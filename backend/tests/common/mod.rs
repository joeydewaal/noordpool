#![allow(dead_code)]

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
    push::{Notification, PushBackend},
    routes,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use tower::Service;
use uuid::Uuid;

static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TestApp {
    state: AppState,
    router: axum::Router,
    /// Name of the temporary database to drop on cleanup (PostgreSQL only).
    test_db: Option<String>,
    /// Base connection URL for creating/dropping databases.
    base_url: Option<String>,
    /// Captured notifications (always populated — TestApp always uses mock push).
    pub notifications: Arc<Mutex<Vec<Notification>>>,
}

pub struct Response {
    inner: axum::response::Response,
}

impl Response {
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    pub fn inner_headers(&self) -> &http::HeaderMap {
        self.inner.headers()
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

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.inner = self.inner.header(name, value);
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
        let (url, test_db, base_url) = match std::env::var("DATABASE_URL") {
            Ok(base) => {
                let id = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
                let pid = std::process::id();
                let db_name = format!("test_{pid}_{id}");
                let (client, conn) = tokio_postgres::connect(&base, tokio_postgres::NoTls)
                    .await
                    .unwrap();
                tokio::spawn(async move {
                    conn.await.ok();
                });
                client
                    .execute(&format!("CREATE DATABASE {db_name}"), &[])
                    .await
                    .unwrap();
                drop(client);
                // Replace the database name in the URL
                let test_url = format!("{}/{}", base.rsplit_once('/').unwrap().0, db_name);
                (test_url, Some(db_name), Some(base))
            }
            Err(_) => ("sqlite::memory:".to_string(), None, None),
        };

        let db = build_db().connect(&url).await.unwrap();
        db.push_schema().await.unwrap();

        let jwt = JwtContext::builder()
            .jwt_secret("test-secret")
            .build::<Claims>();

        let avatar_dir = std::env::temp_dir().join(format!(
            "noordpool-test-avatars-{}-{}",
            std::process::id(),
            TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&avatar_dir).unwrap();

        let (push, notifications) = PushBackend::new_mock();

        let state = AppState {
            db,
            jwt,
            google_oidc: None,
            vapid: None,
            live_hub: noordpool_backend::games::live_ws::LiveHub::new(),
            avatar_dir: Arc::new(avatar_dir),
            push,
        };
        let router = routes::app(state.clone());
        TestApp {
            state,
            router,
            test_db,
            base_url,
            notifications,
        }
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

    pub fn patch(&mut self, uri: impl Into<String>) -> RequestBuilder<'_> {
        RequestBuilder {
            inner: http::Request::patch(uri.into()),
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
        let resp = self.router.call(req).await.unwrap();
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
                "firstName": "Test",
                "lastName": "User",
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
                "firstName": format!("Test {:?}", role),
                "lastName": "",
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
            first_name: format!("Test {:?}", role),
            last_name: String::new(),
            player_id: None,
            team_id: None,
            roles: vec![Role::Player, role],
            exp: Timestamp::now() + 24.hour(),
        };
        self.state.jwt.encode_token(&claims).unwrap()
    }

    /// Create two teams and return (home_team_id, away_team_id).
    pub async fn create_teams(&mut self, token: &str, home: &str, away: &str) -> (String, String) {
        let res = self
            .post("/api/teams")
            .token(token)
            .json(json!({ "name": home }))
            .await;
        let home_id = res.json_value().await["id"].as_str().unwrap().to_string();

        let res = self
            .post("/api/teams")
            .token(token)
            .json(json!({ "name": away }))
            .await;
        let away_id = res.json_value().await["id"].as_str().unwrap().to_string();

        (home_id, away_id)
    }

    pub fn player_token_for(&mut self, player_id: Uuid, team_id: Option<Uuid>) -> String {
        let claims = Claims {
            sub: Uuid::new_v4(),
            email: format!("player-{}@example.com", player_id),
            first_name: "Test".to_string(),
            last_name: "Player".to_string(),
            player_id: Some(player_id),
            team_id,
            roles: vec![Role::Player],
            exp: Timestamp::now() + 24.hour(),
        };
        self.state.jwt.encode_token(&claims).unwrap()
    }
}
