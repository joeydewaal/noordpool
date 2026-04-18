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
    games::live_ws::LiveEvent,
    models::{EventType, Game, GameEvent, Player, Role},
    push,
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

    let event = GameEvent::create()
        .game_id(game_id)
        .player_id(body.player_id)
        .event_type(body.event_type)
        .minute(body.minute)
        .exec(&mut db)
        .await?;

    // We also need the player attached to this event. The create call has no way to do this
    // currently so we refetch the newly inserted event.
    let event = GameEvent::filter_by_id(event.id)
        .include(GameEvent::fields().player())
        .get(&mut db)
        .await?;

    // Touch the parent game so live pollers see the new event on
    // their next tick. Goal/OwnGoal events also bump the correct side of
    // the score based on the player's team.
    let now = Timestamp::now();
    let mut game = Game::get_by_id(&mut db, game_id).await?;
    let next_version = game.version + 1;
    let was_live = game.is_live(now);

    let score_change: Option<(i32, i32, Option<crate::games::live::ScoreSide>)> =
        match event.event_type {
            EventType::Goal => {
                let player = Player::filter_by_id(body.player_id).get(&mut db).await?;
                let team_id = player
                    .team_id
                    .ok_or_else(|| AppError::bad_request("player has no team"))?;
                if team_id == game.home_team_id {
                    Some((
                        game.home_score + 1,
                        game.away_score,
                        Some(crate::games::live::ScoreSide::Home),
                    ))
                } else if team_id == game.away_team_id {
                    Some((
                        game.home_score,
                        game.away_score + 1,
                        Some(crate::games::live::ScoreSide::Away),
                    ))
                } else {
                    return Err(AppError::bad_request(
                        "player does not belong to either team in this game",
                    ));
                }
            }
            EventType::OwnGoal => {
                let player = Player::filter_by_id(body.player_id).get(&mut db).await?;
                let team_id = player
                    .team_id
                    .ok_or_else(|| AppError::bad_request("player has no team"))?;
                // Own goal: the opposing team gets the point
                if team_id == game.home_team_id {
                    Some((
                        game.home_score,
                        game.away_score + 1,
                        Some(crate::games::live::ScoreSide::Away),
                    ))
                } else if team_id == game.away_team_id {
                    Some((
                        game.home_score + 1,
                        game.away_score,
                        Some(crate::games::live::ScoreSide::Home),
                    ))
                } else {
                    return Err(AppError::bad_request(
                        "player does not belong to either team in this game",
                    ));
                }
            }
            _ => None,
        };

    let (new_home, new_away, goal_side) = score_change
        .map(|(h, a, s)| (h, a, s))
        .unwrap_or((game.home_score, game.away_score, None));
    let scores_changed = score_change.is_some();

    let mut update = game.update();
    update.set_version(next_version);
    update.set_updated_at(now);
    if scores_changed {
        update.set_home_score(new_home);
        update.set_away_score(new_away);
    }
    update.exec(&mut db).await?;

    state
        .live_hub
        .publish(game_id, LiveEvent::EventAdded(event.clone()));
    if scores_changed {
        state.live_hub.publish(
            game_id,
            LiveEvent::ScoreUpdate {
                home: new_home,
                away: new_away,
                version: next_version,
                updated_at: now,
            },
        );
    }

    if was_live && scores_changed {
        let fresh = Game::filter_by_id(game_id)
            .include(Game::fields().home_team())
            .include(Game::fields().away_team())
            .get(&mut db)
            .await?;
        let home_name = &fresh.home_team.get().name;
        let away_name = &fresh.away_team.get().name;
        push::notify_goal(&state, &fresh, goal_side, home_name, away_name).await;
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

    // Load the event first so we know whether removing it should also
    // back out a goal from the score.
    let event = GameEvent::filter_by_id(event_id)
        .filter(GameEvent::fields().game_id().eq(game_id))
        .get(&mut db)
        .await?;

    GameEvent::filter_by_id(event_id)
        .filter(GameEvent::fields().game_id().eq(game_id))
        .delete()
        .exec(&mut db)
        .await?;

    // Always bump the parent game so live pollers see the deletion.
    // Decrement the correct side when a goal/own-goal event is removed,
    // based on the player's team, clamped at zero.
    let now = Timestamp::now();
    let mut game = Game::get_by_id(&mut db, game_id).await?;
    let next_version = game.version + 1;

    let score_change: Option<(i32, i32)> = match event.event_type {
        EventType::Goal => {
            let player = Player::filter_by_id(event.player_id).get(&mut db).await?;
            let team_id = player.team_id;
            if team_id == Some(game.home_team_id) {
                Some(((game.home_score - 1).max(0), game.away_score))
            } else {
                Some((game.home_score, (game.away_score - 1).max(0)))
            }
        }
        EventType::OwnGoal => {
            let player = Player::filter_by_id(event.player_id).get(&mut db).await?;
            let team_id = player.team_id;
            // Own goal credited the opponent, so decrement the opponent's score
            if team_id == Some(game.home_team_id) {
                Some((game.home_score, (game.away_score - 1).max(0)))
            } else {
                Some(((game.home_score - 1).max(0), game.away_score))
            }
        }
        _ => None,
    };

    let (new_home, new_away) = score_change.unwrap_or((game.home_score, game.away_score));
    let scores_changed = score_change.is_some();

    let mut update = game.update();
    update.set_version(next_version);
    update.set_updated_at(now);
    if scores_changed {
        update.set_home_score(new_home);
        update.set_away_score(new_away);
    }
    update.exec(&mut db).await?;

    state
        .live_hub
        .publish(game_id, LiveEvent::EventDeleted { id: event_id });
    if scores_changed {
        state.live_hub.publish(
            game_id,
            LiveEvent::ScoreUpdate {
                home: new_home,
                away: new_away,
                version: next_version,
                updated_at: now,
            },
        );
    }

    Ok(StatusCode::NO_CONTENT)
}
