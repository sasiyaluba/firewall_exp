



pub enum ParamStrategy {
    V1,
    V2,
}
//
// 从历史记录中拉取参数可能的范围, 结合不变量与符号执行推断其初步执行范围
// 得到