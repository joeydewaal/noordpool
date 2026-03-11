use jiff::Timestamp;
use serde::Serialize;
use toasty::HasMany;
use uuid::Uuid;

use super::{HomeAway, GameEvent, GameStatus};

#[derive(Debug, Serialize, toasty::Model, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[key]
    #[auto]
    pub id: Uuid,

    pub opponent: String,

    pub location: String,

    pub date_time: Timestamp,

    pub home_away: HomeAway,

    #[default(GameStatus::Scheduled)]
    pub status: GameStatus,

    #[default(0)]
    pub home_score: i32,

    #[default(0)]
    pub away_score: i32,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub events: HasMany<GameEvent>,
}
