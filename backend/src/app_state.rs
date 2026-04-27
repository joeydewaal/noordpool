use std::{path::PathBuf, sync::Arc};

use axum::extract::FromRef;
use axum_security::{jwt::JwtContext, oidc::OidcContext};
use toasty::Db;

use crate::{
    auth::{claims::Claims, google::GoogleHandler},
    games::live_ws::LiveHub,
    push::PushBackend,
};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
    pub google_oidc: Option<OidcContext<GoogleHandler>>,
    pub live_hub: LiveHub,
    pub avatar_dir: Arc<PathBuf>,
    pub public_api_url: Option<Arc<String>>,
    pub push: PushBackend,
}

impl FromRef<AppState> for JwtContext<Claims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
