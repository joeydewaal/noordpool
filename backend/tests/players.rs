mod common;

use axum::body::Body;
use common::{
    auth_json_request, call, get_admin_token, get_moderator_token, get_token, json_request,
    request, setup,
};
use insta::{Settings, assert_json_snapshot};

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    settings.add_redaction(".id", "[uuid]");
    settings.add_redaction(".createdAt", "[createdAt]");
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
async fn create_player_forbidden_for_player_role() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;

    let (status, _) = call(
        &mut app,
        auth_json_request("POST", "/api/players", &token)
            .body(Body::from(
                serde_json::json!({
                    "name": "Jan de Boer",
                    "email": "jan.de.boer@example.com",
                    "shirtNumber": 10,
                    "position": "midfielder"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, 403);
}

#[tokio::test]
async fn create_and_get_player() {
    let (mut app, state) = setup().await;
    let token = get_admin_token(&mut app, &state).await;

    let (status, body) = call(
        &mut app,
        auth_json_request("POST", "/api/players", &token)
            .body(Body::from(
                serde_json::json!({
                    "name": "Jan de Boer",
                    "email": "jan.de.boer@example.com",
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
        assert_json_snapshot!("create_player", body);
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
        assert_json_snapshot!("get_player", body);
    });
}

#[tokio::test]
async fn update_player() {
    let (mut app, state) = setup().await;
    let token = get_moderator_token(&mut app, &state).await;

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
        assert_json_snapshot!("update_player", body);
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
    let (mut app, state) = setup().await;
    let token = get_admin_token(&mut app, &state).await;

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
        assert_json_snapshot!("player_stats_empty", body);
    });
}

#[tokio::test]
async fn delete_player_soft_deletes() {
    let (mut app, state) = setup().await;
    let token = get_admin_token(&mut app, &state).await;

    // Create a player
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

    // Delete (soft)
    let (status, _) = call(
        &mut app,
        auth_json_request("DELETE", &format!("/api/players/{player_id}"), &token)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 204);

    // Player should not appear in list (only active players)
    let (_, body) = call(
        &mut app,
        request("GET", "/api/players").body(Body::empty()).unwrap(),
    )
    .await;
    assert_json_snapshot!(body, @"[]");

    // But get by ID should still work
    let (status, body) = call(
        &mut app,
        request("GET", &format!("/api/players/{player_id}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body["active"], false);
}

#[tokio::test]
async fn delete_player_forbidden_for_moderator() {
    let (mut app, state) = setup().await;
    let admin_token = get_admin_token(&mut app, &state).await;
    let mod_token = get_moderator_token(&mut app, &state).await;

    // Create a player as admin
    let (_, created) = call(
        &mut app,
        auth_json_request("POST", "/api/players", &admin_token)
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

    // Moderator should not be able to delete
    let (status, _) = call(
        &mut app,
        auth_json_request("DELETE", &format!("/api/players/{player_id}"), &mod_token)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 403);
}
