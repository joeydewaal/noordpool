use serde::Serialize;
use toasty::BelongsTo;
use uuid::Uuid;

use crate::models::Player;

use super::{EventType, Game};

#[derive(Debug, Serialize, toasty::Model, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    #[serde(skip)]
    pub game_id: Uuid,

    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    #[belongs_to(key = game_id, references = id)]
    pub game: BelongsTo<Game>,

    #[index]
    #[serde(skip)]
    pub player_id: Uuid,

    #[belongs_to(key = player_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub player: BelongsTo<Player>,

    /// Snapshot of the player's team at the time the event was recorded.
    /// Used by compute_scores so score calculation doesn't depend on current team membership.
    #[serde(skip)]
    pub team_id: Uuid,

    pub event_type: EventType,

    pub minute: i32,
}
