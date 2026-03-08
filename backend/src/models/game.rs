use jiff::Timestamp;
use serde::Serialize;
use toasty::HasMany;
use uuid::Uuid;

use super::{HomeAway, MatchEvent, MatchStatus};

#[derive(Debug, Serialize, toasty::Model)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[key]
    #[auto]
    pub id: Uuid,
    pub opponent: String,
    pub location: String,
    pub date_time: Timestamp,
    pub home_away: HomeAway,
    #[default(MatchStatus::Scheduled)]
    pub status: MatchStatus,
    #[default(0)]
    pub home_score: i32,
    #[default(0)]
    pub away_score: i32,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[serde(skip)]
    #[has_many]
    pub events: HasMany<MatchEvent>,
}
