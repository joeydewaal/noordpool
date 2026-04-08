mod common;

use common::TestApp;
use serde_json::json;

// ── find-player ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn find_player_returns_unlinked_match() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    // Admin creates a player without email
    app.post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Piet",
            "lastName": "Paulsen",
            "shirtNumber": 9,
            "position": "Spits"
        }))
        .await;

    let res = app.get("/api/auth/find-player?name=piet").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let matches = body.as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["firstName"], "Piet");
    assert_eq!(matches[0]["lastName"], "Paulsen");
    assert_eq!(matches[0]["shirtNumber"], 9);
    assert_eq!(matches[0]["position"], "Spits");
}

#[tokio::test]
async fn find_player_excludes_linked_players() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    // Admin creates a player
    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Linked",
            "lastName": "Speler",
            "shirtNumber": 5,
            "position": "Centrale middenvelder"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    // Register a user, then link them to the player
    let register_res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Linked",
            "lastName": "Speler",
            "email": "linked@example.com",
            "password": "test123"
        }))
        .await;
    let user_token = register_res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/auth/link-player")
        .token(&user_token)
        .json(json!({ "player_id": player_id }))
        .await;

    let res = app.get("/api/auth/find-player?name=linked").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn find_player_is_case_insensitive() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    app.post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Karel",
            "lastName": "Karelse",
            "shirtNumber": 3,
            "position": "Centrale verdediger"
        }))
        .await;

    // Search with uppercase
    let res = app.get("/api/auth/find-player?name=KAREL").send().await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn find_player_returns_empty_for_no_match() {
    let mut app = TestApp::new().await;

    let res = app
        .get("/api/auth/find-player?name=doesnotexist")
        .send()
        .await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.json_value().await.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn find_player_excludes_inactive_players() {
    let mut app = TestApp::new().await;
    let token = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&token)
        .json(json!({
            "firstName": "Inactive",
            "lastName": "Ivo",
            "shirtNumber": 11,
            "position": "Keeper"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    // Soft-delete the player
    app.delete(format!("/api/players/{player_id}"))
        .token(&token)
        .send()
        .await;

    let res = app.get("/api/auth/find-player?name=inactive").send().await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.json_value().await.as_array().unwrap().len(), 0);
}

// ── register ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn register_without_player_link_works() {
    let mut app = TestApp::new().await;

    let res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Nieuw",
            "lastName": "Iemand",
            "email": "nieuw@example.com",
            "password": "test123"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["user"]["firstName"], "Nieuw");
    assert_eq!(body["user"]["lastName"], "Iemand");
    assert!(body["token"].as_str().is_some());
}

#[tokio::test]
async fn register_without_player_has_no_player_role() {
    let mut app = TestApp::new().await;

    let res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Vers",
            "lastName": "Geregistreerd",
            "email": "vers@example.com",
            "password": "test123"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let roles = body["user"]["roles"].as_array().unwrap();
    assert!(
        !roles.iter().any(|r| r == "player"),
        "fresh user should not have player role"
    );
}

// ── link-player ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn link_player_links_to_existing_player() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&admin)
        .json(json!({
            "firstName": "Sjaak",
            "lastName": "Swart",
            "shirtNumber": 11,
            "position": "Spits"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    let register_res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Sjaak",
            "lastName": "Swart",
            "email": "sjaak@example.com",
            "password": "geheim123"
        }))
        .await;
    let user_token = register_res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/auth/link-player")
        .token(&user_token)
        .json(json!({ "player_id": player_id }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert_eq!(body["user"]["firstName"], "Sjaak");
    assert_eq!(body["user"]["lastName"], "Swart");
    assert!(body["token"].as_str().is_some());
    let roles = body["user"]["roles"].as_array().unwrap();
    assert!(
        roles.iter().any(|r| r == "player"),
        "linked user should have player role"
    );
}

#[tokio::test]
async fn link_player_linked_user_has_player_role_in_login() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&admin)
        .json(json!({
            "firstName": "Klaas",
            "lastName": "Klaasen",
            "shirtNumber": 6,
            "position": "Centrale middenvelder"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    let register_res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Klaas",
            "lastName": "Klaasen",
            "email": "klaas@example.com",
            "password": "wachtwoord"
        }))
        .await;
    let user_token = register_res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/auth/link-player")
        .token(&user_token)
        .json(json!({ "player_id": player_id }))
        .await;

    let res = app
        .post("/api/auth/login")
        .json(json!({
            "email": "klaas@example.com",
            "password": "wachtwoord"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let roles = body["user"]["roles"].as_array().unwrap();
    assert!(
        roles.iter().any(|r| r == "player"),
        "linked user should have player role on login"
    );
}

#[tokio::test]
async fn login_without_player_has_no_player_role() {
    let mut app = TestApp::new().await;

    app.post("/api/auth/register")
        .json(json!({
            "firstName": "Zonder",
            "lastName": "Speler",
            "email": "zonder@example.com",
            "password": "test123"
        }))
        .await;

    let res = app
        .post("/api/auth/login")
        .json(json!({
            "email": "zonder@example.com",
            "password": "test123"
        }))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    let roles = body["user"]["roles"].as_array().unwrap();
    assert!(
        !roles.iter().any(|r| r == "player"),
        "unlinked user should not have player role on login"
    );
}

#[tokio::test]
async fn link_player_fails_if_player_already_linked() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&admin)
        .json(json!({
            "firstName": "Al",
            "lastName": "Gelinkt",
            "shirtNumber": 1,
            "position": "Keeper"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    // First user links the player
    let res1 = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Al",
            "lastName": "Gelinkt",
            "email": "al@example.com",
            "password": "test123"
        }))
        .await;
    let token1 = res1.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/auth/link-player")
        .token(&token1)
        .json(json!({ "player_id": player_id }))
        .await;

    // Second user tries to link the same player — should conflict
    let res2 = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Iemand",
            "lastName": "Anders",
            "email": "anders@example.com",
            "password": "test123"
        }))
        .await;
    let token2 = res2.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/auth/link-player")
        .token(&token2)
        .json(json!({ "player_id": player_id }))
        .await;
    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn link_player_fails_if_player_not_found() {
    let mut app = TestApp::new().await;

    let res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Iemand",
            "lastName": "",
            "email": "iemand@example.com",
            "password": "test123"
        }))
        .await;
    let user_token = res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/auth/link-player")
        .token(&user_token)
        .json(json!({ "player_id": "00000000-0000-0000-0000-000000000000" }))
        .await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn link_player_no_longer_appears_in_find_player() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    app.post("/api/players")
        .token(&admin)
        .json(json!({
            "firstName": "Nu",
            "lastName": "Gelinkt",
            "shirtNumber": 7,
            "position": "Centrale verdediger"
        }))
        .await;

    // Confirm the player shows up before linking
    let res = app
        .get("/api/auth/find-player?name=Nu%20Gelinkt")
        .send()
        .await;
    assert_eq!(res.json_value().await.as_array().unwrap().len(), 1);

    let find_res = app
        .get("/api/auth/find-player?name=Nu%20Gelinkt")
        .send()
        .await;
    let player_id = find_res.json_value().await[0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let register_res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Nu",
            "lastName": "Gelinkt",
            "email": "gelinkt@example.com",
            "password": "test123"
        }))
        .await;
    let user_token = register_res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    app.post("/api/auth/link-player")
        .token(&user_token)
        .json(json!({ "player_id": player_id }))
        .await;

    // Should no longer appear
    let res = app
        .get("/api/auth/find-player?name=Nu%20Gelinkt")
        .send()
        .await;
    assert_eq!(res.json_value().await.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn link_player_requires_authentication() {
    let mut app = TestApp::new().await;

    let res = app
        .post("/api/auth/link-player")
        .json(json!({ "player_id": "00000000-0000-0000-0000-000000000000" }))
        .await;
    assert_eq!(res.status(), 401);
}

// ── unlink-player ────────────────────────────────────────────────────────────

#[tokio::test]
async fn unlink_player_removes_link() {
    let mut app = TestApp::new().await;
    let admin = app.admin_token().await;

    let res = app
        .post("/api/players")
        .token(&admin)
        .json(json!({
            "firstName": "Henk",
            "lastName": "Ontkoppeld",
            "shirtNumber": 14,
            "position": "Spits"
        }))
        .await;
    let player_id = res.json_value().await["id"].as_str().unwrap().to_string();

    let register_res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Henk",
            "lastName": "Ontkoppeld",
            "email": "henk@example.com",
            "password": "test123"
        }))
        .await;
    let user_token = register_res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    // Link
    let link_res = app
        .post("/api/auth/link-player")
        .token(&user_token)
        .json(json!({ "player_id": player_id }))
        .await;
    let linked_token = link_res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    // Unlink
    let res = app
        .post("/api/auth/unlink-player")
        .token(&linked_token)
        .json(json!({}))
        .await;
    assert_eq!(res.status(), 200);
    let body = res.json_value().await;
    assert!(body["token"].as_str().is_some());
    let roles = body["user"]["roles"].as_array().unwrap();
    assert!(
        !roles.iter().any(|r| r == "player"),
        "unlinked user should not have player role"
    );

    // Player should reappear in find-player
    let res = app
        .get("/api/auth/find-player?name=Henk%20Ontkoppeld")
        .send()
        .await;
    assert_eq!(res.json_value().await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn unlink_player_fails_if_not_linked() {
    let mut app = TestApp::new().await;

    let register_res = app
        .post("/api/auth/register")
        .json(json!({
            "firstName": "Geen",
            "lastName": "Speler",
            "email": "geen@example.com",
            "password": "test123"
        }))
        .await;
    let user_token = register_res.json_value().await["token"]
        .as_str()
        .unwrap()
        .to_string();

    let res = app
        .post("/api/auth/unlink-player")
        .token(&user_token)
        .json(json!({}))
        .await;
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn unlink_player_requires_authentication() {
    let mut app = TestApp::new().await;

    let res = app.post("/api/auth/unlink-player").json(json!({})).await;
    assert_eq!(res.status(), 401);
}
