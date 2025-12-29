use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GameResult {
    pub id: Uuid,
    pub room_id: String,
    pub problem_id: String,
    pub user_id: Option<Uuid>,
    pub placement: i32,
    pub total_players: i32,
    pub solve_time_ms: Option<i64>,
    pub passed_tests: i32,
    pub total_tests: i32,
    pub language: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProblemBest {
    pub problem_id: String,
    pub fastest_solve_ms: i64,
    pub attempts: i64,
    pub best_passed_tests: i32,
    pub best_total_tests: i32,
}

impl GameResult {
    pub async fn get_user_problem_bests(pool: &PgPool, user_id: Uuid) -> Result<Vec<ProblemBest>, sqlx::Error> {
        sqlx::query_as::<_, ProblemBest>(
            r#"
            SELECT
                problem_id,
                MIN(solve_time_ms) as fastest_solve_ms,
                COUNT(*) as attempts,
                MAX(passed_tests) as best_passed_tests,
                MAX(total_tests) as best_total_tests
            FROM game_results
            WHERE user_id = $1 AND solve_time_ms IS NOT NULL
            GROUP BY problem_id
            ORDER BY problem_id
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        room_id: &str,
        problem_id: &str,
        user_id: Option<Uuid>,
        placement: i32,
        total_players: i32,
        solve_time_ms: Option<i64>,
        passed_tests: i32,
        total_tests: i32,
        language: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, GameResult>(
            r#"
            INSERT INTO game_results
            (room_id, problem_id, user_id, placement, total_players, solve_time_ms, passed_tests, total_tests, language)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(room_id)
        .bind(problem_id)
        .bind(user_id)
        .bind(placement)
        .bind(total_players)
        .bind(solve_time_ms)
        .bind(passed_tests)
        .bind(total_tests)
        .bind(language)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_user(pool: &PgPool, user_id: Uuid, limit: i32) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, GameResult>(
            "SELECT * FROM game_results WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

pub async fn update_user_stats_after_game(
    pool: &PgPool,
    user_id: Uuid,
    is_win: bool,
    passed: bool,
    solve_time_ms: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE user_stats SET
            games_played = games_played + 1,
            games_won = games_won + CASE WHEN $2 THEN 1 ELSE 0 END,
            games_lost = games_lost + CASE WHEN NOT $2 THEN 1 ELSE 0 END,
            problems_solved = problems_solved + CASE WHEN $3 THEN 1 ELSE 0 END,
            total_submissions = total_submissions + 1,
            fastest_solve_ms = CASE
                WHEN $4::BIGINT IS NOT NULL AND (fastest_solve_ms IS NULL OR $4 < fastest_solve_ms)
                THEN $4 ELSE fastest_solve_ms END,
            last_played_at = NOW(),
            updated_at = NOW()
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .bind(is_win)
    .bind(passed)
    .bind(solve_time_ms)
    .execute(pool)
    .await?;

    // Update streak if win
    if is_win {
        update_streak(pool, user_id).await?;
    }

    Ok(())
}

async fn update_streak(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    // Check if last win was yesterday or today
    let last_played: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT last_played_at FROM user_stats WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let should_increment = match last_played {
        Some(last) => {
            let days_since = (Utc::now().date_naive() - last.date_naive()).num_days();
            days_since <= 1
        }
        None => true,
    };

    if should_increment {
        sqlx::query(
            r#"
            UPDATE user_stats SET
                current_streak = current_streak + 1,
                longest_streak = GREATEST(longest_streak, current_streak + 1)
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .execute(pool)
        .await?;
    } else {
        // Reset streak
        sqlx::query(
            "UPDATE user_stats SET current_streak = 1 WHERE user_id = $1"
        )
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}
