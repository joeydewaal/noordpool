use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
pub enum Formation {
    #[serde(rename = "4-4-2")]
    #[column(variant = 1)]
    F442,

    #[serde(rename = "4-3-3")]
    #[column(variant = 2)]
    F433,

    #[serde(rename = "4-2-3-1")]
    #[column(variant = 3)]
    F4231,

    #[serde(rename = "3-5-2")]
    #[column(variant = 4)]
    F352,

    #[serde(rename = "5-3-2")]
    #[column(variant = 5)]
    F532,

    #[serde(rename = "4-1-4-1")]
    #[column(variant = 6)]
    F4141,
}

impl std::fmt::Display for Formation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Formation::F442 => "4-4-2",
            Formation::F433 => "4-3-3",
            Formation::F4231 => "4-2-3-1",
            Formation::F352 => "3-5-2",
            Formation::F532 => "5-3-2",
            Formation::F4141 => "4-1-4-1",
        };
        write!(f, "{s}")
    }
}
