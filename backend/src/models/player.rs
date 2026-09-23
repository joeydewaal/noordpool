use jiff::Timestamp;
use serde::Serialize;
use toasty::{Deferred, schema::Model, stmt::List};
use uuid::Uuid;

use crate::models::{GameEvent, Position, User, team::Team};

#[derive(Debug, toasty::Model, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    #[serde(skip)]
    pub user_id: Option<Uuid>,

    pub first_name: String,

    pub last_name: String,

    pub shirt_number: i32,

    pub position: Position,

    #[default(true)]
    pub active: bool,

    #[index]
    pub team_id: Uuid,

    #[belongs_to(key = team_id, references = id)]
    #[serde(skip_serializing_if = "Deferred::is_unloaded")]
    pub team: Deferred<Team>,

    #[has_many]
    #[serde(skip_serializing_if = "Deferred::is_unloaded")]
    pub game_events: Deferred<Vec<GameEvent>>,

    #[has_one]
    #[serde(skip_serializing_if = "Deferred::is_unloaded")]
    pub user: Deferred<Option<User>>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
}

impl Player {
    pub fn all_active() -> <Player as Model>::Query<List<Player>> {
        Player::all().filter(Player::fields().active().eq(true))
    }
}
