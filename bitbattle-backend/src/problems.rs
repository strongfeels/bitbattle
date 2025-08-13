use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub expected_output: String,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub examples: Vec<TestCase>,
    pub test_cases: Vec<TestCase>, // Hidden test cases for validation
    pub starter_code: HashMap<String, String>, // language -> starter code
    pub time_limit_minutes: Option<u32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

pub struct ProblemDatabase {
    problems: HashMap<String, Problem>,
}

impl ProblemDatabase {
    pub fn new() -> Self {
        let mut db = ProblemDatabase {
            problems: HashMap::new(),
        };
        db.load_default_problems();
        db
    }

    pub fn get_problem(&self, id: &str) -> Option<&Problem> {
        self.problems.get(id)
    }

    pub fn get_random_problem(&self) -> Option<&Problem> {
        if self.problems.is_empty() {
            return None;
        }

        let problems: Vec<&Problem> = self.problems.values().collect();
        let index = fastrand::usize(..problems.len());
        Some(problems[index])
    }

    pub fn get_problems_by_difficulty(&self, difficulty: &Difficulty) -> Vec<&Problem> {
        self.problems
            .values()
            .filter(|p| &p.difficulty == difficulty)
            .collect()
    }

    pub fn add_problem(&mut self, problem: Problem) {
        self.problems.insert(problem.id.clone(), problem);
    }

    fn load_default_problems(&mut self) {
        // Problem 1: Two Sum
        let two_sum = Problem {
            id: "two-sum".to_string(),
            title: "Two Sum".to_string(),
            description: r#"Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target.

You may assume that each input would have exactly one solution, and you may not use the same element twice.

You can return the answer in any order."#.to_string(),
            difficulty: Difficulty::Easy,
            examples: vec![
                TestCase {
                    input: "nums = [2,7,11,15], target = 9".to_string(),
                    expected_output: "[0,1]".to_string(),
                    explanation: Some("Because nums[0] + nums[1] == 9, we return [0, 1].".to_string()),
                },
                TestCase {
                    input: "nums = [3,2,4], target = 6".to_string(),
                    expected_output: "[1,2]".to_string(),
                    explanation: None,
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "[2,7,11,15] 9".to_string(),
                    expected_output: "[0,1]".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[3,2,4] 6".to_string(),
                    expected_output: "[1,2]".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[3,3] 6".to_string(),
                    expected_output: "[0,1]".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number[]} nums
 * @param {number} target
 * @return {number[]}
 */
function twoSum(nums, target) {
    // Your solution here

}

// Test your solution
console.log(twoSum([2,7,11,15], 9)); // Should return [0,1]"#.to_string());
                code.insert("python".to_string(), r#"def two_sum(nums, target):
    """
    :type nums: List[int]
    :type target: int
    :rtype: List[int]
    """
    # Your solution here
    pass

# Test your solution
print(two_sum([2,7,11,15], 9))  # Should return [0,1]"#.to_string());
                code.insert("java".to_string(), r#"class Solution {
    public int[] twoSum(int[] nums, int target) {
        // Your solution here
        return new int[]{};
    }

    public static void main(String[] args) {
        Solution solution = new Solution();
        int[] result = solution.twoSum(new int[]{2,7,11,15}, 9);
        System.out.println(java.util.Arrays.toString(result)); // Should return [0,1]
    }
}"#.to_string());
                code
            },
            time_limit_minutes: Some(15),
            tags: vec!["array".to_string(), "hash-table".to_string()],
        };

        // Problem 2: Reverse String
        let reverse_string = Problem {
            id: "reverse-string".to_string(),
            title: "Reverse String".to_string(),
            description: r#"Write a function that reverses a string. The input string is given as an array of characters s.

You must do this by modifying the input array in-place with O(1) extra memory."#.to_string(),
            difficulty: Difficulty::Easy,
            examples: vec![
                TestCase {
                    input: r#"s = ["h","e","l","l","o"]"#.to_string(),
                    expected_output: r#"["o","l","l","e","h"]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: r#"s = ["H","a","n","n","a","h"]"#.to_string(),
                    expected_output: r#"["h","a","n","n","a","H"]"#.to_string(),
                    explanation: None,
                },
            ],
            test_cases: vec![
                TestCase {
                    input: r#"["h","e","l","l","o"]"#.to_string(),
                    expected_output: r#"["o","l","l","e","h"]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: r#"["H","a","n","n","a","h"]"#.to_string(),
                    expected_output: r#"["h","a","n","n","a","H"]"#.to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {character[]} s
 * @return {void} Do not return anything, modify s in-place instead.
 */
function reverseString(s) {
    // Your solution here

}

// Test your solution
let test = ["h","e","l","l","o"];
reverseString(test);
console.log(test); // Should be ["o","l","l","e","h"]"#.to_string());
                code.insert("python".to_string(), r#"def reverse_string(s):
    """
    :type s: List[str]
    :rtype: None Do not return anything, modify s in-place instead.
    """
    # Your solution here
    pass

# Test your solution
test = ["h","e","l","l","o"]
reverse_string(test)
print(test)  # Should be ["o","l","l","e","h"]"#.to_string());
                code.insert("java".to_string(), r#"class Solution {
    public void reverseString(char[] s) {
        // Your solution here

    }

    public static void main(String[] args) {
        Solution solution = new Solution();
        char[] test = {'h','e','l','l','o'};
        solution.reverseString(test);
        System.out.println(java.util.Arrays.toString(test)); // Should be [o,l,l,e,h]
    }
}"#.to_string());
                code
            },
            time_limit_minutes: Some(10),
            tags: vec!["two-pointers".to_string(), "string".to_string()],
        };

        // Problem 3: Valid Parentheses
        let valid_parentheses = Problem {
            id: "valid-parentheses".to_string(),
            title: "Valid Parentheses".to_string(),
            description: r#"Given a string s containing just the characters '(', ')', '{', '}', '[' and ']', determine if the input string is valid.

An input string is valid if:
1. Open brackets must be closed by the same type of brackets.
2. Open brackets must be closed in the correct order.
3. Every close bracket has a corresponding open bracket of the same type."#.to_string(),
            difficulty: Difficulty::Easy,
            examples: vec![
                TestCase {
                    input: r#"s = "()"#.to_string(),
                    expected_output: "true".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: r#"s = "()[]{}"#.to_string(),
                    expected_output: "true".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: r#"s = "(]"#.to_string(),
                    expected_output: "false".to_string(),
                    explanation: None,
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "()".to_string(),
                    expected_output: "true".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "()[()]".to_string(),
                    expected_output: "true".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "([)]".to_string(),
                    expected_output: "false".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {string} s
 * @return {boolean}
 */
function isValid(s) {
    // Your solution here

}

// Test your solution
console.log(isValid("()")); // Should return true
console.log(isValid("([)]")); // Should return false"#.to_string());
                code.insert("python".to_string(), r#"def is_valid(s):
    """
    :type s: str
    :rtype: bool
    """
    # Your solution here
    pass

# Test your solution
print(is_valid("()"))  # Should return True
print(is_valid("([)]"))  # Should return False"#.to_string());
                code.insert("java".to_string(), r#"class Solution {
    public boolean isValid(String s) {
        // Your solution here
        return false;
    }

    public static void main(String[] args) {
        Solution solution = new Solution();
        System.out.println(solution.isValid("()")); // Should return true
        System.out.println(solution.isValid("([)]")); // Should return false
    }
}"#.to_string());
                code
            },
            time_limit_minutes: Some(20),
            tags: vec!["stack".to_string(), "string".to_string()],
        };

        self.add_problem(two_sum);
        self.add_problem(reverse_string);
        self.add_problem(valid_parentheses);
    }
}