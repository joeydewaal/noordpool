mod common;

use common::TestApp;
use insta::{Settings, assert_json_snapshot};
use serde_json::json;

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    settings.add_redaction(".dateTime", "[dateTime]");
    settings.add_redaction(
        ".id",
        insta::dynamic_redaction(|val, _| {
            val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
        }),
    );
    settings.add_redaction(
        ".createdAt",
        insta::dynamic_redaction(|val, _| {
            val.as_str()
                .map(|_| "[timestamp]".into())
                .unwrap_or(val.clone())
        }),
    );
    settings.add_redaction(
        "[].id",
        insta::dynamic_redaction(|val, _| {
            val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
        }),
    );
    settings.add_redaction(
        "[].createdAt",
        insta::dynamic_redaction(|val, _| {
            val.as_str()
                .map(|_| "[timestamp]".into())
                .unwrap_or(val.clone())
        }),
    );
    settings
}

#[tokio::test]
async fn list_games_empty() {
    let mut app = TestApp::new().await;
    let res = app.get("/api/games").send().await;
    assert_eq!(res.status(), 200);
    assert_json_snapshot!(res.json_value().await, @"[]");
}

#[tokio::test]
async fn create_game_requires_auth() {
    let mut app = TestApp::new().await;
    let res = app
        .post("/api/games")
        .json(json!({
            "opponent": "FC Test",
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn create_game_forbidden_for_player_role() {
    let mut app = TestApp::new().await;
    let token = app.player_token().await;

    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "opponent": "FC Test",
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn create_and_get_game() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "opponent": "FC Test",
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("create_game", body);
    });

    let game_id = body["id"].as_str().unwrap();
    let res = app.get(format!("/api/games/{game_id}")).send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    settings.bind(|| {
        assert_json_snapshot!("get_game", body);
    });
}

#[tokio::test]
async fn update_game() {
    let mut app = TestApp::new().await;
    let token = app.moderator_token().await;

    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "opponent": "FC Test",
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    let created = res.json_value().await;

    let game_id = created["id"].as_str().unwrap();
    let res = app
        .put(format!("/api/games/{game_id}"))
        .token(&token)
        .json(json!({
            "homeScore": 3,
            "awayScore": 1
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("update_game", body);
    });
}

#[tokio::test]
async fn upcoming_and_recent() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    // Create a scheduled match (future)
    app.post("/api/games")
        .token(&token)
        .json(json!({
            "opponent": "FC Future",
            "location": "Home Stadium",
            "dateTime": "2027-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;

    // Create a match in the past (automatically completed by date)
    app.post("/api/games")
        .token(&token)
        .json(json!({
            "opponent": "FC Past",
            "location": "Away Stadium",
            "dateTime": "2024-01-10T15:00:00Z",
            "homeAway": "away"
        }))
        .await;

    // Check upcoming
    let res = app.get("/api/games/upcoming").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["opponent"], "FC Future");

    // Check recent
    let res = app.get("/api/games/recent").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["opponent"], "FC Past");

    // Check limit
    let res = app.get("/api/games/upcoming?limit=0").send().await;
    assert_eq!(res.status(), 200);
    assert_json_snapshot!(res.json_value().await, @"[]");
}

#[tokio::test]
async fn get_game_not_found() {
    let mut app = TestApp::new().await;
    let res = app
        .get("/api/games/00000000-0000-0000-0000-000000000000")
        .send()
        .await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_game() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    // Create a match
    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "opponent": "FC Test",
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    let created = res.json_value().await;
    let game_id = created["id"].as_str().unwrap();

    // Delete
    let res = app
        .delete(format!("/api/games/{game_id}"))
        .token(&token)
        .send()
        .await;
    assert_eq!(res.status(), 204);

    // Verify gone
    let res = app.get(format!("/api/games/{game_id}")).send().await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_match_forbidden_for_moderator() {
    let mut app = TestApp::new().await;
    let admin_token = app.admin_token().await;
    let mod_token = app.moderator_token().await;

    // Create a match as admin
    let res = app
        .post("/api/games")
        .token(&admin_token)
        .json(json!({
            "opponent": "FC Test",
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    let created = res.json_value().await;
    let game_id = created["id"].as_str().unwrap();

    // Moderator should not be able to delete
    let res = app
        .delete(format!("/api/games/{game_id}"))
        .token(&mod_token)
        .send()
        .await;
    assert_eq!(res.status(), 403);
}
