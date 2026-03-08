use serde::Serialize;
use toasty::BelongsTo;
use uuid::Uuid;

use super::{EventType, Game, Player};

#[derive(Debug, Serialize, toasty::Model)]
#[serde(rename_all = "camelCase")]
pub struct MatchEvent {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    #[serde(rename = "matchId")]
    pub game_id: Uuid,
    #[serde(skip)]
    #[belongs_to(key = game_id, references = id)]
    pub game: BelongsTo<Game>,
    #[index]
    pub player_id: Uuid,
    #[serde(skip)]
    #[belongs_to(key = player_id, references = id)]
    pub player: BelongsTo<Player>,
    pub event_type: EventType,
    pub minute: i32,
}
