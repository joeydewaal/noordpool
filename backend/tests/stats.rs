mod common;

use common::TestApp;
use insta::{Settings, assert_json_snapshot};
use serde_json::json;

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    for path in &[
        "[].playerId",
        ".topScorers[].playerId",
        ".topAssisters[].playerId",
        ".mostCarded[].playerId",
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

async fn create_player(
    app: &mut TestApp,
    token: &str,
    name: &str,
    number: i32,
    position: &str,
) -> String {
    let res = app
        .post("/api/players")
        .token(token)
        .json(json!({
            "name": name,
            "email": format!("{name}@test.be"),
            "shirtNumber": number,
            "position": position
        }))
        .await;
    let body = res.json_value().await;
    body["id"].as_str().unwrap().to_string()
}

async fn create_game_and_complete(
    app: &mut TestApp,
    token: &str,
    opponent: &str,
    home_score: i32,
    away_score: i32,
) -> String {
    let res = app
        .post("/api/games")
        .token(token)
        .json(json!({
            "opponent": opponent,
            "location": "Stadium",
            "dateTime": "2026-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    let body = res.json_value().await;
    let game_id = body["id"].as_str().unwrap().to_string();
    app.put(format!("/api/games/{game_id}"))
        .token(token)
        .json(json!({
            "status": "completed",
            "homeScore": home_score,
            "awayScore": away_score
        }))
        .await;
    game_id
}

async fn add_event(
    app: &mut TestApp,
    token: &str,
    game_id: &str,
    player_id: &str,
    event_type: &str,
    minute: i32,
) {
    app.post(format!("/api/games/{game_id}/events"))
        .token(token)
        .json(json!({
            "playerId": player_id,
            "eventType": event_type,
            "minute": minute
        }))
        .await;
}

#[tokio::test]
async fn leaderboard_empty() {
    let mut app = TestApp::new().await;
    let res = app.get("/api/stats/leaderboard").send().await;
    assert_eq!(res.status(), 200);
    assert_json_snapshot!(res.json_value().await, @r#"
    {
      "mostCarded": [],
      "topAssisters": [],
      "topScorers": []
    }
    "#);
}

#[tokio::test]
async fn leaderboard_with_data() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let striker = create_player(&mut app, &token, "Striker", 9, "forward").await;
    let midfield = create_player(&mut app, &token, "Playmaker", 10, "midfielder").await;
    let defender = create_player(&mut app, &token, "Tough Guy", 4, "defender").await;

    let game1 = create_game_and_complete(&mut app, &token, "FC Alpha", 3, 0).await;
    let game2 = create_game_and_complete(&mut app, &token, "FC Beta", 2, 1).await;

    // Game 1 events: striker scores 2, midfield assists 2, defender yellow card
    add_event(&mut app, &token, &game1, &striker, "goal", 15).await;
    add_event(&mut app, &token, &game1, &midfield, "assist", 15).await;
    add_event(&mut app, &token, &game1, &striker, "goal", 40).await;
    add_event(&mut app, &token, &game1, &midfield, "assist", 40).await;
    add_event(&mut app, &token, &game1, &defender, "yellow_card", 55).await;
    add_event(&mut app, &token, &game1, &midfield, "goal", 70).await;

    // Game 2 events: striker scores 1, defender red card
    add_event(&mut app, &token, &game2, &striker, "goal", 20).await;
    add_event(&mut app, &token, &game2, &midfield, "assist", 20).await;
    add_event(&mut app, &token, &game2, &defender, "red_card", 80).await;
    add_event(&mut app, &token, &game2, &defender, "yellow_card", 30).await;

    let res = app.get("/api/stats/leaderboard").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;

    // Verify structure
    let top_scorers = body["topScorers"].as_array().unwrap();
    assert_eq!(top_scorers[0]["name"], "Striker");
    assert_eq!(top_scorers[0]["goals"], 3);

    let top_assisters = body["topAssisters"].as_array().unwrap();
    assert_eq!(top_assisters[0]["name"], "Playmaker");
    assert_eq!(top_assisters[0]["assists"], 3);

    let most_carded = body["mostCarded"].as_array().unwrap();
    assert_eq!(most_carded[0]["name"], "Tough Guy");
    assert_eq!(most_carded[0]["totalCards"], 3);
}

#[tokio::test]
async fn player_stats_with_events() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let player = create_player(&mut app, &token, "Star Player", 7, "forward").await;
    let game = create_game_and_complete(&mut app, &token, "FC Rival", 2, 0).await;

    add_event(&mut app, &token, &game, &player, "goal", 10).await;
    add_event(&mut app, &token, &game, &player, "goal", 55).await;
    add_event(&mut app, &token, &game, &player, "assist", 30).await;
    add_event(&mut app, &token, &game, &player, "yellow_card", 70).await;

    let res = app.get(format!("/api/players/{player}/stats")).send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("player_stats_with_events", body, {
            ".playerId" => "[uuid]",
        });
    });
}

#[tokio::test]
async fn stats_ignore_scheduled_game_events() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let player = create_player(&mut app, &token, "Player", 11, "forward").await;

    // Create a scheduled (not completed) match
    let res = app
        .post("/api/games")
        .token(&token)
        .json(json!({
            "opponent": "FC Scheduled",
            "location": "Stadium",
            "dateTime": "2027-06-15T18:00:00Z",
            "homeAway": "home"
        }))
        .await;
    let body = res.json_value().await;
    let game_id = body["id"].as_str().unwrap();

    // Add a goal to the scheduled match
    add_event(&mut app, &token, game_id, &player, "goal", 10).await;

    // Stats should show 0 appearances (game not completed)
    let res = app.get(format!("/api/players/{player}/stats")).send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["appearances"], 0);

    // Leaderboard should also not count it
    let res = app.get("/api/stats/leaderboard").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    // Player appears but with 0 stats since events are in non-completed game
    let top_scorers = body["topScorers"].as_array().unwrap();
    assert_eq!(top_scorers[0]["goals"].as_i64().unwrap(), 0);
}
