mod common;

use axum::body::Body;
use common::{auth_json_request, call, get_token, request, setup};
use insta::{Settings, assert_json_snapshot};

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    for path in &[
        ".id",
        ".matchId",
        ".playerId",
        "[].id",
        "[].matchId",
        "[].playerId",
    ] {
        settings.add_redaction(
            *path,
            insta::dynamic_redaction(|val, _| {
                val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
            }),
        );
    }
    settings
}

async fn create_player(app: &mut axum::Router, token: &str, name: &str, number: i32) -> String {
    let (_, body) = call(
        app,
        auth_json_request("POST", "/api/players", token)
            .body(Body::from(
                serde_json::json!({
                    "name": name,
                    "shirtNumber": number,
                    "position": "forward"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    body["id"].as_str().unwrap().to_string()
}

async fn create_match(app: &mut axum::Router, token: &str) -> String {
    let (_, body) = call(
        app,
        auth_json_request("POST", "/api/matches", token)
            .body(Body::from(
                serde_json::json!({
                    "opponent": "FC Test",
                    "location": "Stadium",
                    "dateTime": "2026-06-15T18:00:00Z",
                    "homeAway": "home"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn list_events_empty() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;
    let match_id = create_match(&mut app, &token).await;

    let (status, body) = call(
        &mut app,
        request("GET", &format!("/api/matches/{match_id}/events"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    assert_json_snapshot!(body, @"[]");
}

#[tokio::test]
async fn create_and_list_events() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;
    let match_id = create_match(&mut app, &token).await;
    let player_id = create_player(&mut app, &token, "Scorer", 9).await;

    // Create a goal event
    let (status, body) = call(
        &mut app,
        auth_json_request("POST", &format!("/api/matches/{match_id}/events"), &token)
            .body(Body::from(
                serde_json::json!({
                    "playerId": player_id,
                    "eventType": "goal",
                    "minute": 25
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("create_event", body);
    });

    // Create an assist event
    call(
        &mut app,
        auth_json_request("POST", &format!("/api/matches/{match_id}/events"), &token)
            .body(Body::from(
                serde_json::json!({
                    "playerId": player_id,
                    "eventType": "assist",
                    "minute": 20
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;

    // Create a yellow card at minute 60
    call(
        &mut app,
        auth_json_request("POST", &format!("/api/matches/{match_id}/events"), &token)
            .body(Body::from(
                serde_json::json!({
                    "playerId": player_id,
                    "eventType": "yellow_card",
                    "minute": 60
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;

    // List events — should be sorted by minute
    let (status, body) = call(
        &mut app,
        request("GET", &format!("/api/matches/{match_id}/events"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["minute"], 20);
    assert_eq!(arr[1]["minute"], 25);
    assert_eq!(arr[2]["minute"], 60);
}

#[tokio::test]
async fn delete_event() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;
    let match_id = create_match(&mut app, &token).await;
    let player_id = create_player(&mut app, &token, "Scorer", 9).await;

    let (_, created) = call(
        &mut app,
        auth_json_request("POST", &format!("/api/matches/{match_id}/events"), &token)
            .body(Body::from(
                serde_json::json!({
                    "playerId": player_id,
                    "eventType": "goal",
                    "minute": 10
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    let event_id = created["id"].as_str().unwrap();

    // Delete
    let (status, _) = call(
        &mut app,
        auth_json_request(
            "DELETE",
            &format!("/api/matches/{match_id}/events/{event_id}"),
            &token,
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(status, 204);

    // Verify gone
    let (_, body) = call(
        &mut app,
        request("GET", &format!("/api/matches/{match_id}/events"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_json_snapshot!(body, @"[]");
}

#[tokio::test]
async fn create_event_requires_auth() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;
    let match_id = create_match(&mut app, &token).await;

    let (status, _) = call(
        &mut app,
        common::json_request("POST", &format!("/api/matches/{match_id}/events"))
            .body(Body::from(
                serde_json::json!({
                    "playerId": "00000000-0000-0000-0000-000000000000",
                    "eventType": "goal",
                    "minute": 10
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, 401);
}
