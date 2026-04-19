use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_security::rbac::requires_any;
use jiff::Timestamp;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    games::live::{LiveSnapshot, ScoreSide},
    games::live_ws::LiveEvent,
    models::{EventType, Game, GameEvent, Player, Role, game::compute_scores},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameEventRequest {
    pub player_id: Uuid,
    pub event_type: EventType,
    pub minute: i32,
}

#[tracing::instrument(skip(state), fields(game_id = %game_id))]
pub async fn list(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<GameEvent>>, AppError> {
    let mut db = state.db;

    let events = GameEvent::filter_by_game_id(game_id)
        .order_by(GameEvent::fields().minute().asc())
        .include(GameEvent::fields().player())
        .exec(&mut db)
        .await?;
    Ok(Json(events))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Json(body): Json<CreateGameEventRequest>,
) -> Result<Json<GameEvent>, AppError> {
    tracing::info!(game_id = %game_id, "events::create");
    let mut db = state.db.clone();

    let player = Player::get_by_id(&mut db, body.player_id).await?;
    let team_id = player.team_id;

    let event = GameEvent::create()
        .game_id(game_id)
        .player_id(body.player_id)
        .team_id(team_id)
        .event_type(body.event_type)
        .minute(body.minute)
        .exec(&mut db)
        .await?;

    // Re-fetch with player relation attached.
    let event = GameEvent::filter_by_id(event.id)
        .include(GameEvent::fields().player())
        .get(&mut db)
        .await?;

    // Bump game version so live pollers see the change.
    let now = Timestamp::now();
    let mut game = Game::get_by_id(&mut db, game_id).await?;
    let was_live = game.is_live(now);
    let next_version = game.version + 1;

    let mut update = game.update();
    update.set_version(next_version);
    update.set_updated_at(now);
    update.exec(&mut db).await?;

    state
        .live_hub
        .publish(game_id, LiveEvent::EventAdded(event.clone()));

    // For goal/own-goal events recompute the total score from all events
    // (stored home_score/away_score are now adjustment-only, not goal counts).
    let is_score_event = matches!(event.event_type, EventType::Goal | EventType::OwnGoal);
    if is_score_event {
        let fresh = Game::filter_by_id(game_id)
            .include(Game::fields().events().player())
            .include(Game::fields().home_team())
            .include(Game::fields().away_team())
            .get(&mut db)
            .await?;

        let snapshot = LiveSnapshot::from_game(&fresh, now);
        state.live_hub.publish(
            game_id,
            LiveEvent::ScoreUpdate {
                home: snapshot.home_score,
                away: snapshot.away_score,
                version: next_version,
                updated_at: now,
            },
        );

        if was_live {
            let player = event.player.get();
            let tid = player.team_id;
            let goal_side = Some(match event.event_type {
                EventType::Goal if tid == fresh.home_team_id => ScoreSide::Home,
                EventType::Goal => ScoreSide::Away,
                _ if tid == fresh.home_team_id => ScoreSide::Away, // OwnGoal
                _ => ScoreSide::Home,
            });

            let home_name = fresh.home_team.get().name.clone();
            let away_name = fresh.away_team.get().name.clone();
            let mut game_for_push = fresh.clone();
            game_for_push.home_score = snapshot.home_score;
            game_for_push.away_score = snapshot.away_score;
            state
                .push
                .notify_goal(&game_for_push, goal_side, &home_name, &away_name)
                .await;
        }
    }

    Ok(Json(event))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn delete(
    State(state): State<AppState>,
    Path((game_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    tracing::info!(game_id = %game_id, event_id = %event_id, "events::delete");
    let mut db = state.db;

    let event = GameEvent::filter_by_id(event_id)
        .filter(GameEvent::fields().game_id().eq(game_id))
        .get(&mut db)
        .await?;

    GameEvent::filter_by_id(event_id)
        .filter(GameEvent::fields().game_id().eq(game_id))
        .delete()
        .exec(&mut db)
        .await?;

    let now = Timestamp::now();
    let mut game = Game::get_by_id(&mut db, game_id).await?;
    let next_version = game.version + 1;

    let mut update = game.update();
    update.set_version(next_version);
    update.set_updated_at(now);
    update.exec(&mut db).await?;

    state
        .live_hub
        .publish(game_id, LiveEvent::EventDeleted { id: event_id });

    let is_score_event = matches!(event.event_type, EventType::Goal | EventType::OwnGoal);
    if is_score_event {
        let fresh = Game::filter_by_id(game_id)
            .include(Game::fields().events().player())
            .get(&mut db)
            .await?;

        let events = fresh.events.get();
        let (ch, ca) = compute_scores(events, fresh.home_team_id);
        state.live_hub.publish(
            game_id,
            LiveEvent::ScoreUpdate {
                home: ch + fresh.home_score,
                away: ca + fresh.away_score,
                version: next_version,
                updated_at: now,
            },
        );
    }

    Ok(StatusCode::NO_CONTENT)
}
