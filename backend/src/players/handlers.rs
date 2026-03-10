use std::collections::HashSet;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_security::rbac::{requires, requires_any};
use toasty::Executor;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    json::{CreatePlayerRequest, PlayerStatsResponse, UpdatePlayerRequest},
    models::{EventType, MatchEvent, MatchStatus, Role, User},
};

pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let players: Vec<_> = User::all_active().collect(&mut state.db).await?;
    Ok(Json(players))
}

pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let player = User::get_by_id(&mut state.db, &id).await?;
    Ok(Json(player))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(mut state): State<AppState>,
    Json(body): Json<CreatePlayerRequest>,
) -> Result<Json<User>, AppError> {
    let user = toasty::create!(
        User, {
                name: body.name,
                email: body.email,
                shirt_number: body.shirt_number,
                position: body.position,
                roles: [{ role: Role::Player }]
            }

    )
    .exec(&mut state.db)
    .await?;

    Ok(Json(user))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn update(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<Json<User>, AppError> {
    let mut user_update = User::update_by_id(id);

    if let Some(name) = body.name {
        user_update.set_name(name);
    }

    if let Some(shirt_number) = body.shirt_number {
        user_update.set_shirt_number(shirt_number);
    }
    if let Some(position) = body.position {
        user_update.set_position(position);
    }
    if let Some(active) = body.active {
        user_update.set_active(active);
    }

    let mut tx = state.db.transaction().await?;

    user_update.exec(&mut tx).await?;
    let player = User::get_by_id(&mut tx, &id).await?;

    tx.commit().await?;
    Ok(Json(player))
}

#[requires(Role::Admin)]
pub async fn delete(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let db = &mut state.db;

    let mut player = User::get_by_id(db, &id).await?;

    let mut update = player.update();
    update.set_active(false);
    update.exec(db).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn stats(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlayerStatsResponse>, AppError> {
    let db = &mut state.db;

    let user = User::filter_by_id(id)
        .include(User::fields().match_events().game())
        .get(db)
        .await?;

    let events = user.match_events.get();

    // Get completed game IDs for appearances
    let game_ids: Vec<Uuid> = events
        .iter()
        .filter(|e| e.game.get().status == MatchStatus::Completed)
        .map(|e| e.game_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut goals = 0i32;
    let mut assists = 0i32;
    let mut yellow_cards = 0i32;
    let mut red_cards = 0i32;
    for event in events {
        match event.event_type {
            EventType::Goal => goals += 1,
            EventType::Assist => assists += 1,
            EventType::YellowCard => yellow_cards += 1,
            EventType::RedCard => red_cards += 1,
        }
    }

    Ok(Json(PlayerStatsResponse {
        player_id: id.to_string(),
        appearances: game_ids.len(),
        goals,
        assists,
        yellow_cards,
        red_cards,
    }))
}
