use axum::{
    Json,
    extract::{Path, State},
};
use axum_security::rbac::requires;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Role, User},
};

#[requires(Role::Admin)]
pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let users = User::all()
        .order_by(User::fields().created_at().asc())
        .exec(&mut state.db)
        .await?;

    Ok(Json(users))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub is_moderator: bool,
}

#[requires(Role::Admin)]
pub async fn update(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    tracing::info!(user_id = %id, "users::update");

    User::update_by_id(id)
        .is_moderator(body.is_moderator)
        .exec(&mut state.db)
        .await?;

    let user = User::get_by_id(&mut state.db, id).await?;
    Ok(Json(user))
}
