mod core_module;
use core_module::state::EvmState;
use core_module::utils::errors::ExecutionError;
use ethers::types::H256;
use evm_rs_emulator::paper::my_filed::minitor::listening_storage;
use std::{env, fs, str::FromStr};
// Colored output
use colored::*;

#[tokio::main]
async fn main() -> Result<(), ExecutionError> {
    let rpc = "wss://wiser-stylish-isle.quiknode.pro/fe971117365d555490242e38972893351f3bcd6a/";
    let from = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    let location = vec![H256::from_low_u64_be(0)];
    println!("Before calling listening_storage");
    let _ = listening_storage(rpc, from, location, None, String::from_str("").unwrap()).await;
    println!("After calling listening_storage");
    Ok(())
}
