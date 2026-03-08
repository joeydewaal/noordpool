use axum::Router;
use axum_security::jwt::JwtContext;
use http_body_util::BodyExt;
use noordpool_backend::{
    app_state::AppState,
    auth::claims::Claims,
    models::{Game, MatchEvent, Player, User, UserRole},
    routes,
};
use toasty::Db;
use tower::ServiceExt;

pub async fn setup() -> (Router, AppState) {
    let mut builder = Db::builder();
    builder.register::<User>();
    builder.register::<UserRole>();
    builder.register::<Player>();
    builder.register::<Game>();
    builder.register::<MatchEvent>();
    let mut db = builder.connect("sqlite::memory:").await.unwrap();
    db.push_schema().await.unwrap();

    let jwt = JwtContext::builder()
        .jwt_secret("test-secret")
        .build::<Claims>();

    let state = AppState { db, jwt };
    let app = routes::app(state.clone());
    (app, state)
}

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

pub fn request(method: &str, uri: &str) -> axum::http::request::Builder {
    axum::http::Request::builder().method(method).uri(uri)
}

pub fn json_request(method: &str, uri: &str) -> axum::http::request::Builder {
    request(method, uri).header("content-type", "application/json")
}

pub fn auth_json_request(
    method: &str,
    uri: &str,
    token: &str,
) -> axum::http::request::Builder {
    json_request(method, uri).header("authorization", format!("Bearer {token}"))
}

pub async fn call(
    app: &mut Router,
    req: axum::http::Request<axum::body::Body>,
) -> (axum::http::StatusCode, serde_json::Value) {
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = if body.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null)
    };
    (status, json)
}
