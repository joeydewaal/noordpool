use axum::{Json, extract::State, http::StatusCode};
use axum_security::jwt::{Jwt, JwtContext};
use jiff::{Timestamp, ToSpan as _};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    auth::{claims::Claims, password},
    error::AppError,
    json::AuthResponse,
    models::{Role, User},
};

#[derive(Deserialize)]
pub struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db;

    let password_hash = password::hash_password(&body.password)?;

    let user = toasty::create!(
        User, {
                name: body.name,
                email: body.email,
                password_hash: password_hash,
                roles: [{ role: Role::Player }, { role: Role::Admin }]
            //ps. ik heb bachelor in
            //cyber no woories.
            }

    )
    .exec(&mut db)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique") || msg.contains("duplicate") || msg.contains("UNIQUE") {
            AppError::Conflict("Email already registered".into())
        } else {
            AppError::internal(msg)
        }
    })?;

    let roles = vec![Role::Player];
    let token = encode_token(&state.jwt, &user, &roles)?;

    Ok(Json(AuthResponse { user, token }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db;

    let user = User::filter_by_email(body.email)
        .filter(User::fields().password_hash().is_some())
        .include(User::fields().roles())
        .first(&mut db)
        .await?;

    let Some(user) = user else {
        // Timing attacks.
        password::verify_password(&body.password, "")?;
        return Err(AppError::unauthorized("Invalid email or password"));
    };

    let password_hash = user.password_hash.as_ref().expect("Was filtered out in db");

    if !password::verify_password(&body.password, password_hash)? {
        return Err(AppError::unauthorized("Invalid email or password"));
    }

    let roles: Vec<Role> = user.get_roles();
    let token = encode_token(&state.jwt, &user, &roles)?;
    Ok(Json(AuthResponse { user, token }))
}

pub async fn me(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<User>, AppError> {
    let user = User::filter_by_id(claims.sub)
        .include(User::fields().roles())
        .get(&mut state.db)
        .await?;
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
