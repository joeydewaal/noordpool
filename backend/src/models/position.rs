use serde::{Deserialize, Serialize};
use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Embed)]
pub enum Position {
    #[column(variant = "goalkeeper")]
    #[serde(rename = "Keeper")]
    Goalkeeper,
    #[column(variant = "centre_back")]
    #[serde(rename = "Centrale verdediger")]
    CentreBack,
    #[column(variant = "left_back")]
    #[serde(rename = "Linksback")]
    LeftBack,
    #[column(variant = "right_back")]
    #[serde(rename = "Rechtsback")]
    RightBack,
    #[column(variant = "defensive_midfielder")]
    #[serde(rename = "Defensieve middenvelder")]
    DefensiveMidfielder,
    #[column(variant = "central_midfielder")]
    #[serde(rename = "Centrale middenvelder")]
    CentralMidfielder,
    #[column(variant = "attacking_midfielder")]
    #[serde(rename = "Aanvallende middenvelder")]
    AttackingMidfielder,
    #[column(variant = "left_winger")]
    #[serde(rename = "Linksvleugel")]
    LeftWinger,
    #[column(variant = "right_winger")]
    #[serde(rename = "Rechtsvleugel")]
    RightWinger,
    #[column(variant = "striker")]
    #[serde(rename = "Spits")]
    Striker,
}
