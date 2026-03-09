use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_security::{
    jwt::JwtContext,
    oauth2::{AfterLoginCookies, OAuth2Handler, TokenResponse},
};
use toasty::Db;

use super::{claims::Claims, password};
use crate::models::{Role, User};

pub struct GoogleHandler {
    pub db: Db,
    pub jwt: JwtContext<Claims>,
    pub frontend_url: String,
}

impl OAuth2Handler for GoogleHandler {
    async fn after_login(
        &self,
        token_res: TokenResponse,
        _context: &mut AfterLoginCookies<'_>,
    ) -> impl IntoResponse {
        todo!();
        StatusCode::OK
        // let email = match token_res.claims.email {
        //     Some(email) => email.to_string(),
        //     None => return Redirect::to(&format!("{}?error=no_email", self.frontend_url)),
        // };

        // let name = token_res
        //     .claims
        //     .name
        //     .map(|n| n.to_string())
        //     .unwrap_or_else(|| email.clone());

        // let picture = token_res.claims.picture.map(|p| p.to_string());

        // let mut db = self.db.clone();

        // // Try to find existing user by email
        // let (user, roles) = match User::filter_by_email(&email)
        //     .include(User::fields().roles())
        //     .first(&mut db)
        //     .await
        // {
        //     Ok(Some(user)) => {
        //         let roles: Vec<Role> = user.roles.get().iter().map(|r| r.role).collect();
        //         (user, roles)
        //     }
        //     Ok(None) => {
        //         // Create new user with a random password hash (can't login with password)
        //         let password_hash =
        //             password::hash_password(&uuid::Uuid::new_v4().to_string()).unwrap();

        //         let user = match toasty::create!(User, {
        //             name: name.clone(),
        //             email: email.clone(),
        //             password_hash: password_hash,
        //             avatar_url: picture,
        //             roles: [{ role: Role::Player }]
        //         })
        //         .exec(&mut db)
        //         .await
        //         {
        //             Ok(user) => user,
        //             Err(_) => {
        //                 return Redirect::to(&format!("{}?error=create_failed", self.frontend_url));
        //             }
        //         };

        //         (user, vec![Role::Player])
        //     }
        //     Err(_) => {
        //         return Redirect::to(&format!("{}?error=db_error", self.frontend_url));
        //     }
        // };

        // let claims = Claims {
        //     sub: user.id.to_string(),
        //     email: user.email.clone(),
        //     name: user.name.clone(),
        //     roles: roles.clone(),
        //     exp: axum_security::jwt::get_current_timestamp() + 24 * 60 * 60,
        // };

        // match self.jwt.encode_token(&claims) {
        //     Ok(token) => Redirect::to(&format!("{}?token={}", self.frontend_url, token)),
        //     Err(_) => Redirect::to(&format!("{}?error=token_failed", self.frontend_url)),
        // }
    }
}
