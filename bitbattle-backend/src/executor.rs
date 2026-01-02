use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, StartContainerOptions,
    WaitContainerOptions,
};
use bollard::models::{HostConfig, ContainerWaitResponse};
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::time::{timeout, Duration};

use crate::problems::{Problem, TestCase};

const SANDBOX_IMAGE: &str = "bitbattle-sandbox:latest";
const EXECUTION_TIMEOUT_SECS: u64 = 10;
const MEMORY_LIMIT: i64 = 128 * 1024 * 1024; // 128MB
const NANO_CPUS: i64 = 500_000_000; // 0.5 CPU

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionRequest {
    pub username: String,
    pub problem_id: String,
    pub code: String,
    pub language: String,
    pub room_id: Option<String>,
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
    docker: Docker,
}

impl CodeExecutor {
    pub fn new() -> Self {
        let docker = Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker daemon");

        CodeExecutor { docker }
    }

    pub async fn execute_submission(
        &self,
        request: SubmissionRequest,
        problem: &Problem,
    ) -> SubmissionResult {
        let start_time = Instant::now();
        let mut test_results = Vec::new();

        for test_case in problem.test_cases.iter() {
            let test_result = self.run_test(&request, test_case).await;
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

    async fn run_test(&self, request: &SubmissionRequest, test_case: &TestCase) -> TestResult {
        let test_start = Instant::now();
        let language = request.language.as_str();

        // Create the full code with test harness
        let full_code = self.create_test_harness(&request.code, test_case, &request.problem_id, language);

        let execution_result = timeout(
            Duration::from_secs(EXECUTION_TIMEOUT_SECS),
            self.run_code(&full_code, language),
        )
        .await;

        match execution_result {
            Ok(Ok(output)) => {
                let actual_output = output.trim().to_string();
                let expected_output = test_case.expected_output.trim();
                let passed = self.compare_outputs(&actual_output, expected_output);

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
                error: Some(format!("Execution timeout ({} seconds)", EXECUTION_TIMEOUT_SECS)),
            },
        }
    }

    fn compare_outputs(&self, actual: &str, expected: &str) -> bool {
        // Normalize whitespace and compare
        let actual_normalized: String = actual.split_whitespace().collect::<Vec<_>>().join(" ");
        let expected_normalized: String = expected.split_whitespace().collect::<Vec<_>>().join(" ");
        actual_normalized == expected_normalized
    }

    async fn run_code(&self, code: &str, language: &str) -> Result<String, String> {
        match language {
            "javascript" | "python" => self.run_interpreted(code, language).await,
            "c" | "cpp" | "rust" | "go" | "java" => self.run_compiled(code, language).await,
            _ => Err(format!("Unsupported language: {}", language)),
        }
    }

    async fn run_interpreted(&self, code: &str, language: &str) -> Result<String, String> {
        let (cmd, filename) = match language {
            "javascript" => (vec!["node", "/tmp/code.js"], "code.js"),
            "python" => (vec!["python3", "/tmp/code.py"], "code.py"),
            _ => return Err("Unsupported interpreted language".to_string()),
        };

        self.execute_in_container(code, filename, cmd, false).await
    }

    async fn run_compiled(&self, code: &str, language: &str) -> Result<String, String> {
        let (compile_cmd, run_cmd, filename) = match language {
            "c" => ("gcc -o /tmp/prog /tmp/code.c -lm", "/tmp/prog", "code.c"),
            "cpp" => ("g++ -o /tmp/prog /tmp/code.cpp", "/tmp/prog", "code.cpp"),
            "rust" => ("rustc -o /tmp/prog /tmp/code.rs 2>&1", "/tmp/prog", "code.rs"),
            "go" => ("go build -o /tmp/prog /tmp/code.go", "/tmp/prog", "code.go"),
            "java" => ("javac /tmp/Solution.java", "java -cp /tmp Solution", "Solution.java"),
            _ => return Err("Unsupported compiled language".to_string()),
        };

        let script = format!("{} && {}", compile_cmd, run_cmd);
        self.execute_in_container(code, filename, vec!["sh", "-c", &script], true).await
    }

    async fn execute_in_container(
        &self,
        code: &str,
        filename: &str,
        cmd: Vec<&str>,
        is_compiled: bool,
    ) -> Result<String, String> {
        let container_name = format!("bitbattle-{}-{}", std::process::id(), fastrand::u64(..));

        let (memory, cpu) = if is_compiled {
            (MEMORY_LIMIT * 2, NANO_CPUS * 2) // More resources for compilation
        } else {
            (MEMORY_LIMIT, NANO_CPUS)
        };

        let host_config = HostConfig {
            memory: Some(memory),
            nano_cpus: Some(cpu),
            network_mode: Some("none".to_string()),
            pids_limit: Some(50),
            ..Default::default()
        };

        let config = Config {
            image: Some(SANDBOX_IMAGE.to_string()),
            cmd: Some(cmd.iter().map(|s| s.to_string()).collect()),
            host_config: Some(host_config),
            user: Some("runner".to_string()),
            working_dir: Some("/tmp".to_string()),
            tty: Some(false),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: &container_name,
                    platform: None,
                }),
                config,
            )
            .await
            .map_err(|e| format!("Failed to create container: {}", e))?;

        // Upload code
        let tar_data = self.create_tar_archive(filename, code)?;
        self.docker
            .upload_to_container::<String>(
                &container.id,
                Some(bollard::container::UploadToContainerOptions {
                    path: "/tmp".to_string(),
                    ..Default::default()
                }),
                tar_data.into(),
            )
            .await
            .map_err(|e| format!("Failed to upload code: {}", e))?;

        // Start container
        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| format!("Failed to start container: {}", e))?;

        // Wait for completion
        let mut wait_stream = self.docker.wait_container(
            &container.id,
            Some(WaitContainerOptions { condition: "not-running" }),
        );
        let wait_result: Option<Result<ContainerWaitResponse, _>> = wait_stream.next().await;

        // Collect output
        let mut stdout = String::new();
        let mut stderr = String::new();

        let mut logs_stream = self.docker.logs::<String>(
            &container.id,
            Some(LogsOptions {
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        );

        while let Some(log_result) = logs_stream.next().await {
            if let Ok(log) = log_result {
                match log {
                    bollard::container::LogOutput::StdOut { message } => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    bollard::container::LogOutput::StdErr { message } => {
                        stderr.push_str(&String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
            }
        }

        // Cleanup
        let _ = self.docker.remove_container(
            &container.id,
            Some(RemoveContainerOptions { force: true, ..Default::default() }),
        ).await;

        match wait_result {
            Some(Ok(response)) if response.status_code == 0 => Ok(stdout),
            Some(Ok(_)) => Err(Self::clean_error(&stderr, is_compiled)),
            Some(Err(_)) => Err("Execution failed. Check your code for errors.".to_string()),
            None => Err("Execution interrupted.".to_string()),
        }
    }

    fn clean_error(stderr: &str, is_compiled: bool) -> String {
        let stderr = stderr.trim();
        if stderr.is_empty() {
            return if is_compiled {
                "Compilation failed with no error output.".to_string()
            } else {
                "Execution failed with no error output.".to_string()
            };
        }

        // Find the most relevant error line
        for line in stderr.lines() {
            if line.contains("error") || line.contains("Error") ||
               line.contains("SyntaxError") || line.contains("TypeError") ||
               line.contains("ReferenceError") || line.contains("Exception") {
                // Clean up file paths
                let cleaned = line
                    .replace("/tmp/code.js:", "Line ")
                    .replace("/tmp/code.py:", "Line ")
                    .replace("/tmp/code.c:", "Line ")
                    .replace("/tmp/code.cpp:", "Line ")
                    .replace("/tmp/code.rs:", "Line ")
                    .replace("/tmp/code.go:", "Line ");
                return cleaned.trim().to_string();
            }
        }

        // Truncate if needed
        if stderr.len() > 200 {
            format!("{}...", &stderr[..200])
        } else {
            stderr.to_string()
        }
    }

    /// Remove Java main method from user code to avoid duplicate main methods
    fn remove_java_main_method(code: &str) -> String {
        // Find and remove "public static void main(String[] args) { ... }"
        let mut result = code.to_string();

        // Look for main method pattern
        if let Some(main_start) = result.find("public static void main") {
            // Find the opening brace of the main method
            if let Some(brace_start) = result[main_start..].find('{') {
                let brace_pos = main_start + brace_start;
                let mut depth = 1;
                let mut end_pos = brace_pos + 1;

                // Find the matching closing brace
                for (i, c) in result[brace_pos + 1..].char_indices() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end_pos = brace_pos + 1 + i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                // Remove the main method
                result = format!("{}{}", &result[..main_start], &result[end_pos..]);
            }
        }

        result.trim().to_string()
    }

    fn create_tar_archive(&self, filename: &str, content: &str) -> Result<Vec<u8>, String> {
        use tar::Builder;

        let mut archive_data = Vec::new();
        {
            let mut builder = Builder::new(&mut archive_data);
            let content_bytes = content.as_bytes();
            let mut header = tar::Header::new_gnu();
            header.set_path(filename).map_err(|e| e.to_string())?;
            header.set_size(content_bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, content_bytes).map_err(|e| e.to_string())?;
            builder.finish().map_err(|e| e.to_string())?;
        }
        Ok(archive_data)
    }

    /// Creates a complete test harness that embeds the test input and calls the user's solution
    fn create_test_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str, language: &str) -> String {
        match language {
            "javascript" => self.js_harness(user_code, test_case, problem_id),
            "python" => self.py_harness(user_code, test_case, problem_id),
            "c" => self.c_harness(user_code, test_case, problem_id),
            "cpp" => self.cpp_harness(user_code, test_case, problem_id),
            "rust" => self.rust_harness(user_code, test_case, problem_id),
            "go" => self.go_harness(user_code, test_case, problem_id),
            "java" => self.java_harness(user_code, test_case, problem_id),
            _ => user_code.to_string(),
        }
    }

    // ============ JavaScript Test Harness ============
    fn js_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        let test_call = match problem_id {
            "two-sum" => {
                let parts: Vec<&str> = test_case.input.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!("console.log(JSON.stringify(twoSum({}, {})));", parts[0], parts[1])
                } else { "console.log('Invalid input');".to_string() }
            }
            "reverse-string" => {
                format!("let s = {}; reverseString(s); console.log(JSON.stringify(s));", test_case.input)
            }
            "valid-parentheses" => {
                format!("console.log(isValid(\"{}\"));", test_case.input)
            }
            "fizzbuzz" => {
                format!("console.log(JSON.stringify(fizzBuzz({})));", test_case.input)
            }
            "palindrome-number" => {
                format!("console.log(isPalindrome({}));", test_case.input)
            }
            "maximum-subarray" => {
                format!("console.log(maxSubArray({}));", test_case.input)
            }
            "merge-intervals" => {
                format!("console.log(JSON.stringify(merge({})));", test_case.input)
            }
            "group-anagrams" => {
                format!("console.log(JSON.stringify(groupAnagrams({})));", test_case.input)
            }
            "longest-substring" => {
                format!("console.log(lengthOfLongestSubstring(\"{}\"));", test_case.input)
            }
            "trapping-rain-water" => {
                format!("console.log(trap({}));", test_case.input)
            }
            "merge-k-sorted-lists" => {
                format!("console.log(JSON.stringify(mergeKLists({})));", test_case.input)
            }
            "median-two-sorted-arrays" => {
                let parts: Vec<&str> = test_case.input.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!("console.log(findMedianSortedArrays({}, {}).toFixed(1));", parts[0], parts[1])
                } else { "console.log('Invalid input');".to_string() }
            }
            _ => "console.log('Unknown problem');".to_string()
        };
        format!("{}\n\n{}", user_code, test_call)
    }

    // ============ Python Test Harness ============
    fn py_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        let test_call = match problem_id {
            "two-sum" => {
                let parts: Vec<&str> = test_case.input.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!("import json; print(json.dumps(twoSum({}, {})))", parts[0], parts[1])
                } else { "print('Invalid input')".to_string() }
            }
            "reverse-string" => {
                format!("import json; s = {}; reverseString(s); print(json.dumps(s))", test_case.input)
            }
            "valid-parentheses" => {
                format!("print(str(isValid(\"{}\")).lower())", test_case.input)
            }
            "fizzbuzz" => {
                format!("import json; print(json.dumps(fizzBuzz({})))", test_case.input)
            }
            "palindrome-number" => {
                format!("print(str(isPalindrome({})).lower())", test_case.input)
            }
            "maximum-subarray" => {
                format!("print(maxSubArray({}))", test_case.input)
            }
            "merge-intervals" => {
                format!("import json; print(json.dumps(merge({})))", test_case.input)
            }
            "group-anagrams" => {
                format!("import json; print(json.dumps(groupAnagrams({})))", test_case.input)
            }
            "longest-substring" => {
                format!("print(lengthOfLongestSubstring(\"{}\"))", test_case.input)
            }
            "trapping-rain-water" => {
                format!("print(trap({}))", test_case.input)
            }
            "merge-k-sorted-lists" => {
                format!("import json; print(json.dumps(mergeKLists({})))", test_case.input)
            }
            "median-two-sorted-arrays" => {
                let parts: Vec<&str> = test_case.input.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!("print(f\"{{findMedianSortedArrays({}, {}):.1f}}\")", parts[0], parts[1])
                } else { "print('Invalid input')".to_string() }
            }
            _ => "print('Unknown problem')".to_string()
        };
        format!("{}\n\n{}", user_code, test_call)
    }

    // ============ C Test Harness ============
    fn c_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        let input = &test_case.input;
        let (includes, main_code) = match problem_id {
            "two-sum" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let nums: Vec<&str> = parts[0].trim_matches(|c| c == '[' || c == ']').split(',').collect();
                    let size = nums.len();
                    let arr_init = nums.join(", ");
                    (
                        "#include <stdio.h>\n#include <stdlib.h>".to_string(),
                        format!(
                            "int main() {{\n    int nums[] = {{{}}};\n    int returnSize;\n    int* result = twoSum(nums, {}, {}, &returnSize);\n    printf(\"[%d,%d]\", result[0], result[1]);\n    return 0;\n}}",
                            arr_init, size, parts[1]
                        )
                    )
                } else { ("#include <stdio.h>".to_string(), "int main() { return 0; }".to_string()) }
            }
            "reverse-string" => {
                let chars: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let size = chars.len();
                let char_init: Vec<String> = chars.iter().map(|c| c.trim().trim_matches('"').to_string()).collect();
                let arr_init = char_init.iter().map(|c| format!("'{}'", c)).collect::<Vec<_>>().join(", ");
                (
                    "#include <stdio.h>".to_string(),
                    format!(
                        "int main() {{\n    char s[] = {{{}}};\n    int size = {};\n    reverseString(s, size);\n    printf(\"[\");\n    for(int i = 0; i < size; i++) printf(\"\\\"%c\\\"%s\", s[i], i < size-1 ? \",\" : \"\");\n    printf(\"]\");\n    return 0;\n}}",
                        arr_init, size
                    )
                )
            }
            "valid-parentheses" => (
                "#include <stdio.h>\n#include <stdbool.h>\n#include <string.h>".to_string(),
                format!("int main() {{ printf(isValid(\"{}\") ? \"true\" : \"false\"); return 0; }}", input)
            ),
            "fizzbuzz" => (
                "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>".to_string(),
                format!(
                    "int main() {{\n    int returnSize;\n    char** result = fizzBuzz({}, &returnSize);\n    printf(\"[\");\n    for(int i = 0; i < returnSize; i++) printf(\"\\\"%s\\\"%s\", result[i], i < returnSize-1 ? \",\" : \"\");\n    printf(\"]\");\n    return 0;\n}}",
                    input
                )
            ),
            "palindrome-number" => (
                "#include <stdio.h>\n#include <stdbool.h>".to_string(),
                format!("int main() {{ printf(isPalindrome({}) ? \"true\" : \"false\"); return 0; }}", input)
            ),
            "maximum-subarray" => {
                let nums: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let size = nums.len();
                let arr_init = nums.join(", ");
                (
                    "#include <stdio.h>".to_string(),
                    format!(
                        "int main() {{\n    int nums[] = {{{}}};\n    printf(\"%d\", maxSubArray(nums, {}));\n    return 0;\n}}",
                        arr_init, size
                    )
                )
            }
            "merge-intervals" => {
                (
                    "#include <stdio.h>\n#include <stdlib.h>".to_string(),
                    format!(
                        "int main() {{\n    // Parse intervals from: {}\n    printf(\"C merge-intervals requires manual setup\");\n    return 0;\n}}",
                        input
                    )
                )
            }
            "group-anagrams" => (
                "#include <stdio.h>".to_string(),
                "int main() { printf(\"C group-anagrams requires manual setup\"); return 0; }".to_string()
            ),
            "longest-substring" => (
                "#include <stdio.h>\n#include <string.h>".to_string(),
                format!("int main() {{ printf(\"%d\", lengthOfLongestSubstring(\"{}\")); return 0; }}", input)
            ),
            "trapping-rain-water" => {
                let nums: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let size = nums.len();
                let arr_init = nums.join(", ");
                (
                    "#include <stdio.h>".to_string(),
                    format!(
                        "int main() {{\n    int height[] = {{{}}};\n    printf(\"%d\", trap(height, {}));\n    return 0;\n}}",
                        arr_init, size
                    )
                )
            }
            "merge-k-sorted-lists" => (
                "#include <stdio.h>".to_string(),
                "int main() { printf(\"C merge-k-sorted-lists requires manual setup\"); return 0; }".to_string()
            ),
            "median-two-sorted-arrays" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let nums1: Vec<&str> = parts[0].trim_matches(|c| c == '[' || c == ']').split(',').collect();
                    let nums2: Vec<&str> = parts[1].trim_matches(|c| c == '[' || c == ']').split(',').collect();
                    let size1 = nums1.len();
                    let size2 = nums2.len();
                    (
                        "#include <stdio.h>".to_string(),
                        format!(
                            "int main() {{\n    int nums1[] = {{{}}};\n    int nums2[] = {{{}}};\n    printf(\"%.1f\", findMedianSortedArrays(nums1, {}, nums2, {}));\n    return 0;\n}}",
                            nums1.join(", "), nums2.join(", "), size1, size2
                        )
                    )
                } else { ("#include <stdio.h>".to_string(), "int main() { return 0; }".to_string()) }
            }
            _ => ("#include <stdio.h>".to_string(), "int main() { printf(\"Unknown problem\"); return 0; }".to_string())
        };
        format!("{}\n\n{}\n\n{}", includes, user_code, main_code)
    }

    // ============ C++ Test Harness ============
    fn cpp_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        let input = &test_case.input;
        let (includes, main_code) = match problem_id {
            "two-sum" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    (
                        "#include <iostream>\n#include <vector>\nusing namespace std;".to_string(),
                        format!(
                            "int main() {{\n    vector<int> nums = {};\n    vector<int> result = twoSum(nums, {});\n    cout << \"[\" << result[0] << \",\" << result[1] << \"]\";\n    return 0;\n}}",
                            parts[0], parts[1]
                        )
                    )
                } else { ("#include <iostream>\nusing namespace std;".to_string(), "int main() { return 0; }".to_string()) }
            }
            "reverse-string" => (
                "#include <iostream>\n#include <vector>\nusing namespace std;".to_string(),
                format!(
                    "int main() {{\n    vector<char> s;\n    for(char c : string(R\"({})\")) if(c != '[' && c != ']' && c != '\"' && c != ',') s.push_back(c);\n    reverseString(s);\n    cout << \"[\";\n    for(int i = 0; i < s.size(); i++) cout << \"\\\"\" << s[i] << \"\\\"\" << (i < s.size()-1 ? \",\" : \"\");\n    cout << \"]\";\n    return 0;\n}}",
                    input
                )
            ),
            "valid-parentheses" => (
                "#include <iostream>\n#include <string>\nusing namespace std;".to_string(),
                format!("int main() {{ cout << (isValid(\"{}\") ? \"true\" : \"false\"); return 0; }}", input)
            ),
            "fizzbuzz" => (
                "#include <iostream>\n#include <vector>\n#include <string>\nusing namespace std;".to_string(),
                format!(
                    "int main() {{\n    vector<string> result = fizzBuzz({});\n    cout << \"[\";\n    for(int i = 0; i < result.size(); i++) cout << \"\\\"\" << result[i] << \"\\\"\" << (i < result.size()-1 ? \",\" : \"\");\n    cout << \"]\";\n    return 0;\n}}",
                    input
                )
            ),
            "palindrome-number" => (
                "#include <iostream>\nusing namespace std;".to_string(),
                format!("int main() {{ cout << (isPalindrome({}) ? \"true\" : \"false\"); return 0; }}", input)
            ),
            "maximum-subarray" => (
                "#include <iostream>\n#include <vector>\nusing namespace std;".to_string(),
                format!("int main() {{ vector<int> nums = {}; cout << maxSubArray(nums); return 0; }}", input)
            ),
            "merge-intervals" => (
                "#include <iostream>\n#include <vector>\nusing namespace std;".to_string(),
                format!(
                    "int main() {{\n    vector<vector<int>> intervals = {};\n    vector<vector<int>> result = merge(intervals);\n    cout << \"[\";\n    for(int i = 0; i < result.size(); i++) {{\n        cout << \"[\" << result[i][0] << \",\" << result[i][1] << \"]\";\n        if(i < result.size()-1) cout << \",\";\n    }}\n    cout << \"]\";\n    return 0;\n}}",
                    input
                )
            ),
            "group-anagrams" => (
                "#include <iostream>\n#include <vector>\n#include <string>\n#include <unordered_map>\n#include <algorithm>\nusing namespace std;".to_string(),
                format!(
                    "int main() {{\n    vector<string> strs = {};\n    vector<vector<string>> result = groupAnagrams(strs);\n    cout << \"[\";\n    for(int i = 0; i < result.size(); i++) {{\n        cout << \"[\";\n        for(int j = 0; j < result[i].size(); j++) {{\n            cout << \"\\\"\" << result[i][j] << \"\\\"\";\n            if(j < result[i].size()-1) cout << \",\";\n        }}\n        cout << \"]\";\n        if(i < result.size()-1) cout << \",\";\n    }}\n    cout << \"]\";\n    return 0;\n}}",
                    input
                )
            ),
            "longest-substring" => (
                "#include <iostream>\n#include <string>\n#include <unordered_set>\nusing namespace std;".to_string(),
                format!("int main() {{ cout << lengthOfLongestSubstring(\"{}\"); return 0; }}", input)
            ),
            "trapping-rain-water" => (
                "#include <iostream>\n#include <vector>\nusing namespace std;".to_string(),
                format!("int main() {{ vector<int> height = {}; cout << trap(height); return 0; }}", input)
            ),
            "merge-k-sorted-lists" => (
                "#include <iostream>\n#include <vector>\n#include <queue>\nusing namespace std;".to_string(),
                format!(
                    "int main() {{\n    vector<vector<int>> lists = {};\n    vector<int> result = mergeKLists(lists);\n    cout << \"[\";\n    for(int i = 0; i < result.size(); i++) {{\n        cout << result[i];\n        if(i < result.size()-1) cout << \",\";\n    }}\n    cout << \"]\";\n    return 0;\n}}",
                    input
                )
            ),
            "median-two-sorted-arrays" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    (
                        "#include <iostream>\n#include <vector>\n#include <iomanip>\nusing namespace std;".to_string(),
                        format!(
                            "int main() {{\n    vector<int> nums1 = {};\n    vector<int> nums2 = {};\n    cout << fixed << setprecision(1) << findMedianSortedArrays(nums1, nums2);\n    return 0;\n}}",
                            parts[0], parts[1]
                        )
                    )
                } else { ("#include <iostream>\nusing namespace std;".to_string(), "int main() { return 0; }".to_string()) }
            }
            _ => ("#include <iostream>\nusing namespace std;".to_string(), "int main() { cout << \"Unknown problem\"; return 0; }".to_string())
        };
        format!("{}\n\n{}\n\n{}", includes, user_code, main_code)
    }

    // ============ Rust Test Harness ============
    fn rust_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        let input = &test_case.input;
        let main_code = match problem_id {
            "two-sum" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!(
                        "fn main() {{\n    let nums = vec!{};\n    let result = two_sum(nums, {});\n    println!(\"[{{}},{{}}]\", result[0], result[1]);\n}}",
                        parts[0], parts[1]
                    )
                } else { "fn main() {}".to_string() }
            }
            "reverse-string" => {
                let chars: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let char_vec: Vec<String> = chars.iter().map(|c| {
                    let ch = c.trim().trim_matches('"');
                    format!("'{}'", ch)
                }).collect();
                format!(
                    "fn main() {{\n    let mut s: Vec<char> = vec![{}];\n    reverse_string(&mut s);\n    print!(\"[\");\n    for (i, c) in s.iter().enumerate() {{\n        print!(\"\\\"{{}}\\\"\", c);\n        if i < s.len() - 1 {{ print!(\",\"); }}\n    }}\n    println!(\"]\");\n}}",
                    char_vec.join(", ")
                )
            }
            "valid-parentheses" => {
                format!("fn main() {{ println!(\"{{}}\", if is_valid(\"{}\".to_string()) {{ \"true\" }} else {{ \"false\" }}); }}", input)
            }
            "fizzbuzz" => {
                format!(
                    "fn main() {{\n    let result = fizz_buzz({});\n    print!(\"[\");\n    for (i, s) in result.iter().enumerate() {{\n        print!(\"\\\"{{}}\\\"\", s);\n        if i < result.len() - 1 {{ print!(\",\"); }}\n    }}\n    println!(\"]\");\n}}",
                    input
                )
            }
            "palindrome-number" => {
                format!("fn main() {{ println!(\"{{}}\", if is_palindrome({}) {{ \"true\" }} else {{ \"false\" }}); }}", input)
            }
            "maximum-subarray" => {
                format!("fn main() {{ let nums = vec!{}; println!(\"{{}}\", max_sub_array(nums)); }}", input)
            }
            "merge-intervals" => {
                format!(
                    "fn main() {{\n    let intervals: Vec<Vec<i32>> = vec!{};\n    let result = merge(intervals);\n    print!(\"[\");\n    for (i, interval) in result.iter().enumerate() {{\n        print!(\"[{{}},{{}}]\", interval[0], interval[1]);\n        if i < result.len() - 1 {{ print!(\",\"); }}\n    }}\n    println!(\"]\");\n}}",
                    input
                )
            }
            "group-anagrams" => {
                format!(
                    "fn main() {{\n    let strs: Vec<String> = vec!{}.iter().map(|s: &&str| s.to_string()).collect();\n    let result = group_anagrams(strs);\n    print!(\"[\");\n    for (i, group) in result.iter().enumerate() {{\n        print!(\"[\");\n        for (j, s) in group.iter().enumerate() {{\n            print!(\"\\\"{{}}\\\"\", s);\n            if j < group.len() - 1 {{ print!(\",\"); }}\n        }}\n        print!(\"]\");\n        if i < result.len() - 1 {{ print!(\",\"); }}\n    }}\n    println!(\"]\");\n}}",
                    input
                )
            }
            "longest-substring" => {
                format!("fn main() {{ println!(\"{{}}\", length_of_longest_substring(\"{}\".to_string())); }}", input)
            }
            "trapping-rain-water" => {
                format!("fn main() {{ let height = vec!{}; println!(\"{{}}\", trap(height)); }}", input)
            }
            "merge-k-sorted-lists" => {
                format!(
                    "fn main() {{\n    let lists: Vec<Vec<i32>> = vec!{};\n    let result = merge_k_lists(lists);\n    print!(\"[\");\n    for (i, n) in result.iter().enumerate() {{\n        print!(\"{{}}\", n);\n        if i < result.len() - 1 {{ print!(\",\"); }}\n    }}\n    println!(\"]\");\n}}",
                    input
                )
            }
            "median-two-sorted-arrays" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    format!(
                        "fn main() {{ let nums1 = vec!{}; let nums2 = vec!{}; println!(\"{{:.1}}\", find_median_sorted_arrays(nums1, nums2)); }}",
                        parts[0], parts[1]
                    )
                } else { "fn main() {}".to_string() }
            }
            _ => "fn main() { println!(\"Unknown problem\"); }".to_string()
        };
        format!("{}\n\n{}", user_code, main_code)
    }

    // ============ Go Test Harness ============
    fn go_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        let input = &test_case.input;
        let (imports, main_code) = match problem_id {
            "two-sum" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let arr = parts[0].replace("[", "[]int{").replace("]", "}");
                    (
                        "\"fmt\"".to_string(),
                        format!(
                            "func main() {{\n    nums := {}\n    result := twoSum(nums, {})\n    fmt.Printf(\"[%d,%d]\", result[0], result[1])\n}}",
                            arr, parts[1]
                        )
                    )
                } else { ("\"fmt\"".to_string(), "func main() {}".to_string()) }
            }
            "reverse-string" => {
                let chars: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let byte_vec: Vec<String> = chars.iter().map(|c| {
                    let ch = c.trim().trim_matches('"');
                    format!("'{}'", ch)
                }).collect();
                (
                    "\"fmt\"".to_string(),
                    format!(
                        "func main() {{\n    s := []byte{{{}}}\n    reverseString(s)\n    fmt.Print(\"[\")\n    for i, c := range s {{\n        fmt.Printf(\"\\\"%c\\\"\", c)\n        if i < len(s)-1 {{ fmt.Print(\",\") }}\n    }}\n    fmt.Print(\"]\")\n}}",
                        byte_vec.join(", ")
                    )
                )
            }
            "valid-parentheses" => (
                "\"fmt\"".to_string(),
                format!("func main() {{ if isValid(\"{}\") {{ fmt.Print(\"true\") }} else {{ fmt.Print(\"false\") }} }}", input)
            ),
            "fizzbuzz" => (
                "\"fmt\"".to_string(),
                format!(
                    "func main() {{\n    result := fizzBuzz({})\n    fmt.Print(\"[\")\n    for i, s := range result {{\n        fmt.Printf(\"\\\"%s\\\"\", s)\n        if i < len(result)-1 {{ fmt.Print(\",\") }}\n    }}\n    fmt.Print(\"]\")\n}}",
                    input
                )
            ),
            "palindrome-number" => (
                "\"fmt\"".to_string(),
                format!("func main() {{ if isPalindrome({}) {{ fmt.Print(\"true\") }} else {{ fmt.Print(\"false\") }} }}", input)
            ),
            "maximum-subarray" => {
                let arr = input.replace("[", "[]int{").replace("]", "}");
                (
                    "\"fmt\"".to_string(),
                    format!("func main() {{ nums := {}; fmt.Print(maxSubArray(nums)) }}", arr)
                )
            }
            "merge-intervals" => {
                let arr = input.replace("[[", "[][]int{{").replace("]]", "}}").replace("],[", "},{");
                (
                    "\"fmt\"".to_string(),
                    format!(
                        "func main() {{\n    intervals := {}\n    result := merge(intervals)\n    fmt.Print(\"[\")\n    for i, interval := range result {{\n        fmt.Printf(\"[%d,%d]\", interval[0], interval[1])\n        if i < len(result)-1 {{ fmt.Print(\",\") }}\n    }}\n    fmt.Print(\"]\")\n}}",
                        arr
                    )
                )
            }
            "group-anagrams" => (
                "\"fmt\"\n    \"strings\"".to_string(),
                format!(
                    "func main() {{\n    strs := []string{{{}}}\n    result := groupAnagrams(strs)\n    fmt.Print(\"[\")\n    for i, group := range result {{\n        fmt.Print(\"[\")\n        for j, s := range group {{\n            fmt.Printf(\"\\\"%s\\\"\", s)\n            if j < len(group)-1 {{ fmt.Print(\",\") }}\n        }}\n        fmt.Print(\"]\")\n        if i < len(result)-1 {{ fmt.Print(\",\") }}\n    }}\n    fmt.Print(\"]\")\n    _ = strings.Join(nil, \"\") // use strings\n}}",
                    input.trim_matches(|c| c == '[' || c == ']')
                )
            ),
            "longest-substring" => (
                "\"fmt\"".to_string(),
                format!("func main() {{ fmt.Print(lengthOfLongestSubstring(\"{}\")) }}", input)
            ),
            "trapping-rain-water" => {
                let arr = input.replace("[", "[]int{").replace("]", "}");
                (
                    "\"fmt\"".to_string(),
                    format!("func main() {{ height := {}; fmt.Print(trap(height)) }}", arr)
                )
            }
            "merge-k-sorted-lists" => {
                let arr = input.replace("[[", "[][]int{{").replace("]]", "}}").replace("],[", "},{");
                (
                    "\"fmt\"".to_string(),
                    format!(
                        "func main() {{\n    lists := {}\n    result := mergeKLists(lists)\n    fmt.Print(\"[\")\n    for i, n := range result {{\n        fmt.Print(n)\n        if i < len(result)-1 {{ fmt.Print(\",\") }}\n    }}\n    fmt.Print(\"]\")\n}}",
                        arr
                    )
                )
            }
            "median-two-sorted-arrays" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let arr1 = parts[0].replace("[", "[]int{").replace("]", "}");
                    let arr2 = parts[1].replace("[", "[]int{").replace("]", "}");
                    (
                        "\"fmt\"".to_string(),
                        format!(
                            "func main() {{ nums1 := {}; nums2 := {}; fmt.Printf(\"%.1f\", findMedianSortedArrays(nums1, nums2)) }}",
                            arr1, arr2
                        )
                    )
                } else { ("\"fmt\"".to_string(), "func main() {}".to_string()) }
            }
            _ => ("\"fmt\"".to_string(), "func main() { fmt.Println(\"Unknown problem\") }".to_string())
        };
        format!("package main\n\nimport (\n    {}\n)\n\n{}\n\n{}", imports, user_code, main_code)
    }

    // ============ Java Test Harness ============
    fn java_harness(&self, user_code: &str, test_case: &TestCase, problem_id: &str) -> String {
        let input = &test_case.input;

        // Extract just the method content from user code if it contains "class Solution"
        let method_content = if user_code.contains("class Solution") {
            // Find the content between the first { and last }
            if let Some(start) = user_code.find('{') {
                if let Some(end) = user_code.rfind('}') {
                    let content = user_code[start + 1..end].trim().to_string();
                    // Remove user's main method if present (we'll add our own)
                    Self::remove_java_main_method(&content)
                } else {
                    user_code.to_string()
                }
            } else {
                user_code.to_string()
            }
        } else {
            // If no class wrapper, just use the code but remove any main method
            Self::remove_java_main_method(user_code)
        };

        let (imports, main_code) = match problem_id {
            "two-sum" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let nums: Vec<&str> = parts[0].trim_matches(|c| c == '[' || c == ']').split(',').collect();
                    let arr_init = nums.join(", ");
                    (
                        "import java.util.*;".to_string(),
                        format!(
                            "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        int[] nums = new int[]{{{}}};\n        int[] result = sol.twoSum(nums, {});\n        System.out.print(\"[\" + result[0] + \",\" + result[1] + \"]\");\n    }}",
                            arr_init, parts[1]
                        )
                    )
                } else { ("".to_string(), "    public static void main(String[] args) {}".to_string()) }
            }
            "reverse-string" => {
                let chars: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let char_init: Vec<String> = chars.iter().map(|c| {
                    let ch = c.trim().trim_matches('"');
                    format!("'{}'", ch)
                }).collect();
                (
                    "".to_string(),
                    format!(
                        "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        char[] s = new char[]{{{}}};\n        sol.reverseString(s);\n        System.out.print(\"[\");\n        for(int i = 0; i < s.length; i++) {{\n            System.out.print(\"\\\"\" + s[i] + \"\\\"\");\n            if(i < s.length-1) System.out.print(\",\");\n        }}\n        System.out.print(\"]\");\n    }}",
                        char_init.join(", ")
                    )
                )
            }
            "valid-parentheses" => (
                "".to_string(),
                format!(
                    "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        System.out.print(sol.isValid(\"{}\") ? \"true\" : \"false\");\n    }}",
                    input
                )
            ),
            "fizzbuzz" => (
                "import java.util.*;".to_string(),
                format!(
                    "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        List<String> result = sol.fizzBuzz({});\n        System.out.print(\"[\");\n        for(int i = 0; i < result.size(); i++) {{\n            System.out.print(\"\\\"\" + result.get(i) + \"\\\"\");\n            if(i < result.size()-1) System.out.print(\",\");\n        }}\n        System.out.print(\"]\");\n    }}",
                    input
                )
            ),
            "palindrome-number" => (
                "".to_string(),
                format!(
                    "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        System.out.print(sol.isPalindrome({}) ? \"true\" : \"false\");\n    }}",
                    input
                )
            ),
            "maximum-subarray" => {
                let nums: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let arr_init = nums.join(", ");
                (
                    "".to_string(),
                    format!(
                        "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        int[] nums = new int[]{{{}}};\n        System.out.print(sol.maxSubArray(nums));\n    }}",
                        arr_init
                    )
                )
            }
            "merge-intervals" => {
                // Parse input like [[1,3],[2,6],[8,10],[15,18]]
                let inner = input.trim_start_matches('[').trim_end_matches(']');
                let mut intervals = Vec::new();
                let mut depth = 0;
                let mut current = String::new();

                for c in inner.chars() {
                    match c {
                        '[' => {
                            depth += 1;
                            if depth == 1 {
                                current.clear();
                            }
                        }
                        ']' => {
                            depth -= 1;
                            if depth == 0 {
                                let nums: Vec<&str> = current.split(',').filter(|s| !s.is_empty()).collect();
                                intervals.push(format!("new int[]{{{}}}", nums.join(", ")));
                            }
                        }
                        ',' if depth == 0 => {}
                        _ if depth > 0 => current.push(c),
                        _ => {}
                    }
                }

                let intervals_init = intervals.join(", ");
                (
                    "import java.util.*;".to_string(),
                    format!(
                        "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        int[][] intervals = new int[][]{{{}}};\n        int[][] result = sol.merge(intervals);\n        StringBuilder sb = new StringBuilder(\"[\");\n        for(int i = 0; i < result.length; i++) {{\n            sb.append(\"[\").append(result[i][0]).append(\",\").append(result[i][1]).append(\"]\");\n            if(i < result.length - 1) sb.append(\",\");\n        }}\n        sb.append(\"]\");\n        System.out.print(sb.toString());\n    }}",
                        intervals_init
                    )
                )
            }
            "group-anagrams" => (
                "import java.util.*;".to_string(),
                format!(
                    "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        String[] strs = new String[]{{{}}};\n        List<List<String>> result = sol.groupAnagrams(strs);\n        System.out.print(\"[\");\n        for(int i = 0; i < result.size(); i++) {{\n            System.out.print(\"[\");\n            List<String> group = result.get(i);\n            for(int j = 0; j < group.size(); j++) {{\n                System.out.print(\"\\\"\" + group.get(j) + \"\\\"\");\n                if(j < group.size()-1) System.out.print(\",\");\n            }}\n            System.out.print(\"]\");\n            if(i < result.size()-1) System.out.print(\",\");\n        }}\n        System.out.print(\"]\");\n    }}",
                    input.trim_matches(|c| c == '[' || c == ']')
                )
            ),
            "longest-substring" => (
                "".to_string(),
                format!(
                    "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        System.out.print(sol.lengthOfLongestSubstring(\"{}\"));\n    }}",
                    input
                )
            ),
            "trapping-rain-water" => {
                let nums: Vec<&str> = input.trim_matches(|c| c == '[' || c == ']').split(',').collect();
                let arr_init = nums.join(", ");
                (
                    "".to_string(),
                    format!(
                        "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        int[] height = new int[]{{{}}};\n        System.out.print(sol.trap(height));\n    }}",
                        arr_init
                    )
                )
            }
            "merge-k-sorted-lists" => {
                // Parse input like [[1,4,5],[1,3,4],[2,6]]
                let inner = input.trim_start_matches('[').trim_end_matches(']');
                let mut arrays = Vec::new();
                let mut depth = 0;
                let mut current = String::new();

                for c in inner.chars() {
                    match c {
                        '[' => {
                            depth += 1;
                            if depth == 1 {
                                current.clear();
                            }
                        }
                        ']' => {
                            depth -= 1;
                            if depth == 0 {
                                let nums: Vec<&str> = current.split(',').filter(|s| !s.is_empty()).collect();
                                arrays.push(format!("new int[]{{{}}}", nums.join(", ")));
                            }
                        }
                        ',' if depth == 0 => {}
                        _ if depth > 0 => current.push(c),
                        _ => {}
                    }
                }

                let arrays_init = arrays.join(", ");
                (
                    "import java.util.*;".to_string(),
                    format!(
                        "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        int[][] lists = new int[][]{{{}}};\n        int[] result = sol.mergeKLists(lists);\n        StringBuilder sb = new StringBuilder(\"[\");\n        for(int i = 0; i < result.length; i++) {{\n            sb.append(result[i]);\n            if(i < result.length - 1) sb.append(\",\");\n        }}\n        sb.append(\"]\");\n        System.out.print(sb.toString());\n    }}",
                        arrays_init
                    )
                )
            }
            "median-two-sorted-arrays" => {
                let parts: Vec<&str> = input.split_whitespace().collect();
                if parts.len() >= 2 {
                    let nums1: Vec<&str> = parts[0].trim_matches(|c| c == '[' || c == ']').split(',').collect();
                    let nums2: Vec<&str> = parts[1].trim_matches(|c| c == '[' || c == ']').split(',').collect();
                    (
                        "".to_string(),
                        format!(
                            "    public static void main(String[] args) {{\n        Solution sol = new Solution();\n        int[] nums1 = new int[]{{{}}};\n        int[] nums2 = new int[]{{{}}};\n        System.out.printf(\"%.1f\", sol.findMedianSortedArrays(nums1, nums2));\n    }}",
                            nums1.join(", "), nums2.join(", ")
                        )
                    )
                } else { ("".to_string(), "    public static void main(String[] args) {}".to_string()) }
            }
            _ => ("".to_string(), "    public static void main(String[] args) { System.out.print(\"Unknown problem\"); }".to_string())
        };
        // Wrap method content in Solution class (we extracted just the methods earlier)
        let imports_str = if imports.is_empty() { "".to_string() } else { format!("{}\n\n", imports) };
        format!("{}class Solution {{\n{}\n\n{}\n}}", imports_str, method_content, main_code)
    }
}
