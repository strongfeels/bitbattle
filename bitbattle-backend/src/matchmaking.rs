use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::problems::Difficulty;

/// A player in the matchmaking queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedPlayer {
    /// User ID (UUID for authenticated users, None for guests)
    pub user_id: Option<Uuid>,
    /// Display name
    pub username: String,
    /// Their rating for the selected difficulty (1200 default for unrated)
    pub rating: i32,
    /// Preferred difficulty
    pub difficulty: QueueDifficulty,
    /// Preferred game mode
    pub game_mode: GameMode,
    /// When they joined the queue
    pub queued_at: DateTime<Utc>,
    /// WebSocket connection identifier (for notifying them)
    pub connection_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueDifficulty {
    Easy,
    Medium,
    Hard,
    Any, // Will match with any difficulty
}

impl QueueDifficulty {
    pub fn matches(&self, other: &QueueDifficulty) -> bool {
        match (self, other) {
            (QueueDifficulty::Any, _) | (_, QueueDifficulty::Any) => true,
            (a, b) => a == b,
        }
    }

    pub fn to_problem_difficulty(&self) -> Option<Difficulty> {
        match self {
            QueueDifficulty::Easy => Some(Difficulty::Easy),
            QueueDifficulty::Medium => Some(Difficulty::Medium),
            QueueDifficulty::Hard => Some(Difficulty::Hard),
            QueueDifficulty::Any => None, // Random
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    Casual,
    Ranked,
}

/// A successful match between players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub players: Vec<QueuedPlayer>,
    pub difficulty: QueueDifficulty,
    pub game_mode: GameMode,
    pub room_code: String,
    pub created_at: DateTime<Utc>,
}

/// The matchmaking queue
pub struct MatchmakingQueue {
    /// Players waiting to be matched, keyed by connection_id
    queue: Arc<RwLock<HashMap<String, QueuedPlayer>>>,
    /// Recently created matches (for notification lookup)
    recent_matches: Arc<RwLock<Vec<Match>>>,
    /// Rating difference threshold for matching (expands over time)
    base_rating_threshold: i32,
    /// Maximum wait time before loosening criteria (in seconds)
    max_wait_seconds: i64,
}

impl MatchmakingQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(HashMap::new())),
            recent_matches: Arc::new(RwLock::new(Vec::new())),
            base_rating_threshold: 200,
            max_wait_seconds: 60,
        }
    }

    /// Add a player to the queue
    pub async fn join_queue(&self, player: QueuedPlayer) {
        let mut queue = self.queue.write().await;
        queue.insert(player.connection_id.clone(), player);
    }

    /// Remove a player from the queue
    pub async fn leave_queue(&self, connection_id: &str) -> Option<QueuedPlayer> {
        let mut queue = self.queue.write().await;
        queue.remove(connection_id)
    }

    /// Get queue status for a player
    pub async fn get_queue_position(&self, connection_id: &str) -> Option<usize> {
        let queue = self.queue.read().await;
        let mut players: Vec<_> = queue.values().collect();
        players.sort_by(|a, b| a.queued_at.cmp(&b.queued_at));

        players
            .iter()
            .position(|p| p.connection_id == connection_id)
    }

    /// Get current queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Get queue size for a specific difficulty and game mode
    pub async fn queue_size_for(&self, difficulty: QueueDifficulty, game_mode: GameMode) -> usize {
        self.queue
            .read()
            .await
            .values()
            .filter(|p| p.difficulty.matches(&difficulty) && p.game_mode == game_mode)
            .count()
    }

    /// Try to find matches in the queue
    /// Returns a list of matches that were made
    pub async fn process_queue(&self) -> Vec<Match> {
        let mut matches = Vec::new();
        let mut queue = self.queue.write().await;
        let now = Utc::now();

        // Get all players and sort by queue time (oldest first)
        let mut players: Vec<_> = queue.values().cloned().collect();
        players.sort_by(|a, b| a.queued_at.cmp(&b.queued_at));

        let mut matched_ids: Vec<String> = Vec::new();

        // Try to match players
        for i in 0..players.len() {
            if matched_ids.contains(&players[i].connection_id) {
                continue;
            }

            let player1 = &players[i];
            let wait_time = (now - player1.queued_at).num_seconds();

            // Calculate expanded rating threshold based on wait time
            let rating_threshold = self.calculate_rating_threshold(wait_time);

            // Find a suitable opponent
            for j in (i + 1)..players.len() {
                if matched_ids.contains(&players[j].connection_id) {
                    continue;
                }

                let player2 = &players[j];

                // Check if players are compatible
                if self.are_compatible(player1, player2, rating_threshold) {
                    // Create a match
                    let room_code = generate_room_code();
                    let difficulty = resolve_difficulty(&player1.difficulty, &player2.difficulty);

                    let match_result = Match {
                        id: Uuid::new_v4().to_string(),
                        players: vec![player1.clone(), player2.clone()],
                        difficulty,
                        game_mode: player1.game_mode,
                        room_code,
                        created_at: now,
                    };

                    matched_ids.push(player1.connection_id.clone());
                    matched_ids.push(player2.connection_id.clone());
                    matches.push(match_result);
                    break;
                }
            }
        }

        // Remove matched players from queue
        for id in &matched_ids {
            queue.remove(id);
        }

        // Store recent matches
        if !matches.is_empty() {
            let mut recent = self.recent_matches.write().await;
            recent.extend(matches.clone());

            // Keep only last 100 matches
            let len = recent.len();
            if len > 100 {
                recent.drain(0..len - 100);
            }
        }

        matches
    }

    /// Calculate rating threshold based on wait time
    fn calculate_rating_threshold(&self, wait_seconds: i64) -> i32 {
        let wait_factor = (wait_seconds as f64 / self.max_wait_seconds as f64).min(1.0);
        let expansion = (wait_factor * 500.0) as i32; // Expand up to 500 rating points
        self.base_rating_threshold + expansion
    }

    /// Check if two players are compatible for matching
    fn are_compatible(&self, p1: &QueuedPlayer, p2: &QueuedPlayer, rating_threshold: i32) -> bool {
        // Must have same game mode
        if p1.game_mode != p2.game_mode {
            return false;
        }

        // Difficulty must be compatible
        if !p1.difficulty.matches(&p2.difficulty) {
            return false;
        }

        // For ranked, check rating difference
        if p1.game_mode == GameMode::Ranked {
            let rating_diff = (p1.rating - p2.rating).abs();
            if rating_diff > rating_threshold {
                return false;
            }
        }

        true
    }

    /// Get a match by connection ID (for recently matched players)
    pub async fn get_match_for_player(&self, connection_id: &str) -> Option<Match> {
        let recent = self.recent_matches.read().await;
        recent
            .iter()
            .find(|m| m.players.iter().any(|p| p.connection_id == connection_id))
            .cloned()
    }
}

/// Generate a room code for matched players
fn generate_room_code() -> String {
    let adjectives = ["SWIFT", "SHARP", "QUICK", "SMART", "BRAVE", "FAST", "COOL", "EPIC"];
    let nouns = ["CODER", "HACKER", "NINJA", "MASTER", "WIZARD", "GENIUS", "HERO", "CHAMP"];

    let adj = adjectives[fastrand::usize(..adjectives.len())];
    let noun = nouns[fastrand::usize(..nouns.len())];
    let num = fastrand::u16(1000..10000);

    format!("{}-{}-{}", adj, noun, num)
}

/// Resolve difficulty when matching two players with potentially different preferences
fn resolve_difficulty(d1: &QueueDifficulty, d2: &QueueDifficulty) -> QueueDifficulty {
    match (d1, d2) {
        (QueueDifficulty::Any, QueueDifficulty::Any) => {
            // Pick a random difficulty
            let difficulties = [QueueDifficulty::Easy, QueueDifficulty::Medium, QueueDifficulty::Hard];
            difficulties[fastrand::usize(..difficulties.len())]
        }
        (QueueDifficulty::Any, other) | (other, QueueDifficulty::Any) => *other,
        (d, _) => *d, // Same difficulty
    }
}

/// Request to join the matchmaking queue
#[derive(Debug, Clone, Deserialize)]
pub struct JoinQueueRequest {
    pub username: String,
    pub difficulty: QueueDifficulty,
    pub game_mode: GameMode,
}

/// Response for queue status
#[derive(Debug, Clone, Serialize)]
pub struct QueueStatus {
    pub in_queue: bool,
    pub position: Option<usize>,
    pub queue_size: usize,
    pub estimated_wait_seconds: Option<i64>,
}

/// Match found notification
#[derive(Debug, Clone, Serialize)]
pub struct MatchFoundNotification {
    pub room_code: String,
    pub opponent: String,
    pub difficulty: String,
    pub game_mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_player(id: &str, rating: i32, difficulty: QueueDifficulty, game_mode: GameMode) -> QueuedPlayer {
        QueuedPlayer {
            user_id: None,
            username: format!("player_{}", id),
            rating,
            difficulty,
            game_mode,
            queued_at: Utc::now(),
            connection_id: id.to_string(),
        }
    }

    #[tokio::test]
    async fn test_join_and_leave_queue() {
        let queue = MatchmakingQueue::new();
        let player = create_test_player("1", 1200, QueueDifficulty::Medium, GameMode::Casual);

        queue.join_queue(player.clone()).await;
        assert_eq!(queue.queue_size().await, 1);

        let removed = queue.leave_queue("1").await;
        assert!(removed.is_some());
        assert_eq!(queue.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_match_two_players_casual() {
        let queue = MatchmakingQueue::new();

        let p1 = create_test_player("1", 1200, QueueDifficulty::Medium, GameMode::Casual);
        let p2 = create_test_player("2", 1200, QueueDifficulty::Medium, GameMode::Casual);

        queue.join_queue(p1).await;
        queue.join_queue(p2).await;

        let matches = queue.process_queue().await;
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].players.len(), 2);
        assert_eq!(queue.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_no_match_different_game_modes() {
        let queue = MatchmakingQueue::new();

        let p1 = create_test_player("1", 1200, QueueDifficulty::Medium, GameMode::Casual);
        let p2 = create_test_player("2", 1200, QueueDifficulty::Medium, GameMode::Ranked);

        queue.join_queue(p1).await;
        queue.join_queue(p2).await;

        let matches = queue.process_queue().await;
        assert!(matches.is_empty());
        assert_eq!(queue.queue_size().await, 2);
    }

    #[tokio::test]
    async fn test_no_match_different_difficulties() {
        let queue = MatchmakingQueue::new();

        let p1 = create_test_player("1", 1200, QueueDifficulty::Easy, GameMode::Casual);
        let p2 = create_test_player("2", 1200, QueueDifficulty::Hard, GameMode::Casual);

        queue.join_queue(p1).await;
        queue.join_queue(p2).await;

        let matches = queue.process_queue().await;
        assert!(matches.is_empty());
    }

    #[tokio::test]
    async fn test_any_difficulty_matches() {
        let queue = MatchmakingQueue::new();

        let p1 = create_test_player("1", 1200, QueueDifficulty::Any, GameMode::Casual);
        let p2 = create_test_player("2", 1200, QueueDifficulty::Hard, GameMode::Casual);

        queue.join_queue(p1).await;
        queue.join_queue(p2).await;

        let matches = queue.process_queue().await;
        assert_eq!(matches.len(), 1);
    }

    #[tokio::test]
    async fn test_rating_threshold_ranked() {
        let queue = MatchmakingQueue::new();

        // Players with large rating difference shouldn't match immediately in ranked
        let p1 = create_test_player("1", 1200, QueueDifficulty::Medium, GameMode::Ranked);
        let p2 = create_test_player("2", 1800, QueueDifficulty::Medium, GameMode::Ranked);

        queue.join_queue(p1).await;
        queue.join_queue(p2).await;

        let matches = queue.process_queue().await;
        assert!(matches.is_empty()); // Rating diff of 600 exceeds base threshold of 200
    }

    #[test]
    fn test_queue_difficulty_matches() {
        assert!(QueueDifficulty::Any.matches(&QueueDifficulty::Easy));
        assert!(QueueDifficulty::Any.matches(&QueueDifficulty::Medium));
        assert!(QueueDifficulty::Any.matches(&QueueDifficulty::Hard));
        assert!(QueueDifficulty::Easy.matches(&QueueDifficulty::Any));

        assert!(QueueDifficulty::Easy.matches(&QueueDifficulty::Easy));
        assert!(!QueueDifficulty::Easy.matches(&QueueDifficulty::Medium));
        assert!(!QueueDifficulty::Easy.matches(&QueueDifficulty::Hard));
    }
}
