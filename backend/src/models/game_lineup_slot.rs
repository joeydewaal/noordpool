use serde::Serialize;
use toasty::BelongsTo;
use uuid::Uuid;

use crate::models::Player;

use super::game_lineup::GameLineup;

#[derive(Debug, Serialize, toasty::Model, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameLineupSlot {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    #[serde(skip)]
    pub lineup_id: Uuid,

    #[belongs_to(key = lineup_id, references = id)]
    #[serde(skip)]
    pub game_lineup: BelongsTo<GameLineup>,

    #[index]
    #[serde(skip)]
    pub player_id: Uuid,

    #[belongs_to(key = player_id, references = id)]
    #[serde(skip)]
    pub player: BelongsTo<Player>,

    /// 0–10 = starting XI slot index (per formation definition), 11–17 = bench.
    pub slot: i32,

    #[default(false)]
    pub captain: bool,
}
