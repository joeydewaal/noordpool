use std::collections::HashSet;

use axum::{Json, extract::State};
use serde::Serialize;
use uuid::Uuid;

use jiff::{Timestamp, ToSpan};

use crate::{
    app_state::AppState,
    error::AppError,
    models::{EventType, Player},
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntryResponse {
    pub player_id: Uuid,
    pub first_name: String,
    pub last_name: String,
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

#[tracing::instrument(skip(state))]
pub async fn leaderboard(
    State(state): State<AppState>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    let mut db = state.db;

    // Load all active players with their events and each event's game in one query
    let players = Player::all_active()
        .include(Player::fields().game_events().game())
        .exec(&mut db)
        .await?;

    let now = Timestamp::now();
    let match_duration = 90.minutes();
    let is_completed =
        |g: &crate::models::Game| !g.cancelled && g.date_time + match_duration <= now;

    // Build one leaderboard entry per player
    let entries: Vec<LeaderboardEntryResponse> = players
        .iter()
        .map(|player| {
            let mut goals = 0i32;
            let mut assists = 0i32;
            let mut yellow_cards = 0i32;
            let mut red_cards = 0i32;
            let mut game_ids: HashSet<uuid::Uuid> = HashSet::new();

            for event in player.game_events.get() {
                if !is_completed(event.game.get()) {
                    continue;
                }
                game_ids.insert(event.game_id);
                match event.event_type {
                    EventType::Goal => goals += 1,
                    EventType::Assist { .. } => assists += 1,
                    EventType::YellowCard => yellow_cards += 1,
                    EventType::RedCard => red_cards += 1,
                    EventType::OwnGoal => {}
                }
            }

            LeaderboardEntryResponse {
                player_id: player.id,
                first_name: player.first_name.clone(),
                last_name: player.last_name.clone(),
                shirt_number: player.shirt_number,
                appearances: game_ids.len() as i32,
                goals,
                assists,
                yellow_cards,
                red_cards,
                total_cards: yellow_cards + red_cards,
            }
        })
        .collect();

    let mut indices: Vec<usize> = (0..entries.len()).collect();

    indices.sort_unstable_by(|&a, &b| entries[b].goals.cmp(&entries[a].goals));
    let top_scorers: Vec<_> = indices.iter().map(|&i| entries[i].clone()).collect();

    indices.sort_unstable_by(|&a, &b| entries[b].assists.cmp(&entries[a].assists));
    let top_assisters: Vec<_> = indices.iter().map(|&i| entries[i].clone()).collect();

    indices.sort_unstable_by(|&a, &b| entries[b].total_cards.cmp(&entries[a].total_cards));
    let most_carded: Vec<_> = indices.iter().map(|&i| entries[i].clone()).collect();

    let response = LeaderboardResponse {
        top_scorers,
        top_assisters,
        most_carded,
    };
    Ok(Json(response))
}
