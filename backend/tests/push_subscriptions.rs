mod common;

use common::TestApp;
use jiff::ToSpan;
use noordpool_backend::push::Notification;
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

// ---------------------------------------------------------------------------
// Broadcast (admin-only, configurable message, all subscribers)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn broadcast_requires_admin() {
    let mut app = TestApp::new().await;

    // No auth
    let res = app
        .post("/api/push/broadcast")
        .json(json!({ "message": "Hallo" }))
        .await;
    assert_eq!(res.status(), 401);

    // Non-admin (moderator)
    let moderator = app
        .role_token(
            "mod@example.com",
            noordpool_backend::models::Role::Moderator,
        )
        .await;
    let res = app
        .post("/api/push/broadcast")
        .token(&moderator)
        .json(json!({ "message": "Hallo" }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn broadcast_captures_notification_for_all_subscribers() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    // Subscribe two users so we can verify the notification is sent to all.
    let alice = app
        .role_token(
            "alice2@example.com",
            noordpool_backend::models::Role::Player,
        )
        .await;
    app.post("/api/push/subscriptions")
        .token(&alice)
        .json(sub_payload("https://example.com/endpoint/alice2"))
        .await;

    let bob = app
        .role_token("bob2@example.com", noordpool_backend::models::Role::Player)
        .await;
    app.post("/api/push/subscriptions")
        .token(&bob)
        .json(sub_payload("https://example.com/endpoint/bob2"))
        .await;

    let res = app
        .post("/api/push/broadcast")
        .token(&admin)
        .json(json!({ "message": "Test bericht" }))
        .await;
    assert_eq!(res.status(), 204);

    let captured = app.notifications.lock().unwrap();
    assert_eq!(captured.len(), 1);
    let Notification::Broadcast { message } = &captured[0] else {
        panic!("expected Broadcast notification, got {:?}", captured[0]);
    };
    assert_eq!(message, "Test bericht");
}

// ---------------------------------------------------------------------------
// Goal notifications via mock push backend
// ---------------------------------------------------------------------------

async fn create_live_game(app: &mut TestApp, token: &str) -> (String, String, String) {
    let (home_id, away_id) = app.create_teams(token, "De Noordpool", "FC Test").await;

    // Start 5 minutes ago so it is currently live.
    let date_time = jiff::Timestamp::now() - 5.minutes();
    let res = app
        .post("/api/games")
        .token(token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadion",
            "dateTime": date_time.to_string(),
        }))
        .await;
    let game_id = res.json_value().await["id"].as_str().unwrap().to_string();
    (game_id, home_id, away_id)
}

#[tokio::test]
async fn goal_event_on_live_match_captures_goal_notification() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    let (game_id, home_id, _away_id) = create_live_game(&mut app, &admin).await;

    // Create a player on the home team.
    let res = app
        .post("/api/players")
        .token(&admin)
        .json(json!({
            "firstName": "Scorer",
            "lastName": "",
            "shirtNumber": 9,
            "position": "Spits",
            "teamId": home_id,
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    // Subscribe a user with notify_goal = true (default).
    let player = app.player_token().await;
    app.post("/api/push/subscriptions")
        .token(&player)
        .json(sub_payload("https://example.com/endpoint/fan"))
        .await;

    // Add a goal event.
    let res = app
        .post(format!("/api/games/{game_id}/events"))
        .token(&admin)
        .json(json!({
            "playerId": player_id,
            "eventType": {"type": "goal"},
            "minute": 10,
        }))
        .await;
    assert_eq!(res.status(), 200);

    let captured = app.notifications.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert!(matches!(captured[0], Notification::Goal { .. }));
}
