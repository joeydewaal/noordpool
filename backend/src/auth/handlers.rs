use axum::{Json, extract::{Query, State}, http::StatusCode};
use axum_security::jwt::{Jwt, JwtContext};
use jiff::{Timestamp, ToSpan as _};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::{claims::Claims, password},
    error::AppError,
    json::{AuthResponse, PlayerMatchResponse},
    models::{Role, User},
};

#[derive(Deserialize)]
pub struct RegisterRequest {
    name: String,
    email: String,
    password: String,
    player_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct FindPlayerQuery {
    name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[tracing::instrument(skip_all)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db;

    let password_hash = password::hash_password(&body.password).await?;

    let user = if let Some(player_id) = body.player_id {
        // Link to existing player (created by admin, no email yet)
        let mut player = User::filter_by_id(player_id)
            .include(User::fields().roles())
            .first()
            .exec(&mut db)
            .await?
            .ok_or_else(|| AppError::not_found("Player not found"))?;

        if player.email.is_some() {
            return Err(AppError::conflict("Player already has an account"));
        }

        let mut update = player.update();
        update.set_email(Some(body.email.clone()));
        update.set_password_hash(Some(password_hash));
        update.exec(&mut db).await?;

        // Set updated fields on the local struct for token encoding / response
        player.email = Some(body.email);

        player
    } else {
        toasty::create!(
            User {
                name: body.name,
                email: body.email,
                password_hash: password_hash,
                roles: [{ role: Role::Player }]
            }
        )
        .exec(&mut db)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") || msg.contains("UNIQUE") {
                AppError::conflict("Email already registered")
            } else {
                AppError::internal(msg)
            }
        })?
    };

    let roles = vec![Role::Player];
    let token = encode_token(&state.jwt, &user, &roles)?;

    Ok(Json(AuthResponse { user, token }))
}

#[tracing::instrument(skip_all)]
pub async fn find_player(
    State(mut state): State<AppState>,
    Query(query): Query<FindPlayerQuery>,
) -> Result<Json<Vec<PlayerMatchResponse>>, AppError> {
    let name = query.name.to_lowercase();
    let all = User::all_active().exec(&mut state.db).await?;

    let matches = all
        .into_iter()
        .filter(|u| u.email.is_none() && u.name.to_lowercase().contains(&name))
        .map(|u| PlayerMatchResponse {
            id: u.id,
            name: u.name,
            shirt_number: u.shirt_number,
            position: u.position,
        })
        .collect();

    Ok(Json(matches))
}

#[tracing::instrument(skip_all)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db;

    let user = User::filter_by_email(body.email)
        .filter(User::fields().password_hash().is_some())
        .include(User::fields().roles())
        .first()
        .exec(&mut db)
        .await?;

    let Some(user) = user else {
        // Timing attacks.
        password::verify_password(&body.password, "").await?;
        return Err(AppError::unauthorized("Invalid email or password"));
    };

    let password_hash = user.password_hash.as_ref().expect("Was filtered out in db");

    if !password::verify_password(&body.password, password_hash).await? {
        return Err(AppError::unauthorized("Invalid email or password"));
    }

    let roles: Vec<Role> = user.get_roles();
    let token = encode_token(&state.jwt, &user, &roles)?;
    Ok(Json(AuthResponse { user, token }))
}

#[tracing::instrument(skip(state), fields(user_id = %claims.sub))]
pub async fn me(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<User>, AppError> {
    let user = User::filter_by_id(claims.sub)
        .include(User::fields().roles())
        .first()
        .exec(&mut state.db)
        .await?;
    let Some(user) = user else {
        return Err(AppError::unauthorized("User not found"));
    };
    tracing::debug!("response:\n{:#?}", user);
    Ok(Json(user))
}

pub async fn logout() -> StatusCode {
    StatusCode::OK
}

fn encode_token(jwt: &JwtContext<Claims>, user: &User, roles: &[Role]) -> Result<String, AppError> {
    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        name: user.name.clone(),
        roles: roles.to_vec(),
        exp: Timestamp::now() + 24.hours(),
    };
    jwt.encode_token(&claims).map_err(AppError::internal)
}
