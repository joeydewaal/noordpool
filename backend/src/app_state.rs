use axum::extract::FromRef;
use axum_security::jwt::JwtContext;
use toasty::Db;

use crate::auth::claims::Claims;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
}

impl FromRef<AppState> for JwtContext<Claims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
