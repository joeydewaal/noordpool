use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    #[column(variant = 1)]
    Scheduled,
    #[column(variant = 2)]
    Completed,
    #[column(variant = 3)]
    Cancelled,
}
