use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{AppError, AppResult};
use crate::AppState;

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub sort_by: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Serialize)]
pub struct LeaderboardEntry {
    pub rank: i32,
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub games_played: i32,
    pub games_won: i32,
    pub win_rate: f64,
    pub problems_solved: i32,
    pub fastest_solve_ms: Option<i64>,
    pub longest_streak: i32,
}

#[derive(Serialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total: i64,
}

#[derive(FromRow)]
struct LeaderboardRow {
    user_id: String,
    display_name: String,
    avatar_url: Option<String>,
    games_played: i32,
    games_won: i32,
    win_rate: f64,
    problems_solved: i32,
    fastest_solve_ms: Option<i64>,
    longest_streak: i32,
}

// GET /leaderboard
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardQuery>,
) -> AppResult<Json<LeaderboardResponse>> {
    let sort_by = params.sort_by.unwrap_or_else(|| "wins".to_string());
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Validate sort_by parameter
    let valid_sort_options = ["wins", "problems_solved", "fastest", "streak"];
    if !valid_sort_options.contains(&sort_by.as_str()) {
        return Err(AppError::validation(
            "sort_by",
            format!("Invalid sort option. Valid options: {}", valid_sort_options.join(", ")),
        ));
    }

    let order_clause = match sort_by.as_str() {
        "problems_solved" => "us.problems_solved DESC",
        "fastest" => "us.fastest_solve_ms ASC NULLS LAST",
        "streak" => "us.longest_streak DESC",
        _ => "us.games_won DESC",
    };

    let query = format!(
        r#"
        SELECT
            u.id::text as user_id,
            u.display_name,
            u.avatar_url,
            us.games_played,
            us.games_won,
            us.problems_solved,
            us.fastest_solve_ms,
            us.longest_streak,
            CASE WHEN us.games_played > 0
                 THEN (us.games_won::float / us.games_played::float) * 100
                 ELSE 0 END as win_rate
        FROM users u
        JOIN user_stats us ON u.id = us.user_id
        WHERE us.games_played > 0
        ORDER BY {}
        LIMIT $1 OFFSET $2
        "#,
        order_clause
    );

    let rows = sqlx::query_as::<_, LeaderboardRow>(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db_pool)
        .await?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_stats WHERE games_played > 0"
    )
    .fetch_one(&state.db_pool)
    .await?;

    let entries: Vec<LeaderboardEntry> = rows
        .into_iter()
        .enumerate()
        .map(|(i, row)| LeaderboardEntry {
            rank: offset + i as i32 + 1,
            user_id: row.user_id,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            games_played: row.games_played,
            games_won: row.games_won,
            win_rate: row.win_rate,
            problems_solved: row.problems_solved,
            fastest_solve_ms: row.fastest_solve_ms,
            longest_streak: row.longest_streak,
        })
        .collect();

    Ok(Json(LeaderboardResponse { entries, total }))
}
