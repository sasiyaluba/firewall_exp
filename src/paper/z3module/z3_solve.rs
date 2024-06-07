use z3::*;

pub fn test() {
    let config = z3::Config::new();
    let context = z3::Context::new(&config);
    let solver = z3::Solver::new(&context);
    let mut _new_param: Vec<u64> = vec![];
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
            _new_param.push(mid_value.as_u64().unwrap());
            // 根据中间值，向两边继续寻找可用解
            let mut min_value = mid_value.clone();
            let mut max_value = mid_value.clone();
            loop {
                // 保留回退点
                solver.push();
                // 添加约束
                solver.assert(&x.lt(&min_value));
                // println!("now constraint {:?}", solver.get_assertions());
                // 检查是否有解
                match solver.check() {
                    SatResult::Sat => {
                        // 更新最小值
                        min_value = solver.get_model().unwrap().eval(&x, true).unwrap();
                        _new_param.push(min_value.as_u64().unwrap());
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
                // println!("now constraint {:?}", solver.get_assertions());
                // 检查是否有解
                match solver.check() {
                    SatResult::Sat => {
                        // 更新最小值
                        max_value = solver.get_model().unwrap().eval(&x, true).unwrap();
                        _new_param.push(max_value.as_u64().unwrap());
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
            // println!("_new_param {:?}", _new_param);
        }
        SatResult::Unsat => {}
        SatResult::Unknown => {}
    }
}
