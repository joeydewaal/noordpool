use axum::response::{IntoResponse, Redirect};
use axum_security::{
    jwt::JwtContext,
    oidc::{AfterLoginCookies, OidcHandler, OidcTokenResponse},
};
use jiff::{Timestamp, ToSpan};
use toasty::Db;

use super::claims::Claims;
use crate::models::{Player, User};

pub struct GoogleHandler {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
    pub frontend_url: String,
}

impl OidcHandler for GoogleHandler {
    async fn after_login(
        &self,
        token_res: OidcTokenResponse<'_>,
        _context: &mut AfterLoginCookies<'_>,
    ) -> impl IntoResponse {
        after_login(self, token_res, _context).await
    }
}

async fn after_login(
    this: &GoogleHandler,
    token_res: OidcTokenResponse<'_>,
    _context: &mut AfterLoginCookies<'_>,
) -> crate::Result<Redirect> {
    let mut db = this.db.clone();

    let email = token_res
        .claims
        .email
        .ok_or_else(|| Redirect::to(&format!("{}?error=no_email", this.frontend_url)))?;

    let first_name = token_res.claims.name.unwrap_or_default();
    let last_name = token_res.claims.family_name.unwrap_or_default();

    let opt_user = User::filter_by_email(email)
        .first()
        .exec(&mut db)
        .await
        .map_err(|_| Redirect::to(&format!("{}?error=db_error", this.frontend_url)))?;

    // Try to find existing user by email
    let (roles, user) = match opt_user {
        Some(user) => (user.get_roles(), user),
        None => {
            let user = toasty::create!(User {
                first_name: first_name,
                last_name: last_name,
                email: email,
            })
            .exec(&mut db)
            .await
            .map_err(|_| Redirect::to(&format!("{}?error=create_failed", this.frontend_url)))?;

            (vec![], user)
        }
    };

    let player_id = Player::filter_by_user_id(user.id)
        .first()
        .exec(&mut db)
        .await
        .ok()
        .flatten()
        .map(|p| p.id);

    let claims = Claims {
        sub: user.id,
        player_id,
        email: user.email.clone(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        roles: roles.clone(),
        exp: Timestamp::now() + 24.days(),
    };

    let token = this
        .jwt
        .encode_token(&claims)
        .map_err(|_| Redirect::to(&format!("{}?error=token_failed", this.frontend_url)))?;

    Ok(Redirect::to(&format!(
        "{}?token={}",
        this.frontend_url, token
    )))
}
