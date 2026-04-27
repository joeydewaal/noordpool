use serde::{Deserialize, Serialize};
use toasty::Embed;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum EventType {
    #[column(variant = "goal")]
    Goal,
    #[column(variant = "assist")]
    Assist {
        #[serde(rename = "goalEventId")]
        goal_event_id: Uuid,
    },
    #[column(variant = "yellow_card")]
    YellowCard,
    #[column(variant = "red_card")]
    RedCard,
    #[column(variant = "own_goal")]
    OwnGoal,
}
