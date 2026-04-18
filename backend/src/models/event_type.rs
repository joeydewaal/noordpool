use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    #[column(variant = 1)]
    Goal,
    #[column(variant = 2)]
    Assist,
    #[column(variant = 3)]
    YellowCard,
    #[column(variant = 4)]
    RedCard,
    #[column(variant = 5)]
    OwnGoal,
}
