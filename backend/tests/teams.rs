mod common;

use common::TestApp;

#[tokio::test]
async fn list_teams_is_public() {
    let mut app = TestApp::new().await;

    let res = app.get("/api/teams").send().await;
    assert_eq!(res.status(), 200);

    let teams: Vec<serde_json::Value> = res.json().await;
    assert!(teams.is_empty());
}

#[tokio::test]
async fn admin_can_create_team() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let res = app
        .post("/api/teams")
        .token(&token)
        .json(serde_json::json!({ "name": "FC Test" }))
        .await;
    assert_eq!(res.status(), 200);

    let body = res.json_value().await;
    assert_eq!(body["name"].as_str().unwrap(), "FC Test");
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn create_team_requires_admin() {
    let mut app = TestApp::new().await;
    let token = app.player_token().await;

    let res = app
        .post("/api/teams")
        .token(&token)
        .json(serde_json::json!({ "name": "FC Test" }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn create_team_duplicate_returns_409() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    app.post("/api/teams")
        .token(&token)
        .json(serde_json::json!({ "name": "FC Test" }))
        .await;

    let res = app
        .post("/api/teams")
        .token(&token)
        .json(serde_json::json!({ "name": "FC Test" }))
        .await;
    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn list_teams_returns_created_teams() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    app.post("/api/teams")
        .token(&token)
        .json(serde_json::json!({ "name": "Bravo" }))
        .await;
    app.post("/api/teams")
        .token(&token)
        .json(serde_json::json!({ "name": "Alpha" }))
        .await;

    let res = app.get("/api/teams").send().await;
    assert_eq!(res.status(), 200);

    let teams: Vec<serde_json::Value> = res.json().await;
    assert_eq!(teams.len(), 2);
    // Sorted by name
    assert_eq!(teams[0]["name"].as_str().unwrap(), "Alpha");
    assert_eq!(teams[1]["name"].as_str().unwrap(), "Bravo");
}
