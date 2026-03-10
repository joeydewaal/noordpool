use axum::{
    Json,
    extract::{Path, Query, State},
};
use axum_security::rbac::{requires, requires_any};
use serde::Deserialize;
use toasty::Executor;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    json::{CreateMatchRequest, UpdateMatchRequest},
    models::{Game, Role},
};

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<Game>>, AppError> {
    let db = &mut state.db;

    // TODO: toasty does not support order by timestamp.
    let mut games: Vec<Game> = Game::all()
        .order_by(Game::fields().date_time().asc())
        .collect(db)
        .await?;
    // games.sort_by(|a, b| b.date_time.cmp(&a.date_time));
    Ok(Json(games))
}

pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Game>, AppError> {
    Ok(Json(Game::get_by_id(&mut state.db, &id).await?))
}

pub async fn upcoming(
    State(mut state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let db = &mut state.db;

    let mut game_query = Game::all().filter(Game::fields().status().is_scheduled());

    if let Some(limit) = query.limit {
        game_query = game_query.limit(limit);
    }

    let mut games: Vec<Game> = game_query.collect(db).await?;
    games.sort_by(|a, b| a.date_time.cmp(&b.date_time));
    Ok(Json(games))
}

pub async fn recent(
    State(mut state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let db = &mut state.db;

    let mut game_query = Game::all().filter(Game::fields().status().is_completed());

    if let Some(limit) = query.limit {
        game_query = game_query.limit(limit);
    }

    let mut games: Vec<Game> = game_query.collect(db).await?;
    games.sort_by(|a, b| a.date_time.cmp(&b.date_time));
    Ok(Json(games))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(mut state): State<AppState>,
    Json(body): Json<CreateMatchRequest>,
) -> Result<Json<Game>, AppError> {
    let db = &mut state.db;

    let game = toasty::create!(Game, {
        opponent: body.opponent,
        location: body.location,
        date_time: body.date_time,
        home_away: body.home_away,
    })
    .exec(db)
    .await?;
    Ok(Json(game))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn update(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMatchRequest>,
) -> Result<Json<Game>, AppError> {
    let db = &mut state.db;

    let mut update = Game::update_by_id(id);

    if let Some(opponent) = body.opponent {
        update.set_opponent(opponent);
    }
    if let Some(location) = body.location {
        update.set_location(location);
    }
    if let Some(date_time) = body.date_time {
        update.set_date_time(date_time);
    }

    if let Some(home_away) = body.home_away {
        update.set_home_away(home_away);
    }
    if let Some(status) = body.status {
        update.set_status(status);
    }
    if let Some(home_score) = body.home_score {
        update.set_home_score(home_score);
    }
    if let Some(away_score) = body.away_score {
        update.set_away_score(away_score);
    }

    let mut tx = db.transaction().await?;
    update.exec(&mut tx).await?;
    let game = Game::get_by_id(&mut tx, &id).await?;
    tx.commit().await?;
    Ok(Json(game))
}

#[requires(Role::Admin)]
pub async fn delete(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let db = &mut state.db;

    // Verify match exists
    let game = Game::get_by_id(db, &id).await?;

    game.delete().exec(db).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
