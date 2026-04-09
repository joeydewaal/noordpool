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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub is_moderator: Option<bool>,
}

#[requires(Role::Admin)]
pub async fn update(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    tracing::info!(user_id = %id, "users::update");

    let mut user = User::filter_by_id(id)
        .first()
        .exec(&mut state.db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut update = user.update();
    let mut has_changes = false;

    if let Some(is_moderator) = body.is_moderator {
        update.set_is_moderator(is_moderator);
        has_changes = true;
    }

    if has_changes {
        update.exec(&mut state.db).await?;
        user = User::filter_by_id(id).get(&mut state.db).await?;
    }

    Ok(Json(user))
}
