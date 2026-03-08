use serde::Serialize;

use crate::models::{Role, User};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub roles: Vec<Role>,
}

impl UserResponse {
    pub fn from_user(user: &User, roles: &[Role]) -> Self {
        UserResponse {
            id: user.id.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
            avatar_url: user.avatar_url.clone(),
            roles: roles.to_vec(),
        }
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}
