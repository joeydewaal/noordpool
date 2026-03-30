use jiff::Timestamp;
use serde::Serialize;
use toasty::HasMany;
use uuid::Uuid;

use super::UserRole;
use crate::models::Role;

#[derive(Debug, toasty::Model, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    pub email: String,

    #[serde(skip)]
    pub password_hash: Option<String>,

    pub first_name: String,

    pub last_name: String,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub roles: HasMany<UserRole>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
}

impl User {
    pub fn get_roles(&self) -> Vec<Role> {
        self.roles.get().iter().map(|r| r.role).collect()
    }
}
