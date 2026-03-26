use std::collections::HashSet;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_security::rbac::{requires, requires_any};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    json::{CreatePlayerRequest, PlayerStatsResponse, UpdatePlayerRequest},
    models::{EventType, GameStatus, Role, User},
};

#[tracing::instrument(skip(state))]
pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let players = User::all_active().exec(&mut state.db).await?;
    tracing::debug!("response:\n{:#?}", players);
    Ok(Json(players))
}

#[tracing::instrument(skip(state), fields(player_id = %id))]
pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let player = User::get_by_id(&mut state.db, &id).await?;
    tracing::debug!("response:\n{:#?}", player);
    Ok(Json(player))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(mut state): State<AppState>,
    Json(body): Json<CreatePlayerRequest>,
) -> Result<Json<User>, AppError> {
    tracing::info!("players::create");
    let mut user = toasty::create!(
        User {
                name: body.name,
                email: body.email,
                shirt_number: body.shirt_number,
                position: body.position,
                roles: [{ role: Role::Player }]
            }

    )
    .exec(&mut state.db)
    .await?;

    user.roles.unload();
    tracing::debug!("response:\n{:#?}", user);
    Ok(Json(user))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn update(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<Json<User>, AppError> {
    tracing::info!(player_id = %id, "players::update");
    let mut user = User::get_by_id(&mut state.db, id).await?;
    let mut user_update = user.update();

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

    user_update.exec(&mut state.db).await?;
    tracing::debug!("response:\n{:#?}", user);
    Ok(Json(user))
}

#[requires(Role::Admin)]
pub async fn delete(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    tracing::info!(player_id = %id, "players::delete");
    let db = &mut state.db;

    let mut player = User::get_by_id(db, &id).await?;

    let mut update = player.update();
    update.set_active(false);
    update.exec(db).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state), fields(player_id = %id))]
pub async fn stats(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlayerStatsResponse>, AppError> {
    let db = &mut state.db;

    let user = User::filter_by_id(id)
        .include(User::fields().game_events().game())
        .get(db)
        .await?;

    let events = user.game_events.get();

    // Get completed game IDs for appearances
    let game_ids: Vec<Uuid> = events
        .iter()
        .filter(|e| e.game.get().status == GameStatus::Completed)
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

    let response = PlayerStatsResponse {
        player_id: id.to_string(),
        appearances: game_ids.len(),
        goals,
        assists,
        yellow_cards,
        red_cards,
    };
    tracing::debug!("response:\n{:#?}", response);
    Ok(Json(response))
}
