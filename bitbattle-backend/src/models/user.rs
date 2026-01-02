use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub google_id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserStats {
    pub user_id: Uuid,
    pub games_played: i32,
    pub games_won: i32,
    pub games_lost: i32,
    pub problems_solved: i32,
    pub total_submissions: i32,
    pub fastest_solve_ms: Option<i64>,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_played_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    // Per-difficulty ratings
    pub easy_rating: i32,
    pub easy_peak_rating: i32,
    pub easy_ranked_games: i32,
    pub easy_ranked_wins: i32,
    pub medium_rating: i32,
    pub medium_peak_rating: i32,
    pub medium_ranked_games: i32,
    pub medium_ranked_wins: i32,
    pub hard_rating: i32,
    pub hard_peak_rating: i32,
    pub hard_ranked_games: i32,
    pub hard_ranked_wins: i32,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub user: User,
    pub stats: UserStats,
}

impl User {
    pub async fn find_by_google_id(pool: &PgPool, google_id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE google_id = $1"
        )
        .bind(google_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        google_id: &str,
        email: &str,
        display_name: &str,
        avatar_url: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (google_id, email, display_name, avatar_url)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(google_id)
        .bind(email)
        .bind(display_name)
        .bind(avatar_url)
        .fetch_one(pool)
        .await?;

        // Create initial stats record
        sqlx::query(
            "INSERT INTO user_stats (user_id) VALUES ($1)"
        )
        .bind(user.id)
        .execute(pool)
        .await?;

        Ok(user)
    }

    pub async fn update_display_name(pool: &PgPool, user_id: Uuid, display_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE users SET display_name = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(display_name)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl UserStats {
    pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, UserStats>(
            "SELECT * FROM user_stats WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Calculate ELO rating change
    /// K-factor is 32 for standard play, adjusted based on game count
    pub fn calculate_elo_change(player_rating: i32, opponent_rating: i32, won: bool, games_played: i32) -> i32 {
        // K-factor: higher for new players, lower for experienced
        let k_factor = if games_played < 10 {
            40.0
        } else if games_played < 30 {
            32.0
        } else {
            24.0
        };

        let expected_score = 1.0 / (1.0 + 10.0_f64.powf((opponent_rating as f64 - player_rating as f64) / 400.0));
        let actual_score = if won { 1.0 } else { 0.0 };

        (k_factor * (actual_score - expected_score)).round() as i32
    }

    /// Update rating after a ranked game for a specific difficulty
    pub async fn update_rating(
        pool: &PgPool,
        user_id: Uuid,
        rating_change: i32,
        won: bool,
        difficulty: &str,
    ) -> Result<(), sqlx::Error> {
        let query = match difficulty.to_lowercase().as_str() {
            "easy" => r#"
                UPDATE user_stats
                SET easy_rating = GREATEST(100, easy_rating + $2),
                    easy_peak_rating = GREATEST(easy_peak_rating, GREATEST(100, easy_rating + $2)),
                    easy_ranked_games = easy_ranked_games + 1,
                    easy_ranked_wins = easy_ranked_wins + $3,
                    updated_at = NOW()
                WHERE user_id = $1
            "#,
            "hard" => r#"
                UPDATE user_stats
                SET hard_rating = GREATEST(100, hard_rating + $2),
                    hard_peak_rating = GREATEST(hard_peak_rating, GREATEST(100, hard_rating + $2)),
                    hard_ranked_games = hard_ranked_games + 1,
                    hard_ranked_wins = hard_ranked_wins + $3,
                    updated_at = NOW()
                WHERE user_id = $1
            "#,
            _ => r#"
                UPDATE user_stats
                SET medium_rating = GREATEST(100, medium_rating + $2),
                    medium_peak_rating = GREATEST(medium_peak_rating, GREATEST(100, medium_rating + $2)),
                    medium_ranked_games = medium_ranked_games + 1,
                    medium_ranked_wins = medium_ranked_wins + $3,
                    updated_at = NOW()
                WHERE user_id = $1
            "#,
        };

        sqlx::query(query)
            .bind(user_id)
            .bind(rating_change)
            .bind(if won { 1 } else { 0 })
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Get rating for a specific difficulty
    pub fn get_rating_for_difficulty(&self, difficulty: &str) -> i32 {
        match difficulty.to_lowercase().as_str() {
            "easy" => self.easy_rating,
            "hard" => self.hard_rating,
            _ => self.medium_rating,
        }
    }

    /// Get ranked games count for a specific difficulty
    pub fn get_ranked_games_for_difficulty(&self, difficulty: &str) -> i32 {
        match difficulty.to_lowercase().as_str() {
            "easy" => self.easy_ranked_games,
            "hard" => self.hard_ranked_games,
            _ => self.medium_ranked_games,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod elo_calculation {
        use super::*;

        #[test]
        fn test_equal_ratings_win() {
            // When equal ratings, winning should give ~+20 points (K=40 for new players)
            let change = UserStats::calculate_elo_change(1200, 1200, true, 5);
            assert_eq!(change, 20); // K * (1.0 - 0.5) = 40 * 0.5 = 20
        }

        #[test]
        fn test_equal_ratings_loss() {
            // When equal ratings, losing should give ~-20 points
            let change = UserStats::calculate_elo_change(1200, 1200, false, 5);
            assert_eq!(change, -20); // K * (0.0 - 0.5) = 40 * -0.5 = -20
        }

        #[test]
        fn test_higher_rated_wins_against_lower() {
            // Higher rated player wins against lower rated - should get fewer points
            let change = UserStats::calculate_elo_change(1400, 1200, true, 5);
            assert!(change < 20); // Less than equal rating win
            assert!(change > 0);  // But still positive
        }

        #[test]
        fn test_lower_rated_wins_against_higher() {
            // Lower rated player wins against higher rated - should get more points
            let change = UserStats::calculate_elo_change(1200, 1400, true, 5);
            assert!(change > 20); // More than equal rating win
        }

        #[test]
        fn test_higher_rated_loses_to_lower() {
            // Higher rated loses to lower - should lose more points
            let change = UserStats::calculate_elo_change(1400, 1200, false, 5);
            assert!(change < -20); // More negative than equal rating loss
        }

        #[test]
        fn test_k_factor_new_player() {
            // New player (< 10 games) has K=40
            let change = UserStats::calculate_elo_change(1200, 1200, true, 5);
            assert_eq!(change, 20); // 40 * 0.5 = 20
        }

        #[test]
        fn test_k_factor_intermediate_player() {
            // Intermediate player (10-29 games) has K=32
            let change = UserStats::calculate_elo_change(1200, 1200, true, 15);
            assert_eq!(change, 16); // 32 * 0.5 = 16
        }

        #[test]
        fn test_k_factor_experienced_player() {
            // Experienced player (30+ games) has K=24
            let change = UserStats::calculate_elo_change(1200, 1200, true, 50);
            assert_eq!(change, 12); // 24 * 0.5 = 12
        }

        #[test]
        fn test_extreme_rating_difference() {
            // Very high rated vs very low rated
            let change = UserStats::calculate_elo_change(2000, 1000, true, 5);
            // Expected score is very high (~0.99), so winning gives almost nothing
            assert!(change < 5);
            assert!(change >= 0);
        }

        #[test]
        fn test_upset_victory() {
            // Low rated beats high rated (upset)
            let change = UserStats::calculate_elo_change(1000, 2000, true, 5);
            // Expected score is very low (~0.01), so winning gives a lot
            assert!(change > 35);
        }

        #[test]
        fn test_rating_change_symmetry() {
            // In a match, total ELO should be roughly conserved (zero-sum)
            let winner_change = UserStats::calculate_elo_change(1200, 1300, true, 20);
            let loser_change = UserStats::calculate_elo_change(1300, 1200, false, 20);
            // With same K-factor, these should sum to 0
            assert_eq!(winner_change + loser_change, 0);
        }
    }

    mod user_stats_helpers {
        use super::*;
        use chrono::Utc;

        fn create_test_stats() -> UserStats {
            UserStats {
                user_id: uuid::Uuid::new_v4(),
                games_played: 10,
                games_won: 5,
                games_lost: 5,
                problems_solved: 20,
                total_submissions: 30,
                fastest_solve_ms: Some(30000),
                current_streak: 3,
                longest_streak: 7,
                last_played_at: Some(Utc::now()),
                updated_at: Utc::now(),
                easy_rating: 1200,
                easy_peak_rating: 1250,
                easy_ranked_games: 10,
                easy_ranked_wins: 6,
                medium_rating: 1100,
                medium_peak_rating: 1150,
                medium_ranked_games: 8,
                medium_ranked_wins: 4,
                hard_rating: 1000,
                hard_peak_rating: 1050,
                hard_ranked_games: 5,
                hard_ranked_wins: 2,
            }
        }

        #[test]
        fn test_get_rating_for_easy() {
            let stats = create_test_stats();
            assert_eq!(stats.get_rating_for_difficulty("easy"), 1200);
            assert_eq!(stats.get_rating_for_difficulty("Easy"), 1200);
            assert_eq!(stats.get_rating_for_difficulty("EASY"), 1200);
        }

        #[test]
        fn test_get_rating_for_medium() {
            let stats = create_test_stats();
            assert_eq!(stats.get_rating_for_difficulty("medium"), 1100);
            assert_eq!(stats.get_rating_for_difficulty("Medium"), 1100);
        }

        #[test]
        fn test_get_rating_for_hard() {
            let stats = create_test_stats();
            assert_eq!(stats.get_rating_for_difficulty("hard"), 1000);
            assert_eq!(stats.get_rating_for_difficulty("Hard"), 1000);
        }

        #[test]
        fn test_get_rating_default_to_medium() {
            let stats = create_test_stats();
            // Unknown difficulty defaults to medium
            assert_eq!(stats.get_rating_for_difficulty("unknown"), 1100);
            assert_eq!(stats.get_rating_for_difficulty(""), 1100);
        }

        #[test]
        fn test_get_ranked_games_for_difficulty() {
            let stats = create_test_stats();
            assert_eq!(stats.get_ranked_games_for_difficulty("easy"), 10);
            assert_eq!(stats.get_ranked_games_for_difficulty("medium"), 8);
            assert_eq!(stats.get_ranked_games_for_difficulty("hard"), 5);
            assert_eq!(stats.get_ranked_games_for_difficulty("unknown"), 8); // defaults to medium
        }
    }
}
