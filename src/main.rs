mod core_module;
use core_module::utils::errors::ExecutionError;
use primitive_types::U256;
use std::borrow::BorrowMut;
use std::str::FromStr;
// Colored output
use colored::*;
use evm_rs_emulator::paper::my_filed::expression::evaluate_exp_with_unknown;
use evm_rs_emulator::paper::my_filed::sym_exec::sym_exec;
use evm_rs_emulator::paper::my_filed::Handler::Handler;
use evm_rs_emulator::paper::my_filed::Thread_test::{self, HandlerTest};
use local_ip_address::local_ip;
use std::collections::HashMap;
use std::sync::Arc;
// #[tokio::main]
// async fn main() {
//     let handler = Arc::new(
//         Handler::new(
//             "wss://go.getblock.io/4f364318713f46aba8d5b6de9b7e3ae6",
//             "mysql://root:1234@172.29.231.192:3306/invariantregistry",
//             vec![String::from_str("0x4b00a35Eb8CAe62337f37FE561d7fF48987a4FED").unwrap()],
//             vec![String::from_str("0x29e99f07").unwrap()],
//         )
//         .await
//         .unwrap(),
//     );

//     handler.handle().await;
// }
#[tokio::main]
async fn main() {
    let ipaddr = local_ip().unwrap().to_string();
    println!("现在的ip地址为：{:?}", ipaddr);
    let sql_url = format!("mysql://root:1234@{}:3306/new_data", "172.29.218.244");
    let mut _handler = HandlerTest::new(
        "wss://go.getblock.io/4f364318713f46aba8d5b6de9b7e3ae6",
        sql_url,
    )
    .await
    .unwrap();
    let handler = Arc::new(_handler);
    let handler_clone1 = handler.clone();
    let handler_clone2 = handler.clone();
    // 下面开始加线程
    // 该线程用于处理区块轮询
    tokio::task::spawn(async move { handler_clone1.check_invariant().await });
    tokio::task::spawn(async move { handler_clone2.sysm_exec().await });

    tokio::task::spawn(async move {
        handler.get_block().await;
    })
    .await;
}

/*
1.可扩展的数据结构，解决参数离散值的问题，不只是上限和下限
2.

*/
