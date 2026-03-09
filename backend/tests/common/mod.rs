use axum::Router;
use axum_security::jwt::{JwtContext, get_current_timestamp};
use http_body_util::BodyExt;
use noordpool_backend::{
    app_state::AppState,
    auth::claims::Claims,
    models::{
        EventType, Game, HomeAway, MatchEvent, MatchStatus, Player, Position, Role, User, UserRole,
    },
    routes,
};
use toasty::Db;
use tower::ServiceExt;

pub async fn setup() -> (Router, AppState) {
    let mut builder = Db::builder();
    builder.register::<User>();
    builder.register::<UserRole>();
    builder.register::<Role>();
    builder.register::<Player>();
    builder.register::<Position>();
    builder.register::<Game>();
    builder.register::<MatchStatus>();
    builder.register::<HomeAway>();
    builder.register::<MatchEvent>();
    builder.register::<EventType>();
    let mut db = builder.connect("sqlite::memory:").await.unwrap();
    db.push_schema().await.unwrap();

    let jwt = JwtContext::builder()
        .jwt_secret("test-secret")
        .build::<Claims>();

    let state = AppState {
        db,
        jwt,
        google_oauth2: None,
    };
    let app = routes::app(state.clone());
    (app, state)
}

/// Register a user and return a token with Player role (default).
pub async fn get_token(app: &mut Router) -> String {
    let (_, body) = call(
        app,
        axum::http::Request::builder()
            .method("POST")
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                serde_json::json!({
                    "name": "Test User",
                    "email": "test@example.com",
                    "password": "password123"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    body["token"].as_str().unwrap().to_string()
}

/// Encode a token with Admin + Player roles (no DB registration needed for role check).
pub async fn get_admin_token(app: &mut Router, state: &AppState) -> String {
    get_role_token(app, state, "admin@example.com", Role::Admin).await
}

/// Encode a token with Moderator + Player roles.
pub async fn get_moderator_token(app: &mut Router, state: &AppState) -> String {
    get_role_token(app, state, "moderator@example.com", Role::Moderator).await
}

async fn get_role_token(
    app: &mut Router,
    state: &AppState,
    email: &str,
    role: Role,
) -> String {
    // Register the user (gets Player role in DB)
    let (_, body) = call(
        app,
        axum::http::Request::builder()
            .method("POST")
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                serde_json::json!({
                    "name": format!("Test {:?}", role),
                    "email": email,
                    "password": "password123"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    let user_id = body["user"]["id"].as_str().unwrap();

    // Encode a JWT with the elevated roles — RBAC checks the token, not the DB
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        name: format!("Test {:?}", role),
        roles: vec![Role::Player, role],
        exp: get_current_timestamp() + 24 * 60 * 60,
    };
    state.jwt.encode_token(&claims).unwrap()
}

pub fn request(method: &str, uri: &str) -> axum::http::request::Builder {
    axum::http::Request::builder().method(method).uri(uri)
}

pub fn json_request(method: &str, uri: &str) -> axum::http::request::Builder {
    request(method, uri).header("content-type", "application/json")
}

pub fn auth_json_request(method: &str, uri: &str, token: &str) -> axum::http::request::Builder {
    json_request(method, uri).header("authorization", format!("Bearer {token}"))
}

pub async fn call(
    app: &mut Router,
    req: axum::http::Request<axum::body::Body>,
) -> (axum::http::StatusCode, serde_json::Value) {
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = if body.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null)
    };
    (status, json)
}
