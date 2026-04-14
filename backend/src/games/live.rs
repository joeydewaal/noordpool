use axum::{
    Json,
    extract::{Path, State},
};
use axum_security::rbac::requires_any;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    games::live_ws::{LiveEvent, publish},
    models::{Game, GameEvent, Role},
    push,
};

/// Snapshot of a match's live state, used as both the initial WebSocket
/// frame a client receives and as the body of the score-adjust response.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LiveSnapshot {
    pub id: Uuid,
    pub status: &'static str,
    pub home_score: i32,
    pub away_score: i32,
    pub version: i64,
    pub updated_at: Timestamp,
    pub events: Vec<GameEvent>,
}

impl LiveSnapshot {
    pub fn from_game(game: &Game, now: Timestamp) -> Self {
        let events: Vec<GameEvent> = game.events.get().to_vec();
        Self {
            id: game.id,
            status: game.derived_status(now),
            home_score: game.home_score,
            away_score: game.away_score,
            version: game.version,
            updated_at: game.updated_at,
            events,
        }
    }
}

#[derive(Deserialize, Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoreSide {
    Home,
    Away,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdjustScoreRequest {
    pub side: ScoreSide,
    pub delta: i32,
}

/// Moderator quick-action to bump a side's score by +/-1. On +1, fires a
/// goal push notification and publishes a `ScoreUpdate` to the live hub
/// so connected WebSocket viewers update instantly.
#[requires_any(Role::Admin, Role::Moderator)]
#[tracing::instrument(skip(state, body), fields(game_id = %id))]
pub async fn adjust_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<AdjustScoreRequest>,
) -> Result<Json<LiveSnapshot>, AppError> {
    if body.delta != 1 && body.delta != -1 {
        return Err(AppError::conflict("delta must be +1 or -1"));
    }

    let mut db = state.db.clone();
    let mut game = Game::get_by_id(&mut db, id).await?;

    let now = Timestamp::now();
    if !game.is_live(now) {
        return Err(AppError::conflict("game is not currently live"));
    }

    let (new_home, new_away) = match body.side {
        ScoreSide::Home => ((game.home_score + body.delta).max(0), game.away_score),
        ScoreSide::Away => (game.home_score, (game.away_score + body.delta).max(0)),
    };
    let next_version = game.version + 1;

    let mut update = game.update();
    update.set_home_score(new_home);
    update.set_away_score(new_away);
    update.set_version(next_version);
    update.set_updated_at(now);
    update.exec(&mut db).await?;

    // Re-load with events so the response includes the full picture.
    let fresh = Game::filter_by_id(id)
        .include(Game::fields().events().player())
        .include(Game::fields().home_team())
        .include(Game::fields().away_team())
        .get(&mut db)
        .await?;

    publish(
        &state.live_hub,
        id,
        LiveEvent::ScoreUpdate {
            home: fresh.home_score,
            away: fresh.away_score,
            version: fresh.version,
            updated_at: fresh.updated_at,
        },
    );

    if body.delta == 1 {
        let home_name = &fresh.home_team.get().name;
        let away_name = &fresh.away_team.get().name;
        push::notify_goal(&state, &fresh, Some(body.side), home_name, away_name).await;
    }

    Ok(Json(LiveSnapshot::from_game(&fresh, now)))
}
