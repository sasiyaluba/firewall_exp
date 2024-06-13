use num_bigint::BigInt;
use num_traits::{FromPrimitive, One, Zero};
use regex::bytes::Regex;
use std::fmt;

#[derive(Debug)]
pub enum ExpressionCaclError {
    InvalidToken(String),
    MismatchedParentheses,
    DivisionByZero,
    SyntaxError,
}

impl fmt::Display for ExpressionCaclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionCaclError::InvalidToken(token) => write!(f, "Invalid token: {}", token),
            ExpressionCaclError::MismatchedParentheses => write!(f, "Mismatched parentheses"),
            ExpressionCaclError::DivisionByZero => write!(f, "Division by zero"),
            ExpressionCaclError::SyntaxError => write!(f, "Syntax error"),
        }
    }
}
pub fn evaluate_exp(_expression: &str) -> Result<bool, ExpressionCaclError> {
    let re = Regex::new(r"(\d+|[+*/()-])|>=|<=|>|<|!=|==").unwrap();
    let x: Vec<String> = re
        .find_iter(_expression.as_bytes())
        .map(|mat| String::from_utf8(mat.as_bytes().to_vec()).unwrap())
        .collect();
    let y = infix_to_postfix(x);
    let result = evaluate_postfix(y);
    Ok(result >= BigInt::one())
}
pub fn get_precedence(op: &str) -> BigInt {
    match op {
        "+" | "-" => BigInt::one(),
        "*" | "/" => BigInt::from_i32(2).unwrap(),
        "==" | "!=" | "<" | ">" | "<=" | ">=" => BigInt::zero(),
        _ => BigInt::from_i32(-1).unwrap(),
    }
}
pub fn infix_to_postfix(tokens: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    let mut operators = Vec::new();

    for token in tokens {
        if token.chars().all(char::is_numeric) {
            output.push(token);
        } else if token == "(" {
            operators.push(token);
        } else if token == ")" {
            while let Some(top) = operators.pop() {
                if top == "(" {
                    break;
                }
                output.push(top);
            }
        } else {
            while let Some(top) = operators.last() {
                if get_precedence(top) >= get_precedence(&token) {
                    output.push(operators.pop().unwrap());
                } else {
                    break;
                }
            }
            operators.push(token);
        }
    }

    while let Some(top) = operators.pop() {
        output.push(top);
    }

    output
}
pub fn evaluate_postfix(tokens: Vec<String>) -> BigInt {
    let mut stack = Vec::new();

    for token in tokens {
        if token.chars().all(char::is_numeric) {
            stack.push(token.parse::<BigInt>().unwrap());
        } else {
            let b = stack.pop().unwrap();
            let a = stack.pop().unwrap();
            let result = match token.as_str() {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => a / b,
                "==" => {
                    if a == b {
                        BigInt::one()
                    } else {
                        BigInt::zero()
                    }
                }
                "!=" => {
                    if a != b {
                        BigInt::one()
                    } else {
                        BigInt::zero()
                    }
                }
                "<=" => {
                    if a <= b {
                        BigInt::one()
                    } else {
                        BigInt::zero()
                    }
                }
                ">=" => {
                    if a >= b {
                        BigInt::one()
                    } else {
                        BigInt::zero()
                    }
                }
                "<" => {
                    if a < b {
                        BigInt::one()
                    } else {
                        BigInt::zero()
                    }
                }
                ">" => {
                    if a > b {
                        BigInt::one()
                    } else {
                        BigInt::zero()
                    }
                }
                _ => panic!("Invalid operator"),
            };
            stack.push(result);
        }
    }

    stack.pop().unwrap()
}

pub fn get_all_variables(_expression: &str) -> Vec<String> {
    let re = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
    re.find_iter(_expression.as_bytes())
        .map(|mat| String::from_utf8(mat.as_bytes().to_vec()).unwrap())
        .collect()
}
#[test]
fn main() {
    match evaluate_exp("(100 + 150) * 100 < 100") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
