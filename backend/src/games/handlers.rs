use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_security::rbac::{requires, requires_any};
use jiff::{Timestamp, ToSpan};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Game, Role, game::MATCH_DURATION_MINUTES},
};

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameRequest {
    pub home_team_id: Uuid,
    pub away_team_id: Uuid,
    pub location: String,
    pub date_time: Timestamp,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGameRequest {
    pub home_team_id: Option<Uuid>,
    pub away_team_id: Option<Uuid>,
    pub location: Option<String>,
    pub date_time: Option<Timestamp>,
    pub cancelled: Option<bool>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

/// Wraps a `Game` so the JSON output carries the server-derived
/// `status` field (`scheduled` | `live` | `finished` | `cancelled`).
/// The frontend never computes its own liveness.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResponse {
    #[serde(flatten)]
    pub game: Game,
    pub status: &'static str,
}

impl GameResponse {
    pub fn new(mut game: Game) -> Self {
        game.apply_computed_scores();
        let status = game.derived_status(Timestamp::now());
        Self { game, status }
    }

    pub fn many(games: Vec<Game>) -> Vec<Self> {
        let now = Timestamp::now();
        games
            .into_iter()
            .map(|mut g| {
                g.apply_computed_scores();
                let status = g.derived_status(now);
                Self { game: g, status }
            })
            .collect()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamesSummaryResponse {
    pub upcoming: Vec<GameResponse>,
    pub recent: Vec<GameResponse>,
}

#[tracing::instrument(skip(state))]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<GameResponse>>, AppError> {
    let mut db = state.db;

    let games = Game::all()
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .include(Game::fields().events().player())
        .order_by(Game::fields().date_time().asc())
        .exec(&mut db)
        .await?;
    Ok(Json(GameResponse::many(games)))
}

#[tracing::instrument(skip(state), fields(game_id = %id))]
pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<GameResponse>, AppError> {
    let game = Game::filter_by_id(id)
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .include(Game::fields().events().player())
        .get(&mut state.db)
        .await?;
    Ok(Json(GameResponse::new(game)))
}

#[tracing::instrument(skip(state, query))]
pub async fn upcoming(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<GameResponse>>, AppError> {
    let mut db = state.db;

    let now = Timestamp::now() - MATCH_DURATION_MINUTES.minutes();

    let mut game_query = Game::all()
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .filter(Game::fields().cancelled().eq(false))
        .filter(Game::fields().date_time().gt(now))
        .order_by(Game::fields().date_time().asc());

    if let Some(limit) = query.limit {
        game_query = game_query.limit(limit);
    }

    let games = game_query.exec(&mut db).await?;
    Ok(Json(GameResponse::many(games)))
}

#[tracing::instrument(skip(state, query))]
pub async fn recent(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<GameResponse>>, AppError> {
    let mut db = state.db;

    let mut game_query = Game::all()
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .include(Game::fields().events().player())
        .filter(Game::fields().cancelled().eq(false))
        .filter(Game::fields().date_time().lt(Timestamp::now()))
        .order_by(Game::fields().date_time().desc());

    if let Some(limit) = query.limit {
        game_query = game_query.limit(limit);
    }

    let games = game_query.exec(&mut db).await?;
    Ok(Json(GameResponse::many(games)))
}

#[tracing::instrument(skip(state, query))]
pub async fn summary(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<GamesSummaryResponse>, AppError> {
    let mut db = state.db;
    let now = Timestamp::now();
    let limit = query.limit.unwrap_or(3);

    let upcoming = Game::all()
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .filter(Game::fields().cancelled().eq(false))
        .filter(Game::fields().date_time().gt(now))
        .order_by(Game::fields().date_time().asc())
        .limit(limit)
        .exec(&mut db)
        .await?;

    let recent = Game::all()
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .include(Game::fields().events().player())
        .filter(Game::fields().cancelled().eq(false))
        .filter(Game::fields().date_time().lt(now))
        .order_by(Game::fields().date_time().desc())
        .limit(limit)
        .exec(&mut db)
        .await?;

    Ok(Json(GamesSummaryResponse {
        upcoming: GameResponse::many(upcoming),
        recent: GameResponse::many(recent),
    }))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateGameRequest>,
) -> Result<Json<GameResponse>, AppError> {
    tracing::info!("games::create");
    let mut db = state.db;

    if body.home_team_id == body.away_team_id {
        return Err(AppError::bad_request(
            "home and away team must be different",
        ));
    }

    let created = toasty::create!(Game {
        home_team_id: body.home_team_id,
        away_team_id: body.away_team_id,
        location: body.location,
        date_time: body.date_time,
    })
    .exec(&mut db)
    .await?;

    // Re-fetch with team relations so the response carries full team
    // objects (the create! call returns a Game with unloaded relations).
    let game = Game::filter_by_id(created.id)
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .get(&mut db)
        .await?;
    Ok(Json(GameResponse::new(game)))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGameRequest>,
) -> Result<Json<GameResponse>, AppError> {
    tracing::info!(game_id = %id, "games::update");
    let mut db = state.db;

    let mut game = Game::get_by_id(&mut db, id).await?;

    // Determine the final home/away so we can validate the pair even
    // when only one side is being changed.
    let next_home = req.home_team_id.unwrap_or(game.home_team_id);
    let next_away = req.away_team_id.unwrap_or(game.away_team_id);
    if next_home == next_away {
        return Err(AppError::bad_request(
            "home and away team must be different",
        ));
    }

    let next_version = game.version + 1;
    let mut update = game.update();

    if let Some(home_team_id) = req.home_team_id {
        update.set_home_team_id(home_team_id);
    }
    if let Some(away_team_id) = req.away_team_id {
        update.set_away_team_id(away_team_id);
    }
    if let Some(location) = req.location {
        update.set_location(location);
    }
    if let Some(date_time) = req.date_time {
        update.set_date_time(date_time);
    }
    if let Some(cancelled) = req.cancelled {
        update.set_cancelled(cancelled);
    }
    if let Some(home_score) = req.home_score {
        update.set_home_score(home_score);
    }
    if let Some(away_score) = req.away_score {
        update.set_away_score(away_score);
    }

    // Every edit touches version + updated_at so ETag clients always
    // see a fresh version after any admin/mod mutation.
    let now = Timestamp::now();
    update.set_version(next_version);
    update.set_updated_at(now);

    update.exec(&mut db).await?;
    let fresh = Game::filter_by_id(id)
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .get(&mut db)
        .await?;
    Ok(Json(GameResponse::new(fresh)))
}

#[requires(Role::Admin)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    tracing::info!(game_id = %id, "games::delete");
    let mut db = state.db;

    let game = Game::get_by_id(&mut db, id).await?;

    game.delete().exec(&mut db).await?;
    Ok(StatusCode::NO_CONTENT)
}
