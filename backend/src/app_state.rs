use axum::extract::FromRef;
use axum_security::{jwt::JwtContext, oidc::OidcContext};
use toasty::Db;

use crate::auth::{claims::Claims, google::GoogleHandler};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
    pub google_oidc: Option<OidcContext<GoogleHandler>>,
}

impl FromRef<AppState> for JwtContext<Claims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
