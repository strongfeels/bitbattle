use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{GameResult, ProblemBest, User, UserStats};
use crate::AppState;

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i32>,
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub stats: StatsResponse,
    pub problem_bests: Vec<ProblemBestResponse>,
}

#[derive(Serialize)]
pub struct ProblemBestResponse {
    pub problem_id: String,
    pub fastest_solve_ms: i64,
    pub attempts: i64,
}

#[derive(Serialize)]
pub struct DifficultyRankedStats {
    pub rating: i32,
    pub peak_rating: i32,
    pub games_played: i32,
    pub games_won: i32,
    pub win_rate: f64,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub games_played: i32,
    pub games_won: i32,
    pub games_lost: i32,
    pub problems_solved: i32,
    pub fastest_solve_ms: Option<i64>,
    pub current_streak: i32,
    pub longest_streak: i32,
    // Per-difficulty ranked stats
    pub easy_ranked: DifficultyRankedStats,
    pub medium_ranked: DifficultyRankedStats,
    pub hard_ranked: DifficultyRankedStats,
}

#[derive(Serialize)]
pub struct GameHistoryEntry {
    pub id: String,
    pub room_id: String,
    pub problem_id: String,
    pub placement: i32,
    pub total_players: i32,
    pub solve_time_ms: Option<i64>,
    pub passed_tests: i32,
    pub total_tests: i32,
    pub language: String,
    pub created_at: String,
}

// GET /users/:id/profile
pub async fn get_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&user_id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid user ID").into_response(),
    };

    let user = match User::find_by_id(&state.db_pool, uuid).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let stats = match UserStats::find_by_user_id(&state.db_pool, uuid).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Stats not found").into_response()
        }
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let problem_bests = GameResult::get_user_problem_bests(&state.db_pool, uuid)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|pb| ProblemBestResponse {
            problem_id: pb.problem_id,
            fastest_solve_ms: pb.fastest_solve_ms,
            attempts: pb.attempts,
        })
        .collect();

    // Helper to calculate win rate
    fn calc_win_rate(games: i32, wins: i32) -> f64 {
        if games > 0 {
            (wins as f64 / games as f64) * 100.0
        } else {
            0.0
        }
    }

    Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        stats: StatsResponse {
            games_played: stats.games_played,
            games_won: stats.games_won,
            games_lost: stats.games_lost,
            problems_solved: stats.problems_solved,
            fastest_solve_ms: stats.fastest_solve_ms,
            current_streak: stats.current_streak,
            longest_streak: stats.longest_streak,
            easy_ranked: DifficultyRankedStats {
                rating: stats.easy_rating,
                peak_rating: stats.easy_peak_rating,
                games_played: stats.easy_ranked_games,
                games_won: stats.easy_ranked_wins,
                win_rate: calc_win_rate(stats.easy_ranked_games, stats.easy_ranked_wins),
            },
            medium_ranked: DifficultyRankedStats {
                rating: stats.medium_rating,
                peak_rating: stats.medium_peak_rating,
                games_played: stats.medium_ranked_games,
                games_won: stats.medium_ranked_wins,
                win_rate: calc_win_rate(stats.medium_ranked_games, stats.medium_ranked_wins),
            },
            hard_ranked: DifficultyRankedStats {
                rating: stats.hard_rating,
                peak_rating: stats.hard_peak_rating,
                games_played: stats.hard_ranked_games,
                games_won: stats.hard_ranked_wins,
                win_rate: calc_win_rate(stats.hard_ranked_games, stats.hard_ranked_wins),
            },
        },
        problem_bests,
    })
    .into_response()
}

// GET /users/:id/history
pub async fn get_game_history(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&user_id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid user ID").into_response(),
    };

    let limit = params.limit.unwrap_or(20).min(100);

    let results = match GameResult::find_by_user(&state.db_pool, uuid, limit).await {
        Ok(r) => r,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let history: Vec<GameHistoryEntry> = results
        .into_iter()
        .map(|r| GameHistoryEntry {
            id: r.id.to_string(),
            room_id: r.room_id,
            problem_id: r.problem_id,
            placement: r.placement,
            total_players: r.total_players,
            solve_time_ms: r.solve_time_ms,
            passed_tests: r.passed_tests,
            total_tests: r.total_tests,
            language: r.language,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    Json(history).into_response()
}
