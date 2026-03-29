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
    json::{
        CreatePlayerRequest, PlayerGoalMatchResponse, PlayerStatsResponse, UpdatePlayerRequest,
    },
    models::{EventType, GameStatus, Role, User},
};

#[tracing::instrument(skip(state))]
pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let players = User::all_active().exec(&mut state.db).await?;
    Ok(Json(players))
}

#[tracing::instrument(skip(state), fields(player_id = %id))]
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
    // Group goal minutes by game_id for goal_matches
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
