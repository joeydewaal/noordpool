mod common;

use common::TestApp;
use insta::{Settings, assert_json_snapshot};
use serde_json::json;

/// Creates a finished game (past date), adds goal events for players on known
/// teams, and verifies that GET /api/games/summary returns the correct scores
/// computed from those events rather than the zero-value stored adjustments.
#[tokio::test]
async fn recent_games_score_computed_from_events() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let (home_id, away_id) = app
        .create_teams(&token, "De Noordpool", "FC Tegenstander")
        .await;

    // Create a player on the home team
    let player_res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Scorer",
            "lastName": "",
            "shirtNumber": 9,
            "position": "Spits",
            "teamId": home_id
        }))
        .await;
    let player_id = player_res.json_value().await["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Game in the past → status "finished"
    let game_res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Veld",
            "dateTime": "2024-03-01T15:00:00Z"
        }))
        .await;
    let game_id = game_res.json_value().await["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Two home goals by the home player
    for minute in [20, 55] {
        app.post(format!("/api/games/{game_id}/events"))
            .token(&token)
            .json(json!({ "playerId": player_id, "eventType": "goal", "minute": minute }))
            .await;
    }
    // One own goal by the home player (counts for away)
    app.post(format!("/api/games/{game_id}/events"))
        .token(&token)
        .json(json!({ "playerId": player_id, "eventType": "own_goal", "minute": 70 }))
        .await;

    let summary = app.get("/api/games/summary").send().await;
    assert_eq!(summary.status(), 200);
    let body = summary.json_value().await;
    let recent = body["recent"].as_array().unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0]["homeScore"], 2, "two home goals from events");
    assert_eq!(recent[0]["awayScore"], 1, "one own-goal counts for away");
}

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    settings.add_redaction(".dateTime", "[dateTime]");
    settings.add_redaction(".updatedAt", "[timestamp]");
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
    settings.add_redaction(".homeTeam.id", "[uuid]");
    settings.add_redaction(".awayTeam.id", "[uuid]");
    settings.add_redaction(".homeTeamId", "[uuid]");
    settings.add_redaction(".awayTeamId", "[uuid]");
    settings.add_redaction("[].homeTeam.id", "[uuid]");
    settings.add_redaction("[].awayTeam.id", "[uuid]");
    settings.add_redaction("[].homeTeamId", "[uuid]");
    settings.add_redaction("[].awayTeamId", "[uuid]");
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
    let token = app.admin_token().await;
    let (home_id, away_id) = app.create_teams(&token, "Home FC", "Away FC").await;

    let res = app
        .post("/api/games")
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z"
        }))
        .await;
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn create_game_forbidden_for_player_role() {
    let mut app = TestApp::new().await;
    let admin_token = app.admin_token().await;
    let (home_id, away_id) = app.create_teams(&admin_token, "Home FC", "Away FC").await;
    let token = app.player_token().await;

    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z"
        }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn create_and_get_game() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;
    let (home_id, away_id) = app.create_teams(&token, "De Noordpool", "FC Test").await;

    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z"
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
    let admin = app.admin_token().await;
    let (home_id, away_id) = app.create_teams(&admin, "De Noordpool", "FC Test").await;
    let token = app.moderator_token().await;

    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z"
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
    let (home_id, future_id) = app.create_teams(&token, "De Noordpool", "FC Future").await;

    let res = app
        .post("/api/teams")
        .token(&token)
        .json(json!({ "name": "FC Past" }))
        .await;
    let past_id = res.json_value().await["id"].as_str().unwrap().to_string();

    // Create a scheduled match (future)
    app.post("/api/games")
        .token(&token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": future_id,
            "location": "Home Stadium",
            "dateTime": "2027-06-15T18:00:00Z"
        }))
        .await;

    // Create a match in the past (automatically completed by date)
    app.post("/api/games")
        .token(&token)
        .json(json!({
            "homeTeamId": past_id,
            "awayTeamId": home_id,
            "location": "Away Stadium",
            "dateTime": "2024-01-10T15:00:00Z"
        }))
        .await;

    // Check upcoming
    let res = app.get("/api/games/upcoming").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["awayTeam"]["name"], "FC Future");

    // Check recent
    let res = app.get("/api/games/recent").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["homeTeam"]["name"], "FC Past");

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
    let (home_id, away_id) = app.create_teams(&token, "De Noordpool", "FC Test").await;

    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z"
        }))
        .await;
    let created = res.json_value().await;
    let game_id = created["id"].as_str().unwrap();

    let res = app
        .delete(format!("/api/games/{game_id}"))
        .token(&token)
        .send()
        .await;
    assert_eq!(res.status(), 204);

    let res = app.get(format!("/api/games/{game_id}")).send().await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_match_forbidden_for_moderator() {
    let mut app = TestApp::new().await;
    let admin_token = app.admin_token().await;
    let mod_token = app.moderator_token().await;
    let (home_id, away_id) = app
        .create_teams(&admin_token, "De Noordpool", "FC Test")
        .await;

    let res = app
        .post("/api/games")
        .token(&admin_token)
        .json(json!({
            "homeTeamId": home_id,
            "awayTeamId": away_id,
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z"
        }))
        .await;
    let created = res.json_value().await;
    let game_id = created["id"].as_str().unwrap();

    let res = app
        .delete(format!("/api/games/{game_id}"))
        .token(&mod_token)
        .send()
        .await;
    assert_eq!(res.status(), 403);
}
