use axum::{Json, extract::{Path, State}};
use axum_security::jwt::Jwt;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    json::CreateMatchEventRequest,
    models::{Game, MatchEvent},
};

pub async fn list(
    State(state): State<AppState>,
    Path(match_id): Path<Uuid>,
) -> Result<Json<Vec<MatchEvent>>, AppError> {
    let mut db = state.db.clone();

    let mut events: Vec<MatchEvent> = MatchEvent::filter_by_game_id(&match_id)
        .collect(&mut db)
        .await?;
    events.sort_by_key(|e| e.minute);
    Ok(Json(events))
}

pub async fn create(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Path(match_id): Path<Uuid>,
    Json(body): Json<CreateMatchEventRequest>,
) -> Result<Json<MatchEvent>, AppError> {
    let mut db = state.db.clone();
    let player_id: Uuid = body
        .player_id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid player ID".into()))?;

    // Verify match exists
    Game::get_by_id(&mut db, &match_id).await?;

    let event = toasty::create!(MatchEvent, {
        game_id: match_id,
        player_id: player_id,
        event_type: body.event_type,
        minute: body.minute,
    })
    .exec(&mut db)
    .await?;
    Ok(Json(event))
}

pub async fn delete(
    _claims: Jwt<Claims>,
    State(state): State<AppState>,
    Path((match_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let mut db = state.db.clone();

    let event = MatchEvent::get_by_id(&mut db, &event_id).await?;

    if event.game_id != match_id {
        return Err(AppError::BadRequest("Event does not belong to this match".into()));
    }

    event.delete(&mut db).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
