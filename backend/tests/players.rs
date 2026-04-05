mod common;

use common::TestApp;
use insta::{Settings, assert_json_snapshot};
use serde_json::json;
use uuid::Uuid;

fn redact_settings() -> Settings {
    let mut settings = Settings::clone_current();
    settings.add_redaction(".id", "[uuid]");
    settings.add_redaction(".playerId", "[uuid]");
    settings.add_redaction(".createdAt", "[createdAt]");
    settings
}

#[tokio::test]
async fn list_players_empty() {
    let mut app = TestApp::new().await;
    let res = app.get("/api/players").send().await;
    assert_eq!(res.status(), 200);
    assert_json_snapshot!(res.json_value().await, @"[]");
}

#[tokio::test]
async fn create_player_requires_auth() {
    let mut app = TestApp::new().await;
    let res = app
        .post("/api/players")
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Centrale middenvelder"
        }))
        .await;
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn create_player_forbidden_for_player_role() {
    let mut app = TestApp::new().await;
    let token = app.player_token().await;

    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Centrale middenvelder"
        }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn create_and_get_player() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Centrale middenvelder"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("create_player", body);
    });

    let player_id = body["id"].as_str().unwrap();
    let res = app.get(format!("/api/players/{player_id}")).send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    settings.bind(|| {
        assert_json_snapshot!("get_player", body);
    });
}

#[tokio::test]
async fn update_player() {
    let mut app = TestApp::new().await;
    let token = app.moderator_token().await;

    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Centrale middenvelder"
        }))
        .await;
    let created = res.json_value().await;

    let player_id = created["id"].as_str().unwrap();
    let res = app
        .put(format!("/api/players/{player_id}"))
        .token(&token)
        .json(json!({
            "shirtNumber": 7,
            "position": "Spits"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("update_player", body);
    });
}

#[tokio::test]
async fn get_player_not_found() {
    let mut app = TestApp::new().await;
    let res = app
        .get("/api/players/00000000-0000-0000-0000-000000000000")
        .send()
        .await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn player_stats_empty() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Centrale middenvelder"
        }))
        .await;
    let created = res.json_value().await;

    let player_id = created["id"].as_str().unwrap();
    let res = app
        .get(format!("/api/players/{player_id}/stats"))
        .send()
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let settings = redact_settings();
    settings.bind(|| {
        assert_json_snapshot!("player_stats_empty", body);
    });
}

#[tokio::test]
async fn delete_player_soft_deletes() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    // Create a player
    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Centrale middenvelder"
        }))
        .await;
    let created = res.json_value().await;
    let player_id = created["id"].as_str().unwrap();

    // Delete (soft)
    let res = app
        .delete(format!("/api/players/{player_id}"))
        .token(&token)
        .send()
        .await;
    assert_eq!(res.status(), 204);

    // Player should not appear in list (only active players)
    let res = app.get("/api/players").send().await;
    let players = res.json_value().await;
    let names: Vec<_> = players
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["firstName"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"Jan"));

    // But get by ID should still work
    let res = app.get(format!("/api/players/{player_id}")).send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["active"], false);
}

#[tokio::test]
async fn delete_player_forbidden_for_moderator() {
    let mut app = TestApp::new().await;
    let admin_token = app.admin_token().await;
    let mod_token = app.moderator_token().await;

    // Create a player as admin
    let res = app
        .post("/api/players")
        .token(&admin_token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Centrale middenvelder"
        }))
        .await;
    let created = res.json_value().await;
    let player_id = created["id"].as_str().unwrap();

    // Moderator should not be able to delete
    let res = app
        .delete(format!("/api/players/{player_id}"))
        .token(&mod_token)
        .send()
        .await;
    assert_eq!(res.status(), 403);
}

// --- Self-service tests ---

/// Helper: create a player and return (player_id, player_token)
async fn create_player_with_token(app: &mut TestApp) -> (String, String) {
    let admin_token = app.admin_token().await;
    let res = app
        .post("/api/players")
        .token(&admin_token)
        .json(json!({
            "firstName": "Eigen",
            "lastName": "Speler",
            "shirtNumber": 5,
            "position": "Linksback"
        }))
        .await;
    let body = res.json_value().await;
    let player_id = body["id"].as_str().unwrap().to_string();
    let uuid: Uuid = player_id.parse().unwrap();
    let token = app.player_token_for(uuid);
    (player_id, token)
}

#[tokio::test]
async fn self_service_update_own_shirt_number() {
    let mut app = TestApp::new().await;
    let (player_id, token) = create_player_with_token(&mut app).await;

    let res = app
        .put(format!("/api/players/{player_id}"))
        .token(&token)
        .json(json!({ "shirtNumber": 99 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["shirtNumber"], 99);
}

#[tokio::test]
async fn self_service_update_own_position() {
    let mut app = TestApp::new().await;
    let (player_id, token) = create_player_with_token(&mut app).await;

    let res = app
        .put(format!("/api/players/{player_id}"))
        .token(&token)
        .json(json!({ "position": "Spits" }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["position"], "Spits");
}

#[tokio::test]
async fn self_service_cannot_update_name() {
    let mut app = TestApp::new().await;
    let (player_id, token) = create_player_with_token(&mut app).await;

    let res = app
        .put(format!("/api/players/{player_id}"))
        .token(&token)
        .json(json!({ "firstName": "Hacked", "lastName": "Name", "shirtNumber": 7 }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    // Name should be unchanged, only shirt number updated
    assert_eq!(body["firstName"], "Eigen");
    assert_eq!(body["lastName"], "Speler");
    assert_eq!(body["shirtNumber"], 7);
}

#[tokio::test]
async fn self_service_cannot_update_active() {
    let mut app = TestApp::new().await;
    let (player_id, token) = create_player_with_token(&mut app).await;

    let res = app
        .put(format!("/api/players/{player_id}"))
        .token(&token)
        .json(json!({ "active": false }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["active"], true);
}

#[tokio::test]
async fn self_service_cannot_update_other_player() {
    let mut app = TestApp::new().await;
    let admin_token = app.admin_token().await;

    // Create two players
    let res = app
        .post("/api/players")
        .token(&admin_token)
        .json(json!({
            "firstName": "Speler",
            "lastName": "Een",
            "shirtNumber": 1,
            "position": "Keeper"
        }))
        .await;
    let player_one_id: Uuid = res.json_value().await["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let res = app
        .post("/api/players")
        .token(&admin_token)
        .json(json!({
            "firstName": "Speler",
            "lastName": "Twee",
            "shirtNumber": 2,
            "position": "Spits"
        }))
        .await;
    let player_two_id = res.json_value().await["id"].as_str().unwrap().to_string();

    // Player one tries to update player two
    let token = app.player_token_for(player_one_id);
    let res = app
        .put(format!("/api/players/{player_two_id}"))
        .token(&token)
        .json(json!({ "shirtNumber": 99 }))
        .await;
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn self_service_update_requires_auth() {
    let mut app = TestApp::new().await;
    let admin_token = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&admin_token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Spits"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    // No auth token
    let res = app
        .put(format!("/api/players/{player_id}"))
        .json(json!({ "shirtNumber": 99 }))
        .await;
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn admin_can_still_update_all_fields() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Jan",
            "lastName": "de Boer",
            "shirtNumber": 10,
            "position": "Spits"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    let res = app
        .put(format!("/api/players/{player_id}"))
        .token(&token)
        .json(json!({
            "firstName": "Piet",
            "lastName": "Ansen",
            "shirtNumber": 7,
            "position": "Keeper",
            "active": false
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["firstName"], "Piet");
    assert_eq!(body["lastName"], "Ansen");
    assert_eq!(body["shirtNumber"], 7);
    assert_eq!(body["position"], "Keeper");
    assert_eq!(body["active"], false);
}
