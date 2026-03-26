use jiff::Timestamp;
use serde::Serialize;
use toasty::{BelongsTo, HasMany, Model};
use uuid::Uuid;

use crate::models::{GameEvent, Position, Role, team::Team};

use super::UserRole;

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
    pub name: String,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub roles: HasMany<UserRole>,

    #[default(0)]
    pub shirt_number: i32,

    #[default(Position::Goalkeeper)]
    pub position: Position,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub game_events: HasMany<GameEvent>,

    #[index]
    #[serde(skip)]
    pub team_id: Option<Uuid>,

    #[belongs_to(key = team_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub team: BelongsTo<Option<Team>>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,

    #[default(true)]
    pub active: bool,
}

impl User {
    pub fn all_active() -> <User as Model>::Query {
        User::all().filter(User::fields().active().eq(true))
    }

    pub fn get_roles(&self) -> Vec<Role> {
        self.roles.get().iter().map(|r| r.role).collect()
    }
}
