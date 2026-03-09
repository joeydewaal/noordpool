use jiff::Timestamp;
use serde::Serialize;
use toasty::HasMany;
use uuid::Uuid;

use super::UserRole;

#[derive(Debug, toasty::Model, Serialize)]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    pub email: String,

    pub password_hash: Option<String>,
    pub name: String,

    pub avatar_url: Option<String>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub roles: HasMany<UserRole>,
}
