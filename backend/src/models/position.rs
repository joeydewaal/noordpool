use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
pub enum Position {
    #[column(variant = 1)]
    #[serde(rename = "Keeper")]
    Goalkeeper,
    #[column(variant = 2)]
    #[serde(rename = "Centrale verdediger")]
    CentreBack,
    #[column(variant = 3)]
    #[serde(rename = "Linksback")]
    LeftBack,
    #[column(variant = 4)]
    #[serde(rename = "Rechtsback")]
    RightBack,
    #[column(variant = 5)]
    #[serde(rename = "Defensieve middenvelder")]
    DefensiveMidfielder,
    #[column(variant = 6)]
    #[serde(rename = "Centrale middenvelder")]
    CentralMidfielder,
    #[column(variant = 7)]
    #[serde(rename = "Aanvallende middenvelder")]
    AttackingMidfielder,
    #[column(variant = 8)]
    #[serde(rename = "Linksvleugel")]
    LeftWinger,
    #[column(variant = 9)]
    #[serde(rename = "Rechtsvleugel")]
    RightWinger,
    #[column(variant = 10)]
    #[serde(rename = "Spits")]
    Striker,
}
