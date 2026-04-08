mod common;

use common::TestApp;
use jiff::{Timestamp, ToSpan};
use serde_json::json;

/// Helper: create a game whose date_time is `offset` from now.
async fn create_game(app: &mut TestApp, token: &str, offset: jiff::Span) -> String {
    let date_time = Timestamp::now() + offset;
    let res = app
        .post("/api/games")
        .token(token)
        .json(json!({
            "opponent": "FC Live",
            "location": "Stadium",
            "dateTime": date_time.to_string(),
            "homeAway": "home"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn poll_live_returns_status_and_etag() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    // kick-off was 5 minutes ago → inside the live window
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app.get(format!("/api/games/{id}/live")).send().await;
    assert_eq!(res.status(), 200);
    let etag = res
        .inner_headers()
        .get("etag")
        .expect("etag header")
        .to_str()
        .unwrap()
        .to_string();
    assert!(etag.starts_with("W/\""));
    let body = res.json_value().await;
    assert_eq!(body["status"], "live");
    assert_eq!(body["version"], 0);
}

#[tokio::test]
async fn poll_live_returns_304_when_etag_matches() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app.get(format!("/api/games/{id}/live")).send().await;
    let etag = res
        .inner_headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let res = app
        .get(format!("/api/games/{id}/live"))
        .header("if-none-match", &etag)
        .send()
        .await;
    assert_eq!(res.status(), 304);
}

#[tokio::test]
async fn poll_live_marks_scheduled_game_as_scheduled() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, 168.hours()).await;

    let res = app.get(format!("/api/games/{id}/live")).send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["status"], "scheduled");
}

#[tokio::test]
async fn adjust_opponent_score_when_not_live_returns_conflict() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, 168.hours()).await;

    let res = app
        .post(format!("/api/games/{id}/live/opponent_score"))
        .token(&token)
        .json(json!({ "delta": 1 }))
        .await;
    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn adjust_opponent_score_requires_admin_or_moderator() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;
    let id = create_game(&mut app, &admin, -5.minutes()).await;

    // No auth
    let res = app
        .post(format!("/api/games/{id}/live/opponent_score"))
        .json(json!({ "delta": 1 }))
        .await;
    assert_eq!(res.status(), 401);

    // Player role
    let player = app.player_token().await;
    let res = app
        .post(format!("/api/games/{id}/live/opponent_score"))
        .token(&player)
        .json(json!({ "delta": 1 }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn adjust_opponent_score_increments_and_bumps_version() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    // create_game uses homeAway: home, so opponent is the away side.
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app
        .post(format!("/api/games/{id}/live/opponent_score"))
        .token(&token)
        .json(json!({ "delta": 1 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["homeScore"], 0);
    assert_eq!(body["awayScore"], 1);
    assert_eq!(body["version"], 1);

    let res = app
        .post(format!("/api/games/{id}/live/opponent_score"))
        .token(&token)
        .json(json!({ "delta": 1 }))
        .await;
    let body = res.json_value().await;
    assert_eq!(body["homeScore"], 0);
    assert_eq!(body["awayScore"], 2);
    assert_eq!(body["version"], 2);
}

#[tokio::test]
async fn adjust_opponent_score_minus_one_clamps_at_zero() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app
        .post(format!("/api/games/{id}/live/opponent_score"))
        .token(&token)
        .json(json!({ "delta": -1 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["awayScore"], 0);
}

#[tokio::test]
async fn adjust_opponent_score_rejects_invalid_delta() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app
        .post(format!("/api/games/{id}/live/opponent_score"))
        .token(&token)
        .json(json!({ "delta": 2 }))
        .await;
    assert_eq!(res.status(), 409);
}
