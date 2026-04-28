//! Web Push notifications (RFC 8030 / Web Push protocol).
//!
//! Generate VAPID keys with:
//!   npx web-push generate-vapid-keys
//! `VAPID_SUBJECT` must be a `mailto:` or `https:` URI per RFC 8292.
//!
//! Routes mounted at `/api/push`:
//!   GET    /vapid-public-key      → public, returns the server's VAPID pub key
//!   POST   /subscriptions         → upsert (by endpoint) for current user
//!   DELETE /subscriptions         → delete one of current user's subs by endpoint
//!   GET    /subscriptions/me      → list current user's subs
//!   PATCH  /subscriptions/{id}    → update notify_* prefs (must own row)
//!   POST   /broadcast             → admin-only, send a message to all subscribers

use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use axum_security::{jwt::Jwt, rbac::requires};
use serde::{Deserialize, Serialize};
use serde_json::json;
use toasty::Db;
use uuid::Uuid;
use web_push::{
    ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushMessage, WebPushMessageBuilder,
};

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    games::live::ScoreSide,
    models::{Game, PushSubscription, Role},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vapid-public-key", get(vapid_public_key))
        .route("/subscriptions", post(subscribe).delete(unsubscribe))
        .route("/subscriptions/me", get(list_mine))
        .route("/subscriptions/{id}", patch(update_prefs))
        .route("/broadcast", post(broadcast))
}

pub struct VapidConfig {
    /// Base64url-encoded P-256 public key (raw, uncompressed, 65 bytes).
    pub public_key: String,
    /// Base64url-encoded P-256 private key (32 bytes).
    pub private_key: String,
    /// `mailto:` or `https:` URI per RFC 8292.
    pub subject: String,
}

#[derive(Debug, Clone)]
pub enum Notification {
    Goal {
        game_id: Uuid,
        home_team_id: Uuid,
        away_team_id: Uuid,
        home_team_name: String,
        away_team_name: String,
        home_score: i32,
        away_score: i32,
        side: Option<ScoreSide>,
    },
    Broadcast {
        message: String,
    },
}

#[derive(Clone)]
pub enum PushBackend {
    /// Sends real Web Push messages over HTTP/2 with rustls.
    Live {
        db: Db,
        vapid: Arc<VapidConfig>,
        http: reqwest::Client,
    },
    /// Captures notifications in memory for testing.
    Mock(Arc<Mutex<Vec<Notification>>>),
    /// No-op — VAPID not configured.
    Disabled,
}

impl PushBackend {
    pub fn new_live(db: Db, vapid: Arc<VapidConfig>) -> Self {
        let http = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        PushBackend::Live { db, vapid, http }
    }

    pub fn new_mock() -> (Self, Arc<Mutex<Vec<Notification>>>) {
        let store = Arc::new(Mutex::new(Vec::new()));
        (PushBackend::Mock(Arc::clone(&store)), store)
    }

    /// Returns the VAPID public key if this backend is configured.
    pub fn vapid_public_key(&self) -> Option<&str> {
        match self {
            PushBackend::Live { vapid, .. } => Some(&vapid.public_key),
            _ => None,
        }
    }

    pub async fn notify(&self, notification: Notification) {
        match self {
            PushBackend::Live { db, vapid, http } => {
                send_real(notification, db, vapid, http).await;
            }
            PushBackend::Mock(store) => {
                store.lock().unwrap().push(notification);
            }
            PushBackend::Disabled => {}
        }
    }

    pub async fn notify_goal(
        &self,
        game: &Game,
        side: Option<ScoreSide>,
        home_team_name: &str,
        away_team_name: &str,
    ) {
        self.notify(Notification::Goal {
            game_id: game.id,
            home_team_id: game.home_team_id,
            away_team_id: game.away_team_id,
            home_team_name: home_team_name.to_string(),
            away_team_name: away_team_name.to_string(),
            home_score: game.home_score,
            away_score: game.away_score,
            side,
        })
        .await;
    }
}

async fn send_real(
    notification: Notification,
    db: &Db,
    vapid: &VapidConfig,
    http: &reqwest::Client,
) {
    let mut db = db.clone();

    let notify_goal_only = matches!(notification, Notification::Goal { .. });
    let subs = if notify_goal_only {
        PushSubscription::all()
            .filter(PushSubscription::fields().notify_goal().eq(true))
            .exec(&mut db)
            .await
    } else {
        PushSubscription::all().exec(&mut db).await
    };
    let subs = match subs {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(error = %err, "push: failed to load subscriptions");
            return;
        }
    };

    if subs.is_empty() {
        return;
    }

    let payload_bytes = match &notification {
        Notification::Goal {
            game_id,
            home_team_id,
            away_team_id,
            home_team_name,
            away_team_name,
            home_score,
            away_score,
            side,
        } => json!({
            "type": "goal",
            "gameId": game_id,
            "homeTeam": { "id": home_team_id, "name": home_team_name },
            "awayTeam": { "id": away_team_id, "name": away_team_name },
            "homeScore": home_score,
            "awayScore": away_score,
            "side": match side {
                Some(ScoreSide::Home) => Some("home"),
                Some(ScoreSide::Away) => Some("away"),
                None => None,
            },
        })
        .to_string()
        .into_bytes(),

        Notification::Broadcast { message } => json!({ "type": "broadcast", "message": message })
            .to_string()
            .into_bytes(),
    };

    let mut to_delete: Vec<Uuid> = Vec::new();

    for sub in &subs {
        let sub_info = SubscriptionInfo::new(&sub.endpoint, &sub.p256dh, &sub.auth);

        let sig = match VapidSignatureBuilder::from_base64(&vapid.private_key, &sub_info).and_then(
            |mut b| {
                b.add_claim("sub", vapid.subject.clone());
                b.build()
            },
        ) {
            Ok(s) => s,
            Err(err) => {
                tracing::error!(error = %err, sub_id = %sub.id, "push: vapid sign failed");
                continue;
            }
        };

        let mut builder = WebPushMessageBuilder::new(&sub_info);
        builder.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);
        builder.set_vapid_signature(sig);

        let message = match builder.build() {
            Ok(m) => m,
            Err(err) => {
                tracing::error!(error = %err, sub_id = %sub.id, "push: build message failed");
                continue;
            }
        };

        match send_message(http, &message).await {
            SendOutcome::Ok => tracing::debug!(sub_id = %sub.id, "push: sent"),
            SendOutcome::Expired => {
                tracing::info!(sub_id = %sub.id, "push: pruning expired endpoint");
                to_delete.push(sub.id);
            }
            SendOutcome::Failed(err) => {
                tracing::warn!(error = %err, sub_id = %sub.id, "push: send failed");
            }
        }
    }

    for id in to_delete {
        if let Ok(sub) = PushSubscription::get_by_id(&mut db, id).await
            && let Err(err) = sub.delete().exec(&mut db).await
        {
            tracing::warn!(error = %err, sub_id = %id, "push: prune delete failed");
        }
    }
}

enum SendOutcome {
    Ok,
    Expired,
    Failed(String),
}

/// POST a `WebPushMessage` to its subscription endpoint over HTTP/2 + rustls.
/// Endpoint, headers and encrypted body all come from the web-push builder.
async fn send_message(http: &reqwest::Client, message: &WebPushMessage) -> SendOutcome {
    let mut req = http
        .post(message.endpoint.to_string())
        .header("TTL", message.ttl.to_string());

    if let Some(topic) = &message.topic {
        req = req.header("Topic", topic);
    }
    if let Some(urgency) = &message.urgency {
        req = req.header("Urgency", urgency_str(urgency));
    }

    if let Some(payload) = &message.payload {
        for (name, value) in &payload.crypto_headers {
            req = req.header(*name, value);
        }
        req = req
            .header("Content-Type", "application/octet-stream")
            .header("Content-Encoding", encoding_str(&payload.content_encoding))
            .body(payload.content.clone());
    } else {
        req = req.header("Content-Length", "0");
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(err) => return SendOutcome::Failed(err.to_string()),
    };
    let status = resp.status();
    if status.is_success() {
        return SendOutcome::Ok;
    }
    // 404 / 410 mean the subscription is gone; 401 / 403 mean we're unauthorized
    // for it (often = stale endpoint after key rotation). Prune in either case.
    if status == reqwest::StatusCode::NOT_FOUND
        || status == reqwest::StatusCode::GONE
        || status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
    {
        return SendOutcome::Expired;
    }
    SendOutcome::Failed(format!("HTTP {status}"))
}

fn urgency_str(u: &web_push::Urgency) -> &'static str {
    match u {
        web_push::Urgency::VeryLow => "very-low",
        web_push::Urgency::Low => "low",
        web_push::Urgency::Normal => "normal",
        web_push::Urgency::High => "high",
    }
}

fn encoding_str(e: &ContentEncoding) -> &'static str {
    match e {
        ContentEncoding::Aes128Gcm => "aes128gcm",
        ContentEncoding::AesGcm => "aesgcm",
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VapidKeyResponse {
    pub key: String,
}

pub async fn vapid_public_key(
    State(state): State<AppState>,
) -> Result<Json<VapidKeyResponse>, AppError> {
    let key = state
        .push
        .vapid_public_key()
        .ok_or_else(|| AppError::Internal("Web Push not configured".into()))?;
    Ok(Json(VapidKeyResponse {
        key: key.to_string(),
    }))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    #[serde(default)]
    pub notify_goal: Option<bool>,
}

/// Upsert: if a row already exists for this endpoint we update its keys
/// and reassign it to the current user (handles re-subscribe / device
/// handoff). Otherwise we insert a new row.
#[tracing::instrument(skip(state, body))]
pub async fn subscribe(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
    Json(body): Json<SubscribeRequest>,
) -> Result<Json<PushSubscription>, AppError> {
    let existing = PushSubscription::filter_by_endpoint(body.endpoint.clone())
        .first()
        .exec(&mut state.db)
        .await?;

    let notify_goal = body.notify_goal.unwrap_or(true);

    let sub = if let Some(mut sub) = existing {
        let mut update = sub.update();
        update.set_user_id(claims.sub);
        update.set_p256dh(body.p256dh);
        update.set_auth(body.auth);
        update.set_notify_goal(notify_goal);
        update.exec(&mut state.db).await?;
        PushSubscription::get_by_id(&mut state.db, sub.id).await?
    } else {
        toasty::create!(PushSubscription {
            user_id: claims.sub,
            endpoint: body.endpoint,
            p256dh: body.p256dh,
            auth: body.auth,
            notify_goal: notify_goal,
        })
        .exec(&mut state.db)
        .await?
    };

    Ok(Json(sub))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

#[tracing::instrument(skip(state, body))]
pub async fn unsubscribe(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
    Json(body): Json<UnsubscribeRequest>,
) -> Result<StatusCode, AppError> {
    let Some(sub) = PushSubscription::filter_by_endpoint(body.endpoint)
        .first()
        .exec(&mut state.db)
        .await?
    else {
        return Ok(StatusCode::NO_CONTENT);
    };

    if sub.user_id != claims.sub {
        return Err(AppError::forbidden("Not your subscription"));
    }

    sub.delete().exec(&mut state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state))]
pub async fn list_mine(
    State(mut state): State<AppState>,
    Jwt(claims): Jwt<Claims>,
) -> Result<Json<Vec<PushSubscription>>, AppError> {
    let subs = PushSubscription::filter_by_user_id(claims.sub)
        .exec(&mut state.db)
        .await?;
    Ok(Json(subs))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePrefsRequest {
    pub notify_goal: Option<bool>,
}

#[tracing::instrument(skip(state, body))]
pub async fn update_prefs(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Jwt(claims): Jwt<Claims>,
    Json(body): Json<UpdatePrefsRequest>,
) -> Result<Json<PushSubscription>, AppError> {
    let mut sub = PushSubscription::get_by_id(&mut state.db, id).await?;
    if sub.user_id != claims.sub {
        return Err(AppError::forbidden("Not your subscription"));
    }

    let mut update = sub.update();
    if let Some(notify_goal) = body.notify_goal {
        update.set_notify_goal(notify_goal);
    }
    update.exec(&mut state.db).await?;

    let fresh = PushSubscription::get_by_id(&mut state.db, id).await?;
    Ok(Json(fresh))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastRequest {
    pub message: String,
}

/// Admin-only: send a push notification with a configurable message to all subscribers.
#[requires(Role::Admin)]
#[tracing::instrument(skip(state, body))]
pub async fn broadcast(
    State(state): State<AppState>,
    Json(body): Json<BroadcastRequest>,
) -> Result<StatusCode, AppError> {
    state
        .push
        .notify(Notification::Broadcast {
            message: body.message,
        })
        .await;
    Ok(StatusCode::NO_CONTENT)
}
