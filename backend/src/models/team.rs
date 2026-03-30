use serde::Serialize;
use toasty::HasMany;
use uuid::Uuid;

use crate::models::Player;

#[derive(Debug, toasty::Model, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    #[key]
    #[auto]
    id: Uuid,

    #[unique]
    name: String,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    players: HasMany<Player>,
}
