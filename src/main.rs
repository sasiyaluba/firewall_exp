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
        vec![String::from_str("0x70ccd19d14552da0fb0712fd3920aeb1f9f65f59").unwrap()],
    )
    .await
    .unwrap();
    handler.handle().await;
}
