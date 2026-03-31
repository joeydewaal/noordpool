use jiff::Timestamp;
use serde::Serialize;
use toasty::{BelongsTo, HasMany, HasOne, schema::Model};
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
    #[serde(skip)]
    pub team_id: Option<Uuid>,

    #[belongs_to(key = team_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub team: BelongsTo<Option<Team>>,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub game_events: HasMany<GameEvent>,

    #[has_one]
    #[serde(skip_serializing_if = "HasOne::is_unloaded")]
    pub player: HasOne<Option<User>>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
}

impl Player {
    pub fn all_active() -> <Player as Model>::Query {
        Player::all().filter(Player::fields().active().eq(true))
    }
}
