use axum::{
    Json,
    extract::{Path, Query, State},
};
use axum_security::rbac::{requires, requires_any};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    json::{CreateGameRequest, UpdateGameRequest},
    models::{Game, Role},
};

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db;

    let games = Game::all()
        .order_by(Game::fields().date_time().asc())
        .exec(&mut db)
        .await?;
    Ok(Json(games))
}

pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Game>, AppError> {
    Ok(Json(Game::get_by_id(&mut state.db, &id).await?))
}

pub async fn upcoming(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db;

    let mut game_query = Game::all()
        .filter(Game::fields().status().is_scheduled())
        .order_by(Game::fields().date_time().desc());

    if let Some(limit) = query.limit {
        game_query = game_query.limit(limit);
    }

    let games = game_query.exec(&mut db).await?;
    Ok(Json(games))
}

pub async fn recent(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db;

    let mut game_query = Game::all()
        .filter(Game::fields().status().is_completed())
        .order_by(Game::fields().date_time().desc());

    if let Some(limit) = query.limit {
        game_query = game_query.limit(limit);
    }

    let games = game_query.exec(&mut db).await?;
    Ok(Json(games))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(mut state): State<AppState>,
    Json(body): Json<CreateGameRequest>,
) -> Result<Json<Game>, AppError> {
    let db = &mut state.db;

    let game = toasty::create!(Game {
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
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGameRequest>,
) -> Result<Json<Game>, AppError> {
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
    if let Some(status) = req.status {
        update.set_status(status);
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
    let mut db = state.db;

    // Verify match exists
    let game = Game::get_by_id(&mut db, &id).await?;

    game.delete().exec(&mut db).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
