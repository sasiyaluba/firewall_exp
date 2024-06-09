use ethers::middleware::gas_oracle::blocknative::Response;
use ethers::prelude::TxHash;
use ethers::types::{GethDebugBuiltInTracerConfig, GethTrace};
use ethers::types::{GethDebugTracerConfig, GethDebugTracingOptions};
use ethers_providers::{JsonRpcClient, JsonRpcClientWrapper, Middleware, Provider, Ws};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
fn extract_op_values(logs: &Value) -> Vec<String> {
    let mut op_values = Vec::new();

    if let Some(logs_array) = logs.as_array() {
        for log in logs_array {
            if let Some(op_value) = log["op"].as_str() {
                op_values.push(op_value.to_string());
            }
        }
    }
    op_values
}

fn extract_op_values_str(logs: &Value) -> Vec<String> {
    let mut op_values = Vec::new();

    if let Some(logs_array) = logs.as_array() {
        for log in logs_array {
            if let Some(op_value) = log["op"].as_str() {
                op_values.push(op_value.to_string());
            }
        }
    }
    op_values
}
fn extract_pc_op(logs: &Value) -> Vec<(usize, String)> {
    let mut values: Vec<(usize, String)> = vec![];

    if let Some(logs_array) = logs.as_array() {
        for log in logs_array {
            if let Some(op_value) = log["op"].as_str() {
                if let Some(pc_value) = log["pc"].as_u64() {
                    values.push((pc_value as usize, op_value.to_string()));
                }
            }
        }
    }
    values
}

pub async fn get_opcode_list(_rpc: &str, _attack_hash: &str) -> Vec<String> {
    let client = Client::new();
    let res = client
        .post(_rpc)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "debug_traceTransaction",
            "params": [
                _attack_hash,
                {
                    "enableMemory": false,
                    "disableStack": true,
                    "disableStorage": true,
                    "enableReturnData": false
                }
            ]
        }))
        .send()
        .await
        .expect("rpc error");
    let tracer_data = res.json::<Value>().await.expect("json lib error");

    let mut opcode_list: Vec<String> = Vec::new();
    if tracer_data["result"]["failed"].eq(&true) {
        return opcode_list;
    }

    // 获取 result 下的 structLogs 字段
    let struct_logs = tracer_data["result"]["structLogs"].clone();
    extract_op_values(&struct_logs)
}

// get_op_code_list ["PUSH1", "PUSH1", "MSTORE", "PUSH1", "CALLDATASIZE", "LT"]
pub async fn get_opcode_list_str(_rpc: &str, _attack_hash: &str) -> Vec<String> {
    let client = Client::new();
    let res: reqwest::Response = client
        .post(_rpc)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "debug_traceTransaction",
            "params": [
                _attack_hash,
                {
                    "enableMemory": false,
                    "disableStack": true,
                    "disableStorage": true,
                    "enableReturnData": false
                }
            ]
        }))
        .send()
        .await
        .expect("rpc error");
    let tracer_data = res.json::<Value>().await.expect("json lib error");

    let mut opcode_list: Vec<String> = Vec::new();
    if tracer_data["result"]["failed"].eq(&true) {
        return opcode_list;
    }

    // 获取 result 下的 structLogs 字段
    let struct_logs = tracer_data["result"]["structLogs"].clone();
    extract_op_values_str(&struct_logs)
}

pub async fn get_pc_op(_rpc: &str, _attack_hash: &str) -> Vec<(usize, String)> {
    let client = Client::new();
    let res = client
        .post(_rpc)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "debug_traceTransaction",
            "params": [
                _attack_hash,
                {
                    "enableMemory": false,
                    "disableStack": true,
                    "disableStorage": true,
                    "enableReturnData": false
                }
            ]
        }))
        .send()
        .await
        .expect("rpc error");
    let tracer_data = res.json::<Value>().await.expect("json lib error");

    let mut opcode_list: Vec<(usize, String)> = Vec::new();
    if tracer_data["result"]["failed"].eq(&true) {
        return opcode_list;
    }

    // 获取 result 下的 structLogs 字段
    let struct_logs = tracer_data["result"]["structLogs"].clone();
    extract_pc_op(&struct_logs)
}

// 比较两个指令序列
pub fn compare_list(_op_list1: Vec<String>, _op_list2: Vec<String>) -> f64 {
    let mut control_flow_op_list1: Vec<String> = Vec::new();
    let mut control_flow_op_list2: Vec<String> = Vec::new();
    // 提取两个指令序列的控制流指令，jumpi，jump，return，stop，revert，invalid，call，delegatecall，callcode，create，create2
    for op in _op_list1 {
        if op.eq("JUMPI")
            || op.eq("JUMP")
            || op.eq("RETURN")
            || op.eq("STOP")
            || op.eq("REVERT")
            || op.eq("INVALID")
            || op.eq("CALL")
            || op.eq("DELEGATECALL")
            || op.eq("CALLCODE")
            || op.eq("CREATE")
            || op.eq("CREATE2")
        {
            control_flow_op_list1.push(op);
        }
    }

    for op in _op_list2 {
        if op.eq("JUMPI")
            || op.eq("JUMP")
            || op.eq("RETURN")
            || op.eq("STOP")
            || op.eq("REVERT")
            || op.eq("INVALID")
            || op.eq("CALL")
            || op.eq("DELEGATECALL")
            || op.eq("CALLCODE")
            || op.eq("CREATE")
            || op.eq("CREATE2")
        {
            control_flow_op_list2.push(op);
        }
    }

    // 进行逐条比较，记录相似率
    let mut same_count = 0;

    for i in 0..control_flow_op_list1.len() {
        if control_flow_op_list2.len() > i && control_flow_op_list1[i] == control_flow_op_list2[i] {
            same_count += 1;
        }
    }
    let simliarity = same_count as f64 / control_flow_op_list1.len() as f64;

    simliarity
}

#[tokio::test]
async fn test_get_opcode_list() {
    let rpc = "https://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/";
    let attack_hash = "0x0d51c6fbc9182bf90bcb1f24323bf18aebcce02521023789ce8e58a23a2c6ada";

    let op_list = get_pc_op(rpc, attack_hash).await;
    println!("{:?}", op_list);
}
