use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::atomic::{AtomicBool, Ordering};
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod config;
mod db;
mod executor;
mod handlers;
mod models;
mod problems;

use config::Config;
use problems::{Problem, ProblemDatabase};
use executor::{CodeExecutor, SubmissionRequest, SubmissionResult};
use auth::OptionalAuthUser;
use models::game_result::update_user_stats_after_game;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Setup tracing subscriber (logging)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration from environment");
    tracing::info!("Configuration loaded successfully");

    // Create database pool
    let db_pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");
    tracing::info!("Database pool created");

    // Run migrations
    db::run_migrations(&db_pool)
        .await
        .expect("Failed to run database migrations");

    // Create shared state for rooms and problems
    let problem_db = Arc::new(ProblemDatabase::new());
    let code_executor = Arc::new(CodeExecutor::new());
    let rooms = Arc::new(RwLock::new(HashMap::<String, Room>::new()));

    let state = AppState {
        rooms,
        problem_db,
        code_executor,
        db_pool,
        config: Arc::new(config),
    };

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build app with routes and shared state
    let app = Router::new()
        // Existing routes
        .route("/", get(root_handler))
        .route("/ws", get(ws_handler))
        .route("/problems", get(get_problems_handler))
        .route("/problems/:id", get(get_problem_handler))
        .route("/submit", post(submit_code_handler))
        // Auth routes
        .route("/auth/google", get(handlers::google_auth_redirect))
        .route("/auth/callback", get(handlers::google_auth_callback))
        .route("/auth/me", get(handlers::get_current_user))
        .route("/auth/set-username", post(handlers::set_username))
        // User routes
        .route("/users/:id/profile", get(handlers::get_user_profile))
        .route("/users/:id/history", get(handlers::get_game_history))
        // Leaderboard
        .route("/leaderboard", get(handlers::get_leaderboard))
        .layer(cors)
        .with_state(state);

    // Run server on localhost:4000
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
    pub problem_db: Arc<ProblemDatabase>,
    pub code_executor: Arc<CodeExecutor>,
    pub db_pool: PgPool,
    pub config: Arc<Config>,
}

#[derive(Clone)]
struct Room {
    tx: broadcast::Sender<String>,
    users: Arc<RwLock<Vec<String>>>,
    current_problem: Arc<RwLock<Option<Problem>>>,
    user_codes: Arc<RwLock<HashMap<String, String>>>,
    required_players: usize,
    game_started: Arc<RwLock<bool>>,
    game_mode: String,
}

impl Room {
    fn new(problem: Option<Problem>, required_players: usize, game_mode: String) -> Self {
        let (tx, _rx) = broadcast::channel::<String>(100);
        Room {
            tx,
            users: Arc::new(RwLock::new(Vec::new())),
            current_problem: Arc::new(RwLock::new(problem)),
            user_codes: Arc::new(RwLock::new(HashMap::new())),
            required_players,
            game_started: Arc::new(RwLock::new(false)),
            game_mode,
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

async fn get_problems_handler(State(state): State<AppState>) -> impl IntoResponse {
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
    OptionalAuthUser(auth_user): OptionalAuthUser,
    Json(request): Json<SubmissionRequest>,
) -> impl IntoResponse {
    tracing::info!("Code submission from user: {}", request.username);

    if let Some(problem) = state.problem_db.get_problem(&request.problem_id) {
        let result = state.code_executor.execute_submission(request.clone(), problem).await;

        // Record game result if user is authenticated
        if let Some(user) = &auth_user {
            let is_win = result.passed;
            let solve_time = if result.passed {
                Some(result.execution_time_ms as i64)
            } else {
                None
            };

            // Record the game result
            if let Err(e) = models::GameResult::create(
                &state.db_pool,
                request.room_id.as_deref().unwrap_or("default"),
                &request.problem_id,
                Some(user.user_id),
                if result.passed { 1 } else { 0 },
                1, // For now, assume 1 player; room logic can update this
                solve_time,
                result.passed_tests as i32,
                result.total_tests as i32,
                &request.language,
            ).await {
                tracing::error!("Failed to record game result: {:?}", e);
            }

            // Update user stats
            if let Err(e) = update_user_stats_after_game(
                &state.db_pool,
                user.user_id,
                is_win,
                result.passed,
                solve_time,
            ).await {
                tracing::error!("Failed to update user stats: {:?}", e);
            }
        }

        // Broadcast submission result to all users in the room
        let rooms = state.rooms.read().await;
        let room_id = request.room_id.as_deref().unwrap_or(&request.problem_id);
        if let Some(room) = rooms.get(room_id) {
            let broadcast_message = serde_json::json!({
                "type": "submission_result",
                "data": {
                    "result": result
                }
            });
            let _ = room.tx.send(broadcast_message.to_string());
        } else if let Some(room) = rooms.get("default") {
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
    let difficulty = params.get("difficulty").cloned();
    let required_players = params.get("players")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    let game_mode = params.get("mode").cloned().unwrap_or_else(|| "casual".to_string());
    ws.on_upgrade(move |socket| handle_socket(socket, state, room_id, difficulty, required_players, game_mode))
}

async fn handle_socket(socket: WebSocket, state: AppState, room_id: String, difficulty: Option<String>, required_players: usize, game_mode: String) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!("User joining room: {} with difficulty: {:?}, required players: {}, mode: {}", room_id, difficulty, required_players, game_mode);

    // Get or create room with a problem based on difficulty
    let room = {
        let mut rooms = state.rooms.write().await;
        let game_mode_clone = game_mode.clone();
        rooms.entry(room_id.clone()).or_insert_with(|| {
            let problem = state.problem_db.get_random_problem_by_difficulty(difficulty.as_deref()).cloned();
            tracing::info!("Created new room '{}' with difficulty {:?}, required players: {}, mode: {}, problem: {:?}",
                room_id,
                difficulty,
                required_players,
                game_mode_clone,
                problem.as_ref().map(|p| &p.title));
            Room::new(problem, required_players, game_mode_clone)
        }).clone()
    };

    let mut rx = room.tx.subscribe();
    let room_clone = room.clone();

    // Track if this connection is still active
    let connection_active = Arc::new(AtomicBool::new(true));
    let connection_active_clone = connection_active.clone();

    // Task to handle incoming messages from client
    let recv_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    tracing::info!("Received message: {}", text);

                    if let Ok(parsed_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match parsed_msg.msg_type.as_str() {
                            "code_change" => {
                                if let (Ok(code), Ok(username)) = (
                                    serde_json::from_value::<String>(parsed_msg.data["code"].clone()),
                                    serde_json::from_value::<String>(parsed_msg.data["username"].clone())
                                ) {
                                    room_clone.user_codes.write().await.insert(username, code);
                                }
                                let _ = room_clone.tx.send(text);
                            }
                            "user_joined" => {
                                if let Ok(username) = serde_json::from_value::<String>(parsed_msg.data["username"].clone()) {
                                    let current_players = room_clone.users.read().await.len();
                                    let game_already_started = *room_clone.game_started.read().await;

                                    // Check if room is full (game already started or at capacity)
                                    if game_already_started || current_players >= room_clone.required_players {
                                        tracing::info!("Room {} is full, rejecting user {}", room_id, username);
                                        let room_full_message = serde_json::json!({
                                            "type": "room_full",
                                            "data": {
                                                "message": "This room is full. The game has already started.",
                                                "current": current_players,
                                                "required": room_clone.required_players
                                            }
                                        });
                                        let _ = room_clone.tx.send(room_full_message.to_string());
                                        continue;
                                    }

                                    // Send existing users to the new joiner
                                    let existing_users: Vec<String> = room_clone.users.read().await.clone();
                                    for existing_user in &existing_users {
                                        let existing_user_msg = serde_json::json!({
                                            "type": "user_joined",
                                            "data": {
                                                "username": existing_user
                                            }
                                        });
                                        let _ = room_clone.tx.send(existing_user_msg.to_string());
                                    }

                                    room_clone.users.write().await.push(username.clone());

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

                                    let current_players = room_clone.users.read().await.len();
                                    let player_count_message = serde_json::json!({
                                        "type": "player_count",
                                        "data": {
                                            "current": current_players,
                                            "required": room_clone.required_players
                                        }
                                    });
                                    let _ = room_clone.tx.send(player_count_message.to_string());

                                    if current_players >= room_clone.required_players {
                                        *room_clone.game_started.write().await = true;
                                        tracing::info!("All {} players joined room, starting game!", room_clone.required_players);
                                        let game_start_message = serde_json::json!({
                                            "type": "game_start",
                                            "data": {}
                                        });
                                        let _ = room_clone.tx.send(game_start_message.to_string());
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
                    tracing::debug!("Received non-text message");
                }
            }
        }

        connection_active_clone.store(false, Ordering::Relaxed);
        tracing::info!("Receive task ended");
    });

    // Task to send messages to client
    let send_task: JoinHandle<()> = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
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
