use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_security::rbac::{requires, requires_any};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use jiff::ToSpan;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Game, HomeAway, Role},
};

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameRequest {
    pub opponent: String,
    pub location: String,
    pub date_time: Timestamp,
    pub home_away: HomeAway,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGameRequest {
    pub opponent: Option<String>,
    pub location: Option<String>,
    pub date_time: Option<Timestamp>,
    pub home_away: Option<HomeAway>,
    pub cancelled: Option<bool>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamesSummaryResponse {
    pub upcoming: Vec<Game>,
    pub recent: Vec<Game>,
}

#[tracing::instrument(skip(state))]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db;

    let games = Game::all()
        .order_by(Game::fields().date_time().asc())
        .exec(&mut db)
        .await?;
    Ok(Json(games))
}

#[tracing::instrument(skip(state), fields(game_id = %id))]
pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Game>, AppError> {
    let game = Game::filter_by_id(id)
        .include(Game::fields().events().player())
        .get(&mut state.db)
        .await?;
    Ok(Json(game))
}

#[tracing::instrument(skip(state, query))]
pub async fn upcoming(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db;
    let now = Timestamp::now();

    let all_games = Game::all()
        .filter(Game::fields().cancelled().eq(false))
        .order_by(Game::fields().date_time().asc())
        .exec(&mut db)
        .await?;

    let mut games: Vec<Game> = all_games
        .into_iter()
        .filter(|g| g.date_time > now)
        .collect();

    if let Some(limit) = query.limit {
        games.truncate(limit);
    }

    Ok(Json(games))
}

#[tracing::instrument(skip(state, query))]
pub async fn recent(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db;
    let now = Timestamp::now();
    let match_duration = 90.minutes();

    let all_games = Game::all()
        .filter(Game::fields().cancelled().eq(false))
        .order_by(Game::fields().date_time().desc())
        .exec(&mut db)
        .await?;

    let mut games: Vec<Game> = all_games
        .into_iter()
        .filter(|g| g.date_time.checked_add(match_duration).is_ok_and(|end| end <= now))
        .collect();

    if let Some(limit) = query.limit {
        games.truncate(limit);
    }

    Ok(Json(games))
}

#[tracing::instrument(skip(state, query))]
pub async fn summary(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<GamesSummaryResponse>, AppError> {
    let mut db = state.db;
    let now = Timestamp::now();
    let match_duration = 90.minutes();
    let limit = query.limit.unwrap_or(3);

    let all_games = Game::all()
        .filter(Game::fields().cancelled().eq(false))
        .order_by(Game::fields().date_time().asc())
        .exec(&mut db)
        .await?;

    // Nearest upcoming games first (asc)
    let upcoming: Vec<Game> = all_games
        .iter()
        .filter(|g| g.date_time > now)
        .cloned()
        .take(limit)
        .collect();

    // Most recent results first (desc)
    let recent: Vec<Game> = all_games
        .iter()
        .rev()
        .filter(|g| g.date_time.checked_add(match_duration).is_ok_and(|end| end <= now))
        .cloned()
        .take(limit)
        .collect();

    Ok(Json(GamesSummaryResponse { upcoming, recent }))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateGameRequest>,
) -> Result<Json<Game>, AppError> {
    tracing::info!("games::create");
    let mut db = state.db;

    let game = toasty::create!(Game {
        opponent: body.opponent,
        location: body.location,
        date_time: body.date_time,
        home_away: body.home_away,
    })
    .exec(&mut db)
    .await?;
    Ok(Json(game))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGameRequest>,
) -> Result<Json<Game>, AppError> {
    tracing::info!(game_id = %id, "games::update");
    let mut db = state.db;

    let mut game = Game::get_by_id(&mut db, id).await?;

    let mut update = game.update();

    if let Some(opponent) = req.opponent {
        update.set_opponent(opponent);
    }
    if let Some(location) = req.location {
        update.set_location(location);
    }
    if let Some(date_time) = req.date_time {
        update.set_date_time(date_time);
    }

    if let Some(home_away) = req.home_away {
        update.set_home_away(home_away);
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

    update.exec(&mut db).await?;
    Ok(Json(game))
}

#[requires(Role::Admin)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    tracing::info!(game_id = %id, "games::delete");
    let mut db = state.db;

    let game = Game::get_by_id(&mut db, id).await?;

    game.delete().exec(&mut db).await?;
    Ok(StatusCode::NO_CONTENT)
}
