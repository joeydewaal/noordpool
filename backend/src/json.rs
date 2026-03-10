use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{EventType, HomeAway, MatchStatus, Position, Role, User};

// ── Auth ──

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub roles: Vec<Role>,
}

impl UserResponse {
    pub fn from_user(user: &User, roles: &[Role]) -> Self {
        UserResponse {
            id: user.id.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
            avatar_url: user.avatar_url.clone(),
            roles: roles.to_vec(),
        }
    }
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

// ── Players ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlayerRequest {
    pub name: String,
    pub email: String,
    pub shirt_number: i32,
    pub position: Position,

}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerRequest {
    pub name: Option<String>,
    pub shirt_number: Option<i32>,
    pub position: Option<Position>,
    pub active: Option<bool>,
}

// ── Matches ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMatchRequest {
    pub opponent: String,
    pub location: String,
    pub date_time: Timestamp,
    pub home_away: HomeAway,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMatchRequest {
    pub opponent: Option<String>,
    pub location: Option<String>,
    pub date_time: Option<Timestamp>,
    pub home_away: Option<HomeAway>,
    pub status: Option<MatchStatus>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

// ── Events ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMatchEventRequest {
    pub player_id: Uuid,
    pub event_type: EventType,
    pub minute: i32,
}

// ── Stats ──

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsResponse {
    pub player_id: String,
    pub appearances: usize,
    pub goals: i32,
    pub assists: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntryResponse {
    pub player_id: String,
    pub name: String,
    pub shirt_number: i32,
    pub appearances: i32,
    pub goals: i32,
    pub assists: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
    pub total_cards: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardResponse {
    pub top_scorers: Vec<LeaderboardEntryResponse>,
    pub top_assisters: Vec<LeaderboardEntryResponse>,
    pub most_carded: Vec<LeaderboardEntryResponse>,
}
