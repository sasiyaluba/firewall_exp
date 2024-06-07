use reqwest::Client;
use serde_json::{json, Value};

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

// get_op_code_list ["PUSH1", "PUSH1", "MSTORE", "PUSH1", "CALLDATASIZE", "LT"]
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

pub fn compare(list1: Vec<(usize, String)>, list2: Vec<(usize, String)>) -> bool {
    let mut controlflow1: Vec<(usize, String)> = Vec::new();
    let mut controlflow2: Vec<(usize, String)> = Vec::new();
    let mut count = 0;
    // 提取两个list中每个控制流
    for i in list1 {
        if i.1 == "JUMPI"
            || i.1 == "JUMP"
            || i.1 == "CALL"
            || i.1 == "DELEGATECALL"
            || i.1 == "CREATE"
            || i.1 == "CREATE2"
        {
            controlflow1.push(i);
        }
    }
    for i in list2 {
        if i.1 == "JUMPI"
            || i.1 == "JUMP"
            || i.1 == "CALL"
            || i.1 == "DELEGATECALL"
            || i.1 == "CREATE"
            || i.1 == "CREATE2"
        {
            controlflow2.push(i);
        }
    }
    println!("{:?}", controlflow1);
    println!("{:?}", controlflow2);
    // // 比较两个control
    // for i in 0..controlflow1.len() {
    //     if controlflow1[i].1 == controlflow2[i].1 {
    //         count += 1;
    //     }
    // }
    // // 输出比较结果
    // println!(
    //     "control flow match percentage {:?}",
    //     count as f64 / controlflow1.len() as f64
    // );

    true
}

#[tokio::test]
async fn test_get_opcode_list() {
    let rpc = "https://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/";
    let attack_hash = "0x0d51c6fbc9182bf90bcb1f24323bf18aebcce02521023789ce8e58a23a2c6ada";

    let op_list = get_pc_op(rpc, attack_hash).await;
    println!("{:?}", op_list);
}
