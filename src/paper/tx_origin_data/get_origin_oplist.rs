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

use crate::paper::strategy::simiarity;
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
    let _rpc = "https://go.getblock.io/6969c2c44a9f4e3893299c50da1d1364";
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
    let opcode_list: Vec<String> = Vec::new();
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

pub fn compare_list(
    _op_list1: Vec<([u8; 20], usize, String)>,
    _op_list2: Vec<([u8; 20], usize, String)>,
) -> f64 {
    let mut control_flow_op_list1: Vec<([u8; 20], usize, String)> = Vec::new();
    let mut control_flow_op_list2: Vec<([u8; 20], usize, String)> = Vec::new();

    for op in _op_list1.clone() {
        if op.2.as_str().eq("JUMPDEST") {
            control_flow_op_list1.push(op);
        }
    }
    for op in _op_list2.clone() {
        if op.2.as_str().eq("JUMPDEST") {
            control_flow_op_list2.push(op);
        }
    }

    let mut same_count = 0;
    let max_len = std::cmp::max(control_flow_op_list1.len(), control_flow_op_list2.len());

    for i in 0..max_len {
        if control_flow_op_list1.len() > i && control_flow_op_list2.len() > i {
            if control_flow_op_list1[i] == control_flow_op_list2[i] {
                same_count += 1;
            }
        }
    }

    let simiarity = same_count as f64 / control_flow_op_list1.len() as f64;
    simiarity
}

#[tokio::test]
async fn test_get() {
    let tx_hash = "0xb9be87b2a62070b0a645e342f73742a17abd9c152c4b3f297bf753d1b768f9c0";
    get_opcode_list_str("", &tx_hash).await;
}
