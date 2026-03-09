use jiff::Timestamp;
use serde::Serialize;
use toasty::{HasMany, Model};
use uuid::Uuid;

use crate::models::{MatchEvent, Position};

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

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub events: HasMany<MatchEvent>,

    #[default(0)]
    pub shirt_number: i32,

    #[default(Position::Goalkeeper)]
    pub position: Position,

    #[default(true)]
    pub active: bool,
}

impl User {
    pub fn all_active() -> <User as Model>::Query {
        User::all().filter(User::fields().active().eq(true))
    }
}
