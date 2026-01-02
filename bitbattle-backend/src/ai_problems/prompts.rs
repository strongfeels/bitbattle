use crate::problems::Difficulty;

/// System prompt for problem generation
pub const SYSTEM_PROMPT: &str = r#"You are an expert competitive programming problem creator for a real-time coding battle game. Generate coding problems that are:
- Clear and unambiguous
- Self-contained (no external data, files, or APIs needed)
- Solvable with standard algorithms and data structures
- Fun and engaging for competitive play

CRITICAL RULES:
1. Input/output must be simple, parseable formats (JSON arrays, numbers, strings)
2. Test cases must include edge cases and be comprehensive
3. The problem MUST be solvable - you will provide a working reference solution
4. Do NOT use any external libraries beyond standard library

SUPPORTED LANGUAGES (provide starter code for ALL):
- javascript, python, rust, go, java, c, cpp

OUTPUT FORMAT - Return ONLY valid JSON with these exact keys:
- title: string (2-5 words)
- description: string (full problem description with constraints)
- examples: array of {input, expected_output, explanation}
- test_cases: array of {input, expected_output} (hidden tests, 3-6 items)
- starter_code: object with language keys (javascript, python, rust, go, java, c, cpp)
- time_limit_minutes: number (10-40 based on difficulty)
- tags: array of strings (e.g., ["array", "math"])
- reference_solution: object with {language, code} - MUST be a working solution

IMPORTANT: The reference_solution MUST work correctly with all test cases. This will be used to validate the problem is solvable."#;

/// Build the user prompt for a specific difficulty
pub fn build_generation_prompt(difficulty: Difficulty) -> String {
    match difficulty {
        Difficulty::Easy => r#"Generate an EASY difficulty coding problem.

EASY problem requirements:
- Single concept: arrays, strings, basic math, or simple iteration
- Time complexity: O(n) or O(n log n) at most
- 3-4 test cases with simple edge cases
- Solvable in 10-15 minutes by an intermediate programmer
- Examples: sum of array, reverse string, find max, count occurrences, palindrome check

Focus on clarity and straightforward logic. No tricky edge cases.

Generate a unique, original problem (not Two Sum, FizzBuzz, or other common problems).

Return only valid JSON."#.to_string(),

        Difficulty::Medium => r#"Generate a MEDIUM difficulty coding problem.

MEDIUM problem requirements:
- Combines 2-3 concepts: hash maps, stacks, two pointers, sliding window, sorting
- Time complexity: O(n log n) to O(n^2) acceptable
- 4-5 test cases including tricky edge cases
- Solvable in 20-25 minutes by an intermediate programmer
- Examples: merge intervals, group anagrams, valid parentheses with multiple types

Include at least one non-obvious edge case. Requires some algorithmic thinking.

Generate a unique, original problem (not common LeetCode problems).

Return only valid JSON."#.to_string(),

        Difficulty::Hard => r#"Generate a HARD difficulty coding problem.

HARD problem requirements:
- Complex algorithms: dynamic programming, graphs, trees, advanced data structures
- Optimization is key - brute force should time out
- 5-6 test cases with complex edge cases and large inputs
- Solvable in 30-40 minutes by an experienced programmer
- Examples: longest increasing subsequence, shortest path, tree serialization

The naive solution should be obvious but inefficient. The optimal solution requires insight.

Generate a unique, original problem (not common LeetCode problems).

Return only valid JSON."#.to_string(),
    }
}

/// Parse the LLM response into problem components
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GeneratedProblem {
    pub title: String,
    pub description: String,
    pub examples: Vec<TestCaseJson>,
    pub test_cases: Vec<TestCaseJson>,
    pub starter_code: std::collections::HashMap<String, String>,
    pub time_limit_minutes: Option<u32>,
    pub tags: Vec<String>,
    pub reference_solution: ReferenceSolution,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TestCaseJson {
    pub input: String,
    pub expected_output: String,
    #[serde(default)]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReferenceSolution {
    pub language: String,
    pub code: String,
}

impl GeneratedProblem {
    /// Parse from LLM response text
    pub fn from_llm_response(response: &str) -> Result<Self, serde_json::Error> {
        // Try to extract JSON from the response (in case there's extra text)
        let json_str = extract_json(response);
        serde_json::from_str(json_str)
    }
}

/// Extract JSON object from a string that may have extra text
fn extract_json(text: &str) -> &str {
    // Find first { and last }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return &text[start..=end];
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json() {
        let text = r#"Here's the problem:
{"title": "Test"}
Done!"#;
        assert_eq!(extract_json(text), r#"{"title": "Test"}"#);
    }

    #[test]
    fn test_difficulty_prompts() {
        let easy = build_generation_prompt(Difficulty::Easy);
        assert!(easy.contains("EASY"));

        let medium = build_generation_prompt(Difficulty::Medium);
        assert!(medium.contains("MEDIUM"));

        let hard = build_generation_prompt(Difficulty::Hard);
        assert!(hard.contains("HARD"));
    }
}
