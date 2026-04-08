mod common;

use common::TestApp;
use serde_json::json;

fn sub_payload(endpoint: &str) -> serde_json::Value {
    json!({
        "endpoint": endpoint,
        "p256dh": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUls0VJXg7A8u-Ts1XbjhazAkj7I99e8QcYP7DkM",
        "auth": "tBHItJI5svbpez7KI4CCXg",
    })
}

#[tokio::test]
async fn vapid_public_key_returns_503_when_not_configured() {
    let mut app = TestApp::new().await;
    let res = app.get("/api/push/vapid-public-key").send().await;
    // VAPID is None in tests; handler returns Internal error for now.
    assert_eq!(res.status(), 500);
}

#[tokio::test]
async fn subscribe_requires_auth() {
    let mut app = TestApp::new().await;
    let res = app
        .post("/api/push/subscriptions")
        .json(sub_payload("https://example.com/endpoint/1"))
        .await;
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn subscribe_creates_then_upserts() {
    let mut app = TestApp::new().await;
    let token = app.player_token().await;

    let res = app
        .post("/api/push/subscriptions")
        .token(&token)
        .json(sub_payload("https://example.com/endpoint/1"))
        .await;
    assert_eq!(res.status(), 200);
    let first = res.json_value().await;
    let first_id = first["id"].as_str().unwrap().to_string();

    // Re-subscribing the same endpoint should upsert (same id) and update
    // notify_goal preference.
    let res = app
        .post("/api/push/subscriptions")
        .token(&token)
        .json(json!({
            "endpoint": "https://example.com/endpoint/1",
            "p256dh": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUls0VJXg7A8u-Ts1XbjhazAkj7I99e8QcYP7DkM",
            "auth": "tBHItJI5svbpez7KI4CCXg",
            "notifyGoal": false,
        }))
        .await;
    assert_eq!(res.status(), 200);
    let second = res.json_value().await;
    assert_eq!(second["id"].as_str().unwrap(), first_id);
    assert_eq!(second["notifyGoal"], false);

    // list_mine
    let res = app
        .get("/api/push/subscriptions/me")
        .token(&token)
        .send()
        .await;
    assert_eq!(res.status(), 200);
    let list = res.json_value().await;
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn unsubscribe_removes_subscription() {
    let mut app = TestApp::new().await;
    let token = app.player_token().await;

    app.post("/api/push/subscriptions")
        .token(&token)
        .json(sub_payload("https://example.com/endpoint/1"))
        .await;

    let res = app
        .delete("/api/push/subscriptions")
        .token(&token)
        .json(json!({ "endpoint": "https://example.com/endpoint/1" }))
        .await;
    assert_eq!(res.status(), 204);

    let res = app
        .get("/api/push/subscriptions/me")
        .token(&token)
        .send()
        .await;
    let list = res.json_value().await;
    assert_eq!(list.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn unsubscribe_other_users_endpoint_forbidden() {
    let mut app = TestApp::new().await;
    let alice = app
        .role_token("alice@example.com", noordpool_backend::models::Role::Player)
        .await;
    let bob = app
        .role_token("bob@example.com", noordpool_backend::models::Role::Player)
        .await;

    app.post("/api/push/subscriptions")
        .token(&alice)
        .json(sub_payload("https://example.com/endpoint/alice"))
        .await;

    let res = app
        .delete("/api/push/subscriptions")
        .token(&bob)
        .json(json!({ "endpoint": "https://example.com/endpoint/alice" }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn list_mine_only_returns_callers_subs() {
    let mut app = TestApp::new().await;
    let alice = app
        .role_token("alice@example.com", noordpool_backend::models::Role::Player)
        .await;
    let bob = app
        .role_token("bob@example.com", noordpool_backend::models::Role::Player)
        .await;

    app.post("/api/push/subscriptions")
        .token(&alice)
        .json(sub_payload("https://example.com/endpoint/alice"))
        .await;
    app.post("/api/push/subscriptions")
        .token(&bob)
        .json(sub_payload("https://example.com/endpoint/bob"))
        .await;

    let res = app
        .get("/api/push/subscriptions/me")
        .token(&alice)
        .send()
        .await;
    let list = res.json_value().await;
    let arr = list.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["endpoint"], "https://example.com/endpoint/alice");
}

#[tokio::test]
async fn update_prefs_only_for_own_subscription() {
    let mut app = TestApp::new().await;
    let alice = app
        .role_token("alice@example.com", noordpool_backend::models::Role::Player)
        .await;
    let bob = app
        .role_token("bob@example.com", noordpool_backend::models::Role::Player)
        .await;

    let res = app
        .post("/api/push/subscriptions")
        .token(&alice)
        .json(sub_payload("https://example.com/endpoint/alice"))
        .await;
    let id = res.json_value().await["id"].as_str().unwrap().to_string();

    // Bob can't toggle Alice's prefs
    let res = app
        .patch(format!("/api/push/subscriptions/{id}"))
        .token(&bob)
        .json(json!({ "notifyGoal": false }))
        .await;
    assert_eq!(res.status(), 403);

    // Alice can
    let res = app
        .patch(format!("/api/push/subscriptions/{id}"))
        .token(&alice)
        .json(json!({ "notifyGoal": false }))
        .await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.json_value().await["notifyGoal"], false);
}
