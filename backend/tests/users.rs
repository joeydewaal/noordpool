mod common;

use common::TestApp;
use serde_json::json;

async fn register_user(app: &mut TestApp, email: &str) -> (String, String) {
    let res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Test",
            "lastName": "User",
            "email": email,
            "password": "password123"
        }))
        .await;
    let body = res.json_value().await;
    let id = body["user"]["id"].as_str().unwrap().to_string();
    let token = body["token"].as_str().unwrap().to_string();
    (id, token)
}

#[tokio::test]
async fn admin_can_promote_user_to_moderator() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;
    let (target_id, _) = register_user(&mut app, "promoteme@example.com").await;

    let res = app
        .patch(format!("/api/users/{target_id}"))
        .token(&admin)
        .json(json!({ "isModerator": true }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["isModerator"], true);
    let roles = body["roles"].as_array().unwrap();
    assert!(roles.iter().any(|r| r == "moderator"));
}

#[tokio::test]
async fn admin_can_demote_moderator() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;
    let (target_id, _) = register_user(&mut app, "demoteme@example.com").await;

    // Promote first
    app.patch(format!("/api/users/{target_id}"))
        .token(&admin)
        .json(json!({ "isModerator": true }))
        .await;

    // Demote
    let res = app
        .patch(format!("/api/users/{target_id}"))
        .token(&admin)
        .json(json!({ "isModerator": false }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["isModerator"], false);
    let roles = body["roles"].as_array().unwrap();
    assert!(!roles.iter().any(|r| r == "moderator"));
}

#[tokio::test]
async fn promote_user_requires_admin() {
    let mut app = TestApp::new().await;
    let player_token = app.player_token().await;
    let (target_id, _) = register_user(&mut app, "victim@example.com").await;

    let res = app
        .patch(format!("/api/users/{target_id}"))
        .token(&player_token)
        .json(json!({ "isModerator": true }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn promote_user_requires_authentication() {
    let mut app = TestApp::new().await;
    let (target_id, _) = register_user(&mut app, "victim@example.com").await;

    let res = app
        .patch(format!("/api/users/{target_id}"))
        .json(json!({ "isModerator": true }))
        .await;
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn promote_unknown_user_returns_404() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    let res = app
        .patch("/api/users/00000000-0000-0000-0000-000000000000")
        .token(&admin)
        .json(json!({ "isModerator": true }))
        .await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn admin_can_list_users() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;
    register_user(&mut app, "alice@example.com").await;
    register_user(&mut app, "bob@example.com").await;

    let res = app.get("/api/users").token(&admin).send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let users = body.as_array().unwrap();
    // At least the 2 we registered plus the admin seed user
    assert!(users.len() >= 3);
    let emails: Vec<&str> = users.iter().map(|u| u["email"].as_str().unwrap()).collect();
    assert!(emails.contains(&"alice@example.com"));
    assert!(emails.contains(&"bob@example.com"));
}

#[tokio::test]
async fn list_users_requires_admin() {
    let mut app = TestApp::new().await;
    let player_token = app.player_token().await;

    let res = app.get("/api/users").token(&player_token).send().await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn list_users_requires_authentication() {
    let mut app = TestApp::new().await;

    let res = app.get("/api/users").send().await;
    assert_eq!(res.status(), 401);
}
