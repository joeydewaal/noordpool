mod common;

use common::TestApp;
use jiff::{Timestamp, ToSpan};
use serde_json::json;

/// Helper: create two teams and a game whose date_time is `offset` from now.
async fn create_game(app: &mut TestApp, token: &str, offset: jiff::Span) -> String {
    let (home_id, away_id) = app.create_teams(token, "Home FC", "FC Live").await;
    let date_time = Timestamp::now() + offset;
    let res = app
        .post("/api/games")
        .token(token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadium",
            "dateTime": date_time.to_string()
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn adjust_score_when_not_live_returns_conflict() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, 168.hours()).await;

    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .token(&token)
        .json(json!({ "side": "away", "delta": 1 }))
        .await;
    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn adjust_score_requires_admin_or_moderator() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;
    let id = create_game(&mut app, &admin, -5.minutes()).await;

    // No auth
    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .json(json!({ "side": "away", "delta": 1 }))
        .await;
    assert_eq!(res.status(), 401);

    // Player role
    let player = app.player_token().await;
    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .token(&player)
        .json(json!({ "side": "away", "delta": 1 }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn moderator_can_adjust_score_during_live_match() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;
    let id = create_game(&mut app, &admin, -5.minutes()).await;

    let moderator = app.moderator_token().await;
    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .token(&moderator)
        .json(json!({ "side": "away", "delta": 1 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["homeScore"], 0);
    assert_eq!(body["awayScore"], 1);
    assert_eq!(body["version"], 1);
}

#[tokio::test]
async fn moderator_can_update_live_game_via_put() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;
    let id = create_game(&mut app, &admin, -5.minutes()).await;

    let res = app.get(format!("/api/games/{id}")).send().await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.json_value().await["status"], "live");

    let moderator = app.moderator_token().await;
    let res = app
        .put(format!("/api/games/{id}"))
        .token(&moderator)
        .json(json!({ "homeScore": 2, "awayScore": 1 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["homeScore"], 2);
    assert_eq!(body["awayScore"], 1);
    assert_eq!(body["status"], "live");
    assert_eq!(body["version"], 1);
}

#[tokio::test]
async fn adjust_score_increments_and_bumps_version() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .token(&token)
        .json(json!({ "side": "away", "delta": 1 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["homeScore"], 0);
    assert_eq!(body["awayScore"], 1);
    assert_eq!(body["version"], 1);

    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .token(&token)
        .json(json!({ "side": "home", "delta": 1 }))
        .await;
    let body = res.json_value().await;
    assert_eq!(body["homeScore"], 1);
    assert_eq!(body["awayScore"], 1);
    assert_eq!(body["version"], 2);
}

#[tokio::test]
async fn adjust_score_minus_one_clamps_at_zero() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .token(&token)
        .json(json!({ "side": "away", "delta": -1 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["awayScore"], 0);
}

#[tokio::test]
async fn adjust_score_rejects_invalid_delta() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let id = create_game(&mut app, &token, -5.minutes()).await;

    let res = app
        .post(format!("/api/games/{id}/live/score"))
        .token(&token)
        .json(json!({ "side": "away", "delta": 2 }))
        .await;
    assert_eq!(res.status(), 409);
}
