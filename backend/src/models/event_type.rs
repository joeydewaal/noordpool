use serde::{Deserialize, Serialize};
use toasty::Embed;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EventType {
    #[column(variant = 1)]
    Goal,
    #[column(variant = 2)]
    Assist {
        #[serde(rename = "goalEventId")]
        goal_event_id: Uuid,
    },
    #[column(variant = 3)]
    YellowCard,
    #[column(variant = 4)]
    RedCard,
    #[column(variant = 5)]
    OwnGoal,
}
