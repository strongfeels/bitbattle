use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::problems::{Difficulty, Problem, TestCase};

/// Status of an AI-generated problem
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemStatus {
    PendingValidation,
    Validating,
    Validated,
    Rejected,
}

impl ProblemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProblemStatus::PendingValidation => "pending_validation",
            ProblemStatus::Validating => "validating",
            ProblemStatus::Validated => "validated",
            ProblemStatus::Rejected => "rejected",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending_validation" => ProblemStatus::PendingValidation,
            "validating" => ProblemStatus::Validating,
            "validated" => ProblemStatus::Validated,
            "rejected" => ProblemStatus::Rejected,
            _ => ProblemStatus::PendingValidation,
        }
    }
}

/// AI-generated problem stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiProblem {
    pub id: Uuid,
    pub problem_id: String,
    pub title: String,
    pub description: String,
    pub difficulty: String,
    pub examples: serde_json::Value,
    pub test_cases: serde_json::Value,
    pub starter_code: serde_json::Value,
    pub time_limit_minutes: Option<i32>,
    pub tags: serde_json::Value,
    pub status: String,
    pub provider: String,
    pub model: String,
    pub validation_attempts: i32,
    pub last_validation_error: Option<String>,
    pub validated_at: Option<DateTime<Utc>>,
    pub times_used: i32,
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new AI problem
#[derive(Debug, Clone)]
pub struct NewAiProblem {
    pub problem_id: String,
    pub title: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub examples: Vec<TestCase>,
    pub test_cases: Vec<TestCase>,
    pub starter_code: std::collections::HashMap<String, String>,
    pub time_limit_minutes: Option<u32>,
    pub tags: Vec<String>,
    pub provider: String,
    pub model: String,
}

/// Pool counts by difficulty
#[derive(Debug, Clone, Default)]
pub struct PoolCounts {
    pub easy: i64,
    pub medium: i64,
    pub hard: i64,
}

impl AiProblem {
    /// Convert database model to Problem struct for game use
    pub fn to_problem(&self) -> Result<Problem, serde_json::Error> {
        let difficulty = match self.difficulty.as_str() {
            "Easy" => Difficulty::Easy,
            "Medium" => Difficulty::Medium,
            "Hard" => Difficulty::Hard,
            _ => Difficulty::Medium,
        };

        Ok(Problem {
            id: self.problem_id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            difficulty,
            examples: serde_json::from_value(self.examples.clone())?,
            test_cases: serde_json::from_value(self.test_cases.clone())?,
            starter_code: serde_json::from_value(self.starter_code.clone())?,
            time_limit_minutes: self.time_limit_minutes.map(|m| m as u32),
            tags: serde_json::from_value(self.tags.clone())?,
        })
    }

    /// Get validated pool counts by difficulty
    pub async fn get_pool_counts(pool: &PgPool) -> Result<PoolCounts, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT difficulty, COUNT(*) as count
            FROM ai_problems
            WHERE status = 'validated'
            GROUP BY difficulty
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut counts = PoolCounts::default();
        for (difficulty, count) in rows {
            match difficulty.as_str() {
                "Easy" => counts.easy = count,
                "Medium" => counts.medium = count,
                "Hard" => counts.hard = count,
                _ => {}
            }
        }

        Ok(counts)
    }

    /// Find a validated problem by difficulty that players haven't seen
    pub async fn find_unseen_by_difficulty(
        pool: &PgPool,
        difficulty: &str,
        player_ids: &[Uuid],
    ) -> Result<Option<Self>, sqlx::Error> {
        if player_ids.is_empty() {
            // No players with IDs, just get least used problem
            return sqlx::query_as::<_, Self>(
                r#"
                SELECT * FROM ai_problems
                WHERE status = 'validated' AND difficulty = $1
                ORDER BY times_used ASC, RANDOM()
                LIMIT 1
                "#,
            )
            .bind(difficulty)
            .fetch_optional(pool)
            .await;
        }

        // Find problem not seen by any of the players
        sqlx::query_as::<_, Self>(
            r#"
            SELECT ap.* FROM ai_problems ap
            WHERE ap.status = 'validated'
              AND ap.difficulty = $1
              AND NOT EXISTS (
                SELECT 1 FROM player_problem_history pph
                WHERE pph.problem_id = ap.problem_id
                  AND pph.user_id = ANY($2)
              )
            ORDER BY ap.times_used ASC, RANDOM()
            LIMIT 1
            "#,
        )
        .bind(difficulty)
        .bind(player_ids)
        .fetch_optional(pool)
        .await
    }

    /// Get a pending problem for validation
    pub async fn get_pending_for_validation(pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE ai_problems
            SET status = 'validating'
            WHERE id = (
                SELECT id FROM ai_problems
                WHERE status = 'pending_validation'
                AND validation_attempts < 3
                ORDER BY created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
        )
        .fetch_optional(pool)
        .await
    }

    /// Insert a new AI problem
    pub async fn insert(pool: &PgPool, problem: NewAiProblem) -> Result<Self, sqlx::Error> {
        let difficulty_str = match problem.difficulty {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        };

        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO ai_problems (
                problem_id, title, description, difficulty,
                examples, test_cases, starter_code,
                time_limit_minutes, tags, provider, model
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(&problem.problem_id)
        .bind(&problem.title)
        .bind(&problem.description)
        .bind(difficulty_str)
        .bind(serde_json::to_value(&problem.examples).unwrap_or_default())
        .bind(serde_json::to_value(&problem.test_cases).unwrap_or_default())
        .bind(serde_json::to_value(&problem.starter_code).unwrap_or_default())
        .bind(problem.time_limit_minutes.map(|m| m as i32))
        .bind(serde_json::to_value(&problem.tags).unwrap_or_default())
        .bind(&problem.provider)
        .bind(&problem.model)
        .fetch_one(pool)
        .await
    }

    /// Update problem status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: ProblemStatus,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let validated_at = if status == ProblemStatus::Validated {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE ai_problems
            SET status = $2,
                last_validation_error = $3,
                validated_at = $4,
                validation_attempts = validation_attempts + 1
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status.as_str())
        .bind(error)
        .bind(validated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Increment usage count
    pub async fn mark_as_used(pool: &PgPool, problem_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE ai_problems
            SET times_used = times_used + 1
            WHERE problem_id = $1
            "#,
        )
        .bind(problem_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record that a player has seen a problem
    pub async fn record_player_history(
        pool: &PgPool,
        user_id: Uuid,
        problem_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO player_problem_history (user_id, problem_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(problem_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
