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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tower_http::cors::{AllowOrigin, CorsLayer};
use axum::http::{HeaderName, HeaderValue, Method};

mod ai_problems;
mod auth;
mod config;
mod db;
mod error;
mod executor;
mod handlers;
mod llm;
mod matchmaking;
mod middleware;
mod models;
mod problems;
mod rate_limit;
mod validation;
mod spectate;

use config::Config;
use problems::{Problem, ProblemDatabase};
use executor::{CodeExecutor, SubmissionRequest, SubmissionResult};
use auth::OptionalAuthUser;
use models::game_result::update_user_stats_after_game;
use matchmaking::MatchmakingQueue;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Load configuration first (needed for logging setup)
    let config = Config::from_env().expect("Failed to load configuration from environment");

    // Setup tracing subscriber (logging)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    if config.json_logging {
        // JSON logging for production (log aggregation)
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        // Pretty logging for development
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    tracing::info!("Configuration loaded successfully");

    // Create database pool
    let db_pool = db::create_pool(&config)
        .await
        .expect("Failed to create database pool");
    tracing::info!(
        max_connections = config.db_max_connections,
        min_connections = config.db_min_connections,
        "Database pool created"
    );

    // Run migrations
    db::run_migrations(&db_pool)
        .await
        .expect("Failed to run database migrations");

    // Create shared state for rooms and problems
    let problem_db = Arc::new(ProblemDatabase::new());
    let code_executor = Arc::new(CodeExecutor::new());
    let rooms = Arc::new(RwLock::new(HashMap::<String, Room>::new()));
    let matchmaking_queue = Arc::new(MatchmakingQueue::new());

    // Extract allowed origins before moving config
    let allowed_origins_config = config.allowed_origins.clone();

    let state = AppState {
        rooms,
        problem_db,
        code_executor,
        db_pool,
        config: Arc::new(config),
        matchmaking_queue: matchmaking_queue.clone(),
    };

    // Spawn background task to process matchmaking queue
    let matchmaking_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let matches = matchmaking_state.matchmaking_queue.process_queue().await;
            for match_result in matches {
                tracing::info!(
                    "Match created: {} - {} vs {}",
                    match_result.room_code,
                    match_result.players[0].username,
                    match_result.players[1].username
                );
                // Create the room for the matched players
                let problem = matchmaking_state
                    .problem_db
                    .get_random_problem_by_difficulty(
                        match_result.difficulty.to_problem_difficulty()
                            .map(|d| match d {
                                problems::Difficulty::Easy => "easy",
                                problems::Difficulty::Medium => "medium",
                                problems::Difficulty::Hard => "hard",
                            })
                    )
                    .cloned();

                let game_mode = match match_result.game_mode {
                    matchmaking::GameMode::Casual => "casual",
                    matchmaking::GameMode::Ranked => "ranked",
                };

                let mut rooms = matchmaking_state.rooms.write().await;
                rooms.insert(
                    match_result.room_code.clone(),
                    Room::new(problem, 2, game_mode.to_string()),
                );
            }
        }
    });

    // Start AI problem generator if enabled
    if state.config.ai_problems_enabled {
        if let Some(llm_provider) = llm::create_provider(&state.config) {
            let generator = Arc::new(ai_problems::ProblemGenerator::new(
                state.db_pool.clone(),
                llm_provider,
                state.code_executor.clone(),
                state.config.clone(),
            ));
            let generator_clone = generator.clone();
            tokio::spawn(async move {
                generator_clone.start().await;
            });
            tracing::info!(
                "AI problem generator started with provider: {}",
                state.config.ai_provider
            );
        } else {
            tracing::warn!(
                "AI problems enabled but no LLM provider configured (set OPENAI_API_KEY)"
            );
        }
    } else {
        tracing::info!("AI problem generation disabled");
    }

    // Build CORS layer with configured origins
    let allowed_origins: Vec<HeaderValue> = allowed_origins_config
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins.clone()))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("accept"),
            HeaderName::from_static("origin"),
            HeaderName::from_static("x-requested-with"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600));

    tracing::info!("CORS configured for origins: {:?}", allowed_origins_config);

    // Rate-limited routes for code submission (strict: 2 req/sec)
    let submit_routes = Router::new()
        .route("/submit", post(submit_code_handler))
        .layer(rate_limit::RateLimitLayer::submit());

    // Rate-limited routes for auth (strict: 5 req/sec)
    let auth_routes = Router::new()
        .route("/auth/google", get(handlers::google_auth_redirect))
        .route("/auth/callback", get(handlers::google_auth_callback))
        .route("/auth/me", get(handlers::get_current_user))
        .route("/auth/set-username", post(handlers::set_username))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/logout-all", post(handlers::logout_all))
        .layer(rate_limit::RateLimitLayer::auth());

    // Rate-limited routes for matchmaking (moderate: 10 req/sec)
    let matchmaking_routes = Router::new()
        .route("/matchmaking/join", post(join_matchmaking_queue))
        .route("/matchmaking/leave", post(leave_matchmaking_queue))
        .route("/matchmaking/status", get(get_matchmaking_status))
        .layer(rate_limit::RateLimitLayer::matchmaking());

    // Health check routes (no rate limiting)
    let health_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler));

    // General routes with standard rate limiting (100 req/sec)
    let general_routes = Router::new()
        .route("/", get(root_handler))
        .route("/problems", get(get_problems_handler))
        .route("/problems/:id", get(get_problem_handler))
        .route("/users/:id/profile", get(handlers::get_user_profile))
        .route("/users/:id/history", get(handlers::get_game_history))
        .route("/leaderboard", get(handlers::get_leaderboard))
        .layer(rate_limit::RateLimitLayer::general());

    // WebSocket route (no rate limiting - connection-based)
    let ws_routes = Router::new()
        .route("/ws", get(ws_handler))
        .route("/ws/spectate", get(spectate::spectate_ws_handler))
        .route("/rooms/live", get(spectate::live_rooms_handler));

    // Merge all routes
    let app = Router::new()
        .merge(health_routes)
        .merge(submit_routes)
        .merge(auth_routes)
        .merge(matchmaking_routes)
        .merge(general_routes)
        .merge(ws_routes)
        .layer(axum::middleware::from_fn(middleware::security_headers))
        .layer(axum::middleware::from_fn(middleware::request_id))
        .layer(axum::middleware::from_fn(middleware::request_timing))
        .layer(cors)
        .with_state(state);

    tracing::info!("Rate limiting enabled: submit=2/s, auth=5/s, matchmaking=10/s, general=100/s");

    // Get host and port from environment or use defaults
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4000);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid HOST:PORT configuration");

    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Graceful shutdown handling
    let shutdown_signal = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        tracing::info!("Shutdown signal received, starting graceful shutdown...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .unwrap();

    tracing::info!("Server shut down gracefully");
}

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
    pub problem_db: Arc<ProblemDatabase>,
    pub code_executor: Arc<CodeExecutor>,
    pub db_pool: PgPool,
    pub config: Arc<Config>,
    pub matchmaking_queue: Arc<MatchmakingQueue>,
}

#[derive(Clone)]
struct Room {
    tx: broadcast::Sender<String>,
    users: Arc<RwLock<Vec<String>>>,
    current_problem: Arc<RwLock<Option<Problem>>>,
    user_codes: Arc<RwLock<HashMap<String, String>>>,
    required_players: usize,
    game_started: Arc<RwLock<bool>>,
    game_ended: Arc<RwLock<bool>>,
    game_mode: String,
    winner: Arc<RwLock<Option<String>>>,
    // Store user_id mappings for rating calculations
    user_ids: Arc<RwLock<HashMap<String, uuid::Uuid>>>,
    spectator_count: Arc<std::sync::atomic::AtomicUsize>,
    is_public: bool,
    created_at: std::time::Instant,
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
            game_ended: Arc::new(RwLock::new(false)),
            game_mode,
            winner: Arc::new(RwLock::new(None)),
            user_ids: Arc::new(RwLock::new(HashMap::new())),
            spectator_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            is_public: true,
            created_at: std::time::Instant::now(),
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

/// Health check endpoint for load balancers and orchestrators
async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Readiness check - verifies database connectivity
async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Check database connection
    match sqlx::query("SELECT 1").execute(&state.db_pool).await {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "database": "connected",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        ),
        Err(e) => (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "database": "disconnected",
                "error": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        ),
    }
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
) -> Result<Json<SubmissionResult>, validation::ValidationError> {
    // Validate input
    validation::validate_username(&request.username)?;
    validation::validate_code(&request.code)?;
    validation::validate_problem_id(&request.problem_id)?;

    // Validate language - check against supported languages
    let language = request.language.trim().to_lowercase();
    if !validation::SUPPORTED_LANGUAGES.contains(&language.as_str()) {
        return Err(validation::ValidationError::new(format!(
            "Unsupported language '{}'. Supported: {}",
            request.language,
            validation::SUPPORTED_LANGUAGES.join(", ")
        ))
        .field("language"));
    }

    // Validate room_id if provided
    if let Some(ref room_id) = request.room_id {
        validation::validate_room_code(room_id)?;
    }

    tracing::info!("Code submission from user: {}", request.username);

    if let Some(problem) = state.problem_db.get_problem(&request.problem_id) {
        let result = state.code_executor.execute_submission(request.clone(), problem).await;
        let room_id = request.room_id.as_deref().unwrap_or(&request.problem_id);

        // Get difficulty as string for rating calculations
        let difficulty_str = match problem.difficulty {
            problems::Difficulty::Easy => "easy",
            problems::Difficulty::Medium => "medium",
            problems::Difficulty::Hard => "hard",
        };

        // Check if this submission wins the game
        let mut game_over_data: Option<serde_json::Value> = None;

        if result.passed {
            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(room_id) {
                let mut game_ended = room.game_ended.write().await;
                let game_started = *room.game_started.read().await;

                // Only process win if game started and hasn't ended
                if game_started && !*game_ended {
                    *game_ended = true;
                    *room.winner.write().await = Some(request.username.clone());
                    drop(game_ended);

                    let users = room.users.read().await.clone();
                    let user_ids = room.user_ids.read().await.clone();
                    let game_mode = room.game_mode.clone();

                    // Calculate rating changes for ranked games
                    let mut rating_changes: HashMap<String, RatingChange> = HashMap::new();

                    if game_mode == "ranked" {
                        // First, collect all player ratings
                        let mut player_ratings: HashMap<String, (uuid::Uuid, i32, i32)> = HashMap::new(); // username -> (user_id, rating, games_played)

                        for username in &users {
                            if let Some(uid) = user_ids.get(username).copied() {
                                if let Ok(Some(stats)) = models::UserStats::find_by_user_id(&state.db_pool, uid).await {
                                    let rating = stats.get_rating_for_difficulty(difficulty_str);
                                    let games = stats.get_ranked_games_for_difficulty(difficulty_str);
                                    player_ratings.insert(username.clone(), (uid, rating, games));
                                }
                            }
                        }

                        // Get winner's rating
                        let winner_rating = player_ratings.get(&request.username)
                            .map(|(_, r, _)| *r)
                            .unwrap_or(1200);

                        // Calculate average opponent rating for winner
                        let opponent_ratings: Vec<i32> = users.iter()
                            .filter(|u| *u != &request.username)
                            .map(|u| player_ratings.get(u).map(|(_, r, _)| *r).unwrap_or(1200))
                            .collect();

                        let avg_opponent_rating = if opponent_ratings.is_empty() {
                            1200
                        } else {
                            opponent_ratings.iter().sum::<i32>() / opponent_ratings.len() as i32
                        };

                        // Process each player
                        for username in &users {
                            let is_winner = username == &request.username;

                            if let Some((uid, current_rating, games_played)) = player_ratings.get(username).copied() {
                                // Calculate rating change based on matchup
                                let opponent_rating = if is_winner {
                                    // Winner: use average of all opponents
                                    avg_opponent_rating
                                } else {
                                    // Loser: calculate loss based on their rating vs winner's rating
                                    // Higher rated losers lose more, lower rated losers lose less
                                    winner_rating
                                };

                                let rating_change = models::UserStats::calculate_elo_change(
                                    current_rating,
                                    opponent_rating,
                                    is_winner,
                                    games_played,
                                );

                                let new_rating = (current_rating + rating_change).max(100);

                                // Update rating in database
                                if let Err(e) = models::UserStats::update_rating(
                                    &state.db_pool,
                                    uid,
                                    rating_change,
                                    is_winner,
                                    difficulty_str,
                                ).await {
                                    tracing::error!("Failed to update rating for {}: {:?}", username, e);
                                }

                                rating_changes.insert(username.clone(), RatingChange {
                                    old_rating: current_rating,
                                    new_rating,
                                    change: rating_change,
                                });
                            } else {
                                // Guest player - no rating change
                                rating_changes.insert(username.clone(), RatingChange {
                                    old_rating: 1200,
                                    new_rating: 1200,
                                    change: 0,
                                });
                            }
                        }
                    }

                    game_over_data = Some(serde_json::json!({
                        "winner": request.username,
                        "solve_time_ms": result.execution_time_ms,
                        "problem_id": request.problem_id,
                        "difficulty": difficulty_str,
                        "game_mode": game_mode,
                        "rating_changes": rating_changes,
                        "players": users,
                    }));
                }
            }
        }

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
                room_id,
                &request.problem_id,
                Some(user.user_id),
                if result.passed { 1 } else { 0 },
                1,
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
        if let Some(room) = rooms.get(room_id) {
            let broadcast_message = serde_json::json!({
                "type": "submission_result",
                "data": {
                    "result": result
                }
            });
            let _ = room.tx.send(broadcast_message.to_string());

            // Broadcast game_over if someone won
            if let Some(game_over) = game_over_data {
                let game_over_message = serde_json::json!({
                    "type": "game_over",
                    "data": game_over
                });
                tracing::info!("Broadcasting game_over: {:?}", game_over_message);
                let _ = room.tx.send(game_over_message.to_string());
            }
        } else if let Some(room) = rooms.get("default") {
            let broadcast_message = serde_json::json!({
                "type": "submission_result",
                "data": {
                    "result": result
                }
            });
            let _ = room.tx.send(broadcast_message.to_string());
        }

        Ok(Json(result))
    } else {
        // Problem not found - return validation error
        Err(validation::ValidationError::new("Problem not found")
            .field("problem_id")
            .detail("problem_id", request.problem_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RatingChange {
    old_rating: i32,
    new_rating: i32,
    change: i32,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, validation::ValidationError> {
    // Validate room ID
    let room_id = params.get("room").cloned().unwrap_or_else(|| "default".to_string());
    let room_id = validation::validate_room_code(&room_id)?;

    // Validate difficulty if provided
    let difficulty = if let Some(diff) = params.get("difficulty") {
        if diff != "random" {
            validation::validate_difficulty(diff)?;
        }
        Some(diff.clone())
    } else {
        None
    };

    // Validate player count
    let required_players = params.get("players")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(2);
    validation::validate_player_count(required_players)?;

    // Validate game mode
    let game_mode = params.get("mode").cloned().unwrap_or_else(|| "casual".to_string());
    validation::validate_game_mode(&game_mode)?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, room_id, difficulty, required_players, game_mode)))
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

// Matchmaking handlers
#[derive(Debug, Deserialize)]
struct JoinQueueRequest {
    username: String,
    difficulty: matchmaking::QueueDifficulty,
    game_mode: matchmaking::GameMode,
    connection_id: String,
}

#[derive(Debug, Serialize)]
struct JoinQueueResponse {
    success: bool,
    message: String,
    queue_size: usize,
}

async fn join_matchmaking_queue(
    State(state): State<AppState>,
    OptionalAuthUser(auth_user): OptionalAuthUser,
    Json(request): Json<JoinQueueRequest>,
) -> Result<Json<JoinQueueResponse>, validation::ValidationError> {
    // Validate input
    validation::validate_username(&request.username)?;
    validation::validate_connection_id(&request.connection_id)?;

    // For ranked mode, require authentication
    if request.game_mode == matchmaking::GameMode::Ranked && auth_user.is_none() {
        return Err(validation::ValidationError::new(
            "Authentication required for ranked matchmaking"
        ).field("game_mode"));
    }

    // Get user rating if authenticated
    let (user_id, rating) = if let Some(user) = &auth_user {
        // Try to get user stats for rating
        match models::UserStats::find_by_user_id(&state.db_pool, user.user_id).await {
            Ok(Some(stats)) => {
                let difficulty_str = match request.difficulty {
                    matchmaking::QueueDifficulty::Easy => "easy",
                    matchmaking::QueueDifficulty::Medium => "medium",
                    matchmaking::QueueDifficulty::Hard => "hard",
                    matchmaking::QueueDifficulty::Any => "medium",
                };
                (Some(user.user_id), stats.get_rating_for_difficulty(difficulty_str))
            }
            _ => (Some(user.user_id), 1200),
        }
    } else {
        (None, 1200)
    };

    let player = matchmaking::QueuedPlayer {
        user_id,
        username: request.username,
        rating,
        difficulty: request.difficulty,
        game_mode: request.game_mode,
        queued_at: chrono::Utc::now(),
        connection_id: request.connection_id,
    };

    state.matchmaking_queue.join_queue(player).await;
    let queue_size = state.matchmaking_queue.queue_size().await;

    tracing::info!("Player joined matchmaking queue. Queue size: {}", queue_size);

    Ok(Json(JoinQueueResponse {
        success: true,
        message: "Added to matchmaking queue".to_string(),
        queue_size,
    }))
}

#[derive(Debug, Deserialize)]
struct LeaveQueueRequest {
    connection_id: String,
}

#[derive(Debug, Serialize)]
struct LeaveQueueResponse {
    success: bool,
    message: String,
}

async fn leave_matchmaking_queue(
    State(state): State<AppState>,
    Json(request): Json<LeaveQueueRequest>,
) -> Result<Json<LeaveQueueResponse>, validation::ValidationError> {
    // Validate connection_id
    validation::validate_connection_id(&request.connection_id)?;

    let removed = state.matchmaking_queue.leave_queue(&request.connection_id).await;

    if removed.is_some() {
        tracing::info!("Player left matchmaking queue: {}", request.connection_id);
        Ok(Json(LeaveQueueResponse {
            success: true,
            message: "Removed from matchmaking queue".to_string(),
        }))
    } else {
        Ok(Json(LeaveQueueResponse {
            success: false,
            message: "Not found in queue".to_string(),
        }))
    }
}

#[derive(Debug, Serialize)]
struct MatchmakingStatusResponse {
    in_queue: bool,
    position: Option<usize>,
    queue_size: usize,
    match_found: bool,
    match_info: Option<MatchInfo>,
}

#[derive(Debug, Serialize)]
struct MatchInfo {
    room_code: String,
    opponent: String,
    difficulty: String,
    game_mode: String,
}

async fn get_matchmaking_status(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<MatchmakingStatusResponse>, validation::ValidationError> {
    let connection_id = params.get("connection_id").cloned().unwrap_or_default();

    // Validate connection_id
    validation::validate_connection_id(&connection_id)?;

    // Check if a match was found
    if let Some(match_result) = state.matchmaking_queue.get_match_for_player(&connection_id).await {
        let opponent = match_result
            .players
            .iter()
            .find(|p| p.connection_id != connection_id)
            .map(|p| p.username.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let difficulty = match match_result.difficulty {
            matchmaking::QueueDifficulty::Easy => "easy",
            matchmaking::QueueDifficulty::Medium => "medium",
            matchmaking::QueueDifficulty::Hard => "hard",
            matchmaking::QueueDifficulty::Any => "random",
        };

        let game_mode = match match_result.game_mode {
            matchmaking::GameMode::Casual => "casual",
            matchmaking::GameMode::Ranked => "ranked",
        };

        return Ok(Json(MatchmakingStatusResponse {
            in_queue: false,
            position: None,
            queue_size: state.matchmaking_queue.queue_size().await,
            match_found: true,
            match_info: Some(MatchInfo {
                room_code: match_result.room_code,
                opponent,
                difficulty: difficulty.to_string(),
                game_mode: game_mode.to_string(),
            }),
        }));
    }

    // Check queue position
    let position = state.matchmaking_queue.get_queue_position(&connection_id).await;
    let queue_size = state.matchmaking_queue.queue_size().await;

    Ok(Json(MatchmakingStatusResponse {
        in_queue: position.is_some(),
        position,
        queue_size,
        match_found: false,
        match_info: None,
    }))
}
