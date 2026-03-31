use jiff::Timestamp;
use serde::Serialize;
use toasty::BelongsTo;
use uuid::Uuid;

use crate::models::{Player, Role};

#[derive(Debug, toasty::Model, Clone)]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    pub email: String,

    pub password_hash: Option<String>,

    pub first_name: String,

    pub last_name: String,

    #[index]
    pub player_id: Option<Uuid>,

    #[belongs_to(key = player_id, references = id)]
    pub player: BelongsTo<Option<Player>>,

    pub avatar_url: Option<String>,

    #[default(false)]
    pub is_admin: bool,

    #[default(false)]
    pub is_moderator: bool,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
}

impl User {
    pub fn get_roles(&self) -> Vec<Role> {
        let mut roles = Vec::new();

        if self.is_admin {
            roles.push(Role::Admin);
        }

        if self.is_moderator {
            roles.push(Role::Moderator);
        }

        if self.player_id.is_some() {
            roles.push(Role::Player);
        }
        roles
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSer<'a> {
    pub id: &'a Uuid,
    pub email: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub avatar_url: &'a Option<String>,
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub player: &'a BelongsTo<Option<Player>>,
    pub is_admin: bool,
    pub is_moderator: bool,
    pub created_at: Timestamp,
    pub roles: Vec<Role>,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        UserSer {
            id: &self.id,
            email: &self.email,
            first_name: &self.first_name,
            last_name: &self.last_name,
            avatar_url: &self.avatar_url,
            player: &self.player,
            is_admin: self.is_admin,
            is_moderator: self.is_moderator,
            created_at: self.created_at,
            roles: self.get_roles(),
        }
        .serialize(serializer)
    }
}
