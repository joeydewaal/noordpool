use serde::{Deserialize, Serialize};
use toasty::Embed;
use uuid::Uuid;

use super::User;

#[derive(Debug, toasty::Model)]
pub struct UserRole {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub user_id: uuid::Uuid,
    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<User>,
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
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
