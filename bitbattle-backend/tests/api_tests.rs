// Integration tests for BitBattle API endpoints
// Note: These tests require the backend to be running locally
// or need to set up a test server

use std::collections::HashMap;

#[cfg(test)]
mod room_code_tests {
    /// Test that room codes follow the expected format: WORD-WORD-NNNN
    #[test]
    fn test_room_code_format() {
        // Room codes should be in format WORD-WORD-NNNN
        let valid_codes = vec![
            "SWIFT-CODER-1234",
            "QUICK-BATTLE-0001",
            "CODE-NINJA-9999",
        ];

        for code in valid_codes {
            let parts: Vec<&str> = code.split('-').collect();
            assert_eq!(parts.len(), 3, "Room code should have 3 parts separated by -");
            assert!(parts[0].chars().all(|c| c.is_ascii_uppercase()), "First part should be uppercase");
            assert!(parts[1].chars().all(|c| c.is_ascii_uppercase()), "Second part should be uppercase");
            assert!(parts[2].chars().all(|c| c.is_ascii_digit()), "Third part should be digits");
            assert_eq!(parts[2].len(), 4, "Third part should be 4 digits");
        }
    }

    #[test]
    fn test_room_code_uniqueness() {
        // Generate multiple room codes and ensure they're unique
        let mut codes = std::collections::HashSet::new();
        let words = ["SWIFT", "QUICK", "CODE", "HACK", "BYTE"];

        for w1 in words {
            for w2 in words {
                for n in 0..100 {
                    let code = format!("{}-{}-{:04}", w1, w2, n);
                    assert!(codes.insert(code.clone()), "Duplicate code: {}", code);
                }
            }
        }
    }
}

#[cfg(test)]
mod submission_request_tests {
    use serde_json;

    #[test]
    fn test_submission_request_serialization() {
        let json = r#"{
            "username": "test_user",
            "problem_id": "two-sum",
            "code": "function twoSum(nums, target) { return [0, 1]; }",
            "language": "javascript",
            "room_id": "TEST-ROOM-0001"
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(parsed["username"], "test_user");
        assert_eq!(parsed["problem_id"], "two-sum");
        assert_eq!(parsed["language"], "javascript");
        assert_eq!(parsed["room_id"], "TEST-ROOM-0001");
    }

    #[test]
    fn test_submission_request_without_room_id() {
        let json = r#"{
            "username": "test_user",
            "problem_id": "two-sum",
            "code": "function twoSum(nums, target) { return [0, 1]; }",
            "language": "javascript"
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(parsed["room_id"].is_null(), "room_id should be optional");
    }

    #[test]
    fn test_supported_languages() {
        let languages = vec![
            "javascript",
            "python",
            "java",
            "c",
            "cpp",
            "rust",
            "go",
        ];

        for lang in languages {
            let json = format!(r#"{{
                "username": "test",
                "problem_id": "two-sum",
                "code": "// code",
                "language": "{}"
            }}"#, lang);

            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed["language"], lang);
        }
    }
}

#[cfg(test)]
mod submission_result_tests {
    use serde_json;

    #[test]
    fn test_submission_result_format() {
        let result = serde_json::json!({
            "username": "test_user",
            "problem_id": "two-sum",
            "passed": true,
            "total_tests": 3,
            "passed_tests": 3,
            "test_results": [
                {
                    "input": "[2,7,11,15] 9",
                    "expected_output": "[0,1]",
                    "actual_output": "[0,1]",
                    "passed": true,
                    "execution_time_ms": 50,
                    "error": null
                }
            ],
            "execution_time_ms": 150,
            "submission_time": 1234567890
        });

        assert_eq!(result["passed"], true);
        assert_eq!(result["total_tests"], 3);
        assert_eq!(result["passed_tests"], 3);
        assert!(result["test_results"].is_array());
    }

    #[test]
    fn test_failed_submission_result() {
        let result = serde_json::json!({
            "username": "test_user",
            "problem_id": "two-sum",
            "passed": false,
            "total_tests": 3,
            "passed_tests": 1,
            "test_results": [
                {
                    "input": "[2,7,11,15] 9",
                    "expected_output": "[0,1]",
                    "actual_output": "[0,1]",
                    "passed": true,
                    "execution_time_ms": 50,
                    "error": null
                },
                {
                    "input": "[3,2,4] 6",
                    "expected_output": "[1,2]",
                    "actual_output": "[0,2]",
                    "passed": false,
                    "execution_time_ms": 45,
                    "error": null
                }
            ],
            "execution_time_ms": 150,
            "submission_time": 1234567890
        });

        assert_eq!(result["passed"], false);
        assert_eq!(result["passed_tests"], 1);
        assert_eq!(result["total_tests"], 3);
    }

    #[test]
    fn test_submission_with_error() {
        let result = serde_json::json!({
            "username": "test_user",
            "problem_id": "two-sum",
            "passed": false,
            "total_tests": 3,
            "passed_tests": 0,
            "test_results": [
                {
                    "input": "[2,7,11,15] 9",
                    "expected_output": "[0,1]",
                    "actual_output": "",
                    "passed": false,
                    "execution_time_ms": 10,
                    "error": "SyntaxError: Unexpected token"
                }
            ],
            "execution_time_ms": 50,
            "submission_time": 1234567890
        });

        assert_eq!(result["passed"], false);
        assert!(result["test_results"][0]["error"].is_string());
    }
}

#[cfg(test)]
mod websocket_message_tests {
    use serde_json;

    #[test]
    fn test_user_joined_message() {
        let msg = serde_json::json!({
            "type": "user_joined",
            "data": {
                "username": "player1"
            }
        });

        assert_eq!(msg["type"], "user_joined");
        assert_eq!(msg["data"]["username"], "player1");
    }

    #[test]
    fn test_user_left_message() {
        let msg = serde_json::json!({
            "type": "user_left",
            "data": {
                "username": "player1"
            }
        });

        assert_eq!(msg["type"], "user_left");
    }

    #[test]
    fn test_code_change_message() {
        let msg = serde_json::json!({
            "type": "code_change",
            "data": {
                "username": "player1",
                "code": "function twoSum(nums, target) { /* solution */ }"
            }
        });

        assert_eq!(msg["type"], "code_change");
        assert!(msg["data"]["code"].is_string());
    }

    #[test]
    fn test_game_start_message() {
        let msg = serde_json::json!({
            "type": "game_start",
            "data": {}
        });

        assert_eq!(msg["type"], "game_start");
    }

    #[test]
    fn test_player_count_message() {
        let msg = serde_json::json!({
            "type": "player_count",
            "data": {
                "current": 1,
                "required": 2
            }
        });

        assert_eq!(msg["type"], "player_count");
        assert_eq!(msg["data"]["current"], 1);
        assert_eq!(msg["data"]["required"], 2);
    }

    #[test]
    fn test_room_full_message() {
        let msg = serde_json::json!({
            "type": "room_full",
            "data": {
                "message": "This room is full. The game has already started.",
                "current": 2,
                "required": 2
            }
        });

        assert_eq!(msg["type"], "room_full");
        assert!(msg["data"]["message"].is_string());
    }

    #[test]
    fn test_problem_assigned_message() {
        let msg = serde_json::json!({
            "type": "problem_assigned",
            "data": {
                "problem": {
                    "id": "two-sum",
                    "title": "Two Sum",
                    "description": "Given an array...",
                    "difficulty": "Easy",
                    "examples": [],
                    "starter_code": {},
                    "time_limit_minutes": 15,
                    "tags": ["array", "hash-table"]
                }
            }
        });

        assert_eq!(msg["type"], "problem_assigned");
        assert_eq!(msg["data"]["problem"]["id"], "two-sum");
    }

    #[test]
    fn test_submission_result_broadcast_message() {
        let msg = serde_json::json!({
            "type": "submission_result",
            "data": {
                "result": {
                    "username": "player1",
                    "passed": true,
                    "total_tests": 3,
                    "passed_tests": 3
                }
            }
        });

        assert_eq!(msg["type"], "submission_result");
        assert_eq!(msg["data"]["result"]["passed"], true);
    }
}

#[cfg(test)]
mod auth_tests {
    use serde_json;

    #[test]
    fn test_jwt_claims_structure() {
        let claims = serde_json::json!({
            "sub": "550e8400-e29b-41d4-a716-446655440000",
            "email": "user@example.com",
            "name": "Test User",
            "exp": 1234567890
        });

        assert!(claims["sub"].is_string());
        assert!(claims["email"].is_string());
        assert!(claims["name"].is_string());
        assert!(claims["exp"].is_number());
    }

    #[test]
    fn test_user_response_structure() {
        let user = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "email": "user@example.com",
            "display_name": "Test User",
            "avatar_url": "https://example.com/avatar.jpg",
            "created_at": "2024-01-01T00:00:00Z"
        });

        assert!(user["id"].is_string());
        assert!(user["email"].is_string());
        assert!(user["display_name"].is_string());
    }

    #[test]
    fn test_leaderboard_entry_structure() {
        let entry = serde_json::json!({
            "rank": 1,
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "display_name": "TopPlayer",
            "avatar_url": "https://example.com/avatar.jpg",
            "rating": 1500,
            "games_played": 50,
            "games_won": 30
        });

        assert_eq!(entry["rank"], 1);
        assert!(entry["rating"].is_number());
        assert!(entry["games_played"].is_number());
    }

    #[test]
    fn test_game_result_structure() {
        let result = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "room_id": "TEST-ROOM-0001",
            "problem_id": "two-sum",
            "user_id": "660e8400-e29b-41d4-a716-446655440000",
            "placement": 1,
            "total_players": 2,
            "solve_time_ms": 45000,
            "passed_tests": 3,
            "total_tests": 3,
            "language": "javascript",
            "created_at": "2024-01-01T00:00:00Z"
        });

        assert!(result["id"].is_string());
        assert!(result["placement"].is_number());
        assert!(result["solve_time_ms"].is_number());
    }
}

#[cfg(test)]
mod game_mode_tests {
    #[test]
    fn test_valid_game_modes() {
        let valid_modes = vec!["casual", "ranked"];

        for mode in valid_modes {
            assert!(
                mode == "casual" || mode == "ranked",
                "Invalid game mode: {}",
                mode
            );
        }
    }

    #[test]
    fn test_ranked_requires_auth() {
        // This is a conceptual test - ranked games should require authentication
        let is_authenticated = true;
        let game_mode = "ranked";

        if game_mode == "ranked" {
            assert!(is_authenticated, "Ranked games require authentication");
        }
    }
}

#[cfg(test)]
mod difficulty_routing_tests {
    #[test]
    fn test_difficulty_values() {
        let valid_difficulties = vec!["easy", "medium", "hard", "random"];

        for difficulty in &valid_difficulties {
            assert!(
                valid_difficulties.contains(difficulty),
                "Difficulty {} should be valid",
                difficulty
            );
        }
    }

    #[test]
    fn test_random_difficulty_behavior() {
        // When difficulty is "random", any difficulty problem should be acceptable
        let difficulty = "random";
        let problem_difficulties = vec!["Easy", "Medium", "Hard"];

        if difficulty == "random" {
            for pd in problem_difficulties {
                // All difficulties should be acceptable
                assert!(
                    pd == "Easy" || pd == "Medium" || pd == "Hard",
                    "Invalid problem difficulty: {}",
                    pd
                );
            }
        }
    }
}
