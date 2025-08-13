use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::fs;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use crate::problems::{Problem, TestCase};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRequest {
    pub username: String,
    pub problem_id: String,
    pub code: String,
    pub language: String,
    pub room_id: Option<String>, // Add optional room_id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub input: String,
    pub expected_output: String,
    pub actual_output: String,
    pub passed: bool,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionResult {
    pub username: String,
    pub problem_id: String,
    pub passed: bool,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub test_results: Vec<TestResult>,
    pub execution_time_ms: u64,
    pub submission_time: i64,
}

pub struct CodeExecutor {
    temp_dir: String,
}

impl CodeExecutor {
    pub fn new() -> Self {
        // Create a temporary directory for code execution
        let temp_dir = format!("/tmp/bitbattle_{}", std::process::id());
        std::fs::create_dir_all(&temp_dir).unwrap_or_else(|_| {});

        CodeExecutor { temp_dir }
    }

    pub async fn execute_submission(
        &self,
        request: SubmissionRequest,
        problem: &Problem,
    ) -> SubmissionResult {
        let start_time = Instant::now();

        match request.language.as_str() {
            "javascript" => self.execute_javascript(request, problem).await,
            "python" => self.execute_python(request, problem).await,
            _ => SubmissionResult {
                username: request.username,
                problem_id: request.problem_id,
                passed: false,
                total_tests: 0,
                passed_tests: 0,
                test_results: vec![],
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                submission_time: chrono::Utc::now().timestamp(),
            },
        }
    }

    async fn execute_javascript(
        &self,
        request: SubmissionRequest,
        problem: &Problem,
    ) -> SubmissionResult {
        let start_time = Instant::now();
        let mut test_results = Vec::new();

        for (index, test_case) in problem.test_cases.iter().enumerate() {
            let test_result = self.run_javascript_test(&request, test_case, index).await;
            test_results.push(test_result);
        }

        let passed_tests = test_results.iter().filter(|r| r.passed).count();
        let total_tests = test_results.len();

        SubmissionResult {
            username: request.username,
            problem_id: request.problem_id,
            passed: passed_tests == total_tests,
            total_tests,
            passed_tests,
            test_results,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            submission_time: chrono::Utc::now().timestamp(),
        }
    }

    async fn run_javascript_test(
        &self,
        request: &SubmissionRequest,
        test_case: &TestCase,
        test_index: usize,
    ) -> TestResult {
        let test_start = Instant::now();

        // Create a unique filename for this test
        let filename = format!("{}/test_{}_{}_{}.js",
                               self.temp_dir, request.username, request.problem_id, test_index);

        // Create test wrapper code
        let test_code = self.create_javascript_test_wrapper(&request.code, test_case, &request.problem_id);

        // Write code to file
        if let Err(e) = fs::write(&filename, &test_code) {
            return TestResult {
                input: test_case.input.clone(),
                expected_output: test_case.expected_output.clone(),
                actual_output: String::new(),
                passed: false,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error: Some(format!("Failed to write test file: {}", e)),
            };
        }

        // Execute with timeout
        let execution_result = timeout(
            Duration::from_secs(5), // 5 second timeout
            self.run_node_command(&filename)
        ).await;

        // Clean up file
        let _ = fs::remove_file(&filename);

        match execution_result {
            Ok(Ok(output)) => {
                let actual_output = output.trim().to_string();
                let expected_output = test_case.expected_output.trim();
                let passed = actual_output == expected_output;

                TestResult {
                    input: test_case.input.clone(),
                    expected_output: test_case.expected_output.clone(),
                    actual_output,
                    passed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            Ok(Err(error)) => TestResult {
                input: test_case.input.clone(),
                expected_output: test_case.expected_output.clone(),
                actual_output: String::new(),
                passed: false,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error: Some(error),
            },
            Err(_) => TestResult {
                input: test_case.input.clone(),
                expected_output: test_case.expected_output.clone(),
                actual_output: String::new(),
                passed: false,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error: Some("Execution timeout (5 seconds)".to_string()),
            },
        }
    }

    async fn run_node_command(&self, filename: &str) -> Result<String, String> {
        let output = Command::new("node")
            .arg(filename)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute node: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Runtime error: {}", stderr))
        }
    }

    fn create_javascript_test_wrapper(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        match problem_id {
            "two-sum" => {
                let input_parts: Vec<&str> = test_case.input.split_whitespace().collect();
                if input_parts.len() >= 2 {
                    let array_part = input_parts[0];
                    let target = input_parts[1];

                    format!(r#"
{}

// Test execution
try {{
    const nums = {};
    const target = {};
    const result = twoSum(nums, target);
    console.log(JSON.stringify(result));
}} catch (error) {{
    console.error("Error:", error.message);
}}
"#, user_code, array_part, target)
                } else {
                    format!("{}\nconsole.log('Invalid test input');", user_code)
                }
            }
            "reverse-string" => {
                format!(r#"
{}

// Test execution
try {{
    const s = {};
    reverseString(s);
    console.log(JSON.stringify(s));
}} catch (error) {{
    console.error("Error:", error.message);
}}
"#, user_code, test_case.input)
            }
            "valid-parentheses" => {
                format!(r#"
{}

// Test execution
try {{
    const s = "{}";
    const result = isValid(s);
    console.log(result);
}} catch (error) {{
    console.error("Error:", error.message);
}}
"#, user_code, test_case.input)
            }
            _ => {
                format!("{}\nconsole.log('Unknown problem type');", user_code)
            }
        }
    }

    async fn execute_python(
        &self,
        request: SubmissionRequest,
        problem: &Problem,
    ) -> SubmissionResult {
        // Similar to JavaScript but for Python
        // For now, we'll just return a placeholder
        SubmissionResult {
            username: request.username,
            problem_id: request.problem_id,
            passed: false,
            total_tests: 0,
            passed_tests: 0,
            test_results: vec![],
            execution_time_ms: 0,
            submission_time: chrono::Utc::now().timestamp(),
        }
    }
}