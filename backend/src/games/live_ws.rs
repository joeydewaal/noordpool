use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use jiff::Timestamp;
use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    games::live::LiveSnapshot,
    models::{Game, GameEvent},
};

/// Per-game broadcast channel registry. Each entry is a fan-out `Sender`
/// that all connected viewers of that game receive from. Created lazily on
/// first subscriber and dropped implicitly when the last `Sender`/`Receiver`
/// pair goes away (we clean the map entry on disconnect — see `subscribe`).
pub type LiveHub = Arc<Mutex<HashMap<Uuid, broadcast::Sender<LiveEvent>>>>;

pub fn new_hub() -> LiveHub {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Events broadcast to connected WebSocket clients for a specific game.
/// Tag is `type` so the JSON is easy to discriminate on the client:
/// `{"type":"scoreUpdate","home":1,"away":0, ...}`.
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LiveEvent {
    Snapshot(LiveSnapshot),
    ScoreUpdate {
        home: i32,
        away: i32,
        version: i64,
        updated_at: Timestamp,
    },
    EventAdded(GameEvent),
    EventDeleted {
        id: Uuid,
    },
    StatusChange {
        status: &'static str,
    },
}

/// Broadcast an event to all current subscribers of `game_id`. When there
/// are no subscribers (channel absent or zero receivers) this is a no-op —
/// mutation handlers can publish unconditionally without caring whether
/// anyone is listening.
pub fn publish(hub: &LiveHub, game_id: Uuid, event: LiveEvent) {
    let sender = {
        let map = hub.lock().expect("live hub poisoned");
        map.get(&game_id).cloned()
    };
    if let Some(tx) = sender {
        let _ = tx.send(event);
    }
}

/// Subscribe to a game's live channel, creating it on demand. Capacity is
/// small (32) — viewers that lag more than 32 events will miss some frames
/// and reconcile via a fresh snapshot on reconnect.
fn subscribe(hub: &LiveHub, game_id: Uuid) -> broadcast::Receiver<LiveEvent> {
    let mut map = hub.lock().expect("live hub poisoned");
    let sender = map
        .entry(game_id)
        .or_insert_with(|| broadcast::channel::<LiveEvent>(32).0);
    sender.subscribe()
}

/// Remove the per-game sender if nobody is listening anymore. Called after
/// a socket disconnects so the map doesn't grow unbounded.
fn prune(hub: &LiveHub, game_id: Uuid) {
    let mut map = hub.lock().expect("live hub poisoned");
    if let Some(tx) = map.get(&game_id)
        && tx.receiver_count() == 0
    {
        map.remove(&game_id);
    }
}

/// GET /api/games/:id/ws — public, unauthenticated. Upgrades to a
/// WebSocket, sends the current snapshot, then streams live events until
/// the client disconnects.
#[tracing::instrument(skip(ws, state), fields(game_id = %id))]
pub async fn ws_live(
    ws: WebSocketUpgrade,
    State(mut state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Load the snapshot up-front so a 404 surfaces as an HTTP error before
    // the upgrade completes — easier for clients than an immediate close.
    let game = Game::filter_by_id(id)
        .include(Game::fields().events().player())
        .get(&mut state.db)
        .await?;
    let snapshot = LiveSnapshot::from_game(&game, Timestamp::now());

    let hub = state.live_hub.clone();
    Ok(ws.on_upgrade(move |socket| run_socket(socket, hub, id, snapshot)))
}

async fn run_socket(mut socket: WebSocket, hub: LiveHub, game_id: Uuid, snapshot: LiveSnapshot) {
    let mut rx = subscribe(&hub, game_id);

    // Initial snapshot. If the client is gone already, bail out quickly.
    if send(&mut socket, &LiveEvent::Snapshot(snapshot))
        .await
        .is_err()
    {
        prune(&hub, game_id);
        return;
    }

    loop {
        tokio::select! {
            // Outgoing: broadcast → client.
            incoming = rx.recv() => match incoming {
                Ok(event) => {
                    if send(&mut socket, &event).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Slow consumer — skip missed events. The client will
                    // reconcile on the next snapshot (reconnect) if needed.
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
            // Incoming: we only care about Close/Ping; everything else is
            // ignored. Axum handles Ping/Pong automatically.
            msg = socket.recv() => match msg {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(_)) => break,
                _ => {}
            },
        }
    }

    prune(&hub, game_id);
}

async fn send(socket: &mut WebSocket, event: &LiveEvent) -> Result<(), axum::Error> {
    let json = serde_json::to_string(event).expect("LiveEvent serializes");
    socket.send(Message::Text(json.into())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_delivers_to_subscribers() {
        let hub = new_hub();
        let game_id = Uuid::new_v4();
        let mut rx = subscribe(&hub, game_id);

        publish(
            &hub,
            game_id,
            LiveEvent::ScoreUpdate {
                home: 1,
                away: 0,
                version: 1,
                updated_at: Timestamp::now(),
            },
        );

        let event = rx.recv().await.expect("broadcast receives event");
        match event {
            LiveEvent::ScoreUpdate { home, away, .. } => {
                assert_eq!((home, away), (1, 0));
            }
            _ => panic!("wrong event variant"),
        }
    }

    #[tokio::test]
    async fn publish_without_subscribers_is_noop() {
        let hub = new_hub();
        let game_id = Uuid::new_v4();
        publish(
            &hub,
            game_id,
            LiveEvent::EventDeleted { id: Uuid::new_v4() },
        );
        assert!(hub.lock().unwrap().get(&game_id).is_none());
    }

    #[tokio::test]
    async fn prune_removes_entry_when_no_receivers() {
        let hub = new_hub();
        let game_id = Uuid::new_v4();
        {
            let _rx = subscribe(&hub, game_id);
            assert!(hub.lock().unwrap().contains_key(&game_id));
        }
        prune(&hub, game_id);
        assert!(!hub.lock().unwrap().contains_key(&game_id));
    }
}
