use std::env;
use std::str::FromStr;
use std::sync::Arc;
use dotenv::dotenv;
use ethers::addressbook::Address;
use ethers::prelude::{Provider, ProviderError, ProviderExt, TxHash};
use crate::bytes::{_hex_string_to_bytes, to_h256};
use crate::core_module::context::account_state_ex_context::{get_accounts_state_tx, get_tx_after_accounts_state, ISDiff};
use crate::core_module::context::transaction_context::get_transaction_content;
use crate::{EvmState, Runner};
use crate::core_module::context::calldata_info::CallDataInfo;
use crate::core_module::context::evm_context::EvmContext;
use crate::paper::evm_interpreter::get_evm_interpreter::get_evm_interpreter;
use crate::paper::strategy::param_strategy::ParamStrategy;
use crate::paper::strategy::path_strategy::PathStrategy;


pub async fn exec(
    _rpc: &str,
    _tx_hash: &str,
    _function_selector: &str,
    _index: u8,
    _path_strategy: Option<PathStrategy>,// 默认为全路径匹配策略
    _param_strategy: Option<ParamStrategy>
) -> Result<Vec<Vec<u8>>, ProviderError> {

    // 1. 得到路径约束策略

    // 2. 得到参数枚举策略

    // 3.

    let mut return_param_list: Vec<Vec<u8>> = Vec::new();
    Ok(return_param_list)
}

async fn exec_internal(
    _rpc: &str,
    _tx_hash: &'static str,
    _param_origin_data: Vec<u8>,
    _all_possible_param_list: Vec<Vec<u8>>,
    _constraint_path: Option<Vec<&'static str>>,
    _multi_thread: Option<bool>) -> Result<Vec<Vec<u8>>, ProviderError>
{
    let mut interpreter = get_evm_interpreter(_rpc, _tx_hash).await.unwrap();

    // 1. 将路约束加载进入到对应的interpreter
    interpreter.constraint_path = _constraint_path;

    // 2. 更新interpreter对应的calldata
    let mut calldata_info = CallDataInfo::new();
    calldata_info.origin = _param_origin_data;
    interpreter.calldata_info = Some(calldata_info);

    // 3. 开始执行
    let mut return_param_list: Vec<Vec<u8>> = Vec::new();
    let bytecode = interpreter.bytecode.clone();
    if _multi_thread.is_some().eq(&true) {
        // todo! 需要添加多线程的实现
    } else {
        for new_param in _all_possible_param_list {
            // Interpret the bytecode
            let ret = interpreter.interpret_init(bytecode.clone(),new_param.clone(), true);
            if ret.is_ok() {
                println!("{:?}", interpreter.op_list.len());
                return_param_list.push(new_param.clone());
                println!("find new param ");
            }
        }
    }

    Ok(return_param_list)
}