use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[column(variant = "admin")]
    Admin,
    #[column(variant = "moderator")]
    Moderator,
    #[column(variant = "player")]
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
