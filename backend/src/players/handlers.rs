use std::collections::HashSet;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_security::rbac::{requires, requires_any};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{EventType, GameStatus, HomeAway, Player, Position, Role},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlayerRequest {
    pub first_name: String,
    pub last_name: String,
    pub shirt_number: i32,
    pub position: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub shirt_number: Option<i32>,
    pub position: Option<Position>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerGoalMatchResponse {
    pub game_id: String,
    pub opponent: String,
    pub date_time: Timestamp,
    pub home_away: HomeAway,
    pub home_score: i32,
    pub away_score: i32,
    pub status: GameStatus,
    pub minutes: Vec<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsResponse {
    pub player_id: String,
    pub appearances: usize,
    pub goals: i32,
    pub assists: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
    pub goal_matches: Vec<PlayerGoalMatchResponse>,
}

#[tracing::instrument(skip(state))]
pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<Player>>, AppError> {
    let players = Player::all_active().exec(&mut state.db).await?;
    Ok(Json(players))
}

#[tracing::instrument(skip(state), fields(player_id = %id))]
pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Player>, AppError> {
    let player = Player::get_by_id(&mut state.db, id).await?;
    Ok(Json(player))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(mut state): State<AppState>,
    Json(body): Json<CreatePlayerRequest>,
) -> Result<Json<Player>, AppError> {
    tracing::info!("players::create");
    let player = toasty::create!(Player {
        first_name: body.first_name,
        last_name: body.last_name,
        shirt_number: body.shirt_number,
        position: body.position,
    })
    .exec(&mut state.db)
    .await?;

    Ok(Json(player))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn update(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<Json<Player>, AppError> {
    tracing::info!(player_id = %id, "players::update");
    let mut player = Player::get_by_id(&mut state.db, id).await?;
    let mut player_update = player.update();

    if let Some(first_name) = body.first_name {
        player_update.set_first_name(first_name);
    }
    if let Some(last_name) = body.last_name {
        player_update.set_last_name(last_name);
    }
    if let Some(shirt_number) = body.shirt_number {
        player_update.set_shirt_number(shirt_number);
    }
    if let Some(position) = body.position {
        player_update.set_position(position);
    }
    if let Some(active) = body.active {
        player_update.set_active(active);
    }

    player_update.exec(&mut state.db).await?;
    Ok(Json(player))
}

#[requires(Role::Admin)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    tracing::info!(player_id = %id, "players::delete");
    let mut db = state.db;

    let mut player = Player::get_by_id(&mut db, id).await?;

    let mut update = player.update();
    update.set_active(false);
    update.exec(&mut db).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state), fields(player_id = %id))]
pub async fn stats(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlayerStatsResponse>, AppError> {
    let db = &mut state.db;

    let player = Player::filter_by_id(id)
        .include(Player::fields().game_events().game())
        .get(db)
        .await?;

    let events = player.game_events.get();

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
    let mut goal_map: std::collections::HashMap<Uuid, (Vec<i32>, &crate::models::Game)> =
        std::collections::HashMap::new();

    for event in events {
        match event.event_type {
            EventType::Goal => {
                goals += 1;
                let game = event.game.get();
                goal_map
                    .entry(event.game_id)
                    .or_insert_with(|| (Vec::new(), game))
                    .0
                    .push(event.minute);
            }
            EventType::Assist => assists += 1,
            EventType::YellowCard => yellow_cards += 1,
            EventType::RedCard => red_cards += 1,
        }
    }

    let mut goal_matches: Vec<PlayerGoalMatchResponse> = goal_map
        .into_values()
        .map(|(mut minutes, game)| {
            minutes.sort_unstable();
            PlayerGoalMatchResponse {
                game_id: game.id.to_string(),
                opponent: game.opponent.clone(),
                date_time: game.date_time,
                home_away: game.home_away.clone(),
                home_score: game.home_score,
                away_score: game.away_score,
                status: game.status.clone(),
                minutes,
            }
        })
        .collect();
    goal_matches.sort_by(|a, b| b.date_time.cmp(&a.date_time));

    let response = PlayerStatsResponse {
        player_id: id.to_string(),
        appearances: game_ids.len(),
        goals,
        assists,
        yellow_cards,
        red_cards,
        goal_matches,
    };
    Ok(Json(response))
}
