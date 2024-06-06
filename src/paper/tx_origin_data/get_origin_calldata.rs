use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fmt::format;

use super::super::invariant::database::Value_range;
use crate::bytes::_hex_string_to_bytes;
use crate::bytes::pad_left;
use crate::paper::invariant::listening_tx::RangeExpression;

// 将call_tracer里面的信息全部读取到一个list里面
fn recursive_read_input(data: &Value) -> Vec<String> {
    let mut ret_calldata = Vec::new();
    if let Some(data_map) = data.as_object() {
        if let Some(input_data) = data_map.get("input") {
            if let Some(input_str) = input_data.as_str() {
                ret_calldata.push(input_str.to_string());
            }
        }
        for value in data_map.values() {
            ret_calldata.extend(recursive_read_input(value));
        }
    } else if let Some(data_list) = data.as_array() {
        for item in data_list {
            ret_calldata.extend(recursive_read_input(item));
        }
    }
    ret_calldata
}

// 找到call调用层中第一个满足to地址是我们输入的调用，并获取其inputdata
fn recursive_read_input_targetAddress(data: &Value, to: &str) -> Result<Vec<String>, ()> {
    let mut ret_calldata = Vec::new();
    if let Some(data_map) = data.as_object() {
        if to == data_map.get("to").unwrap() {
            if let Some(input_data) = data_map.get("input") {
                if let Some(input_str) = input_data.as_str() {
                    ret_calldata.push(input_str.to_string());
                    println!("now retcalldata includes :{:?}", ret_calldata);
                    return Ok(ret_calldata);
                }
            }
        }
        for value in data_map.values() {
            if let Ok(mut result) = recursive_read_input_targetAddress(value, to) {
                ret_calldata.append(&mut result);
                return Ok(ret_calldata); // Stop further recursion if a match is found
            }
        }
    } else if let Some(data_list) = data.as_array() {
        for item in data_list {
            if let Ok(mut result) = recursive_read_input_targetAddress(item, to) {
                ret_calldata.append(&mut result);
                return Ok(ret_calldata); // Stop further recursion if a match is found
            }
        }
    }
    Err(())
}

fn func_name_to_selector(func_name_str: &str) -> String {
    let selector = &ethers::core::utils::keccak256(func_name_str)[0..4];
    // 将十进制数组转换为十六进制字符串
    let selector_str: String = [
        "0x",
        &selector
            .iter()
            .map(|&x| format!("{:02X}", x).to_lowercase())
            .collect::<Vec<String>>()
            .join(""),
    ]
    .concat();

    println!("{:?}", selector_str);
    selector_str
}

// func_name_str ""
///@param _index表示目标参数在函数中的位置
async fn get_origin_calldata(
    _rpc: &str,
    _attack_hash: &str,
    func_name_str: &str,
    _index: u8,
) -> Vec<u8> {
    // 拿到 call_tracer
    let client = Client::new();

    let res = client
        .post(_rpc)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "debug_traceTransaction",
            "params": [
                _attack_hash,
                {"tracer": "callTracer"}
            ]
        }))
        .send()
        .await
        .expect("rpc error");
    let tracer_data = res.json::<Value>().await.expect("json lib error");
    println!("trace_data is :{:?}", tracer_data);

    let mut param_data: Vec<u8> = Vec::new();
    if tracer_data["result"]["failed"].eq(&true) {
        return param_data;
    }

    // 这里如果是加上["calls"]的话，是没有将最外层的call调用算进去
    let call_data = tracer_data["result"]["calls"].clone();
    println!("call_data is :{:?}", call_data);

    let input_data_list = recursive_read_input(&call_data);
    println!("Whole inputdata is :{:?}", input_data_list);

    let func_selector = func_name_to_selector(func_name_str);

    // 过滤并去重
    // 找到调用目标函数的inputdata
    let mut unique_calldata_param = HashSet::new();
    for item in input_data_list {
        if item.starts_with(func_selector.as_str()) {
            unique_calldata_param.insert(item);
        }
    }

    // 将 HashSet 转换回 Vec
    let origin_calldata_param: Vec<String> = unique_calldata_param.into_iter().collect();

    // 计算起始位置和结束位置 0x + 4_bytes_function_selector
    let start_position = (2 + 8 + (_index - 1) * 64) as usize;
    let end_position = start_position + 64;

    // 截取子字符串
    let index_param_value = &origin_calldata_param[0][start_position..end_position];
    let index_param_value_bytes = _hex_string_to_bytes(index_param_value);

    index_param_value_bytes
}

async fn get_origin_calldata_to(
    _rpc: &str,
    _attack_hash: &str,
    func_name_str: &str,
    _index: u8,
    to: &str,
) -> Vec<u8> {
    // 拿到 call_tracer
    let client = Client::new();

    let res = client
        .post(_rpc)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "debug_traceTransaction",
            "params": [
                _attack_hash,
                {"tracer": "callTracer"}
            ]
        }))
        .send()
        .await
        .expect("rpc error");
    let tracer_data = res.json::<Value>().await.expect("json lib error");

    let mut param_data: Vec<u8> = Vec::new();
    if tracer_data["result"]["failed"].eq(&true) {
        return param_data;
    }

    let call_data = tracer_data["result"].clone();

    let input_data_list = recursive_read_input_targetAddress(&call_data, to).unwrap();
    println!("The inputdata is :{:?}", input_data_list);

    let func_selector = func_name_to_selector(func_name_str);

    // 计算起始位置和结束位置 0x + 4_bytes_function_selector
    let start_position = (2 + 8 + (_index - 1) * 64) as usize;
    let end_position = start_position + 64;

    // 截取子字符串
    let index_param_value = &input_data_list[0][start_position..end_position];
    let index_param_value_bytes = _hex_string_to_bytes(index_param_value);

    index_param_value_bytes
}

fn Fill_param(paramType: &str, param_Range: Value_range) -> Vec<String> {
    let mut fill = Vec::<String>::new();

    let mut PR: Vec<primitive_types::U256> = param_Range.getRange();
    for i in PR {
        match paramType {
            // pad_left
            "address" | "uint256" | "bool" => {
                let padded = format!("{:064x}", i);
                println!("Padded value: {}", padded);
                fill.push(padded);
            }
            _ => {}
        }
    }
    fill
}

#[tokio::test]
async fn test_get_opcode_list() {
    let rpc = "https://lb.nodies.app/v1/181a5ebf4c954f8496ae7cbc1ac8d03b";
    let attack_hash = "0x3ed75df83d907412af874b7998d911fdf990704da87c2b1a8cf95ca5d21504cf";

    let origin_param_data = get_origin_calldata_to(
        rpc,
        attack_hash,
        "redeem(address,uint256)",
        1,
        "0x007fe7c498a2cf30971ad8f2cbc36bd14ac51156",
    )
    .await;
    println!("origin inputdata is: {:?}", origin_param_data);
}

#[tokio::test]
async fn test_get_target_address_input() {
    //获取第一个目标地址为to地址的inputdata
    let rpc_sepolia = "https://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/";
    let attack_hash2 = "0x78815002807ae469c843ffb6a9f17f2836d0dc70a033e3d0b77514a2fd23c0c0";

    //"1": 要替换的参数位于函数中的哪个位置
    //"0x42c1c8bf2c0244bbe7755e592252992f580daaf4": calls中的目标地址
    let origin_param_data = get_origin_calldata_to(
        rpc_sepolia,
        attack_hash2,
        "getFunds(uint256,address,uint256)",
        1,
        "0x42c1c8bf2c0244bbe7755e592252992f580daaf4",
    )
    .await;

    println!(
        "The target param in origin inputdata is: {:?}",
        origin_param_data
    );
}

#[tokio::test]
async fn test_change_param() {
    //表达式
    let test_expression =
        vec!["(declare-const fundsAmount Int) (assert (>= (- 3 fundsAmount) 1))".to_string()];
    let mut exp1 = RangeExpression::new(test_expression, "is alright".to_string());
    //求解参数范围
    let mut result = exp1.test_getRange();
    println!("In this case, our Value_range is :{:?}", result);

    //将参数范围中的每一个元素进行encode
    let padded_params = Fill_param("uint256", result);
    println!("填充之后的参数列表")

    //next:将这些参数提放到原来的calldata中形成新的calldata
    
}
