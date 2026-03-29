use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{EventType, Game, GameStatus, HomeAway, Position, User};

// ── Player goal matches ──

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerGoalMatchResponse {
    pub game_id: String,
    pub opponent: String,
    pub date_time: Timestamp,
    pub home_away: HomeAway,
    pub home_score: i32,
    pub away_score: i32,
    pub status: GameStatus,
    pub minutes: Vec<i32>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

// ── Auth ──

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMatchResponse {
    pub id: Uuid,
    pub name: String,
    pub shirt_number: i32,
    pub position: Position,
}

// ── Players ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlayerRequest {
    pub name: String,
    pub email: Option<String>,
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

// ── Games ──

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamesSummaryResponse {
    pub upcoming: Vec<Game>,
    pub recent: Vec<Game>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameRequest {
    pub opponent: String,
    pub location: String,
    pub date_time: Timestamp,
    pub home_away: HomeAway,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGameRequest {
    pub opponent: Option<String>,
    pub location: Option<String>,
    pub date_time: Option<Timestamp>,
    pub home_away: Option<HomeAway>,
    pub status: Option<GameStatus>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

// ── Events ──

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameEventRequest {
    pub player_id: Uuid,
    pub event_type: EventType,
    pub minute: i32,
}

// ── Stats ──

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsResponse {
    pub player_id: String,
    pub appearances: usize,
    pub goals: i32,
    pub assists: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
    pub goal_matches: Vec<PlayerGoalMatchResponse>,
}

#[derive(Clone, Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardResponse {
    pub top_scorers: Vec<LeaderboardEntryResponse>,
    pub top_assisters: Vec<LeaderboardEntryResponse>,
    pub most_carded: Vec<LeaderboardEntryResponse>,
}
