use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use axum_security::jwt::{Jwt, JwtContext};
use jiff::{Timestamp, ToSpan as _};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    auth::{claims::Claims, password},
    error::AppError,
    models::{Player, Position, Role, User},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: User,
    pub player_id: Option<Uuid>,
    pub token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMatchResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub shirt_number: i32,
    pub position: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LinkPlayerRequest {
    player_id: Uuid,
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

    let user = toasty::create!(User {
        first_name: body.first_name,
        last_name: body.last_name,
        email: body.email,
        password_hash: password_hash,
    })
    .exec(&mut db)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique") || msg.contains("duplicate") || msg.contains("UNIQUE") {
            AppError::conflict("Email already registered")
        } else {
            AppError::internal(msg)
        }
    })?;

    let token = encode_token(&state.jwt, &user, &[Role::Player], None)?;
    Ok(Json(AuthResponse {
        user,
        player_id: None,
        token,
    }))
}

#[tracing::instrument(skip_all)]
pub async fn link_player(
    State(state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
    Json(body): Json<LinkPlayerRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db;

    let mut player = Player::get_by_id(&mut db, body.player_id).await?;

    if player.user_id.is_some() {
        return Err(AppError::conflict("Player already has an account"));
    }

    let mut player_update = player.update();
    player_update.set_user_id(Some(claims.sub));
    player_update.exec(&mut db).await?;

    let user = User::filter_by_id(claims.sub).get(&mut db).await?;

    let roles: Vec<Role> = user.get_roles();
    let token = encode_token(&state.jwt, &user, &roles, Some(player.id))?;

    Ok(Json(AuthResponse {
        user,
        player_id: Some(player.id),
        token,
    }))
}

#[tracing::instrument(skip_all)]
pub async fn unlink_player(
    State(state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<AuthResponse>, AppError> {
    let mut db = state.db;

    let player = Player::filter_by_user_id(claims.sub)
        .first()
        .exec(&mut db)
        .await?;

    let Some(mut player) = player else {
        return Err(AppError::not_found("No player linked"));
    };

    let mut player_update = player.update();
    player_update.set_user_id(None);
    player_update.exec(&mut db).await?;

    let user = User::filter_by_id(claims.sub).get(&mut db).await?;
    let roles: Vec<Role> = user.get_roles();
    let token = encode_token(&state.jwt, &user, &roles, None)?;

    Ok(Json(AuthResponse {
        user,
        player_id: None,
        token,
    }))
}

#[tracing::instrument(skip_all)]
pub async fn find_player(
    State(mut state): State<AppState>,
    Query(query): Query<FindPlayerQuery>,
) -> Result<Json<Vec<PlayerMatchResponse>>, AppError> {
    let name = query.name.to_lowercase();

    let all = Player::all_active()
        .filter(Player::fields().user_id().is_none())
        .exec(&mut state.db)
        .await?;

    let matches = all
        .into_iter()
        .filter(|p| {
            let full = format!("{} {}", p.first_name, p.last_name).to_lowercase();
            p.first_name.to_lowercase().contains(&name)
                || p.last_name.to_lowercase().contains(&name)
                || full.contains(&name)
        })
        .map(|p| PlayerMatchResponse {
            id: p.id,
            first_name: p.first_name,
            last_name: p.last_name,
            shirt_number: p.shirt_number,
            position: p.position,
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

    let player_id = Player::filter_by_user_id(user.id)
        .first()
        .exec(&mut db)
        .await?
        .map(|p| p.id);

    let token = encode_token(&state.jwt, &user, &roles, player_id)?;
    Ok(Json(AuthResponse {
        user,
        player_id,
        token,
    }))
}

#[tracing::instrument(skip(state), fields(user_id = %claims.sub))]
pub async fn me(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<User>, AppError> {
    let user = User::filter_by_id(claims.sub)
        .first()
        .exec(&mut state.db)
        .await?;

    let Some(user) = user else {
        return Err(AppError::unauthorized("User not found"));
    };
    Ok(Json(user))
}

pub async fn logout() -> StatusCode {
    StatusCode::OK
}

fn encode_token(
    jwt: &JwtContext<Claims>,
    user: &User,
    roles: &[Role],
    player_id: Option<uuid::Uuid>,
) -> Result<String, AppError> {
    let claims = Claims {
        sub: user.id,
        player_id,
        email: user.email.clone(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        roles: roles.to_vec(),
        exp: Timestamp::now() + 24.hours(),
    };
    jwt.encode_token(&claims).map_err(AppError::internal)
}
