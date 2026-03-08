use axum::{Json, extract::{Path, State}};
use axum_security::jwt::Jwt;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    json::{CreateMatchEventRequest, MatchEventResponse},
    models::{Game, MatchEvent},
};

pub async fn list(
    State(state): State<AppState>,
    Path(match_id): Path<String>,
) -> Result<Json<Vec<MatchEventResponse>>, AppError> {
    let mut db = state.db.clone();
    let match_id: Uuid = match_id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid match ID".into()))?;

    let mut events: Vec<MatchEvent> = MatchEvent::filter_by_game_id(&match_id)
        .collect(&mut db)
        .await?;
    events.sort_by_key(|e| e.minute);
    let resp: Vec<MatchEventResponse> = events.iter().map(MatchEventResponse::from_event).collect();
    Ok(Json(resp))
}

pub async fn create(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Path(match_id): Path<String>,
    Json(body): Json<CreateMatchEventRequest>,
) -> Result<Json<MatchEventResponse>, AppError> {
    let mut db = state.db.clone();
    let match_id: Uuid = match_id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid match ID".into()))?;
    let player_id: Uuid = body
        .player_id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid player ID".into()))?;

    // Verify match exists
    Game::filter_by_id(&match_id)
        .first(&mut db)
        .await?
        .ok_or_else(|| AppError::NotFound("Match not found".into()))?;

    let event = toasty::create!(MatchEvent, {
        game_id: match_id,
        player_id: player_id,
        event_type: body.event_type,
        minute: body.minute,
    })
    .exec(&mut db)
    .await?;
    Ok(Json(MatchEventResponse::from_event(&event)))
}

pub async fn delete(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Path((match_id, event_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode, AppError> {
    let mut db = state.db.clone();
    let match_id: Uuid = match_id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid match ID".into()))?;
    let event_id: Uuid = event_id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid event ID".into()))?;

    let event = MatchEvent::filter_by_id(&event_id)
        .first(&mut db)
        .await?
        .ok_or_else(|| AppError::NotFound("Event not found".into()))?;

    if event.game_id != match_id {
        return Err(AppError::BadRequest("Event does not belong to this match".into()));
    }

    event.delete(&mut db).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
