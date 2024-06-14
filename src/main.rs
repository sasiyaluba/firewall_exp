mod core_module;
use core_module::utils::errors::ExecutionError;
use primitive_types::U256;
use std::str::FromStr;
// Colored output
use colored::*;
use evm_rs_emulator::paper::my_filed::expression::evaluate_exp_with_unknown;
use evm_rs_emulator::paper::my_filed::sym_exec::sym_exec;
use evm_rs_emulator::paper::my_filed::Handler::Handler;
#[tokio::main]
async fn main() {
    let handler = Handler::new(
        "wss://go.getblock.io/4f364318713f46aba8d5b6de9b7e3ae6",
        "mysql://root:1234@172.29.199.74:3306/invariantregistry",
        vec![String::from_str("0x4b00a35Eb8CAe62337f37FE561d7fF48987a4FED").unwrap()],
    )
    .await
    .unwrap();
    handler.handle().await;
}

/*
1.可扩展的数据结构，解决参数离散值的问题，不只是上限和下限
2.

*/
