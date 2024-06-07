use crate::bytes::pad_left;
use crate::bytes::{_hex_string_to_bytes, to_h256};
use crate::core_module::context::account_state_ex_context::{
    get_accounts_state_tx, get_tx_after_accounts_state, ISDiff,
};
use crate::core_module::context::calldata_info::CallDataInfo;
use crate::core_module::context::evm_context::EvmContext;
use crate::core_module::context::transaction_context::get_transaction_content;
use crate::paper::evm_interpreter::get_evm_interpreter::get_evm_interpreter;
use crate::paper::strategy::param_strategy::get_range_temp;
use crate::paper::strategy::param_strategy::ParamStrategy;
use crate::paper::strategy::path_strategy::PathStrategy;
use crate::{EvmState, Runner};
use ethers::prelude::{Provider, ProviderError, ProviderExt, TxHash};
use ethers::types::GethDebugTracingOptions;
use ethers::types::H256;
use ethers::types::{CallConfig, GethDebugTracerConfig, GethDebugTracerType};
use ethers_providers::{Http, Middleware};
use rayon::vec;
use std::str::FromStr;

pub async fn exec(
    _rpc: &'static str,
    _tx_hash: &'static str,
    _address: &'static str,
    _function_selector: &'static str,
    _index: u8,
    _path_strategy: Option<PathStrategy>, // 默认为全路径匹配策略
    _param_strategy: Option<ParamStrategy>,
) -> Result<Vec<Vec<u8>>, ProviderError> {
    match _path_strategy.unwrap() {
        PathStrategy::ControlFlowMatch => {}
        PathStrategy::FullPathMatch => match _param_strategy.unwrap() {
            ParamStrategy::FullParamEnumeration => {
                // 得到范围
                let _param_range = get_range_temp(_address, _function_selector, _index);

                // 创建calldata
                let _ = exec_internal(
                    &_rpc,
                    _tx_hash,
                    vec![0],
                    _param_range,
                    Some(vec![""]),
                    Some(false),
                )
                .await;
            }
        },
    }
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
    _multi_thread: Option<bool>,
) -> Result<Vec<Vec<u8>>, ProviderError> {
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
    // if _multi_thread.is_some().eq(&true) {
    //     println!("here");
    //     // todo! 需要添加多线程的实现
    // } else {
    for new_param in _all_possible_param_list {
        println!("new_param {:?}", new_param);
        // Interpret the bytecode
        let ret = interpreter.interpret_init(bytecode.clone(), new_param.clone(), true);
        if ret.is_ok() {
            println!("{:?}", interpreter.op_list.len());
            return_param_list.push(new_param.clone());
            println!("find new param ");
        }
    }
    // }

    Ok(return_param_list)
}

// 依照call的思路重新封装
async fn sym_exec(
    _rpc: &'static str,
    _tx_hash: &'static str,
    _target_address: &str,
    _selector: &[u8],
    _index: u8,
) {
    // 需要一个provider来找到对应的calldata
    let mut runner = get_evm_interpreter(_rpc, _tx_hash).await.unwrap();
    // 得到第一次call target_address的calldata

    // 得到指定位置的参数
    let mut _calldata = runner.calldata.clone();
    let selector = _calldata.heap.get(0..4).unwrap();
    let index = 0;
    let index_param = _calldata.heap.get(index * 32 + 4..index * 32 + 36).unwrap();
    let param_range = get_range_temp("", "", 1);
    // 构造一个calldata数组
    for item in param_range.iter() {
        // 替换calldata
        _calldata
            .heap
            .splice(index * 32 + 4..=index * 32 + 36, pad_left(&item.as_slice()));
        println!("替换后的calldata {:?}", _calldata.heap);

        // 执行call
        runner.call(
            runner.address,
            [0; 32],
            _calldata.heap.as_slice().to_vec(),
            0,
            false,
        );
    }
}

async fn get_call_data(
    _rpc: &'static str,
    _tx_hash: &'static str,
    _target_address: &str,
    _selector: &[u8],
) {
    // 获得provider
    let mut provider = Provider::<Http>::try_connect(_rpc).await.unwrap();
    // 得到calltrace
    let calltrace = provider
        .debug_trace_transaction(
            TxHash::from_str(_tx_hash).unwrap(),
            GethDebugTracingOptions {
                disable_stack: Some(true),
                disable_storage: Some(true),
                tracer: Some(GethDebugTracerType::BuiltInTracer(
                    ethers::types::GethDebugBuiltInTracerType::CallTracer,
                )),
                enable_memory: Some(false),
                enable_return_data: Some(false),
                tracer_config: None,
                timeout: Some(String::from_str("100").unwrap()),
            },
        )
        .await
        .unwrap();
    println!("{:?}", calltrace);
}

#[tokio::test]
async fn test11() {
    let rpc = "https://lb.nodies.app/v1/181a5ebf4c954f8496ae7cbc1ac8d03b";
    let tx_hash = "0xe915fc3045fa9922d402c3e9e62e0d7145a3a6a49400b3dad91c86d054e45561";
    let address = "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD";
    let function_selector = "0x3593564c";
    let index = 0;
    let path_strategy = PathStrategy::FullPathMatch;
    let param_strategy = ParamStrategy::FullParamEnumeration;
    let result = exec(
        rpc,
        tx_hash,
        address,
        function_selector,
        index,
        Some(path_strategy),
        Some(param_strategy),
    )
    .await;
}
