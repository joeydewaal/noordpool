use axum::extract::FromRef;
use axum_security::{jwt::JwtContext, oauth2::OAuth2Context};
use toasty::Db;

use crate::auth::{claims::Claims, google::GoogleHandler};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
    pub google_oauth2: Option<OAuth2Context<GoogleHandler>>,
}

impl FromRef<AppState> for JwtContext<Claims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
