use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "lowercase")]
pub enum HomeAway {
    #[column(variant = 1)]
    Home,
    #[column(variant = 2)]
    Away,
}
