use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Setup tracing subscriber (logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create shared state for rooms
    let rooms = Arc::new(RwLock::new(HashMap::<String, Room>::new()));

    // Build app with routes and shared state
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/signup", post(signup_handler))
        .route("/ws", get(ws_handler))
        .with_state(AppState { rooms });

    // Run server on localhost:4000
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
struct AppState {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
}

#[derive(Clone)]
struct Room {
    tx: broadcast::Sender<String>,
    users: Arc<RwLock<Vec<String>>>,
    current_code: Arc<RwLock<String>>,
}

impl Room {
    fn new() -> Self {
        let (tx, _rx) = broadcast::channel::<String>(100);
        Room {
            tx,
            users: Arc::new(RwLock::new(Vec::new())),
            current_code: Arc::new(RwLock::new(String::new())),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WebSocketMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: serde_json::Value,
}

async fn root_handler() -> &'static str {
    "BitBattle backend is running"
}

#[derive(Deserialize)]
struct SignupPayload {
    username: String,
}

#[derive(Serialize)]
struct SignupResponse {
    message: String,
}

async fn signup_handler(Json(payload): Json<SignupPayload>) -> impl IntoResponse {
    tracing::info!("New signup: {}", payload.username);
    Json(SignupResponse {
        message: format!("Welcome, {}!", payload.username),
    })
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let room_id = "default".to_string();

    // Get or create room
    let room = {
        let mut rooms = state.rooms.write().await;
        rooms.entry(room_id.clone()).or_insert_with(Room::new).clone()
    };

    let mut rx = room.tx.subscribe();
    let room_clone = room.clone();

    // Task to handle incoming messages from client
    let recv_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                if let Message::Text(text) = msg {
                    tracing::info!("Received message: {}", text);

                    // Try to parse as structured message
                    if let Ok(parsed_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match parsed_msg.msg_type.as_str() {
                            "code_change" => {
                                if let Ok(code) = serde_json::from_value::<String>(parsed_msg.data["code"].clone()) {
                                    *room_clone.current_code.write().await = code;
                                }
                                let _ = room_clone.tx.send(text);
                            }
                            "user_joined" => {
                                if let Ok(username) = serde_json::from_value::<String>(parsed_msg.data["username"].clone()) {
                                    room_clone.users.write().await.push(username);
                                }
                                let _ = room_clone.tx.send(text);
                            }
                            "user_left" => {
                                if let Ok(username) = serde_json::from_value::<String>(parsed_msg.data["username"].clone()) {
                                    room_clone.users.write().await.retain(|u| u != &username);
                                }
                                let _ = room_clone.tx.send(text);
                            }
                            _ => {
                                let _ = room_clone.tx.send(text);
                            }
                        }
                    } else {
                        // Broadcast as-is for backward compatibility
                        let _ = room_clone.tx.send(text);
                    }
                }
            } else {
                break;
            }
        }
        tracing::info!("Receive task ended");
    });

    // Task to send messages to client
    let send_task: JoinHandle<()> = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
        tracing::info!("Send task ended");
    });

    tokio::pin!(recv_task);
    tokio::pin!(send_task);

    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    }
}