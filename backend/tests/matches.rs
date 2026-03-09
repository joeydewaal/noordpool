mod common;

use axum::body::Body;
use common::{
    auth_json_request, call, get_admin_token, get_moderator_token, get_token, json_request,
    request, setup,
};
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
async fn list_matches_empty() {
    let (mut app, _) = setup().await;
    let (status, body) = call(
        &mut app,
        request("GET", "/api/matches").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    assert_json_snapshot!(body, @"[]");
}

#[tokio::test]
async fn create_match_requires_auth() {
    let (mut app, _) = setup().await;
    let (status, _) = call(
        &mut app,
        json_request("POST", "/api/matches")
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
    assert_eq!(status, 401);
}

#[tokio::test]
async fn create_match_forbidden_for_player_role() {
    let (mut app, _) = setup().await;
    let token = get_token(&mut app).await;

    let (status, _) = call(
        &mut app,
        auth_json_request("POST", "/api/matches", &token)
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
    assert_eq!(status, 403);
}

#[tokio::test]
async fn create_and_get_match() {
    let (mut app, state) = setup().await;
    let token = get_admin_token(&mut app, &state).await;

    let (status, body) = call(
        &mut app,
        auth_json_request("POST", "/api/matches", &token)
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
    assert_eq!(status, 200);
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("create_match", body);
    });

    let match_id = body["id"].as_str().unwrap();
    let (status, body) = call(
        &mut app,
        request("GET", &format!("/api/matches/{match_id}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    settings.bind(|| {
        assert_json_snapshot!("get_match", body);
    });
}

#[tokio::test]
async fn update_match() {
    let (mut app, state) = setup().await;
    let token = get_moderator_token(&mut app, &state).await;

    let (_, created) = call(
        &mut app,
        auth_json_request("POST", "/api/matches", &token)
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

    let match_id = created["id"].as_str().unwrap();
    let (status, body) = call(
        &mut app,
        auth_json_request("PUT", &format!("/api/matches/{match_id}"), &token)
            .body(Body::from(
                serde_json::json!({
                    "status": "completed",
                    "homeScore": 3,
                    "awayScore": 1
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("update_match", body);
    });
}

#[tokio::test]
async fn upcoming_and_recent() {
    let (mut app, state) = setup().await;
    let token = get_admin_token(&mut app, &state).await;

    // Create a scheduled match (future)
    call(
        &mut app,
        auth_json_request("POST", "/api/matches", &token)
            .body(Body::from(
                serde_json::json!({
                    "opponent": "FC Future",
                    "location": "Home Stadium",
                    "dateTime": "2027-06-15T18:00:00Z",
                    "homeAway": "home"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;

    // Create and complete a match
    let (_, past_match) = call(
        &mut app,
        auth_json_request("POST", "/api/matches", &token)
            .body(Body::from(
                serde_json::json!({
                    "opponent": "FC Past",
                    "location": "Away Stadium",
                    "dateTime": "2026-01-10T15:00:00Z",
                    "homeAway": "away"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    let past_id = past_match["id"].as_str().unwrap();
    call(
        &mut app,
        auth_json_request("PUT", &format!("/api/matches/{past_id}"), &token)
            .body(Body::from(
                serde_json::json!({ "status": "completed", "homeScore": 2, "awayScore": 1 })
                    .to_string(),
            ))
            .unwrap(),
    )
    .await;

    // Check upcoming
    let (status, body) = call(
        &mut app,
        request("GET", "/api/matches/upcoming")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["opponent"], "FC Future");

    // Check recent
    let (status, body) = call(
        &mut app,
        request("GET", "/api/matches/recent")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["opponent"], "FC Past");

    // Check limit
    let (status, body) = call(
        &mut app,
        request("GET", "/api/matches/upcoming?limit=0")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200);
    assert_json_snapshot!(body, @"[]");
}

#[tokio::test]
async fn get_match_not_found() {
    let (mut app, _) = setup().await;
    let (status, _) = call(
        &mut app,
        request("GET", "/api/matches/00000000-0000-0000-0000-000000000000")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn delete_match() {
    let (mut app, state) = setup().await;
    let token = get_admin_token(&mut app, &state).await;

    // Create a match
    let (_, created) = call(
        &mut app,
        auth_json_request("POST", "/api/matches", &token)
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
    let match_id = created["id"].as_str().unwrap();

    // Delete
    let (status, _) = call(
        &mut app,
        auth_json_request("DELETE", &format!("/api/matches/{match_id}"), &token)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 204);

    // Verify gone
    let (status, _) = call(
        &mut app,
        request("GET", &format!("/api/matches/{match_id}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn delete_match_forbidden_for_moderator() {
    let (mut app, state) = setup().await;
    let admin_token = get_admin_token(&mut app, &state).await;
    let mod_token = get_moderator_token(&mut app, &state).await;

    // Create a match as admin
    let (_, created) = call(
        &mut app,
        auth_json_request("POST", "/api/matches", &admin_token)
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
    let match_id = created["id"].as_str().unwrap();

    // Moderator should not be able to delete
    let (status, _) = call(
        &mut app,
        auth_json_request("DELETE", &format!("/api/matches/{match_id}"), &mod_token)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, 403);
}
