use std::collections::HashSet;

use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    error::AppError,
    json::{LeaderboardEntryResponse, LeaderboardResponse},
    models::{EventType, GameStatus, User},
};

#[tracing::instrument(skip(state))]
pub async fn leaderboard(
    State(mut state): State<AppState>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    let db = &mut state.db;

    // Load all active players with their events and each event's game in one query
    let players = User::all_active()
        .include(User::fields().game_events().game())
        .exec(db)
        .await?;

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
                if event.game.get().status != GameStatus::Completed {
                    continue;
                }
                game_ids.insert(event.game_id);
                match event.event_type {
                    EventType::Goal => goals += 1,
                    EventType::Assist => assists += 1,
                    EventType::YellowCard => yellow_cards += 1,
                    EventType::RedCard => red_cards += 1,
                }
            }

            LeaderboardEntryResponse {
                player_id: player.id.to_string(),
                name: player.name.clone(),
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

    let mut top_scorers = entries.clone();
    top_scorers.sort_by(|a, b| b.goals.cmp(&a.goals));

    let mut top_assisters = entries.clone();
    top_assisters.sort_by(|a, b| b.assists.cmp(&a.assists));

    let mut most_carded = entries;
    most_carded.sort_by(|a, b| b.total_cards.cmp(&a.total_cards));

    let response = LeaderboardResponse {
        top_scorers,
        top_assisters,
        most_carded,
    };
    tracing::debug!("response:\n{:#?}", response);
    Ok(Json(response))
}
