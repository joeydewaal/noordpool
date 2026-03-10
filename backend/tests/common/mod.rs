use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{self, StatusCode},
};
use axum_security::jwt::{JwtContext, get_current_timestamp};
use http_body_util::BodyExt;
use noordpool_backend::{
    app_state::AppState,
    auth::claims::Claims,
    models::{Role, build_db},
    routes,
};
use serde::{Serialize, de::DeserializeOwned};
use tower::{Service, ServiceExt};

pub async fn setup() -> (Router, AppState) {
    let db_path = std::env::temp_dir().join(format!("noordpool-test-{}.db", uuid::Uuid::new_v4()));
    let db_url = format!("sqlite:{}", db_path.display());
    let mut db = build_db().connect(&db_url).await.unwrap();
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

async fn get_role_token(app: &mut Router, state: &AppState, email: &str, role: Role) -> String {
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

pub struct TestApp {
    state: AppState,
    router: Router,
}

pub struct Response {
    inner: axum::response::Response,
}

impl Response {
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    pub async fn string(self) -> String {
        let body = self.inner.into_body().collect().await.unwrap().to_bytes();
        std::str::from_utf8(&body).unwrap().to_string()
    }

    pub async fn json<T: DeserializeOwned>(self) -> T {
        let body = self.string().await;
        serde_json::from_str(&body).unwrap()
    }
    pub async fn json_value(self) -> serde_json::Value {
        self.json().await
    }
}

pub struct RequestBuilder<'a> {
    inner: axum::http::request::Builder,
    app: &'a mut TestApp,
}

impl RequestBuilder<'_> {
    pub fn token(mut self, token: &str) -> Self {
        self.inner = self
            .inner
            .header("authorization", format!("Bearer {token}"));
        self
    }

    pub async fn send(self) -> Response {
        let req = self.inner.body(Body::empty()).unwrap();
        self.app.call(req).await
    }

    pub async fn json(self, body: impl Serialize) -> Response {
        let body = serde_json::to_string(&body).unwrap();
        let req = self.inner.body(Body::from(body)).unwrap();
        self.app.call(req).await
    }
}

impl TestApp {
    pub async fn new() -> Self {
        let db_path =
            std::env::temp_dir().join(format!("noordpool-test-{}.db", uuid::Uuid::new_v4()));
        let db_url = format!("sqlite:{}", db_path.display());
        let mut db = build_db().connect(&db_url).await.unwrap();
        db.push_schema().await.unwrap();

        let jwt = JwtContext::builder()
            .jwt_secret("test-secret")
            .build::<Claims>();

        let state = AppState {
            db,
            jwt,
            google_oauth2: None,
        };
        let router = routes::app(state.clone());
        TestApp { state, router }
    }

    pub fn get(&mut self, uri: impl Into<String>) -> RequestBuilder<'_> {
        RequestBuilder {
            inner: http::Request::get(uri.into()),
            app: self,
        }
    }

    pub fn post(&mut self, uri: impl Into<String>) -> RequestBuilder<'_> {
        RequestBuilder {
            inner: http::Request::post(uri.into()),
            app: self,
        }
    }

    pub fn delete(&mut self, uri: impl Into<String>) -> RequestBuilder<'_> {
        RequestBuilder {
            inner: http::Request::delete(uri.into()),
            app: self,
        }
    }

    pub async fn call(&mut self, req: Request) -> Response {
        let resp = (&mut self.router).call(req).await.unwrap();
        Response { inner: resp }
    }

    pub async fn admin_token(&mut self) -> String {
        self.role_token("admin@example.com", Role::Admin).await
    }

    pub async fn role_token(&mut self, email: &str, role: Role) -> String {
        let (_, body) = self
            .json_req(
                "POST",
                "/api/auth/register",
                serde_json::json!({
                    "name": format!("Test {:?}", role),
                    "email": email,
                    "password": "password123"
                }),
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
        self.state.jwt.encode_token(&claims).unwrap()
    }

    pub async fn json_req(
        &mut self,
        method: &str,
        uri: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let req = axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let (res, body) = self.call(req).await;
        (
            res,
            serde_json::from_str(&body).unwrap_or(serde_json::Value::Null),
        )
    }

    pub async fn json_req_auth(
        &mut self,
        method: &str,
        uri: &str,
        token: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let req = axum::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(body.to_string()))
            .unwrap();

        let (res, body) = self.call(req).await;
        (
            res,
            serde_json::from_str(&body).unwrap_or(serde_json::Value::Null),
        )
    }

    pub async fn get(&mut self, uri: &str, token: Option<&str>) -> (StatusCode, String) {
        let mut req = axum::http::Request::get(uri);

        if let Some(token) = token {
            req = req.header("authorization", format!("Bearer {token}"));
        };

        let req = req.body(Body::empty()).unwrap();

        self.call(req).await
    }
}
