use axum::{
    Json,
    extract::{Path, Query, State},
};
use axum_security::rbac::requires_any;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::{Formation, GameLineup, GameLineupSlot, Role},
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
    pub team_id: Uuid,
    pub formation: Formation,
    pub published: bool,
    pub updated_at: Timestamp,
    pub slots: Vec<LineupSlotResponse>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineupQuery {
    pub team_id: Option<Uuid>,
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
    pub team_id: Uuid,
}

async fn build_response(lineup: GameLineup) -> Result<GameLineupResponse, AppError> {
    let mut slot_responses: Vec<_> = lineup
        .slots
        .get()
        .iter()
        .map(|slot| {
            let player = slot.player.get().clone();
            LineupSlotResponse {
                slot: slot.slot,
                captain: slot.captain,
                player: LineupPlayerResponse {
                    id: player.id,
                    first_name: player.first_name,
                    last_name: player.last_name,
                    shirt_number: player.shirt_number,
                    avatar_url: player
                        .user
                        .get()
                        .as_ref()
                        .and_then(|u| u.avatar_url.clone()),
                },
            }
        })
        .collect();

    slot_responses.sort_by_key(|s| s.slot);

    Ok(GameLineupResponse {
        id: lineup.id,
        game_id: lineup.game_id,
        team_id: lineup.team_id,
        formation: lineup.formation,
        published: lineup.published,
        updated_at: lineup.updated_at,
        slots: slot_responses,
    })
}

pub async fn get_lineup(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Query(query): Query<LineupQuery>,
) -> Result<Json<Option<GameLineupResponse>>, AppError> {
    let mut db = state.db;

    let mut q = GameLineup::filter_by_game_id(game_id)
        .include(GameLineup::fields().slots().player().user());

    if let Some(tid) = query.team_id {
        q = q.filter(GameLineup::fields().team_id().eq(tid));
    }

    match q.first().exec(&mut db).await? {
        None => Ok(Json(None)),
        Some(lineup) => Ok(Json(Some(build_response(lineup).await?))),
    }
}

#[requires_any(Role::Admin, Role::Moderator)]
pub async fn save_lineup(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Json(body): Json<SaveLineupRequest>,
) -> Result<Json<GameLineupResponse>, AppError> {
    let mut db = state.db.clone();
    let mut tx = db.transaction().await?;

    let existing = GameLineup::filter_by_team_id(body.team_id)
        .filter(GameLineup::fields().game_id().eq(game_id))
        .first()
        .exec(&mut tx)
        .await?;

    let lineup = if let Some(mut existing) = existing {
        existing
            .update()
            .formation(body.formation)
            .published(true)
            .exec(&mut tx)
            .await?;
        existing
    } else {
        GameLineup::create()
            .game_id(game_id)
            .team_id(body.team_id)
            .formation(body.formation)
            .published(true)
            .exec(&mut tx)
            .await?
    };

    GameLineupSlot::filter_by_lineup_id(lineup.id)
        .delete()
        .exec(&mut tx)
        .await?;

    for slot_req in &body.slots {
        lineup
            .slots()
            .create()
            .player_id(slot_req.player_id)
            .slot(slot_req.slot)
            .captain(slot_req.captain)
            .exec(&mut tx)
            .await?;
    }

    tx.commit().await?;

    let lineup = GameLineup::filter_by_id(lineup.id)
        .include(GameLineup::fields().slots().player().user())
        .get(&mut db)
        .await?;

    Ok(Json(build_response(lineup).await?))
}
