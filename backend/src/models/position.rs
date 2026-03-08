use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    #[column(variant = 1)]
    Goalkeeper,
    #[column(variant = 2)]
    Defender,
    #[column(variant = 3)]
    Midfielder,
    #[column(variant = 4)]
    Forward,
}
