use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
};
use axum_security::rbac::requires_any;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Formation, GameLineup, GameLineupSlot, Player, Role},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineupPlayerResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub shirt_number: i32,
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineupSlotResponse {
    pub slot: i32,
    pub captain: bool,
    pub player: LineupPlayerResponse,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLineupResponse {
    pub id: Uuid,
    pub game_id: Uuid,
    pub formation: Formation,
    pub published: bool,
    pub updated_at: Timestamp,
    pub slots: Vec<LineupSlotResponse>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotRequest {
    pub slot: i32,
    pub player_id: Uuid,
    #[serde(default)]
    pub captain: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLineupRequest {
    pub formation: Formation,
    pub slots: Vec<SlotRequest>,
}

async fn build_response(
    db: &mut toasty::Db,
    lineup: GameLineup,
) -> Result<GameLineupResponse, AppError> {
    let slots: Vec<GameLineupSlot> = GameLineupSlot::filter_by_lineup_id(lineup.id)
        .exec(db)
        .await?;

    let player_ids: Vec<Uuid> = slots.iter().map(|s| s.player_id).collect();

    let players_with_users: Vec<Player> = Player::all()
        .include(Player::fields().user())
        .exec(db)
        .await?
        .into_iter()
        .filter(|p| player_ids.contains(&p.id))
        .collect();

    let player_map: HashMap<Uuid, Player> =
        players_with_users.into_iter().map(|p| (p.id, p)).collect();

    let mut slot_responses: Vec<LineupSlotResponse> = slots
        .into_iter()
        .filter_map(|slot| {
            let player = player_map.get(&slot.player_id)?;
            let avatar_url = player
                .user
                .get()
                .as_ref()
                .and_then(|u| u.avatar_url.clone());
            Some(LineupSlotResponse {
                slot: slot.slot,
                captain: slot.captain,
                player: LineupPlayerResponse {
                    id: player.id,
                    first_name: player.first_name.clone(),
                    last_name: player.last_name.clone(),
                    shirt_number: player.shirt_number,
                    avatar_url,
                },
            })
        })
        .collect();

    slot_responses.sort_by_key(|s| s.slot);

    Ok(GameLineupResponse {
        id: lineup.id,
        game_id: lineup.game_id,
        formation: lineup.formation,
        published: lineup.published,
        updated_at: lineup.updated_at,
        slots: slot_responses,
    })
}

pub async fn get_lineup(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameLineupResponse>, AppError> {
    let mut db = state.db;

    let lineup = GameLineup::filter_by_game_id(game_id)
        .exec(&mut db)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::not_found("Geen opstelling gevonden"))?;

    Ok(Json(build_response(&mut db, lineup).await?))
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn save_lineup(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Json(body): Json<SaveLineupRequest>,
) -> Result<Json<GameLineupResponse>, AppError> {
    let mut db = state.db.clone();

    let existing: Vec<GameLineup> = GameLineup::filter_by_game_id(game_id).exec(&mut db).await?;

    let lineup_id = if let Some(mut existing) = existing.into_iter().next() {
        let lineup_id = existing.id;
        let mut upd = existing.update();
        upd.set_formation(body.formation);
        upd.set_published(true);
        upd.set_updated_at(Timestamp::now());
        upd.exec(&mut db).await?;
        lineup_id
    } else {
        GameLineup::create()
            .game_id(game_id)
            .formation(body.formation)
            .published(true)
            .exec(&mut db)
            .await?
            .id
    };

    GameLineupSlot::filter_by_lineup_id(lineup_id)
        .delete()
        .exec(&mut db)
        .await?;

    for slot_req in &body.slots {
        GameLineupSlot::create()
            .lineup_id(lineup_id)
            .player_id(slot_req.player_id)
            .slot(slot_req.slot)
            .captain(slot_req.captain)
            .exec(&mut db)
            .await?;
    }

    let lineup = GameLineup::filter_by_game_id(game_id)
        .exec(&mut db)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::internal("Lineup not found after save"))?;

    Ok(Json(build_response(&mut db, lineup).await?))
}
