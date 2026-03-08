use axum::{Json, extract::State};
use axum_security::jwt::{Jwt, JwtContext, get_current_timestamp};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    auth::{
        claims::{Claims, Role},
        password,
    },
    error::AppError,
    json::{AuthResponse, UserResponse},
    models::{User, UserRole},
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
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db.clone();

    let password_hash = password::hash_password(&body.password)?;

    let user = User::create()
        .name(&body.name)
        .email(&body.email)
        .password_hash(&password_hash)
        .created_at(jiff::Timestamp::now())
        .exec(&mut db)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unique") || msg.contains("duplicate") || msg.contains("UNIQUE") {
                AppError::Conflict("Email already registered".into())
            } else {
                AppError::Internal(msg)
            }
        })?;

    let _role = user
        .roles()
        .create()
        .role("player")
        .exec(&mut db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

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

    let user = User::get_by_email(&mut db, &body.email)
        .await
        .map_err(|_| AppError::Unauthorized("Invalid email or password".into()))?;

    let valid = password::verify_password(&body.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    let role_records = user.roles().collect::<Vec<_>>(&mut db).await.map_err(|e| AppError::Internal(e.to_string()))?;
    let roles: Vec<Role> = role_records
        .iter()
        .filter_map(|r| Role::from_str(&r.role))
        .collect();

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

fn encode_token(
    jwt: &JwtContext<Claims>,
    user: &User,
    roles: &[Role],
) -> Result<String, AppError> {
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
