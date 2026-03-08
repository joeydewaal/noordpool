mod common;

use axum::body::Body;
use common::{auth_json_request, call, get_token, json_request, request, setup};
use insta::{Settings, assert_json_snapshot};

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    settings.add_redaction(
        ".id",
        insta::dynamic_redaction(|val, _| {
            val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
        }),
    );
    settings.add_redaction(
        ".userId",
        insta::dynamic_redaction(|val, _| {
            val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
        }),
    );
    settings.add_redaction(
        ".playerId",
        insta::dynamic_redaction(|val, _| {
            val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
        }),
    );
    settings.add_redaction(
        "[].id",
        insta::dynamic_redaction(|val, _| {
            val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
        }),
    );
    settings
}

#[tokio::test]
async fn list_players_empty() {
    let (mut app, _) = setup().await;
    let (status, body) = call(
        &mut app,
        request("GET", "/api/players").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    assert_json_snapshot!(body, @"[]");
}

#[tokio::test]
async fn create_player_requires_auth() {
    let (mut app, _) = setup().await;
    let (status, _body) = call(
        &mut app,
        json_request("POST", "/api/players")
            .body(Body::from(
                serde_json::json!({
                    "name": "Jan de Boer",
                    "shirtNumber": 10,
                    "position": "midfielder"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, 401);
}

#[tokio::test]
async fn create_and_get_player() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;

    let (status, body) = call(
        &mut app,
        auth_json_request("POST", "/api/players", &token)
            .body(Body::from(
                serde_json::json!({
                    "name": "Jan de Boer",
                    "shirtNumber": 10,
                    "position": "midfielder"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("create_player", body, {
            ".id" => "[uuid]",
        });
    });

    let player_id = body["id"].as_str().unwrap();
    let (status, body) = call(
        &mut app,
        request("GET", &format!("/api/players/{player_id}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    settings.bind(|| {
        assert_json_snapshot!("get_player", body, {
            ".id" => "[uuid]",
        });
    });
}

#[tokio::test]
async fn update_player() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;

    let (_, created) = call(
        &mut app,
        auth_json_request("POST", "/api/players", &token)
            .body(Body::from(
                serde_json::json!({
                    "name": "Jan de Boer",
                    "shirtNumber": 10,
                    "position": "midfielder"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;

    let player_id = created["id"].as_str().unwrap();
    let (status, body) = call(
        &mut app,
        auth_json_request("PUT", &format!("/api/players/{player_id}"), &token)
            .body(Body::from(
                serde_json::json!({
                    "shirtNumber": 7,
                    "position": "forward"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("update_player", body, {
            ".id" => "[uuid]",
        });
    });
}

#[tokio::test]
async fn get_player_not_found() {
    let (mut app, _) = setup().await;
    let (status, _) = call(
        &mut app,
        request("GET", "/api/players/00000000-0000-0000-0000-000000000000")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn player_stats_empty() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;

    let (_, created) = call(
        &mut app,
        auth_json_request("POST", "/api/players", &token)
            .body(Body::from(
                serde_json::json!({
                    "name": "Jan de Boer",
                    "shirtNumber": 10,
                    "position": "midfielder"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;

    let player_id = created["id"].as_str().unwrap();
    let (status, body) = call(
        &mut app,
        request("GET", &format!("/api/players/{player_id}/stats"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("player_stats_empty", body, {
            ".playerId" => "[uuid]",
        });
    });
}
