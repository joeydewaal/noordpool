use axum_security::rbac::RBAC;
use serde::{Deserialize, Serialize};

use crate::models::Role;

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
