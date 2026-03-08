use axum::{
    Json,
    extract::{Path, State},
};
use axum_security::jwt::Jwt;
use uuid::Uuid;

use crate::{
    app_state::AppState, auth::claims::Claims, error::AppError, json::CreateMatchEventRequest,
    models::MatchEvent,
};

pub async fn list(
    State(mut state): State<AppState>,
    Path(match_id): Path<Uuid>,
) -> Result<Json<Vec<MatchEvent>>, AppError> {
    let db = &mut state.db;

    let events: Vec<MatchEvent> = MatchEvent::filter_by_game_id(match_id)
        .order_by(MatchEvent::fields().minute().asc())
        .collect(db)
        .await?;
    Ok(Json(events))
}

pub async fn create(
    _claims: Jwt<Claims>,
    State(mut state): State<AppState>,
    Path(match_id): Path<Uuid>,
    Json(body): Json<CreateMatchEventRequest>,
) -> Result<Json<MatchEvent>, AppError> {
    let db = &mut state.db;

    let event = toasty::create!(MatchEvent, {
        game_id: match_id,
        player_id: body.player_id,
        event_type: body.event_type,
        minute: body.minute,
    })
    .exec(db)
    .await?;
    Ok(Json(event))
}

pub async fn delete(
    _claims: Jwt<Claims>,
    State(mut state): State<AppState>,
    Path((match_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let db = &mut state.db;

    MatchEvent::filter_by_id(event_id)
        .filter(MatchEvent::fields().game_id().eq(match_id))
        .delete(db)
        .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
