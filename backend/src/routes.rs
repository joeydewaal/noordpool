use axum::Router;
use axum_security::oauth2::OAuth2Ext;
use tower_http::cors::CorsLayer;

use crate::{app_state::AppState, auth, events, matches, players, stats};

pub fn app(state: AppState) -> Router {
    let mut app = Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/players", players::router())
        .nest("/api/matches", matches::router())
        .nest("/api/matches/{match_id}/events", events::router())
        .nest("/api/stats", stats::router())
        .layer(state.jwt.clone());

    if let Some(google_oidc) = state.google_oauth2.clone() {
        app = app.with_oauth2(google_oidc);
    }

    app.layer(CorsLayer::permissive()).with_state(state)
}
