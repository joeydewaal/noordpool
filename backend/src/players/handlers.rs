use std::collections::HashSet;

use axum::{Json, extract::{Path, State}};
use axum_security::jwt::Jwt;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    json::{CreatePlayerRequest, PlayerResponse, PlayerStatsResponse, UpdatePlayerRequest},
    models::{EventType, Game, MatchEvent, MatchStatus, Player},
};

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<PlayerResponse>>, AppError> {
    let mut db = state.db.clone();
    let players: Vec<Player> = Player::all().collect(&mut db).await?;
    let resp: Vec<PlayerResponse> = players.iter().map(PlayerResponse::from_player).collect();
    Ok(Json(resp))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PlayerResponse>, AppError> {
    let mut db = state.db.clone();
    let id: Uuid = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid player ID".into()))?;
    let player = Player::filter_by_id(&id)
        .first(&mut db)
        .await?
        .ok_or_else(|| AppError::NotFound("Player not found".into()))?;
    Ok(Json(PlayerResponse::from_player(&player)))
}

pub async fn create(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Json(body): Json<CreatePlayerRequest>,
) -> Result<Json<PlayerResponse>, AppError> {
    let mut db = state.db.clone();
    let player = toasty::create!(Player, {
        name: body.name,
        shirt_number: body.shirt_number,
        position: body.position,
    })
    .exec(&mut db)
    .await?;
    Ok(Json(PlayerResponse::from_player(&player)))
}

pub async fn update(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<Json<PlayerResponse>, AppError> {
    let mut db = state.db.clone();
    let id: Uuid = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid player ID".into()))?;
    let mut player = Player::filter_by_id(&id)
        .first(&mut db)
        .await?
        .ok_or_else(|| AppError::NotFound("Player not found".into()))?;

    let mut update = player.update();
    if let Some(name) = body.name {
        update = update.name(name);
    }
    if let Some(shirt_number) = body.shirt_number {
        update = update.shirt_number(shirt_number);
    }
    if let Some(position) = body.position {
        update = update.position(position);
    }
    if let Some(active) = body.active {
        update = update.active(active);
    }
    update.exec(&mut db).await?;

    let player = Player::filter_by_id(&id)
        .first(&mut db)
        .await?
        .ok_or_else(|| AppError::NotFound("Player not found".into()))?;
    Ok(Json(PlayerResponse::from_player(&player)))
}

pub async fn stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PlayerStatsResponse>, AppError> {
    let mut db = state.db.clone();
    let id: Uuid = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid player ID".into()))?;

    // Verify player exists
    Player::filter_by_id(&id)
        .first(&mut db)
        .await?
        .ok_or_else(|| AppError::NotFound("Player not found".into()))?;

    // Get all events for this player
    let events: Vec<MatchEvent> = MatchEvent::filter_by_player_id(&id)
        .collect(&mut db)
        .await?;

    // Get completed game IDs for appearances
    let game_ids: Vec<Uuid> = events.iter().map(|e| e.game_id).collect::<HashSet<_>>().into_iter().collect();
    let mut appearances = 0i32;
    for game_id in &game_ids {
        if let Some(game) = Game::filter_by_id(game_id).first(&mut db).await? {
            if game.status == MatchStatus::Completed {
                appearances += 1;
            }
        }
    }

    let mut goals = 0i32;
    let mut assists = 0i32;
    let mut yellow_cards = 0i32;
    let mut red_cards = 0i32;
    for event in &events {
        match event.event_type {
            EventType::Goal => goals += 1,
            EventType::Assist => assists += 1,
            EventType::YellowCard => yellow_cards += 1,
            EventType::RedCard => red_cards += 1,
        }
    }

    Ok(Json(PlayerStatsResponse {
        player_id: id.to_string(),
        appearances,
        goals,
        assists,
        yellow_cards,
        red_cards,
    }))
}
