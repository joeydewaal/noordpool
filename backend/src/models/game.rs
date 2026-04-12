use jiff::{Span, Timestamp};
use serde::Serialize;
use toasty::{BelongsTo, HasMany};
use uuid::Uuid;

use super::GameEvent;
use crate::models::team::Team;

/// How long after kickoff we still consider a match "live". Covers
/// 2x 45 min halves + halftime + stoppage + buffer.
pub const MATCH_DURATION_MINUTES: i64 = 120;

#[derive(Debug, Serialize, toasty::Model, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    pub home_team_id: Uuid,

    #[belongs_to(key = home_team_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub home_team: BelongsTo<Team>,

    #[index]
    pub away_team_id: Uuid,

    #[belongs_to(key = away_team_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub away_team: BelongsTo<Team>,

    pub location: String,

    pub date_time: Timestamp,

    #[default(false)]
    pub cancelled: bool,

    #[default(0)]
    pub home_score: i32,

    #[default(0)]
    pub away_score: i32,

    /// Monotonic counter bumped on every mutation. Used as the ETag
    /// value for the live polling endpoint.
    #[default(0)]
    pub version: i64,

    #[default(Timestamp::now())]
    pub updated_at: Timestamp,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub events: HasMany<GameEvent>,
}

impl Game {
    /// Returns true when `now` is inside `[date_time, date_time + MATCH_DURATION]`
    /// and the game is not cancelled.
    pub fn is_live(&self, now: Timestamp) -> bool {
        if self.cancelled {
            return false;
        }
        let Ok(end) = self
            .date_time
            .checked_add(Span::new().minutes(MATCH_DURATION_MINUTES))
        else {
            return false;
        };
        self.date_time <= now && now <= end
    }

    /// Server-derived status string. Mirrors the frontend `GameStatus` union.
    pub fn derived_status(&self, now: Timestamp) -> &'static str {
        if self.cancelled {
            "cancelled"
        } else if now < self.date_time {
            "scheduled"
        } else if self.is_live(now) {
            "live"
        } else {
            "finished"
        }
    }
}
