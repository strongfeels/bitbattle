use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::Ordering;

use crate::AppState;

/// List all live/spectatable games
pub async fn live_rooms_handler(State(state): State<AppState>) -> impl IntoResponse {
    let rooms = state.rooms.read().await;
    
    let live_games: Vec<_> = rooms.iter()
        .filter(|(_, room)| {
            // Only show public rooms with active games
            room.is_public && *room.game_started.blocking_read()
        })
        .map(|(room_id, room)| {
            let users = room.users.blocking_read();
            let problem = room.current_problem.blocking_read();
            let game_ended = *room.game_ended.blocking_read();
            
            serde_json::json!({
                "room_id": room_id,
                "players": *users,
                "player_count": users.len(),
                "spectator_count": room.spectator_count.load(Ordering::Relaxed),
                "game_mode": room.game_mode,
                "problem": problem.as_ref().map(|p| serde_json::json!({
                    "title": p.title,
                    "difficulty": p.difficulty,
                })),
                "game_ended": game_ended,
                "elapsed_seconds": room.created_at.elapsed().as_secs(),
            })
        })
        .collect();
    
    Json(serde_json::json!({
        "live_games": live_games,
        "total": live_games.len()
    }))
}

/// WebSocket handler for spectators (read-only)
pub async fn spectate_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let room_id = params.get("room").cloned().unwrap_or_default();
    
    ws.on_upgrade(move |socket| handle_spectator_socket(socket, state, room_id))
}

async fn handle_spectator_socket(socket: WebSocket, state: AppState, room_id: String) {
    let (mut sender, mut receiver) = socket.split();
    
    // Find the room
    let room = {
        let rooms = state.rooms.read().await;
        rooms.get(&room_id).cloned()
    };
    
    let Some(room) = room else {
        let _ = sender.send(Message::Text(serde_json::json!({
            "type": "error",
            "data": { "message": "Room not found" }
        }).to_string())).await;
        return;
    };
    
    // Increment spectator count
    room.spectator_count.fetch_add(1, Ordering::Relaxed);
    
    // Subscribe to room broadcasts
    let mut rx = room.tx.subscribe();
    
    // Send initial state to spectator
    {
        let users = room.users.read().await;
        let problem = room.current_problem.read().await;
        let user_codes = room.user_codes.read().await;
        let game_started = *room.game_started.read().await;
        let game_ended = *room.game_ended.read().await;
        let winner = room.winner.read().await.clone();
        
        let init_message = serde_json::json!({
            "type": "spectate_init",
            "data": {
                "room_id": room_id,
                "players": *users,
                "game_mode": room.game_mode,
                "game_started": game_started,
                "game_ended": game_ended,
                "winner": winner,
                "problem": problem.as_ref().map(|p| serde_json::json!({
                    "id": p.id,
                    "title": p.title,
                    "description": p.description,
                    "difficulty": p.difficulty,
                    "examples": p.examples,
                })),
                "player_codes": *user_codes,
                "spectator_count": room.spectator_count.load(Ordering::Relaxed),
            }
        });
        
        if let Err(e) = sender.send(Message::Text(init_message.to_string())).await {
            tracing::error!("Failed to send init to spectator: {}", e);
            room.spectator_count.fetch_sub(1, Ordering::Relaxed);
            return;
        }
    }
    
    let room_clone = room.clone();
    let connection_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let connection_active_clone = connection_active.clone();
    
    // Task to receive messages (spectators can send ping/pong but not game actions)
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {} // Ignore other messages from spectators
            }
        }
        connection_active_clone.store(false, Ordering::Relaxed);
    });
    
    // Task to forward room messages to spectator
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if !connection_active.load(Ordering::Relaxed) {
                break;
            }
            
            // Forward all room messages to spectator
            if let Err(e) = sender.send(Message::Text(msg)).await {
                tracing::error!("Failed to send to spectator: {}", e);
                break;
            }
        }
    });
    
    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }
    
    // Decrement spectator count on disconnect
    room_clone.spectator_count.fetch_sub(1, Ordering::Relaxed);
    tracing::info!("Spectator left room {}", room_id);
}
