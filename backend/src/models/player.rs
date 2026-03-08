use serde::Serialize;
use uuid::Uuid;

use super::Position;

#[derive(Debug, Serialize, toasty::Model)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[key]
    #[auto]
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub shirt_number: i32,
    pub position: Position,
    #[default(true)]
    pub active: bool,
}
