use axum::{
    body::Body,
    extract::Request,
    http::{self, StatusCode},
};
use axum_security::jwt::JwtContext;
use http_body_util::BodyExt;
use jiff::{Timestamp, ToSpan};
use noordpool_backend::{
    app_state::AppState,
    auth::claims::Claims,
    models::{Role, build_db},
    routes,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use tower::Service;
use uuid::Uuid;

pub struct TestApp {
    state: AppState,
    router: axum::Router,
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
        let body = self.string().await;
        if body.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&body).unwrap()
        }
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
        let req = self
            .inner
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();
        self.app.call(req).await
    }
}

impl TestApp {
    pub async fn new() -> Self {
        let mut db = build_db().connect("sqlite::memory:").await.unwrap();
        db.push_schema().await.unwrap();
        // Release the cached connection back to the pool so handlers can use it.
        // With max_connections=1 for in-memory SQLite, the pool only allows one
        // connection. If we keep it cached here, handler clones will deadlock
        // waiting for it.
        let db = {
            let fresh = db.clone();
            drop(db);
            fresh
        };

        let jwt = JwtContext::builder()
            .jwt_secret("test-secret")
            .build::<Claims>();

        let state = AppState {
            db,
            jwt,
            google_oidc: None,
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

    pub fn put(&mut self, uri: impl Into<String>) -> RequestBuilder<'_> {
        RequestBuilder {
            inner: http::Request::put(uri.into()),
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

    pub async fn moderator_token(&mut self) -> String {
        self.role_token("moderator@example.com", Role::Moderator)
            .await
    }

    pub async fn player_token(&mut self) -> String {
        let res = self
            .post("/api/auth/register")
            .json(json!({
                "name": "Test User",
                "email": "test@example.com",
                "password": "password123"
            }))
            .await;
        let body = res.json_value().await;
        body["token"].as_str().unwrap().to_string()
    }

    pub async fn role_token(&mut self, email: &str, role: Role) -> String {
        let res = self
            .post("/api/auth/register")
            .json(json!({
                "name": format!("Test {:?}", role),
                "email": email,
                "password": "password123"
            }))
            .await;
        let body = res.json_value().await;
        let user_id: Uuid = body["user"]["id"].as_str().unwrap().parse().unwrap();

        // Encode a JWT with the elevated roles — RBAC checks the token, not the DB
        let claims = Claims {
            sub: user_id,
            email: email.to_string(),
            name: format!("Test {:?}", role),
            roles: vec![Role::Player, role],
            exp: Timestamp::now() + 24.hour(),
        };
        self.state.jwt.encode_token(&claims).unwrap()
    }
}
