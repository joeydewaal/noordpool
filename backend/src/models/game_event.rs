use serde::Serialize;
use toasty::BelongsTo;
use uuid::Uuid;

use crate::models::User;

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

    #[serde(skip)]
    #[belongs_to(key = game_id, references = id)]
    pub game: BelongsTo<Game>,

    #[index]
    #[serde(skip)]
    pub user_id: Uuid,

    #[belongs_to(key = user_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub user: BelongsTo<User>,

    pub event_type: EventType,

    pub minute: i32,
}
