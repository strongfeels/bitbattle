use std::sync::Arc;

use crate::executor::CodeExecutor;
use crate::problems::{Problem, TestCase};

use super::prompts::{GeneratedProblem, TestCaseJson};

/// Validates that AI-generated problems are solvable
pub struct ProblemValidator {
    executor: Arc<CodeExecutor>,
}

impl ProblemValidator {
    pub fn new(executor: Arc<CodeExecutor>) -> Self {
        Self { executor }
    }

    /// Validate a generated problem by running the reference solution
    pub async fn validate(&self, generated: &GeneratedProblem) -> ValidationResult {
        // First, check the structure
        if let Err(e) = self.validate_structure(generated) {
            return ValidationResult::Invalid(format!("Structure error: {}", e));
        }

        // Run the reference solution against all test cases
        match self.run_reference_solution(generated).await {
            Ok(()) => ValidationResult::Valid,
            Err(e) => ValidationResult::Invalid(e),
        }
    }

    /// Validate the problem structure
    fn validate_structure(&self, problem: &GeneratedProblem) -> Result<(), String> {
        // Check title
        if problem.title.trim().is_empty() {
            return Err("Title is empty".to_string());
        }
        if problem.title.len() > 100 {
            return Err("Title too long".to_string());
        }

        // Check description
        if problem.description.trim().is_empty() {
            return Err("Description is empty".to_string());
        }
        if problem.description.len() < 50 {
            return Err("Description too short".to_string());
        }

        // Check examples
        if problem.examples.is_empty() {
            return Err("No examples provided".to_string());
        }
        if problem.examples.len() > 5 {
            return Err("Too many examples".to_string());
        }

        // Check test cases
        if problem.test_cases.is_empty() {
            return Err("No test cases provided".to_string());
        }
        if problem.test_cases.len() < 3 {
            return Err("Need at least 3 test cases".to_string());
        }
        if problem.test_cases.len() > 10 {
            return Err("Too many test cases".to_string());
        }

        // Check starter code - need at least JavaScript and Python
        let required_langs = ["javascript", "python"];
        for lang in required_langs {
            if !problem.starter_code.contains_key(lang) {
                return Err(format!("Missing starter code for {}", lang));
            }
        }

        // Check reference solution
        if problem.reference_solution.code.trim().is_empty() {
            return Err("Reference solution is empty".to_string());
        }

        // Validate the reference solution language
        let valid_langs = ["javascript", "python", "rust", "go", "java", "c", "cpp"];
        if !valid_langs.contains(&problem.reference_solution.language.as_str()) {
            return Err(format!(
                "Invalid reference solution language: {}",
                problem.reference_solution.language
            ));
        }

        Ok(())
    }

    /// Run the reference solution against all test cases
    async fn run_reference_solution(&self, problem: &GeneratedProblem) -> Result<(), String> {
        let language = &problem.reference_solution.language;
        let code = &problem.reference_solution.code;

        // Convert to Problem struct for executor
        let test_cases: Vec<TestCase> = problem
            .test_cases
            .iter()
            .map(|tc| TestCase {
                input: tc.input.clone(),
                expected_output: tc.expected_output.clone(),
                explanation: tc.explanation.clone(),
            })
            .collect();

        let temp_problem = Problem {
            id: "validation-temp".to_string(),
            title: problem.title.clone(),
            description: problem.description.clone(),
            difficulty: crate::problems::Difficulty::Medium,
            examples: vec![],
            test_cases,
            starter_code: std::collections::HashMap::new(),
            time_limit_minutes: Some(5),
            tags: vec![],
        };

        // Create submission request
        let request = crate::executor::SubmissionRequest {
            username: "validator".to_string(),
            problem_id: "validation-temp".to_string(),
            code: code.clone(),
            language: language.clone(),
            room_id: None,
        };

        // Execute
        let result = self.executor.execute_submission(request, &temp_problem).await;

        if result.passed {
            Ok(())
        } else {
            let failed_tests: Vec<String> = result
                .test_results
                .iter()
                .filter(|r| !r.passed)
                .map(|r| {
                    format!(
                        "Input: {}, Expected: {}, Got: {}",
                        r.input, r.expected_output, r.actual_output
                    )
                })
                .collect();

            Err(format!(
                "Reference solution failed {} of {} tests: {}",
                result.total_tests - result.passed_tests,
                result.total_tests,
                failed_tests.join("; ")
            ))
        }
    }
}

/// Result of problem validation
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::Invalid(msg) => Some(msg),
        }
    }
}

/// Convert generated problem to test cases
pub fn to_test_cases(test_cases: &[TestCaseJson]) -> Vec<TestCase> {
    test_cases
        .iter()
        .map(|tc| TestCase {
            input: tc.input.clone(),
            expected_output: tc.expected_output.clone(),
            explanation: tc.explanation.clone(),
        })
        .collect()
}
