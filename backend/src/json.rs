use serde::{Deserialize, Serialize};

use crate::models::{
    EventType, Game, HomeAway, MatchEvent, MatchStatus, Player, Position, Role, User,
};

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub id: String,
    pub user_id: Option<String>,
    pub name: String,
    pub shirt_number: i32,
    pub position: Position,
    pub active: bool,
}

impl PlayerResponse {
    pub fn from_player(player: &Player) -> Self {
        PlayerResponse {
            id: player.id.to_string(),
            user_id: player.user_id.map(|u| u.to_string()),
            name: player.name.clone(),
            shirt_number: player.shirt_number,
            position: player.position,
            active: player.active,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlayerRequest {
    pub name: String,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchResponse {
    pub id: String,
    pub opponent: String,
    pub location: String,
    pub date_time: String,
    pub home_away: HomeAway,
    pub status: MatchStatus,
    pub home_score: i32,
    pub away_score: i32,
    pub created_at: String,
}

impl MatchResponse {
    pub fn from_game(game: &Game) -> Self {
        MatchResponse {
            id: game.id.to_string(),
            opponent: game.opponent.clone(),
            location: game.location.clone(),
            date_time: game.date_time.to_string(),
            home_away: game.home_away,
            status: game.status,
            home_score: game.home_score,
            away_score: game.away_score,
            created_at: game.created_at.to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMatchRequest {
    pub opponent: String,
    pub location: String,
    pub date_time: String,
    pub home_away: HomeAway,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMatchRequest {
    pub opponent: Option<String>,
    pub location: Option<String>,
    pub date_time: Option<String>,
    pub home_away: Option<HomeAway>,
    pub status: Option<MatchStatus>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

// ── Events ──

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchEventResponse {
    pub id: String,
    pub match_id: String,
    pub player_id: String,
    pub event_type: EventType,
    pub minute: i32,
}

impl MatchEventResponse {
    pub fn from_event(event: &MatchEvent) -> Self {
        MatchEventResponse {
            id: event.id.to_string(),
            match_id: event.game_id.to_string(),
            player_id: event.player_id.to_string(),
            event_type: event.event_type,
            minute: event.minute,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMatchEventRequest {
    pub player_id: String,
    pub event_type: EventType,
    pub minute: i32,
}

// ── Stats ──

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsResponse {
    pub player_id: String,
    pub appearances: i32,
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
