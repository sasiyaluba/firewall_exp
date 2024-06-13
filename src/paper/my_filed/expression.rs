use ast::Ast;
use ethers::types::U256;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One, Zero};
use regex::bytes::Regex;
use std::collections::VecDeque;
use std::fmt;
use std::str::FromStr;
use z3::ast::Int;
use z3::*;

use crate::core_module::context;
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
    let _expression = _expression.replace(" ", "");
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

pub fn evaluate_exp_with_unknown(_expression: &str) -> Result<(u128, u128), ExpressionCaclError> {
    // z3上下文以及求解器
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    // 去除空格
    let _expression = _expression.replace(" ", "");
    // 用&&分割表达式
    let _expressions = _expression.split("&&");

    // 得到变量，此时的exp应该只有一个未知数
    let temps = get_all_variables(&_expression);
    // 第一个即为未知数
    let param = temps.first().unwrap();
    // 创建z3未知数
    let x = Int::new_const(&ctx, param.as_str());
    // 添加基本约束，未知数>0
    solver.assert(&x.ge(&Int::from_i64(&ctx, 0)));

    // 遍历每个表达式，为求解器分别加上约束
    for _exp in _expressions {
        println!("now expression {:?}", _exp);
        // 得到中缀表达式
        let re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*|[0-9]+|[+\-*/()<>!=]+)").unwrap();
        let infix: Vec<String> = re
            .find_iter(_exp.as_bytes())
            .map(|mat| String::from_utf8(mat.as_bytes().to_vec()).unwrap())
            .collect();
        // 中缀转后缀
        let y = infix_to_postfix(infix);
        // 计算后缀表达式，为求解器加上约束
        evaluate_postfix_with_unknown(&solver, y.clone(), x.clone());
        println!("now constraint {:?}", &solver.get_assertions());
    }
    // 求解
    let mut _new_param: Vec<u128> = vec![];
    match solver.check() {
        SatResult::Sat => {
            // 找到第一个解
            let mut mid_value = solver.get_model().unwrap().eval(&x, true).unwrap();
            // 保留
            _new_param.push(mid_value.as_u64().unwrap().into());
            // 做循环，直到没有解情况
            loop {
                // 保留回退点
                solver.push();
                // 添加约束，需要当前解不重复
                solver.assert(&x._eq(&mid_value).not());
                // 检查是否有解
                match solver.check() {
                    SatResult::Sat => {
                        // 更新解
                        mid_value = solver.get_model().unwrap().eval(&x, true).unwrap();
                        // 插入解
                        _new_param.push(mid_value.as_u64().unwrap().into());
                    }
                    SatResult::Unsat | SatResult::Unknown => {
                        // 无解则break
                        solver.pop(1);
                        break;
                    }
                }
            }
        }
        SatResult::Unsat | SatResult::Unknown => {
            return Err(ExpressionCaclError::SyntaxError);
        }
    }
    // 返回值
    let (max, min) = find_max_min(&_new_param).unwrap();
    Ok((min, max))
}

pub fn infix_to_postfix(tokens: Vec<String>) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut operators: VecDeque<String> = VecDeque::new();

    for token in tokens {
        if token.chars().all(char::is_alphanumeric) {
            output.push(token);
        } else if token == "(" {
            operators.push_front(token);
        } else if token == ")" {
            while let Some(op) = operators.pop_front() {
                if op == "(" {
                    break;
                }
                output.push(op);
            }
        } else {
            while let Some(op) = operators.front() {
                if get_precedence(op) >= get_precedence(&token) {
                    output.push(operators.pop_front().unwrap());
                } else {
                    break;
                }
            }
            operators.push_front(token);
        }
    }

    while let Some(op) = operators.pop_front() {
        output.push(op);
    }

    output
}

pub fn evaluate_postfix_with_unknown<'a>(
    solver: &'a Solver<'a>,
    tokens: Vec<String>,
    param: Int<'a>,
) {
    let mut stack = Vec::new();
    let context = solver.get_context();
    for token in tokens {
        if token.chars().all(char::is_numeric) {
            // 是数字常量，转换为Int类型，压入栈中
            stack.push(ast::Int::from_str(context, &token).unwrap());
        } else if token.eq(&param.to_string()) {
            stack.push(param.clone());
        } else {
            // 是操作符，从栈中弹出两个操作数，进行运算
            let b = stack.pop().unwrap();
            let a = stack.pop().unwrap();
            let result = match token.as_str() {
                "+" => ast::Int::add(context, &[&a, &b]),
                "-" => ast::Int::sub(context, &[&a, &b]),
                "*" => ast::Int::mul(context, &[&a, &b]),
                "/" => ast::Int::div(&b, &a),
                "==" => {
                    solver.assert(&a._eq(&b));
                    ast::Int::from_i64(context, -1)
                }
                "!=" => {
                    solver.assert(&a._eq(&b).not());
                    ast::Int::from_i64(context, -1)
                }
                "<=" => {
                    solver.assert(&a.le(&b));
                    ast::Int::from_i64(context, -1)
                }
                ">=" => {
                    solver.assert(&a.ge(&b));
                    ast::Int::from_i64(context, -1)
                }
                ">" => {
                    solver.assert(&a.gt(&b));
                    ast::Int::from_i64(context, -1)
                }
                "<" => {
                    solver.assert(&a.lt(&b));
                    ast::Int::from_i64(context, -1)
                }
                _ => panic!("Invalid operator"),
            };
            stack.push(result);
        }
    }
}

pub fn find_max_min(array: &[u128]) -> Option<(u128, u128)> {
    if array.is_empty() {
        return None;
    }

    let mut max_value = array[0];
    let mut min_value = array[0];

    for &num in array.iter() {
        if num > max_value {
            max_value = num;
        }
        if num < min_value {
            min_value = num;
        }
    }

    Some((max_value, min_value))
}
#[test]
fn test1() {
    let expression = "param0+10<20";
    let result = evaluate_exp_with_unknown(expression);
}
