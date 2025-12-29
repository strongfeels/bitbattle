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

    pub fn get_random_problem_by_difficulty(&self, difficulty: Option<&str>) -> Option<&Problem> {
        let problems: Vec<&Problem> = match difficulty {
            Some("easy") => self.get_problems_by_difficulty(&Difficulty::Easy),
            Some("medium") => self.get_problems_by_difficulty(&Difficulty::Medium),
            Some("hard") => self.get_problems_by_difficulty(&Difficulty::Hard),
            _ => self.problems.values().collect(), // "random" or any other value
        };

        if problems.is_empty() {
            return None;
        }

        let index = fastrand::usize(..problems.len());
        Some(problems[index])
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
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public int[] twoSum(int[] nums, int target) {
        // Your solution here
        return new int[]{};
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        int[] result = sol.twoSum(new int[]{2,7,11,15}, 9);
        System.out.println(Arrays.toString(result)); // Should return [0,1]
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <stdlib.h>

// Return array of 2 indices, caller must free
int* twoSum(int* nums, int numsSize, int target, int* returnSize) {
    // Your solution here
    *returnSize = 2;
    int* result = malloc(2 * sizeof(int));
    result[0] = 0;
    result[1] = 0;
    return result;
}

int main() {
    int nums[] = {2, 7, 11, 15};
    int returnSize;
    int* result = twoSum(nums, 4, 9, &returnSize);
    printf("[%d,%d]\n", result[0], result[1]); // Should return [0,1]
    free(result);
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
using namespace std;

vector<int> twoSum(vector<int>& nums, int target) {
    // Your solution here
    return {};
}

int main() {
    vector<int> nums = {2, 7, 11, 15};
    vector<int> result = twoSum(nums, 9);
    cout << "[" << result[0] << "," << result[1] << "]" << endl; // Should return [0,1]
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    // Your solution here
    vec![]
}

fn main() {
    let nums = vec![2, 7, 11, 15];
    let result = two_sum(nums, 9);
    println!("[{},{}]", result[0], result[1]); // Should return [0,1]
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func twoSum(nums []int, target int) []int {
    // Your solution here
    return []int{}
}

func main() {
    nums := []int{2, 7, 11, 15}
    result := twoSum(nums, 9)
    fmt.Printf("[%d,%d]\n", result[0], result[1]) // Should return [0,1]
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
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public void reverseString(char[] s) {
        // Your solution here

    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        char[] test = {'h','e','l','l','o'};
        sol.reverseString(test);
        System.out.println(Arrays.toString(test)); // Should be [o,l,l,e,h]
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <string.h>

void reverseString(char* s, int sSize) {
    // Your solution here

}

int main() {
    char s[] = {'h','e','l','l','o'};
    reverseString(s, 5);
    printf("[%c,%c,%c,%c,%c]\n", s[0], s[1], s[2], s[3], s[4]); // Should be [o,l,l,e,h]
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
using namespace std;

void reverseString(vector<char>& s) {
    // Your solution here

}

int main() {
    vector<char> s = {'h','e','l','l','o'};
    reverseString(s);
    cout << "[" << s[0] << "," << s[1] << "," << s[2] << "," << s[3] << "," << s[4] << "]" << endl;
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn reverse_string(s: &mut Vec<char>) {
    // Your solution here

}

fn main() {
    let mut s = vec!['h','e','l','l','o'];
    reverse_string(&mut s);
    println!("[{},{},{},{},{}]", s[0], s[1], s[2], s[3], s[4]); // Should be [o,l,l,e,h]
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func reverseString(s []byte) {
    // Your solution here

}

func main() {
    s := []byte{'h','e','l','l','o'}
    reverseString(s)
    fmt.Printf("[%c,%c,%c,%c,%c]\n", s[0], s[1], s[2], s[3], s[4]) // Should be [o,l,l,e,h]
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
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public boolean isValid(String s) {
        // Your solution here
        return false;
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        System.out.println(sol.isValid("()")); // Should return true
        System.out.println(sol.isValid("([)]")); // Should return false
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <stdbool.h>
#include <string.h>

bool isValid(char* s) {
    // Your solution here
    return false;
}

int main() {
    printf("%s\n", isValid("()") ? "true" : "false"); // Should return true
    printf("%s\n", isValid("([)]") ? "true" : "false"); // Should return false
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <string>
#include <stack>
using namespace std;

bool isValid(string s) {
    // Your solution here
    return false;
}

int main() {
    cout << (isValid("()") ? "true" : "false") << endl; // Should return true
    cout << (isValid("([)]") ? "true" : "false") << endl; // Should return false
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn is_valid(s: String) -> bool {
    // Your solution here
    false
}

fn main() {
    println!("{}", is_valid("()".to_string())); // Should return true
    println!("{}", is_valid("([)]".to_string())); // Should return false
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func isValid(s string) bool {
    // Your solution here
    return false
}

func main() {
    fmt.Println(isValid("()")) // Should return true
    fmt.Println(isValid("([)]")) // Should return false
}"#.to_string());
                code
            },
            time_limit_minutes: Some(20),
            tags: vec!["stack".to_string(), "string".to_string()],
        };

        // Problem 4: FizzBuzz (Easy)
        let fizzbuzz = Problem {
            id: "fizzbuzz".to_string(),
            title: "FizzBuzz".to_string(),
            description: r#"Given an integer n, return a string array answer where:
- answer[i] == "FizzBuzz" if i is divisible by 3 and 5.
- answer[i] == "Fizz" if i is divisible by 3.
- answer[i] == "Buzz" if i is divisible by 5.
- answer[i] == i (as a string) if none of the above conditions are true.

Note: The array is 1-indexed."#.to_string(),
            difficulty: Difficulty::Easy,
            examples: vec![
                TestCase {
                    input: "n = 5".to_string(),
                    expected_output: r#"["1","2","Fizz","4","Buzz"]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "n = 15".to_string(),
                    expected_output: r#"["1","2","Fizz","4","Buzz","Fizz","7","8","Fizz","Buzz","11","Fizz","13","14","FizzBuzz"]"#.to_string(),
                    explanation: None,
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "3".to_string(),
                    expected_output: r#"["1","2","Fizz"]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "5".to_string(),
                    expected_output: r#"["1","2","Fizz","4","Buzz"]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "15".to_string(),
                    expected_output: r#"["1","2","Fizz","4","Buzz","Fizz","7","8","Fizz","Buzz","11","Fizz","13","14","FizzBuzz"]"#.to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number} n
 * @return {string[]}
 */
function fizzBuzz(n) {
    // Your solution here

}

// Test your solution
console.log(JSON.stringify(fizzBuzz(5))); // Should return ["1","2","Fizz","4","Buzz"]"#.to_string());
                code.insert("python".to_string(), r#"def fizz_buzz(n):
    """
    :type n: int
    :rtype: List[str]
    """
    # Your solution here
    pass

# Test your solution
import json
print(json.dumps(fizz_buzz(5)))  # Should return ["1","2","Fizz","4","Buzz"]"#.to_string());
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public List<String> fizzBuzz(int n) {
        // Your solution here
        return new ArrayList<>();
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        List<String> result = sol.fizzBuzz(5);
        System.out.println(result); // Should return [1, 2, Fizz, 4, Buzz]
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char** fizzBuzz(int n, int* returnSize) {
    // Your solution here
    *returnSize = n;
    char** result = malloc(n * sizeof(char*));
    for (int i = 0; i < n; i++) {
        result[i] = malloc(10 * sizeof(char));
        sprintf(result[i], "%d", i + 1);
    }
    return result;
}

int main() {
    int size;
    char** result = fizzBuzz(5, &size);
    printf("[");
    for (int i = 0; i < size; i++) {
        printf("\"%s\"%s", result[i], i < size-1 ? "," : "");
        free(result[i]);
    }
    printf("]\n");
    free(result);
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
#include <string>
using namespace std;

vector<string> fizzBuzz(int n) {
    // Your solution here
    return {};
}

int main() {
    vector<string> result = fizzBuzz(5);
    cout << "[";
    for (int i = 0; i < result.size(); i++) {
        cout << "\"" << result[i] << "\"" << (i < result.size()-1 ? "," : "");
    }
    cout << "]" << endl;
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn fizz_buzz(n: i32) -> Vec<String> {
    // Your solution here
    vec![]
}

fn main() {
    let result = fizz_buzz(5);
    print!("[");
    for (i, s) in result.iter().enumerate() {
        print!("\"{}\"", s);
        if i < result.len() - 1 { print!(","); }
    }
    println!("]");
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import (
    "fmt"
    "strings"
)

func fizzBuzz(n int) []string {
    // Your solution here
    return []string{}
}

func main() {
    result := fizzBuzz(5)
    fmt.Printf("[%s]\n", "\""+strings.Join(result, "\",\"")+"\"")
}"#.to_string());
                code
            },
            time_limit_minutes: Some(10),
            tags: vec!["math".to_string(), "string".to_string(), "simulation".to_string()],
        };

        // Problem 5: Palindrome Number (Easy)
        let palindrome_number = Problem {
            id: "palindrome-number".to_string(),
            title: "Palindrome Number".to_string(),
            description: r#"Given an integer x, return true if x is a palindrome, and false otherwise.

An integer is a palindrome when it reads the same forward and backward."#.to_string(),
            difficulty: Difficulty::Easy,
            examples: vec![
                TestCase {
                    input: "x = 121".to_string(),
                    expected_output: "true".to_string(),
                    explanation: Some("121 reads as 121 from left to right and from right to left.".to_string()),
                },
                TestCase {
                    input: "x = -121".to_string(),
                    expected_output: "false".to_string(),
                    explanation: Some("From left to right, it reads -121. From right to left, it becomes 121-. Therefore it is not a palindrome.".to_string()),
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "121".to_string(),
                    expected_output: "true".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "-121".to_string(),
                    expected_output: "false".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "12321".to_string(),
                    expected_output: "true".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "10".to_string(),
                    expected_output: "false".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number} x
 * @return {boolean}
 */
function isPalindrome(x) {
    // Your solution here

}

// Test your solution
console.log(isPalindrome(121)); // Should return true
console.log(isPalindrome(-121)); // Should return false"#.to_string());
                code.insert("python".to_string(), r#"def is_palindrome(x):
    """
    :type x: int
    :rtype: bool
    """
    # Your solution here
    pass

# Test your solution
print(is_palindrome(121))  # Should return True
print(is_palindrome(-121))  # Should return False"#.to_string());
                code.insert("java".to_string(), r#"class Solution {
    public boolean isPalindrome(int x) {
        // Your solution here
        return false;
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        System.out.println(sol.isPalindrome(121)); // Should return true
        System.out.println(sol.isPalindrome(-121)); // Should return false
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <stdbool.h>

bool isPalindrome(int x) {
    // Your solution here
    return false;
}

int main() {
    printf("%s\n", isPalindrome(121) ? "true" : "false"); // Should return true
    printf("%s\n", isPalindrome(-121) ? "true" : "false"); // Should return false
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
using namespace std;

bool isPalindrome(int x) {
    // Your solution here
    return false;
}

int main() {
    cout << (isPalindrome(121) ? "true" : "false") << endl; // Should return true
    cout << (isPalindrome(-121) ? "true" : "false") << endl; // Should return false
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn is_palindrome(x: i32) -> bool {
    // Your solution here
    false
}

fn main() {
    println!("{}", is_palindrome(121)); // Should return true
    println!("{}", is_palindrome(-121)); // Should return false
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func isPalindrome(x int) bool {
    // Your solution here
    return false
}

func main() {
    fmt.Println(isPalindrome(121)) // Should return true
    fmt.Println(isPalindrome(-121)) // Should return false
}"#.to_string());
                code
            },
            time_limit_minutes: Some(10),
            tags: vec!["math".to_string()],
        };

        // Problem 6: Maximum Subarray (Medium)
        let max_subarray = Problem {
            id: "maximum-subarray".to_string(),
            title: "Maximum Subarray".to_string(),
            description: r#"Given an integer array nums, find the subarray with the largest sum, and return its sum.

A subarray is a contiguous non-empty sequence of elements within an array."#.to_string(),
            difficulty: Difficulty::Medium,
            examples: vec![
                TestCase {
                    input: "nums = [-2,1,-3,4,-1,2,1,-5,4]".to_string(),
                    expected_output: "6".to_string(),
                    explanation: Some("The subarray [4,-1,2,1] has the largest sum 6.".to_string()),
                },
                TestCase {
                    input: "nums = [1]".to_string(),
                    expected_output: "1".to_string(),
                    explanation: Some("The subarray [1] has the largest sum 1.".to_string()),
                },
                TestCase {
                    input: "nums = [5,4,-1,7,8]".to_string(),
                    expected_output: "23".to_string(),
                    explanation: Some("The subarray [5,4,-1,7,8] has the largest sum 23.".to_string()),
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "[-2,1,-3,4,-1,2,1,-5,4]".to_string(),
                    expected_output: "6".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[1]".to_string(),
                    expected_output: "1".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[5,4,-1,7,8]".to_string(),
                    expected_output: "23".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[-1]".to_string(),
                    expected_output: "-1".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number[]} nums
 * @return {number}
 */
function maxSubArray(nums) {
    // Your solution here
    // Hint: Consider Kadane's algorithm

}

// Test your solution
console.log(maxSubArray([-2,1,-3,4,-1,2,1,-5,4])); // Should return 6"#.to_string());
                code.insert("python".to_string(), r#"def max_sub_array(nums):
    """
    :type nums: List[int]
    :rtype: int
    """
    # Your solution here
    # Hint: Consider Kadane's algorithm
    pass

# Test your solution
print(max_sub_array([-2,1,-3,4,-1,2,1,-5,4]))  # Should return 6"#.to_string());
                code.insert("java".to_string(), r#"class Solution {
    public int maxSubArray(int[] nums) {
        // Your solution here
        // Hint: Consider Kadane's algorithm
        return 0;
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        System.out.println(sol.maxSubArray(new int[]{-2,1,-3,4,-1,2,1,-5,4})); // Should return 6
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>

int maxSubArray(int* nums, int numsSize) {
    // Your solution here
    // Hint: Consider Kadane's algorithm
    return 0;
}

int main() {
    int nums[] = {-2,1,-3,4,-1,2,1,-5,4};
    printf("%d\n", maxSubArray(nums, 9)); // Should return 6
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
using namespace std;

int maxSubArray(vector<int>& nums) {
    // Your solution here
    // Hint: Consider Kadane's algorithm
    return 0;
}

int main() {
    vector<int> nums = {-2,1,-3,4,-1,2,1,-5,4};
    cout << maxSubArray(nums) << endl; // Should return 6
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn max_sub_array(nums: Vec<i32>) -> i32 {
    // Your solution here
    // Hint: Consider Kadane's algorithm
    0
}

fn main() {
    let nums = vec![-2,1,-3,4,-1,2,1,-5,4];
    println!("{}", max_sub_array(nums)); // Should return 6
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func maxSubArray(nums []int) int {
    // Your solution here
    // Hint: Consider Kadane's algorithm
    return 0
}

func main() {
    nums := []int{-2,1,-3,4,-1,2,1,-5,4}
    fmt.Println(maxSubArray(nums)) // Should return 6
}"#.to_string());
                code
            },
            time_limit_minutes: Some(20),
            tags: vec!["array".to_string(), "divide-and-conquer".to_string(), "dynamic-programming".to_string()],
        };

        // Problem 7: Merge Intervals (Medium)
        let merge_intervals = Problem {
            id: "merge-intervals".to_string(),
            title: "Merge Intervals".to_string(),
            description: r#"Given an array of intervals where intervals[i] = [start_i, end_i], merge all overlapping intervals, and return an array of the non-overlapping intervals that cover all the intervals in the input."#.to_string(),
            difficulty: Difficulty::Medium,
            examples: vec![
                TestCase {
                    input: "intervals = [[1,3],[2,6],[8,10],[15,18]]".to_string(),
                    expected_output: "[[1,6],[8,10],[15,18]]".to_string(),
                    explanation: Some("Since intervals [1,3] and [2,6] overlap, merge them into [1,6].".to_string()),
                },
                TestCase {
                    input: "intervals = [[1,4],[4,5]]".to_string(),
                    expected_output: "[[1,5]]".to_string(),
                    explanation: Some("Intervals [1,4] and [4,5] are considered overlapping.".to_string()),
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "[[1,3],[2,6],[8,10],[15,18]]".to_string(),
                    expected_output: "[[1,6],[8,10],[15,18]]".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[[1,4],[4,5]]".to_string(),
                    expected_output: "[[1,5]]".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[[1,4],[0,4]]".to_string(),
                    expected_output: "[[0,4]]".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number[][]} intervals
 * @return {number[][]}
 */
function merge(intervals) {
    // Your solution here

}

// Test your solution
console.log(JSON.stringify(merge([[1,3],[2,6],[8,10],[15,18]])));
// Should return [[1,6],[8,10],[15,18]]"#.to_string());
                code.insert("python".to_string(), r#"def merge(intervals):
    """
    :type intervals: List[List[int]]
    :rtype: List[List[int]]
    """
    # Your solution here
    pass

# Test your solution
import json
print(json.dumps(merge([[1,3],[2,6],[8,10],[15,18]])))
# Should return [[1,6],[8,10],[15,18]]"#.to_string());
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public int[][] merge(int[][] intervals) {
        // Your solution here
        return new int[][]{};
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        int[][] result = sol.merge(new int[][]{{1,3},{2,6},{8,10},{15,18}});
        System.out.print("[");
        for (int i = 0; i < result.length; i++) {
            System.out.print("[" + result[i][0] + "," + result[i][1] + "]");
            if (i < result.length - 1) System.out.print(",");
        }
        System.out.println("]"); // Should return [[1,6],[8,10],[15,18]]
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <stdlib.h>

int** merge(int** intervals, int intervalsSize, int* intervalsColSize, int* returnSize, int** returnColumnSizes) {
    // Your solution here
    *returnSize = 0;
    return NULL;
}

int main() {
    // Test with [[1,3],[2,6],[8,10],[15,18]]
    printf("[[1,6],[8,10],[15,18]]\n"); // Expected output
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
#include <algorithm>
using namespace std;

vector<vector<int>> merge(vector<vector<int>>& intervals) {
    // Your solution here
    return {};
}

int main() {
    vector<vector<int>> intervals = {{1,3},{2,6},{8,10},{15,18}};
    vector<vector<int>> result = merge(intervals);
    cout << "[";
    for (int i = 0; i < result.size(); i++) {
        cout << "[" << result[i][0] << "," << result[i][1] << "]";
        if (i < result.size() - 1) cout << ",";
    }
    cout << "]" << endl; // Should return [[1,6],[8,10],[15,18]]
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn merge(intervals: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    // Your solution here
    vec![]
}

fn main() {
    let intervals = vec![vec![1,3],vec![2,6],vec![8,10],vec![15,18]];
    let result = merge(intervals);
    print!("[");
    for (i, interval) in result.iter().enumerate() {
        print!("[{},{}]", interval[0], interval[1]);
        if i < result.len() - 1 { print!(","); }
    }
    println!("]"); // Should return [[1,6],[8,10],[15,18]]
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func merge(intervals [][]int) [][]int {
    // Your solution here
    return [][]int{}
}

func main() {
    intervals := [][]int{{1,3},{2,6},{8,10},{15,18}}
    result := merge(intervals)
    fmt.Print("[")
    for i, interval := range result {
        fmt.Printf("[%d,%d]", interval[0], interval[1])
        if i < len(result)-1 { fmt.Print(",") }
    }
    fmt.Println("]") // Should return [[1,6],[8,10],[15,18]]
}"#.to_string());
                code
            },
            time_limit_minutes: Some(25),
            tags: vec!["array".to_string(), "sorting".to_string()],
        };

        // Problem 8: Group Anagrams (Medium)
        let group_anagrams = Problem {
            id: "group-anagrams".to_string(),
            title: "Group Anagrams".to_string(),
            description: r#"Given an array of strings strs, group the anagrams together. You can return the answer in any order.

An Anagram is a word or phrase formed by rearranging the letters of a different word or phrase, typically using all the original letters exactly once."#.to_string(),
            difficulty: Difficulty::Medium,
            examples: vec![
                TestCase {
                    input: r#"strs = ["eat","tea","tan","ate","nat","bat"]"#.to_string(),
                    expected_output: r#"[["bat"],["nat","tan"],["ate","eat","tea"]]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: r#"strs = [""]"#.to_string(),
                    expected_output: r#"[[""]]"#.to_string(),
                    explanation: None,
                },
            ],
            test_cases: vec![
                TestCase {
                    input: r#"["eat","tea","tan","ate","nat","bat"]"#.to_string(),
                    expected_output: r#"[["bat"],["nat","tan"],["ate","eat","tea"]]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: r#"[""]"#.to_string(),
                    expected_output: r#"[[""]]"#.to_string(),
                    explanation: None,
                },
                TestCase {
                    input: r#"["a"]"#.to_string(),
                    expected_output: r#"[["a"]]"#.to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {string[]} strs
 * @return {string[][]}
 */
function groupAnagrams(strs) {
    // Your solution here

}

// Test your solution
console.log(JSON.stringify(groupAnagrams(["eat","tea","tan","ate","nat","bat"])));
// Should group anagrams together"#.to_string());
                code.insert("python".to_string(), r#"def group_anagrams(strs):
    """
    :type strs: List[str]
    :rtype: List[List[str]]
    """
    # Your solution here
    pass

# Test your solution
import json
print(json.dumps(group_anagrams(["eat","tea","tan","ate","nat","bat"])))
# Should group anagrams together"#.to_string());
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public List<List<String>> groupAnagrams(String[] strs) {
        // Your solution here
        return new ArrayList<>();
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        List<List<String>> result = sol.groupAnagrams(new String[]{"eat","tea","tan","ate","nat","bat"});
        System.out.println(result); // Should group anagrams together
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// Return array of string arrays (simplified for C)
char*** groupAnagrams(char** strs, int strsSize, int* returnSize, int** returnColumnSizes) {
    // Your solution here
    *returnSize = 0;
    return NULL;
}

int main() {
    // Test with ["eat","tea","tan","ate","nat","bat"]
    printf("Groups anagrams together\n");
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
#include <string>
#include <unordered_map>
#include <algorithm>
using namespace std;

vector<vector<string>> groupAnagrams(vector<string>& strs) {
    // Your solution here
    return {};
}

int main() {
    vector<string> strs = {"eat","tea","tan","ate","nat","bat"};
    vector<vector<string>> result = groupAnagrams(strs);
    for (auto& group : result) {
        cout << "[";
        for (int i = 0; i < group.size(); i++) {
            cout << "\"" << group[i] << "\"";
            if (i < group.size() - 1) cout << ",";
        }
        cout << "] ";
    }
    cout << endl;
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"use std::collections::HashMap;

fn group_anagrams(strs: Vec<String>) -> Vec<Vec<String>> {
    // Your solution here
    vec![]
}

fn main() {
    let strs = vec!["eat","tea","tan","ate","nat","bat"]
        .iter().map(|s| s.to_string()).collect();
    let result = group_anagrams(strs);
    for group in result {
        print!("[");
        for (i, s) in group.iter().enumerate() {
            print!("\"{}\"", s);
            if i < group.len() - 1 { print!(","); }
        }
        print!("] ");
    }
    println!();
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import (
    "fmt"
    "sort"
)

func groupAnagrams(strs []string) [][]string {
    // Your solution here
    return [][]string{}
}

func main() {
    strs := []string{"eat","tea","tan","ate","nat","bat"}
    result := groupAnagrams(strs)
    for _, group := range result {
        fmt.Print("[")
        for i, s := range group {
            fmt.Printf("\"%s\"", s)
            if i < len(group)-1 { fmt.Print(",") }
        }
        fmt.Print("] ")
    }
    fmt.Println()
}"#.to_string());
                code
            },
            time_limit_minutes: Some(25),
            tags: vec!["array".to_string(), "hash-table".to_string(), "string".to_string(), "sorting".to_string()],
        };

        // Problem 9: Longest Substring Without Repeating Characters (Medium)
        let longest_substring = Problem {
            id: "longest-substring".to_string(),
            title: "Longest Substring Without Repeating Characters".to_string(),
            description: r#"Given a string s, find the length of the longest substring without repeating characters."#.to_string(),
            difficulty: Difficulty::Medium,
            examples: vec![
                TestCase {
                    input: r#"s = "abcabcbb""#.to_string(),
                    expected_output: "3".to_string(),
                    explanation: Some("The answer is \"abc\", with the length of 3.".to_string()),
                },
                TestCase {
                    input: r#"s = "bbbbb""#.to_string(),
                    expected_output: "1".to_string(),
                    explanation: Some("The answer is \"b\", with the length of 1.".to_string()),
                },
                TestCase {
                    input: r#"s = "pwwkew""#.to_string(),
                    expected_output: "3".to_string(),
                    explanation: Some("The answer is \"wke\", with the length of 3.".to_string()),
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "abcabcbb".to_string(),
                    expected_output: "3".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "bbbbb".to_string(),
                    expected_output: "1".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "pwwkew".to_string(),
                    expected_output: "3".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "".to_string(),
                    expected_output: "0".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {string} s
 * @return {number}
 */
function lengthOfLongestSubstring(s) {
    // Your solution here
    // Hint: Use sliding window technique

}

// Test your solution
console.log(lengthOfLongestSubstring("abcabcbb")); // Should return 3"#.to_string());
                code.insert("python".to_string(), r#"def length_of_longest_substring(s):
    """
    :type s: str
    :rtype: int
    """
    # Your solution here
    # Hint: Use sliding window technique
    pass

# Test your solution
print(length_of_longest_substring("abcabcbb"))  # Should return 3"#.to_string());
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public int lengthOfLongestSubstring(String s) {
        // Your solution here
        // Hint: Use sliding window technique
        return 0;
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        System.out.println(sol.lengthOfLongestSubstring("abcabcbb")); // Should return 3
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <string.h>

int lengthOfLongestSubstring(char* s) {
    // Your solution here
    // Hint: Use sliding window technique
    return 0;
}

int main() {
    printf("%d\n", lengthOfLongestSubstring("abcabcbb")); // Should return 3
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <string>
#include <unordered_set>
using namespace std;

int lengthOfLongestSubstring(string s) {
    // Your solution here
    // Hint: Use sliding window technique
    return 0;
}

int main() {
    cout << lengthOfLongestSubstring("abcabcbb") << endl; // Should return 3
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"use std::collections::HashSet;

fn length_of_longest_substring(s: String) -> i32 {
    // Your solution here
    // Hint: Use sliding window technique
    0
}

fn main() {
    println!("{}", length_of_longest_substring("abcabcbb".to_string())); // Should return 3
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func lengthOfLongestSubstring(s string) int {
    // Your solution here
    // Hint: Use sliding window technique
    return 0
}

func main() {
    fmt.Println(lengthOfLongestSubstring("abcabcbb")) // Should return 3
}"#.to_string());
                code
            },
            time_limit_minutes: Some(25),
            tags: vec!["hash-table".to_string(), "string".to_string(), "sliding-window".to_string()],
        };

        // Problem 10: Trapping Rain Water (Hard)
        let trapping_rain_water = Problem {
            id: "trapping-rain-water".to_string(),
            title: "Trapping Rain Water".to_string(),
            description: r#"Given n non-negative integers representing an elevation map where the width of each bar is 1, compute how much water it can trap after raining."#.to_string(),
            difficulty: Difficulty::Hard,
            examples: vec![
                TestCase {
                    input: "height = [0,1,0,2,1,0,1,3,2,1,2,1]".to_string(),
                    expected_output: "6".to_string(),
                    explanation: Some("The elevation map is represented by array [0,1,0,2,1,0,1,3,2,1,2,1]. In this case, 6 units of rain water are being trapped.".to_string()),
                },
                TestCase {
                    input: "height = [4,2,0,3,2,5]".to_string(),
                    expected_output: "9".to_string(),
                    explanation: None,
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "[0,1,0,2,1,0,1,3,2,1,2,1]".to_string(),
                    expected_output: "6".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[4,2,0,3,2,5]".to_string(),
                    expected_output: "9".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[4,2,3]".to_string(),
                    expected_output: "1".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number[]} height
 * @return {number}
 */
function trap(height) {
    // Your solution here
    // Hint: Consider using two pointers or dynamic programming

}

// Test your solution
console.log(trap([0,1,0,2,1,0,1,3,2,1,2,1])); // Should return 6"#.to_string());
                code.insert("python".to_string(), r#"def trap(height):
    """
    :type height: List[int]
    :rtype: int
    """
    # Your solution here
    # Hint: Consider using two pointers or dynamic programming
    pass

# Test your solution
print(trap([0,1,0,2,1,0,1,3,2,1,2,1]))  # Should return 6"#.to_string());
                code.insert("java".to_string(), r#"class Solution {
    public int trap(int[] height) {
        // Your solution here
        // Hint: Consider using two pointers or dynamic programming
        return 0;
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        System.out.println(sol.trap(new int[]{0,1,0,2,1,0,1,3,2,1,2,1})); // Should return 6
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>

int trap(int* height, int heightSize) {
    // Your solution here
    // Hint: Consider using two pointers or dynamic programming
    return 0;
}

int main() {
    int height[] = {0,1,0,2,1,0,1,3,2,1,2,1};
    printf("%d\n", trap(height, 12)); // Should return 6
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
using namespace std;

int trap(vector<int>& height) {
    // Your solution here
    // Hint: Consider using two pointers or dynamic programming
    return 0;
}

int main() {
    vector<int> height = {0,1,0,2,1,0,1,3,2,1,2,1};
    cout << trap(height) << endl; // Should return 6
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn trap(height: Vec<i32>) -> i32 {
    // Your solution here
    // Hint: Consider using two pointers or dynamic programming
    0
}

fn main() {
    let height = vec![0,1,0,2,1,0,1,3,2,1,2,1];
    println!("{}", trap(height)); // Should return 6
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func trap(height []int) int {
    // Your solution here
    // Hint: Consider using two pointers or dynamic programming
    return 0
}

func main() {
    height := []int{0,1,0,2,1,0,1,3,2,1,2,1}
    fmt.Println(trap(height)) // Should return 6
}"#.to_string());
                code
            },
            time_limit_minutes: Some(30),
            tags: vec!["array".to_string(), "two-pointers".to_string(), "dynamic-programming".to_string(), "stack".to_string()],
        };

        // Problem 11: Merge k Sorted Lists (Hard)
        let merge_k_lists = Problem {
            id: "merge-k-sorted-lists".to_string(),
            title: "Merge k Sorted Lists".to_string(),
            description: r#"You are given an array of k linked-lists lists, each linked-list is sorted in ascending order.

Merge all the linked-lists into one sorted linked-list and return it.

For simplicity, represent linked lists as arrays."#.to_string(),
            difficulty: Difficulty::Hard,
            examples: vec![
                TestCase {
                    input: "lists = [[1,4,5],[1,3,4],[2,6]]".to_string(),
                    expected_output: "[1,1,2,3,4,4,5,6]".to_string(),
                    explanation: Some("The linked-lists are: 1->4->5, 1->3->4, 2->6. Merged: 1->1->2->3->4->4->5->6".to_string()),
                },
                TestCase {
                    input: "lists = []".to_string(),
                    expected_output: "[]".to_string(),
                    explanation: None,
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "[[1,4,5],[1,3,4],[2,6]]".to_string(),
                    expected_output: "[1,1,2,3,4,4,5,6]".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[]".to_string(),
                    expected_output: "[]".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[[]]".to_string(),
                    expected_output: "[]".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[[1],[0]]".to_string(),
                    expected_output: "[0,1]".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number[][]} lists - array of sorted arrays
 * @return {number[]} - merged sorted array
 */
function mergeKLists(lists) {
    // Your solution here
    // Hint: Consider using a min-heap or divide-and-conquer

}

// Test your solution
console.log(JSON.stringify(mergeKLists([[1,4,5],[1,3,4],[2,6]])));
// Should return [1,1,2,3,4,4,5,6]"#.to_string());
                code.insert("python".to_string(), r#"def merge_k_lists(lists):
    """
    :type lists: List[List[int]]
    :rtype: List[int]
    """
    # Your solution here
    # Hint: Consider using a min-heap or divide-and-conquer
    pass

# Test your solution
import json
print(json.dumps(merge_k_lists([[1,4,5],[1,3,4],[2,6]])))
# Should return [1,1,2,3,4,4,5,6]"#.to_string());
                code.insert("java".to_string(), r#"import java.util.*;

class Solution {
    public int[] mergeKLists(int[][] lists) {
        // Your solution here
        // Hint: Consider using a min-heap or divide-and-conquer
        return new int[]{};
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        int[] result = sol.mergeKLists(new int[][]{{1,4,5},{1,3,4},{2,6}});
        System.out.println(Arrays.toString(result)); // Should return [1,1,2,3,4,4,5,6]
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>
#include <stdlib.h>

int* mergeKLists(int** lists, int listsSize, int* listSizes, int* returnSize) {
    // Your solution here
    // Hint: Consider using a min-heap or divide-and-conquer
    *returnSize = 0;
    return NULL;
}

int main() {
    // Test with [[1,4,5],[1,3,4],[2,6]]
    printf("[1,1,2,3,4,4,5,6]\n"); // Expected output
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
#include <queue>
using namespace std;

vector<int> mergeKLists(vector<vector<int>>& lists) {
    // Your solution here
    // Hint: Consider using a min-heap or divide-and-conquer
    return {};
}

int main() {
    vector<vector<int>> lists = {{1,4,5},{1,3,4},{2,6}};
    vector<int> result = mergeKLists(lists);
    cout << "[";
    for (int i = 0; i < result.size(); i++) {
        cout << result[i];
        if (i < result.size() - 1) cout << ",";
    }
    cout << "]" << endl; // Should return [1,1,2,3,4,4,5,6]
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"use std::collections::BinaryHeap;
use std::cmp::Reverse;

fn merge_k_lists(lists: Vec<Vec<i32>>) -> Vec<i32> {
    // Your solution here
    // Hint: Consider using a min-heap or divide-and-conquer
    vec![]
}

fn main() {
    let lists = vec![vec![1,4,5],vec![1,3,4],vec![2,6]];
    let result = merge_k_lists(lists);
    print!("[");
    for (i, n) in result.iter().enumerate() {
        print!("{}", n);
        if i < result.len() - 1 { print!(","); }
    }
    println!("]"); // Should return [1,1,2,3,4,4,5,6]
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import (
    "container/heap"
    "fmt"
)

func mergeKLists(lists [][]int) []int {
    // Your solution here
    // Hint: Consider using a min-heap or divide-and-conquer
    return []int{}
}

func main() {
    lists := [][]int{{1,4,5},{1,3,4},{2,6}}
    result := mergeKLists(lists)
    fmt.Print("[")
    for i, n := range result {
        fmt.Print(n)
        if i < len(result)-1 { fmt.Print(",") }
    }
    fmt.Println("]") // Should return [1,1,2,3,4,4,5,6]
}"#.to_string());
                code
            },
            time_limit_minutes: Some(30),
            tags: vec!["linked-list".to_string(), "divide-and-conquer".to_string(), "heap".to_string(), "merge-sort".to_string()],
        };

        // Problem 12: Median of Two Sorted Arrays (Hard)
        let median_two_arrays = Problem {
            id: "median-two-sorted-arrays".to_string(),
            title: "Median of Two Sorted Arrays".to_string(),
            description: r#"Given two sorted arrays nums1 and nums2 of size m and n respectively, return the median of the two sorted arrays.

The overall run time complexity should be O(log (m+n))."#.to_string(),
            difficulty: Difficulty::Hard,
            examples: vec![
                TestCase {
                    input: "nums1 = [1,3], nums2 = [2]".to_string(),
                    expected_output: "2.0".to_string(),
                    explanation: Some("Merged array = [1,2,3] and median is 2.".to_string()),
                },
                TestCase {
                    input: "nums1 = [1,2], nums2 = [3,4]".to_string(),
                    expected_output: "2.5".to_string(),
                    explanation: Some("Merged array = [1,2,3,4] and median is (2 + 3) / 2 = 2.5.".to_string()),
                },
            ],
            test_cases: vec![
                TestCase {
                    input: "[1,3] [2]".to_string(),
                    expected_output: "2.0".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[1,2] [3,4]".to_string(),
                    expected_output: "2.5".to_string(),
                    explanation: None,
                },
                TestCase {
                    input: "[0,0] [0,0]".to_string(),
                    expected_output: "0.0".to_string(),
                    explanation: None,
                },
            ],
            starter_code: {
                let mut code = HashMap::new();
                code.insert("javascript".to_string(), r#"/**
 * @param {number[]} nums1
 * @param {number[]} nums2
 * @return {number}
 */
function findMedianSortedArrays(nums1, nums2) {
    // Your solution here
    // Challenge: Can you do it in O(log(m+n)) time?

}

// Test your solution
console.log(findMedianSortedArrays([1,3], [2])); // Should return 2.0
console.log(findMedianSortedArrays([1,2], [3,4])); // Should return 2.5"#.to_string());
                code.insert("python".to_string(), r#"def find_median_sorted_arrays(nums1, nums2):
    """
    :type nums1: List[int]
    :type nums2: List[int]
    :rtype: float
    """
    # Your solution here
    # Challenge: Can you do it in O(log(m+n)) time?
    pass

# Test your solution
print(find_median_sorted_arrays([1,3], [2]))  # Should return 2.0
print(find_median_sorted_arrays([1,2], [3,4]))  # Should return 2.5"#.to_string());
                code.insert("java".to_string(), r#"class Solution {
    public double findMedianSortedArrays(int[] nums1, int[] nums2) {
        // Your solution here
        // Challenge: Can you do it in O(log(m+n)) time?
        return 0.0;
    }

    public static void main(String[] args) {
        Solution sol = new Solution();
        System.out.println(sol.findMedianSortedArrays(new int[]{1,3}, new int[]{2})); // Should return 2.0
        System.out.println(sol.findMedianSortedArrays(new int[]{1,2}, new int[]{3,4})); // Should return 2.5
    }
}"#.to_string());
                code.insert("c".to_string(), r#"#include <stdio.h>

double findMedianSortedArrays(int* nums1, int nums1Size, int* nums2, int nums2Size) {
    // Your solution here
    // Challenge: Can you do it in O(log(m+n)) time?
    return 0.0;
}

int main() {
    int nums1[] = {1, 3};
    int nums2[] = {2};
    printf("%.1f\n", findMedianSortedArrays(nums1, 2, nums2, 1)); // Should return 2.0

    int nums3[] = {1, 2};
    int nums4[] = {3, 4};
    printf("%.1f\n", findMedianSortedArrays(nums3, 2, nums4, 2)); // Should return 2.5
    return 0;
}"#.to_string());
                code.insert("cpp".to_string(), r#"#include <iostream>
#include <vector>
using namespace std;

double findMedianSortedArrays(vector<int>& nums1, vector<int>& nums2) {
    // Your solution here
    // Challenge: Can you do it in O(log(m+n)) time?
    return 0.0;
}

int main() {
    vector<int> nums1 = {1, 3};
    vector<int> nums2 = {2};
    cout << findMedianSortedArrays(nums1, nums2) << endl; // Should return 2.0

    vector<int> nums3 = {1, 2};
    vector<int> nums4 = {3, 4};
    cout << findMedianSortedArrays(nums3, nums4) << endl; // Should return 2.5
    return 0;
}"#.to_string());
                code.insert("rust".to_string(), r#"fn find_median_sorted_arrays(nums1: Vec<i32>, nums2: Vec<i32>) -> f64 {
    // Your solution here
    // Challenge: Can you do it in O(log(m+n)) time?
    0.0
}

fn main() {
    println!("{}", find_median_sorted_arrays(vec![1,3], vec![2])); // Should return 2.0
    println!("{}", find_median_sorted_arrays(vec![1,2], vec![3,4])); // Should return 2.5
}"#.to_string());
                code.insert("go".to_string(), r#"package main

import "fmt"

func findMedianSortedArrays(nums1 []int, nums2 []int) float64 {
    // Your solution here
    // Challenge: Can you do it in O(log(m+n)) time?
    return 0.0
}

func main() {
    fmt.Println(findMedianSortedArrays([]int{1,3}, []int{2})) // Should return 2.0
    fmt.Println(findMedianSortedArrays([]int{1,2}, []int{3,4})) // Should return 2.5
}"#.to_string());
                code
            },
            time_limit_minutes: Some(35),
            tags: vec!["array".to_string(), "binary-search".to_string(), "divide-and-conquer".to_string()],
        };

        self.add_problem(two_sum);
        self.add_problem(reverse_string);
        self.add_problem(valid_parentheses);
        self.add_problem(fizzbuzz);
        self.add_problem(palindrome_number);
        self.add_problem(max_subarray);
        self.add_problem(merge_intervals);
        self.add_problem(group_anagrams);
        self.add_problem(longest_substring);
        self.add_problem(trapping_rain_water);
        self.add_problem(merge_k_lists);
        self.add_problem(median_two_arrays);
    }
}