//! Avatar handlers — presigned-URL flow.
//!
//! 1. `POST /api/users/me/avatar/presign` returns `{ uploadUrl, publicUrl }`.
//! 2. Browser PUTs the (already-resized 256x256 WebP) bytes to `uploadUrl`.
//! 3. `POST /api/users/me/avatar/commit { url }` sets `users.avatar_url`.
//! 4. `DELETE /api/users/me/avatar` clears `avatar_url` and best-effort deletes
//!    the object.
//!
//! In local-storage mode (no `R2_*` env), `uploadUrl` points at our own
//! signed `PUT /api/_avatars/upload/{key}` route below, which writes to
//! `AVATAR_DIR` for `ServeDir` to serve at `/avatars/<key>`.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::put,
};
use axum_security::jwt::Jwt;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState, auth::claims::Claims, error::AppError, models::User,
    r2::verify_local_upload,
};

const AVATAR_CONTENT_TYPE: &str = "image/webp";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignResponse {
    pub upload_url: String,
    pub public_url: String,
    pub content_type: &'static str,
    pub key: String,
}

#[tracing::instrument(skip_all, fields(user_id = %claims.sub))]
pub async fn presign(
    State(state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<PresignResponse>, AppError> {
    let key = format!(
        "avatars/{}-{}.webp",
        claims.sub,
        Timestamp::now().as_second()
    );
    let presigned = state
        .r2
        .presign_put(&key, AVATAR_CONTENT_TYPE)
        .await
        .map_err(|e| AppError::Internal(format!("presign failed: {e}")))?;
    Ok(Json(PresignResponse {
        upload_url: presigned.upload_url,
        public_url: presigned.public_url,
        content_type: AVATAR_CONTENT_TYPE,
        key,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    pub url: String,
}

#[tracing::instrument(skip_all, fields(user_id = %claims.sub))]
pub async fn commit(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
    Json(body): Json<CommitRequest>,
) -> Result<Json<User>, AppError> {
    User::update_by_id(claims.sub)
        .avatar_url(body.url)
        .exec(&mut state.db)
        .await?;
    let fresh = User::filter_by_id(claims.sub).get(&mut state.db).await?;
    Ok(Json(fresh))
}

#[tracing::instrument(skip_all, fields(user_id = %claims.sub))]
pub async fn delete(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<User>, AppError> {
    let user = User::filter_by_id(claims.sub).get(&mut state.db).await?;
    if let Some(url) = &user.avatar_url
        && let Some(key) = key_from_public_url(url, &state)
    {
        state.r2.delete(&key).await;
    }
    User::update_by_id(claims.sub)
        .avatar_url(Option::<String>::None)
        .exec(&mut state.db)
        .await?;
    let fresh = User::filter_by_id(claims.sub).get(&mut state.db).await?;
    Ok(Json(fresh))
}

fn key_from_public_url(url: &str, state: &AppState) -> Option<String> {
    if let Some(local) = state.r2.local_config() {
        let prefix = format!("{}/avatars/", local.api_base.trim_end_matches('/'));
        if let Some(rest) = url.strip_prefix(&prefix) {
            return Some(rest.to_string());
        }
    }
    if let crate::r2::Backend::R2 { public_url, .. } = &state.r2 {
        let prefix = format!("{}/", public_url.as_str().trim_end_matches('/'));
        if let Some(rest) = url.strip_prefix(&prefix) {
            return Some(rest.to_string());
        }
    }
    None
}

#[derive(Deserialize)]
pub struct LocalUploadQuery {
    exp: i64,
    sig: String,
}

#[tracing::instrument(skip_all, fields(key = %key))]
pub async fn local_upload(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(q): Query<LocalUploadQuery>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let local = state
        .r2
        .local_config()
        .ok_or_else(|| AppError::not_found("local upload disabled"))?;
    if !verify_local_upload(&local.signing_key, &key, q.exp, &q.sig) {
        return Err(AppError::forbidden("invalid or expired upload token"));
    }
    let path = local.dir.join(&key);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, &body).await?;
    Ok(StatusCode::OK)
}

pub fn local_upload_router() -> Router<AppState> {
    Router::new().route("/api/_avatars/upload/{*key}", put(local_upload))
}
