use jiff::Timestamp;
use serde::Serialize;
use toasty::{BelongsTo, HasMany};
use uuid::Uuid;

use super::{Formation, Game, game_lineup_slot::GameLineupSlot};

#[derive(Debug, Serialize, toasty::Model, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameLineup {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    #[serde(skip)]
    pub game_id: Uuid,

    #[belongs_to(key = game_id, references = id)]
    #[serde(skip)]
    pub game: BelongsTo<Game>,

    #[index]
    #[serde(skip)]
    pub team_id: Uuid,

    pub formation: Formation,

    #[auto]
    pub updated_at: Timestamp,

    #[has_many]
    #[serde(skip)]
    pub slots: HasMany<GameLineupSlot>,
}
