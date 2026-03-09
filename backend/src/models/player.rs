use serde::Serialize;
use toasty::{HasMany, Model};
use uuid::Uuid;

use super::{MatchEvent, Position};

#[derive(Debug, Serialize, toasty::Model)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[key]
    #[auto]
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub shirt_number: i32,
    pub position: Position,
    #[default(true)]
    pub active: bool,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub events: HasMany<MatchEvent>,
}

impl Player {
    pub fn all_active() -> <Player as Model>::Query {
        Player::all().filter(Player::fields().active().eq(true))
    }
}
