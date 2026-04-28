use axum::extract::FromRef;
use axum_security::{jwt::JwtContext, oidc::OidcContext};
use toasty::Db;

use crate::{
    auth::{claims::Claims, google::GoogleHandler},
    games::live_ws::LiveHub,
    push::PushBackend,
    r2,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
    pub google_oidc: Option<OidcContext<GoogleHandler>>,
    pub live_hub: LiveHub,
    pub r2: r2::Backend,
    pub push: PushBackend,
}

impl FromRef<AppState> for JwtContext<Claims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
