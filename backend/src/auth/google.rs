use axum::response::{IntoResponse, Redirect};
use axum_security::{
    jwt::JwtContext,
    oidc::{AfterLoginCookies, OidcHandler, OidcTokenResponse},
};
use jiff::{Timestamp, ToSpan};
use toasty::Db;

use super::claims::Claims;
use crate::models::{Player, Role, User};

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
        let email = match token_res.claims.email {
            Some(email) => email.to_string(),
            None => return Redirect::to(&format!("{}?error=no_email", self.frontend_url)),
        };

        let full_name = token_res
            .claims
            .name
            .map(|n| n.to_string())
            .unwrap_or_else(|| email.clone());
        let (first_name, last_name) = match full_name.find(' ') {
            Some(idx) => (
                full_name[..idx].to_string(),
                full_name[idx + 1..].to_string(),
            ),
            None => (full_name, String::new()),
        };

        let mut db = self.db.clone();

        // Try to find existing user by email
        let (user, roles) = match User::filter_by_email(&email)
            .include(User::fields().roles())
            .first()
            .exec(&mut db)
            .await
        {
            Ok(Some(user)) => {
                let roles: Vec<Role> = user.roles.get().iter().map(|r| r.role).collect();
                (user, roles)
            }
            Ok(None) => {
                let user = match toasty::create!(User {
                    first_name: first_name.clone(),
                    last_name: last_name.clone(),
                    email: email.clone(),
                    roles: [{ role: Role::Player }]
                })
                .exec(&mut db)
                .await
                {
                    Ok(user) => user,
                    Err(_) => {
                        return Redirect::to(&format!("{}?error=create_failed", self.frontend_url));
                    }
                };

                (user, vec![Role::Player])
            }
            Err(_) => {
                return Redirect::to(&format!("{}?error=db_error", self.frontend_url));
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

        match self.jwt.encode_token(&claims) {
            Ok(token) => Redirect::to(&format!("{}?token={}", self.frontend_url, token)),
            Err(_) => Redirect::to(&format!("{}?error=token_failed", self.frontend_url)),
        }
    }
}
