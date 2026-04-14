use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_security::rbac::requires_any;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Game, GameEvent, Role},
    push,
};

/// Server-computed ETag for a game's live state. Format is
/// `W/"<id>-<version>"` — weak because the body may be re-serialized
/// with a different timestamp order but represent the same logical
/// state. A bump of `version` is the only thing that changes it.
fn game_etag(game: &Game) -> String {
    format!("W/\"{}-{}\"", game.id, game.version)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePollResponse {
    pub id: Uuid,
    pub status: &'static str,
    pub home_score: i32,
    pub away_score: i32,
    pub version: i64,
    pub updated_at: Timestamp,
    pub events: Vec<GameEvent>,
}

/// Public polling endpoint. Returns 304 Not Modified when the client's
/// `If-None-Match` header equals the current ETag — lets Lambda skip
/// serialization for idle pollers.
#[tracing::instrument(skip(state, headers), fields(game_id = %id))]
pub async fn poll_live(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let game = Game::filter_by_id(id)
        .include(Game::fields().events().player())
        .get(&mut state.db)
        .await?;

    let etag = game_etag(&game);

    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
        && if_none_match.to_str().map(|s| s == etag).unwrap_or(false)
    {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        resp.headers_mut().insert(
            header::ETAG,
            HeaderValue::from_str(&etag).map_err(AppError::internal)?,
        );
        resp.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, must-revalidate"),
        );
        return Ok(resp);
    }

    let now = Timestamp::now();
    let events: Vec<GameEvent> = game.events.get().to_vec();
    let body = LivePollResponse {
        id: game.id,
        status: game.derived_status(now),
        home_score: game.home_score,
        away_score: game.away_score,
        version: game.version,
        updated_at: game.updated_at,
        events,
    };

    let mut resp = Json(body).into_response();
    resp.headers_mut().insert(
        header::ETAG,
        HeaderValue::from_str(&etag).map_err(AppError::internal)?,
    );
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, must-revalidate"),
    );
    Ok(resp)
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

/// Moderator-only quick-action to bump a specific side's score by
/// +/-1. The caller (frontend) determines which side based on the
/// user's team affiliation. On +1, fires a goal push notification
/// (awaited — Lambda freezes after response).
#[requires_any(Role::Admin, Role::Moderator)]
#[tracing::instrument(skip(state, body), fields(game_id = %id))]
pub async fn adjust_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<AdjustScoreRequest>,
) -> Result<Json<LivePollResponse>, AppError> {
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

    // Fire the goal push for +1 only. Awaited on purpose — Lambda
    // freezes the runtime once the HTTP response goes back, so a
    // spawned task would get dropped.
    if body.delta == 1 {
        let home_name = &fresh.home_team.get().name;
        let away_name = &fresh.away_team.get().name;
        push::notify_goal(&state, &fresh, Some(body.side), home_name, away_name).await;
    }

    let events: Vec<GameEvent> = fresh.events.get().to_vec();
    Ok(Json(LivePollResponse {
        id: fresh.id,
        status: fresh.derived_status(now),
        home_score: fresh.home_score,
        away_score: fresh.away_score,
        version: fresh.version,
        updated_at: fresh.updated_at,
        events,
    }))
}
