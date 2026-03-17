use std::collections::{HashMap, HashSet};

use axum::{Json, extract::State};

use crate::{
    app_state::AppState,
    error::AppError,
    json::{LeaderboardEntryResponse, LeaderboardResponse},
    models::{EventType, GameEvent, GameStatus, User},
};

#[derive(Default)]
struct PlayerStats {
    goals: i32,
    assists: i32,
    yellow_cards: i32,
    red_cards: i32,
    game_ids: HashSet<uuid::Uuid>,
}

pub async fn leaderboard(
    State(mut state): State<AppState>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    let db = &mut state.db;

    // Get all events
    let events = GameEvent::all()
        .include(GameEvent::fields().game())
        .exec(db)
        .await?;

    // Aggregate stats per player (only for events in completed games)
    let mut stats_map: HashMap<uuid::Uuid, PlayerStats> = HashMap::new();

    for event in &events {
        if event.game.get().status != GameStatus::Completed {
            continue;
        }

        let entry = stats_map.entry(event.user_id).or_default();
        entry.game_ids.insert(event.game_id);
        match event.event_type {
            EventType::Goal => entry.goals += 1,
            EventType::Assist => entry.assists += 1,
            EventType::YellowCard => entry.yellow_cards += 1,
            EventType::RedCard => entry.red_cards += 1,
        }
    }

    // Get active players
    let players = User::all_active().exec(db).await?;
    let player_map: HashMap<uuid::Uuid, &User> = players.iter().map(|p| (p.id, p)).collect();

    // Build entries
    let mut entries: Vec<LeaderboardEntryResponse> = stats_map
        .iter()
        .filter_map(|(player_id, ps)| {
            let player = player_map.get(player_id)?;
            Some(LeaderboardEntryResponse {
                player_id: player_id.to_string(),
                name: player.name.clone(),
                shirt_number: player.shirt_number,
                appearances: ps.game_ids.len() as i32,
                goals: ps.goals,
                assists: ps.assists,
                yellow_cards: ps.yellow_cards,
                red_cards: ps.red_cards,
                total_cards: ps.yellow_cards + ps.red_cards,
            })
        })
        .collect();

    // Also include active players with no events
    for player in &players {
        if player.active && !stats_map.contains_key(&player.id) {
            entries.push(LeaderboardEntryResponse {
                player_id: player.id.to_string(),
                name: player.name.clone(),
                shirt_number: player.shirt_number,
                appearances: 0,
                goals: 0,
                assists: 0,
                yellow_cards: 0,
                red_cards: 0,
                total_cards: 0,
            });
        }
    }

    let mut top_scorers = entries.clone();
    top_scorers.sort_by(|a, b| b.goals.cmp(&a.goals));

    let mut top_assisters = entries.clone();
    top_assisters.sort_by(|a, b| b.assists.cmp(&a.assists));

    let mut most_carded = entries;
    most_carded.sort_by(|a, b| b.total_cards.cmp(&a.total_cards));

    Ok(Json(LeaderboardResponse {
        top_scorers,
        top_assisters,
        most_carded,
    }))
}
