use axum::{Json, extract::State};
use axum_security::rbac::requires;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Role, team::Team},
};

#[tracing::instrument(skip(state))]
pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<Team>>, AppError> {
    let teams = Team::all()
        .order_by(Team::fields().name().asc())
        .exec(&mut state.db)
        .await?;
    Ok(Json(teams))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTeamRequest {
    pub name: String,
}

#[requires(Role::Admin)]
pub async fn create(
    State(mut state): State<AppState>,
    Json(body): Json<CreateTeamRequest>,
) -> Result<Json<Team>, AppError> {
    tracing::info!(name = %body.name, "teams::create");

    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("name is required"));
    }

    // Duplicate names violate the unique index on `name`.
    if Team::all()
        .filter(Team::fields().name().eq(name))
        .first()
        .exec(&mut state.db)
        .await?
        .is_some()
    {
        return Err(AppError::conflict("team with this name already exists"));
    }

    let team = toasty::create!(Team { name: name })
        .exec(&mut state.db)
        .await?;
    Ok(Json(team))
}
