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

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use axum_security::jwt::Jwt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
    WebPushError, WebPushMessageBuilder,
};

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    games::live::ScoreSide,
    models::{Game, PushSubscription},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vapid-public-key", get(vapid_public_key))
        .route("/subscriptions", post(subscribe).delete(unsubscribe))
        .route("/subscriptions/me", get(list_mine))
        .route("/subscriptions/{id}", patch(update_prefs))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VapidKeyResponse {
    pub key: String,
}

pub async fn vapid_public_key(
    State(state): State<AppState>,
) -> Result<Json<VapidKeyResponse>, AppError> {
    let vapid = state
        .vapid
        .as_ref()
        .ok_or_else(|| AppError::Internal("Web Push not configured".into()))?;
    Ok(Json(VapidKeyResponse {
        key: vapid.public_key.clone(),
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

/// Send a "GOAL!" push notification to every subscription that has
/// `notify_goal = true`. Awaited on purpose — Lambda freezes the
/// runtime once the HTTP response goes back.
///
/// Subscriptions that come back as `EndpointNotValid` (410 Gone) or
/// `EndpointNotFound` (404) are pruned from the database.
pub async fn notify_goal(state: &AppState, game: &Game, side: Option<ScoreSide>) {
    let Some(vapid) = state.vapid.as_ref() else {
        tracing::debug!("notify_goal: VAPID not configured, skipping");
        return;
    };

    let mut db = state.db.clone();
    let subs = match PushSubscription::all()
        .filter(PushSubscription::fields().notify_goal().eq(true))
        .exec(&mut db)
        .await
    {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(error = %err, "notify_goal: failed to load subscriptions");
            return;
        }
    };

    if subs.is_empty() {
        return;
    }

    let payload = json!({
        "type": "goal",
        "gameId": game.id,
        "opponent": game.opponent,
        "homeScore": game.home_score,
        "awayScore": game.away_score,
        "side": side.map(|s| match s {
            ScoreSide::Home => "home",
            ScoreSide::Away => "away",
        }),
    });
    let payload_bytes = payload.to_string().into_bytes();

    let client = match IsahcWebPushClient::new() {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(error = %err, "notify_goal: failed to build web push client");
            return;
        }
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
                tracing::error!(error = %err, sub_id = %sub.id, "notify_goal: vapid sign failed");
                continue;
            }
        };

        let mut builder = WebPushMessageBuilder::new(&sub_info);
        builder.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);
        builder.set_vapid_signature(sig);

        let message = match builder.build() {
            Ok(m) => m,
            Err(err) => {
                tracing::error!(error = %err, sub_id = %sub.id, "notify_goal: build message failed");
                continue;
            }
        };

        match client.send(message).await {
            Ok(()) => {
                tracing::debug!(sub_id = %sub.id, "notify_goal: sent");
            }
            Err(WebPushError::EndpointNotValid(_))
            | Err(WebPushError::EndpointNotFound(_))
            | Err(WebPushError::Unauthorized(_)) => {
                tracing::info!(sub_id = %sub.id, "notify_goal: pruning expired endpoint");
                to_delete.push(sub.id);
            }
            Err(err) => {
                tracing::warn!(error = %err, sub_id = %sub.id, "notify_goal: send failed");
            }
        }
    }

    for id in to_delete {
        if let Ok(sub) = PushSubscription::get_by_id(&mut db, id).await
            && let Err(err) = sub.delete().exec(&mut db).await
        {
            tracing::warn!(error = %err, sub_id = %id, "notify_goal: prune delete failed");
        }
    }
}
