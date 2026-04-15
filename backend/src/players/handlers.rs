use std::collections::{HashMap, HashSet};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_security::{
    jwt::Jwt,
    rbac::{requires, requires_any},
};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use jiff::ToSpan;

use crate::{
    app_state::AppState,
    auth::claims::Claims,
    error::AppError,
    models::{EventType, Game, Player, Position, Role, Team},
};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamSummary {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlayerRequest {
    pub first_name: String,
    pub last_name: String,
    pub shirt_number: i32,
    pub position: Position,
    pub team_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlayerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub shirt_number: Option<i32>,
    pub position: Option<Position>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerGoalMatchResponse {
    pub game_id: Uuid,
    pub home_team: TeamSummary,
    pub away_team: TeamSummary,
    pub date_time: Timestamp,
    pub home_score: i32,
    pub away_score: i32,
    pub minutes: Vec<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTimelineEntry {
    pub game_id: Uuid,
    pub home_team: TeamSummary,
    pub away_team: TeamSummary,
    pub date_time: Timestamp,
    pub goals: i32,
    pub assists: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
    pub cumulative_goals: i32,
    pub cumulative_assists: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsResponse {
    pub player_id: Uuid,
    pub appearances: usize,
    pub goals: i32,
    pub assists: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
    pub goal_matches: Vec<PlayerGoalMatchResponse>,
    pub game_timeline: Vec<GameTimelineEntry>,
}

#[tracing::instrument(skip(state))]
pub async fn list(State(mut state): State<AppState>) -> Result<Json<Vec<Player>>, AppError> {
    let players = Player::all_active()
        .include(Player::fields().player())
        .exec(&mut state.db)
        .await?;
    Ok(Json(players))
}

#[tracing::instrument(skip(state), fields(player_id = %id))]
pub async fn get_one(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Player>, AppError> {
    let player = Player::filter_by_id(id)
        .include(Player::fields().player())
        .get(&mut state.db)
        .await?;
    Ok(Json(player))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn create(
    State(mut state): State<AppState>,
    Json(body): Json<CreatePlayerRequest>,
) -> Result<Json<Player>, AppError> {
    tracing::info!("players::create");
    let mut create = Player::create()
        .first_name(body.first_name)
        .last_name(body.last_name)
        .shirt_number(body.shirt_number)
        .position(body.position);

    if let Some(team_id) = body.team_id {
        create = create.team_id(team_id);
    }

    let player = create.exec(&mut state.db).await?;

    Ok(Json(player))
}

pub async fn update(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
    Jwt(claims): Jwt<Claims>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<Json<Player>, AppError> {
    tracing::info!(player_id = %id, "players::update");

    let is_manager = claims.roles.contains(&Role::Admin) || claims.roles.contains(&Role::Moderator);
    let is_own_player = claims.player_id == Some(id);

    if !is_manager && !is_own_player {
        return Err(AppError::forbidden("Not allowed to update this player"));
    }

    let mut player = Player::get_by_id(&mut state.db, id).await?;
    let mut player_update = player.update();
    let mut has_changes = false;

    // Admins/moderators can update all fields; players can only update shirt number and position
    if is_manager {
        if let Some(first_name) = body.first_name {
            player_update.set_first_name(first_name);
            has_changes = true;
        }
        if let Some(last_name) = body.last_name {
            player_update.set_last_name(last_name);
            has_changes = true;
        }
        if let Some(active) = body.active {
            player_update.set_active(active);
            has_changes = true;
        }
    }

    if let Some(shirt_number) = body.shirt_number {
        player_update.set_shirt_number(shirt_number);
        has_changes = true;
    }
    if let Some(position) = body.position {
        player_update.set_position(position);
        has_changes = true;
    }

    if has_changes {
        player_update.exec(&mut state.db).await?;
    }
    Ok(Json(player))
}

#[requires(Role::Admin)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    tracing::info!(player_id = %id, "players::delete");
    let mut db = state.db;

    let mut player = Player::get_by_id(&mut db, id).await?;

    let mut update = player.update();
    update.set_active(false);
    update.exec(&mut db).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(state), fields(player_id = %id))]
pub async fn stats(
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlayerStatsResponse>, AppError> {
    let db = &mut state.db;

    let player = Player::filter_by_id(id)
        .include(Player::fields().game_events().game())
        .get(db)
        .await?;

    // Team relations aren't loaded through the event→game include chain,
    // so build a lookup map from all teams.
    let all_teams = Team::all().exec(db).await?;
    let team_map: HashMap<Uuid, TeamSummary> = all_teams
        .into_iter()
        .map(|t| {
            (
                t.id,
                TeamSummary {
                    id: t.id,
                    name: t.name,
                },
            )
        })
        .collect();

    let events = player.game_events.get();

    let now = Timestamp::now();
    let match_duration = 90.minutes();
    let is_completed = |g: &Game| !g.cancelled && (g.date_time + match_duration) <= now;

    let game_ids: Vec<Uuid> = events
        .iter()
        .filter(|e| is_completed(e.game.get()))
        .map(|e| e.game_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let mut goals = 0i32;
    let mut assists = 0i32;
    let mut yellow_cards = 0i32;
    let mut red_cards = 0i32;
    let mut goal_map: HashMap<Uuid, (Vec<i32>, &Game)> = HashMap::new();

    for event in events {
        match event.event_type {
            EventType::Goal => {
                goals += 1;
                let game = event.game.get();
                goal_map
                    .entry(event.game_id)
                    .or_insert_with(|| (Vec::new(), game))
                    .0
                    .push(event.minute);
            }
            EventType::Assist => assists += 1,
            EventType::YellowCard => yellow_cards += 1,
            EventType::RedCard => red_cards += 1,
        }
    }

    let unknown = TeamSummary {
        id: Uuid::nil(),
        name: "?".into(),
    };
    let mut goal_matches: Vec<PlayerGoalMatchResponse> = goal_map
        .into_values()
        .map(|(mut minutes, game)| {
            minutes.sort_unstable();
            PlayerGoalMatchResponse {
                game_id: game.id,
                home_team: team_map
                    .get(&game.home_team_id)
                    .cloned()
                    .unwrap_or_else(|| unknown.clone()),
                away_team: team_map
                    .get(&game.away_team_id)
                    .cloned()
                    .unwrap_or_else(|| unknown.clone()),
                date_time: game.date_time,
                home_score: game.home_score,
                away_score: game.away_score,
                minutes,
            }
        })
        .collect();
    goal_matches.sort_by(|a, b| b.date_time.cmp(&a.date_time));

    // Build per-game timeline for charts
    let mut timeline_map: HashMap<Uuid, (i32, i32, i32, i32, &Game)> = HashMap::new();
    for event in events {
        let game = event.game.get();
        if !is_completed(game) {
            continue;
        }
        let entry = timeline_map
            .entry(event.game_id)
            .or_insert((0, 0, 0, 0, game));
        match event.event_type {
            EventType::Goal => entry.0 += 1,
            EventType::Assist => entry.1 += 1,
            EventType::YellowCard => entry.2 += 1,
            EventType::RedCard => entry.3 += 1,
        }
    }

    let mut timeline_entries: Vec<_> = timeline_map.into_iter().collect();
    timeline_entries.sort_by(|a, b| a.1.4.date_time.cmp(&b.1.4.date_time));

    let mut cum_goals = 0i32;
    let mut cum_assists = 0i32;
    let game_timeline: Vec<GameTimelineEntry> = timeline_entries
        .into_iter()
        .map(|(game_id, (g, a, yc, rc, game))| {
            cum_goals += g;
            cum_assists += a;
            GameTimelineEntry {
                game_id,
                home_team: team_map
                    .get(&game.home_team_id)
                    .cloned()
                    .unwrap_or_else(|| unknown.clone()),
                away_team: team_map
                    .get(&game.away_team_id)
                    .cloned()
                    .unwrap_or_else(|| unknown.clone()),
                date_time: game.date_time,
                goals: g,
                assists: a,
                yellow_cards: yc,
                red_cards: rc,
                cumulative_goals: cum_goals,
                cumulative_assists: cum_assists,
            }
        })
        .collect();

    let response = PlayerStatsResponse {
        player_id: id,
        appearances: game_ids.len(),
        goals,
        assists,
        yellow_cards,
        red_cards,
        goal_matches,
        game_timeline,
    };

    Ok(Json(response))
}
