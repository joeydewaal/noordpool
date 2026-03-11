mod common;

use insta::{Settings, assert_json_snapshot};
use serde_json::json;

use crate::common::TestApp;

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    for path in &[".id", ".playerId", "[].id", "[].playerId"] {
        settings.add_redaction(
            *path,
            insta::dynamic_redaction(|val, _| {
                val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
            }),
        );
    }
    settings
}

async fn create_player(app: &mut TestApp, token: &str, name: &str, number: i32) -> String {
    let res = app
        .post("/api/players")
        .token(token)
        .json(json!({
            "name": name,
            "shirtNumber": number,
            "position": "forward",
            "email": format!("{name}@test.be")
        }))
        .await;

    let body = res.json_value().await;
    body["id"].as_str().unwrap().to_string()
}

async fn create_game(app: &mut TestApp, token: &str) -> String {
    dbg!(&token);
    let res = app
        .post("/api/games")
        .token(token)
        .json(json!({
            "opponent": "FC Test",
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    dbg!(res.status());
    let body = res.json_value().await;

    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_and_list_events() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let game_id = create_game(&mut app, &token).await;
    let player_id = create_player(&mut app, &token, "Scorer", 9).await;

    // Create a goal event
    let res = app
        .post(format!("/api/games/{game_id}/events"))
        .token(&token)
        .json(json!({
            "playerId": player_id,
            "eventType": "goal",
            "minute": 25
        }))
        .await;

    assert_eq!(res.status(), 200);
    redact_settings()
        .bind_async(async {
            assert_json_snapshot!("create_event", res.json_value().await);
        })
        .await;

    // Create an assist event
    app.post(format!("/api/games/{game_id}/events"))
        .token(&token)
        .json(json!({
            "playerId": player_id,
            "eventType": "assist",
            "minute": 20
        }))
        .await;

    // Create a yellow card at minute 60
    app.post(format!("/api/games/{game_id}/events"))
        .token(&token)
        .json(json!({
            "playerId": player_id,
            "eventType": "yellow_card",
            "minute": 60
        }))
        .await;

    // List events — should be sorted by minute
    let res = app.get(format!("/api/games/{game_id}/events")).send().await;

    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["minute"], 20);
    assert_eq!(arr[1]["minute"], 25);
    assert_eq!(arr[2]["minute"], 60);
}

#[tokio::test]
async fn delete_event() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let game_id = create_game(&mut app, &token).await;
    let player_id = create_player(&mut app, &token, "Scorer", 9).await;

    let res = app
        .post(format!("/api/games/{game_id}/events"))
        .token(&token)
        .json(json!({
            "playerId": player_id,
            "eventType": "goal",
            "minute": 10
        }))
        .await;

    let json = res.json_value().await;
    let event_id = json["id"].as_str().unwrap();

    // Delete
    let res = app
        .delete(format!("/api/games/{game_id}/events/{event_id}"))
        .token(&token)
        .send()
        .await;

    assert_eq!(res.status(), 204);

    // Verify gone
    let res = app.get(format!("/api/games/{game_id}/events")).send().await;

    assert_json_snapshot!(res.json_value().await, @"[]");
}

#[tokio::test]
async fn create_event_requires_auth() {
    let mut test_app = TestApp::new().await;

    let token = test_app.admin_token().await;
    let match_id = create_game(&mut test_app, &token).await;

    let res = test_app
        .post(format!("/api/games/{match_id}/events"))
        .json(json!({
            "playerId": "00000000-0000-0000-0000-000000000000",
            "eventType": "goal",
            "minute": 10
        }))
        .await;

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn list_events_empty() {
    let mut test_app = TestApp::new().await;
    let token = test_app.admin_token().await;
    let game_id = create_game(&mut test_app, &token).await;

    let res = test_app
        .get(format!("/api/games/{game_id}/events"))
        .send()
        .await;
    assert_eq!(res.status(), 200);
    assert_json_snapshot!(res.json_value().await, @"[]");
}
