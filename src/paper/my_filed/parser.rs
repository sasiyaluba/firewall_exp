use std::collections::HashMap;
pub fn parse_expression(input: &str, variables: Option<&HashMap<&str, i128>>) -> i128 {
    // 去掉所有空格
    let input = input.replace(" ", "");
    let mut pos = 0;
    let mut current_char = input.chars().next();

    // 前进函数
    fn advance(input: &str, pos: &mut usize, current_char: &mut Option<char>) {
        *pos += 1;
        if *pos < input.len() {
            *current_char = input[*pos..].chars().next();
        } else {
            *current_char = None;
        }
    }

    // 解析数字常量
    fn number(input: &str, pos: &mut usize, current_char: &mut Option<char>) -> i128 {
        let mut result = String::new();
        // 只要当前的字符是数字，就一直读取
        while let Some(c) = *current_char {
            if c.is_digit(10) {
                result.push(c);
                advance(input, pos, current_char);
            } else {
                break;
            }
        }
        // 将读取到的数字转换为i128类型
        result.parse::<i128>().unwrap()
    }

    // 解析变量名
    fn variable(input: &str, pos: &mut usize, current_char: &mut Option<char>) -> String {
        let mut result = String::new();
        // 变量名必须以字母或下划线开头
        if let Some(c) = *current_char {
            if c.is_alphabetic() || c == '_' {
                result.push(c);
                advance(input, pos, current_char);
            } else {
                return result; // 不符合变量名的规则，返回空字符串
            }
        }
        // 继续读取字母、数字和下划线
        while let Some(c) = *current_char {
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
                advance(input, pos, current_char);
            } else {
                break;
            }
        }
        result
    }

    // 解析因子
    fn factor(
        input: &str,
        pos: &mut usize,
        current_char: &mut Option<char>,
        variables: Option<&HashMap<&str, i128>>,
    ) -> i128 {
        if let Some(c) = *current_char {
            if c == '(' {
                advance(input, pos, current_char);
                let result = expr(input, pos, current_char, variables);
                if *current_char == Some(')') {
                    advance(input, pos, current_char);
                } else {
                    panic!("期望一个右括号");
                }
                result
            } else if c.is_digit(10) {
                number(input, pos, current_char)
            } else if c.is_alphabetic() || c == '_' {
                let var_name = variable(input, pos, current_char);
                *variables
                    .unwrap()
                    .get(var_name.as_str())
                    .unwrap_or_else(|| panic!("变量 {} 未定义", var_name))
            } else {
                panic!("当前字符不属于factor范围");
            }
        } else {
            panic!("当前input已经解析完毕");
        }
    }

    // 解析项
    fn term(
        input: &str,
        pos: &mut usize,
        current_char: &mut Option<char>,
        variables: Option<&HashMap<&str, i128>>,
    ) -> i128 {
        let mut result = factor(input, pos, current_char, variables);
        while let Some(c) = *current_char {
            if c == '*' {
                advance(input, pos, current_char);
                result *= factor(input, pos, current_char, variables);
            } else if c == '/' {
                advance(input, pos, current_char);
                result /= factor(input, pos, current_char, variables);
            } else {
                break;
            }
        }
        result
    }

    // 解析表达式
    fn expr(
        input: &str,
        pos: &mut usize,
        current_char: &mut Option<char>,
        variables: Option<&HashMap<&str, i128>>,
    ) -> i128 {
        let mut result = term(input, pos, current_char, variables);
        while let Some(c) = *current_char {
            if c == '+' {
                advance(input, pos, current_char);
                result += term(input, pos, current_char, variables);
            } else if c == '-' {
                advance(input, pos, current_char);
                result -= term(input, pos, current_char, variables);
            } else if c == '>' {
                advance(input, pos, current_char);
                result = if *current_char == Some('=') {
                    advance(input, pos, current_char);
                    (result >= term(input, pos, current_char, variables)) as i128
                } else {
                    (result > term(input, pos, current_char, variables)) as i128
                };
            } else if c == '<' {
                advance(input, pos, current_char);
                result = if *current_char == Some('=') {
                    advance(input, pos, current_char);
                    (result <= term(input, pos, current_char, variables)) as i128
                } else {
                    (result < term(input, pos, current_char, variables)) as i128
                };
            } else {
                break;
            }
        }
        result
    }

    expr(&input, &mut pos, &mut current_char, variables)
}

#[test]
fn main() {
    let mut variables = HashMap::new();
    variables.insert("x1", 3);
    variables.insert("x2", 4);
    variables.insert("x3", 2);
    let result = parse_expression("x1 + x2 * x3 / (6 - 5)>100", Some(&variables));

    println!("Result: {}", result); // 输出结果
}
