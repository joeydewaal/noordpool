use axum_security::rbac::RBAC;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Moderator,
    Player,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Moderator => "moderator",
            Role::Player => "player",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Role::Admin),
            "moderator" => Some(Role::Moderator),
            "player" => Some(Role::Player),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<Role>,
    pub exp: u64,
}

impl RBAC for Role {
    type Resource = Claims;

    fn extract_roles(resource: &Claims) -> impl IntoIterator<Item = &Role> {
        &resource.roles
    }
}
