use toasty::BelongsTo;
use uuid::Uuid;

use super::{EventType, Game, Player};

#[derive(Debug, toasty::Model)]
pub struct MatchEvent {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub game_id: Uuid,
    #[belongs_to(key = game_id, references = id)]
    pub game: BelongsTo<Game>,
    #[index]
    pub player_id: Uuid,
    #[belongs_to(key = player_id, references = id)]
    pub player: BelongsTo<Player>,
    pub event_type: EventType,
    pub minute: i32,
}
