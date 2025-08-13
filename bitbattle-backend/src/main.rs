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
use std::sync::atomic::{AtomicBool, Ordering};
use tower_http::cors::CorsLayer;

mod problems;
mod executor;

use problems::{Problem, ProblemDatabase};
use executor::{CodeExecutor, SubmissionRequest, SubmissionResult};

#[tokio::main]
async fn main() {
    // Setup tracing subscriber (logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create shared state for rooms and problems
    let problem_db = Arc::new(ProblemDatabase::new());
    let code_executor = Arc::new(CodeExecutor::new());
    let rooms = Arc::new(RwLock::new(HashMap::<String, Room>::new()));

    // Build app with routes and shared state
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/signup", post(signup_handler))
        .route("/ws", get(ws_handler))
        .route("/problems", get(get_problems_handler))
        .route("/problems/:id", get(get_problem_handler))
        .route("/submit", post(submit_code_handler))
        .layer(CorsLayer::permissive()) // Add CORS support
        .with_state(AppState { rooms, problem_db, code_executor });

    // Run server on localhost:4000
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}

#[derive(Clone)]
struct AppState {
    rooms: Arc<RwLock<HashMap<String, Room>>>,
    problem_db: Arc<ProblemDatabase>,
    code_executor: Arc<CodeExecutor>,
}

#[derive(Clone)]
struct Room {
    tx: broadcast::Sender<String>,
    users: Arc<RwLock<Vec<String>>>,
    current_problem: Arc<RwLock<Option<Problem>>>,
    user_codes: Arc<RwLock<HashMap<String, String>>>,
}

impl Room {
    fn new(problem: Option<Problem>) -> Self {
        let (tx, _rx) = broadcast::channel::<String>(100);
        Room {
            tx,
            users: Arc::new(RwLock::new(Vec::new())),
            current_problem: Arc::new(RwLock::new(problem)),
            user_codes: Arc::new(RwLock::new(HashMap::new())),
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

async fn get_problems_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Return list of all problems (without test cases for security)
    let problems: Vec<_> = ["two-sum", "reverse-string", "valid-parentheses"]
        .iter()
        .filter_map(|id| {
            state.problem_db.get_problem(id).map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "title": p.title,
                    "difficulty": p.difficulty,
                    "tags": p.tags
                })
            })
        })
        .collect();

    Json(problems)
}

async fn get_problem_handler(
    axum::extract::Path(problem_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(problem) = state.problem_db.get_problem(&problem_id) {
        // Return problem without hidden test cases
        let public_problem = serde_json::json!({
            "id": problem.id,
            "title": problem.title,
            "description": problem.description,
            "difficulty": problem.difficulty,
            "examples": problem.examples,
            "starter_code": problem.starter_code,
            "time_limit_minutes": problem.time_limit_minutes,
            "tags": problem.tags
        });
        Json(public_problem)
    } else {
        Json(serde_json::json!({"error": "Problem not found"}))
    }
}

async fn submit_code_handler(
    State(state): State<AppState>,
    Json(request): Json<SubmissionRequest>,
) -> impl IntoResponse {
    tracing::info!("Code submission from user: {}", request.username);

    if let Some(problem) = state.problem_db.get_problem(&request.problem_id) {
        let result = state.code_executor.execute_submission(request.clone(), problem).await;

        // Broadcast submission result to all users in the room
        let rooms = state.rooms.read().await;
        if let Some(room) = rooms.get(&request.problem_id) { // Try to find room by problem_id first
            let broadcast_message = serde_json::json!({
                "type": "submission_result",
                "data": {
                    "result": result
                }
            });
            let _ = room.tx.send(broadcast_message.to_string());
        } else if let Some(room) = rooms.get("default") { // Fallback to default room
            let broadcast_message = serde_json::json!({
                "type": "submission_result",
                "data": {
                    "result": result
                }
            });
            let _ = room.tx.send(broadcast_message.to_string());
        }

        Json(result)
    } else {
        Json(SubmissionResult {
            username: request.username,
            problem_id: request.problem_id,
            passed: false,
            total_tests: 0,
            passed_tests: 0,
            test_results: vec![],
            execution_time_ms: 0,
            submission_time: chrono::Utc::now().timestamp(),
        })
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let room_id = params.get("room").unwrap_or(&"default".to_string()).clone();
    ws.on_upgrade(move |socket| handle_socket(socket, state, room_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, room_id: String) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!("User joining room: {}", room_id);

    // Get or create room with a random problem
    let room = {
        let mut rooms = state.rooms.write().await;
        rooms.entry(room_id.clone()).or_insert_with(|| {
            let random_problem = state.problem_db.get_random_problem().cloned();
            tracing::info!("Created new room '{}' with problem: {:?}", room_id, random_problem.as_ref().map(|p| &p.title));
            Room::new(random_problem)
        }).clone()
    };

    let mut rx = room.tx.subscribe();
    let room_clone = room.clone();
    let state_clone = state.clone();

    // Track if this connection is still active
    let connection_active = Arc::new(AtomicBool::new(true));
    let connection_active_clone = connection_active.clone();

    // Task to handle incoming messages from client
    let recv_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    tracing::info!("Received message: {}", text);

                    // Try to parse as structured message
                    if let Ok(parsed_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match parsed_msg.msg_type.as_str() {
                            "code_change" => {
                                if let (Ok(code), Ok(username)) = (
                                    serde_json::from_value::<String>(parsed_msg.data["code"].clone()),
                                    serde_json::from_value::<String>(parsed_msg.data["username"].clone())
                                ) {
                                    // Store user's code
                                    room_clone.user_codes.write().await.insert(username, code);
                                }
                                let _ = room_clone.tx.send(text);
                            }
                            "user_joined" => {
                                if let Ok(username) = serde_json::from_value::<String>(parsed_msg.data["username"].clone()) {
                                    room_clone.users.write().await.push(username.clone());

                                    // Send current problem to the new user
                                    if let Some(problem) = room_clone.current_problem.read().await.as_ref() {
                                        let problem_message = serde_json::json!({
                                            "type": "problem_assigned",
                                            "data": {
                                                "problem": {
                                                    "id": problem.id,
                                                    "title": problem.title,
                                                    "description": problem.description,
                                                    "difficulty": problem.difficulty,
                                                    "examples": problem.examples,
                                                    "starter_code": problem.starter_code,
                                                    "time_limit_minutes": problem.time_limit_minutes,
                                                    "tags": problem.tags
                                                }
                                            }
                                        });
                                        let _ = room_clone.tx.send(problem_message.to_string());
                                    }
                                }
                                let _ = room_clone.tx.send(text);
                            }
                            "user_left" => {
                                if let Ok(username) = serde_json::from_value::<String>(parsed_msg.data["username"].clone()) {
                                    room_clone.users.write().await.retain(|u| u != &username);
                                    room_clone.user_codes.write().await.remove(&username);
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
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket connection closed gracefully");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {
                    // Handle other message types (Binary, Ping, Pong)
                    tracing::debug!("Received non-text message");
                }
            }
        }

        // Mark connection as inactive when receive task ends
        connection_active_clone.store(false, Ordering::Relaxed);
        tracing::info!("Receive task ended");
    });

    // Task to send messages to client
    let send_task: JoinHandle<()> = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Check if connection is still active
            if !connection_active.load(Ordering::Relaxed) {
                tracing::info!("Connection inactive, stopping send task");
                break;
            }

            if let Err(e) = sender.send(Message::Text(msg)).await {
                tracing::error!("Failed to send message: {}", e);
                break;
            }
        }
        tracing::info!("Send task ended");
    });

    tokio::pin!(recv_task);
    tokio::pin!(send_task);

    // Wait for either task to complete, then clean up
    tokio::select! {
        _ = &mut recv_task => {
            tracing::info!("Receive task completed, cleaning up");
            send_task.abort();
        },
        _ = &mut send_task => {
            tracing::info!("Send task completed, cleaning up");
            recv_task.abort();
        }
    }

    tracing::info!("WebSocket connection fully cleaned up");
}