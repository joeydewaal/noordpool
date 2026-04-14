use serde::Serialize;
use toasty::HasMany;
use uuid::Uuid;

use crate::models::Player;

#[derive(Debug, toasty::Model, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    #[key]
    #[auto]
    pub id: Uuid,

    #[unique]
    pub name: String,

    #[has_many]
    #[serde(skip_serializing_if = "HasMany::is_unloaded")]
    pub players: HasMany<Player>,
}
