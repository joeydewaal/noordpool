use std::collections::HashSet;

use axum::{Json, extract::{Path, State}};
use axum_security::jwt::Jwt;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    json::{CreatePlayerRequest, PlayerStatsResponse, UpdatePlayerRequest},
    models::{EventType, Game, MatchEvent, MatchStatus, Player},
};

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<Player>>, AppError> {
    let mut db = state.db.clone();
    let players: Vec<Player> = Player::all().collect(&mut db).await?;
    Ok(Json(players))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Player>, AppError> {
    let mut db = state.db.clone();
    let player = Player::get_by_id(&mut db, &id).await?;
    Ok(Json(player))
}

pub async fn create(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Json(body): Json<CreatePlayerRequest>,
) -> Result<Json<Player>, AppError> {
    let mut db = state.db.clone();
    let player = toasty::create!(Player, {
        name: body.name,
        shirt_number: body.shirt_number,
        position: body.position,
    })
    .exec(&mut db)
    .await?;
    Ok(Json(player))
}

pub async fn update(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<Json<Player>, AppError> {
    let mut db = state.db.clone();
    let mut player = Player::get_by_id(&mut db, &id).await?;

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

    let player = Player::get_by_id(&mut db, &id).await?;
    Ok(Json(player))
}

pub async fn stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlayerStatsResponse>, AppError> {
    let mut db = state.db.clone();

    // Verify player exists
    Player::get_by_id(&mut db, &id).await?;

    // Get all events for this player
    let events: Vec<MatchEvent> = MatchEvent::filter_by_player_id(&id)
        .collect(&mut db)
        .await?;

    // Get completed game IDs for appearances
    let game_ids: Vec<Uuid> = events.iter().map(|e| e.game_id).collect::<HashSet<_>>().into_iter().collect();
    let mut appearances = 0i32;
    for game_id in &game_ids {
        let game = Game::get_by_id(&mut db, game_id).await?;
        if game.status == MatchStatus::Completed {
            appearances += 1;
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
