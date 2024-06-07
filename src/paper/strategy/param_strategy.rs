use ethers::types::{Address, U256};
use revm_primitives::HashMap;
use z3::*;

pub enum ParamStrategy {
    FullParamEnumeration,
    // V2,
}
//
// 从历史记录中拉取参数可能的范围, 结合不变量与符号执行推断其初步执行范围
// 得到
pub struct ValueRange {
    range: Vec<Vec<u8>>,
}
pub struct ParamRange {
    // 地址 -> selector -> index -> 范围
    param_range: HashMap<Address, HashMap<[u8; 4], HashMap<u64, ValueRange>>>,
}

pub fn simple_test() -> Vec<Vec<u8>> {
    let config = z3::Config::new();
    let context = z3::Context::new(&config);
    let solver = z3::Solver::new(&context);
    let mut _new_param: Vec<Vec<u8>> = vec![];
    // 状态变量的值
    let address_balance = &ast::Int::from_u64(&context, 20);
    let invariant = &ast::Int::from_u64(&context, 1);
    // 函数参数，在上下文创建函数参数作为变量
    let x = ast::Int::new_const(&context, "x");

    // 定义表达式
    let expr = x.le(&ast::Int::sub(&context, &[&address_balance, &invariant]));
    solver.assert(&expr);
    solver.assert(&x.gt(&ast::Int::from_u64(&context, 0)));
    // 解
    match solver.check() {
        SatResult::Sat => {
            let mid_value = solver.get_model().unwrap().eval(&x, true).unwrap();
            _new_param.push(mid_value.as_u64().unwrap().to_be_bytes().to_vec());
            // 根据中间值，向两边继续寻找可用解
            let mut min_value = mid_value.clone();
            let mut max_value = mid_value.clone();
            loop {
                // 保留回退点
                solver.push();
                // 添加约束
                solver.assert(&x.lt(&min_value));
                // 检查是否有解
                match solver.check() {
                    SatResult::Sat => {
                        // 更新最小值
                        min_value = solver.get_model().unwrap().eval(&x, true).unwrap();
                        _new_param.push(min_value.as_u64().unwrap().to_be_bytes().to_vec());
                    }
                    SatResult::Unsat => {
                        // 无解则break
                        solver.pop(1);
                        break;
                    }
                    SatResult::Unknown => {
                        // 无解则break
                        solver.pop(1);
                        break;
                    }
                }
            }
            loop {
                // 保留回退点
                solver.push();
                // 添加约束
                solver.assert(&x.gt(&max_value));
                // 检查是否有解
                match solver.check() {
                    SatResult::Sat => {
                        // 更新最小值
                        max_value = solver.get_model().unwrap().eval(&x, true).unwrap();
                        _new_param.push(max_value.as_u64().unwrap().to_be_bytes().to_vec());
                    }
                    SatResult::Unsat => {
                        // 无解则break
                        solver.pop(1);
                        break;
                    }
                    SatResult::Unknown => {
                        // 无解则break
                        solver.pop(1);
                        break;
                    }
                }
            }
        }
        SatResult::Unsat => {}
        SatResult::Unknown => {}
    }
    return _new_param;
}

// todo!从数据库获取
pub fn get_range(address: &str, selector: [u8; 4], index: u8) {}

pub fn get_range_temp(address: &str, selector: &str, index: u8) -> Vec<Vec<u8>> {
    return simple_test();
}
