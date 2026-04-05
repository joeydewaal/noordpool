use axum_security::rbac::RBAC;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub player_id: Option<Uuid>,
    pub roles: Vec<Role>,
    #[serde(with = "jiff::fmt::serde::timestamp::second::required")]
    pub exp: Timestamp,
}

impl RBAC for Role {
    type Resource = Claims;

    fn extract_roles(resource: &Claims) -> impl IntoIterator<Item = &Role> {
        &resource.roles
    }
}
