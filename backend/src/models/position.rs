use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    #[column(variant = 1)]
    Goalkeeper,
    #[column(variant = 2)]
    CentreBack,
    #[column(variant = 3)]
    LeftBack,
    #[column(variant = 4)]
    RightBack,
    #[column(variant = 5)]
    DefensiveMidfielder,
    #[column(variant = 6)]
    CentralMidfielder,
    #[column(variant = 7)]
    AttackingMidfielder,
    #[column(variant = 8)]
    LeftWinger,
    #[column(variant = 9)]
    RightWinger,
    #[column(variant = 10)]
    Striker,
}
