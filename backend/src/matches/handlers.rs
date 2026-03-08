use axum::{Json, extract::{Path, Query, State}};
use axum_security::jwt::Jwt;
use jiff::Timestamp;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    json::{CreateMatchRequest, UpdateMatchRequest},
    models::{Game, MatchStatus},
};

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<usize>,
}

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db.clone();
    let mut games: Vec<Game> = Game::all().collect(&mut db).await?;
    games.sort_by(|a, b| b.date_time.cmp(&a.date_time));
    Ok(Json(games))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Game>, AppError> {
    let mut db = state.db.clone();
    let game = Game::get_by_id(&mut db, &id).await?;
    Ok(Json(game))
}

pub async fn upcoming(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db.clone();
    let mut games: Vec<Game> = Game::all().collect(&mut db).await?;
    games.retain(|g| g.status == MatchStatus::Scheduled);
    games.sort_by(|a, b| a.date_time.cmp(&b.date_time));
    if let Some(limit) = query.limit {
        games.truncate(limit);
    }
    Ok(Json(games))
}

pub async fn recent(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    let mut db = state.db.clone();
    let mut games: Vec<Game> = Game::all().collect(&mut db).await?;
    games.retain(|g| g.status == MatchStatus::Completed);
    games.sort_by(|a, b| b.date_time.cmp(&a.date_time));
    if let Some(limit) = query.limit {
        games.truncate(limit);
    }
    Ok(Json(games))
}

pub async fn create(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Json(body): Json<CreateMatchRequest>,
) -> Result<Json<Game>, AppError> {
    let mut db = state.db.clone();
    let date_time: Timestamp = body
        .date_time
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid dateTime format".into()))?;
    let game = toasty::create!(Game, {
        opponent: body.opponent,
        location: body.location,
        date_time: date_time,
        home_away: body.home_away,
    })
    .exec(&mut db)
    .await?;
    Ok(Json(game))
}

pub async fn update(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMatchRequest>,
) -> Result<Json<Game>, AppError> {
    let mut db = state.db.clone();
    let mut game = Game::get_by_id(&mut db, &id).await?;

    let mut update = game.update();
    if let Some(opponent) = body.opponent {
        update = update.opponent(opponent);
    }
    if let Some(location) = body.location {
        update = update.location(location);
    }
    if let Some(date_time) = body.date_time {
        let ts: Timestamp = date_time
            .parse()
            .map_err(|_| AppError::BadRequest("Invalid dateTime format".into()))?;
        update = update.date_time(ts);
    }
    if let Some(home_away) = body.home_away {
        update = update.home_away(home_away);
    }
    if let Some(status) = body.status {
        update = update.status(status);
    }
    if let Some(home_score) = body.home_score {
        update = update.home_score(home_score);
    }
    if let Some(away_score) = body.away_score {
        update = update.away_score(away_score);
    }
    update.exec(&mut db).await?;

    let game = Game::get_by_id(&mut db, &id).await?;
    Ok(Json(game))
}
