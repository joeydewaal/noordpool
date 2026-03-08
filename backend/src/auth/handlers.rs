use axum::{Json, extract::State};
use axum_security::jwt::{Jwt, JwtContext, get_current_timestamp};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    auth::{claims::Claims, password},
    error::AppError,
    json::{AuthResponse, UserResponse},
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

const TOKEN_LIFETIME_SECS: u64 = 24 * 60 * 60;

pub async fn register(
    State(mut state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let db = &mut state.db;

    let password_hash = password::hash_password(&body.password)?;

    let user = toasty::create!(
        User, {
                name: body.name,
                email: body.email,
                password_hash: password_hash,
                roles: [{ role: Role::Player }]
            }

    )
    .exec(db)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique") || msg.contains("duplicate") || msg.contains("UNIQUE") {
            AppError::Conflict("Email already registered".into())
        } else {
            AppError::Internal(msg)
        }
    })?;

    let roles = vec![Role::Player];
    let token = encode_token(&state.jwt, &user, &roles)?;

    Ok(Json(AuthResponse {
        user: UserResponse::from_user(&user, &roles),
        token,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db.clone();

    let user = User::filter_by_email(body.email)
        .include(User::fields().roles())
        .first(&mut db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    let valid = password::verify_password(&body.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    let roles: Vec<Role> = user.roles.get().iter().map(|r| r.role).collect();

    let token = encode_token(&state.jwt, &user, &roles)?;

    Ok(Json(AuthResponse {
        user: UserResponse::from_user(&user, &roles),
        token,
    }))
}

pub async fn me(Jwt(claims): Jwt<Claims>) -> Json<UserResponse> {
    Json(UserResponse {
        id: claims.sub,
        email: claims.email,
        name: claims.name,
        avatar_url: None,
        roles: claims.roles,
    })
}

pub async fn logout() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

fn encode_token(jwt: &JwtContext<Claims>, user: &User, roles: &[Role]) -> Result<String, AppError> {
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        name: user.name.clone(),
        roles: roles.to_vec(),
        exp: get_current_timestamp() + TOKEN_LIFETIME_SECS,
    };
    jwt.encode_token(&claims)
        .map_err(|e| AppError::Internal(e.to_string()))
}
