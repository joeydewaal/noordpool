use serde::{Deserialize, Serialize};
use toasty::{BelongsTo, Embed};
use uuid::Uuid;

use super::User;

#[derive(Debug, toasty::Model, Serialize)]
pub struct UserRole {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    pub user_id: Uuid,

    #[belongs_to(key = user_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub user: BelongsTo<User>,

    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[column(variant = 1)]
    Admin,
    #[column(variant = 2)]
    Moderator,
    #[column(variant = 3)]
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
}
