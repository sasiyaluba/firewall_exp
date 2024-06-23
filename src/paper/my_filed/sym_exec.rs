use crate::bytes::pad_left;
use crate::core_module::context::account_state_ex_context::{
    get_accounts_state_tx, AccountStateEx, ISDiff,
};
use crate::core_module::context::evm_context::EvmContext;
use crate::core_module::context::transaction_context::{get_transaction_content, TransactionEnv};
use crate::errors::ExecutionError;
use crate::paper::tx_origin_data::get_origin_oplist::compare_list;
use crate::EvmState;
use crate::Runner;
use ethers::prelude::{Provider, ProviderError, TxHash, Ws};
use ethers::types::H256;
use hex::FromHex;
use primitive_types::H160;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get_evm_interpreter(
    rpc: &str,
    tx_hash: &str,
    _target_address: &str,
    _target_index: u8,
    _new_param: Vec<u8>,
    evm_context: Option<EvmContext>,
    accounts_before_tx: Option<BTreeMap<H160, AccountStateEx>>,
    transaction_content: Option<TransactionEnv>,
) -> Result<Runner, ProviderError> {
    let accounts_before_tx: BTreeMap<H160, AccountStateEx> = accounts_before_tx.unwrap();
    let transaction_content = transaction_content.unwrap();
    let state: EvmState;
    state = EvmState::new(None);

    // 4. Set the transaction context for the virtual machine
    let caller = transaction_content.from;
    let origin = transaction_content.from;
    let address = transaction_content.to.unwrap();
    let value = transaction_content.value;
    let mut _simulate = false;

    let data = if address.eq(&<[u8; 20]>::from_hex(&_target_address[2..]).unwrap())
        && _target_index != 255
    {
        // 直接在此处更换参数
        let start = 4 + _target_index as usize * 32;
        let end = start + 32;
        let mut calldata = transaction_content.calldata.heap.clone();
        calldata.splice(start..end, pad_left(_new_param.clone().as_slice()));
        _simulate = true;
        // println!("calldata {:?}", &calldata);
        calldata
    } else {
        transaction_content.calldata.heap
    };

    // 5. Create a new interpreter
    let mut interpreter = Runner::new_paper(
        caller,
        Some(origin),
        Some(address),
        Some(value),
        Some(data),
        Some(state),
        None,
        None,
        Some(<[u8; 20]>::from_hex(&_target_address[2..]).expect("invaild address")),
        Some(_target_index),
        Some(_new_param),
    );
    if _simulate {
        interpreter.exchange_flag = true;
    }
    // 6. insert account_state to evm
    accounts_before_tx
        .iter()
        .for_each(|(_addr, _account_state_ex)| {
            interpreter.modify_account_state(_addr.0, _account_state_ex.clone());
        });

    interpreter.evm_context = evm_context;
    interpreter.bytecode = interpreter
        .state
        .get_code_at(interpreter.address)
        .unwrap()
        .clone();
    Ok(interpreter)
}

pub async fn sym_exec(
    _rpc: &str,
    _tx_hash: &str,
    _target_address: &str,
    _index: u8,
    _min_value: u128,
    _max_value: u128,
) -> Result<Vec<u128>, ExecutionError> {
    // 记录时间
    let start = std::time::Instant::now();
    // 参数范围
    let mut kill_range: Vec<u128> = vec![];
    // 得到的参数范围
    let _param_range: Vec<u128> = (_min_value..=_max_value).collect();
    // 获取一系列信息
    let (accounts_before_tx, transaction_content, evm_context) =
        get_prepare_info(_rpc, _tx_hash).await;

    let mut runner = get_evm_interpreter(
        _rpc,
        _tx_hash,
        _target_address,
        255,
        vec![0],
        Some(evm_context.clone()),
        Some(accounts_before_tx.clone()),
        Some(transaction_content.clone()),
    )
    .await
    .unwrap();
    let _ = runner.interpret(runner.bytecode.clone(), false);
    let origin_address_pc_op = runner.address_pc_op.clone();
    // 替换执行
    for new_param in _param_range {
        let mut runner = get_evm_interpreter(
            _rpc,
            _tx_hash,
            _target_address,
            _index,
            new_param.clone().to_be_bytes().to_vec(),
            Some(evm_context.clone()),
            Some(accounts_before_tx.clone()),
            Some(transaction_content.clone()),
        )
        .await
        .unwrap();
        let _ = runner.interpret(runner.bytecode.clone(), false);
        let new_address_pc_op = runner.address_pc_op.clone();
        // 计算与原始的相似度
        let result: f64 = compare_list(origin_address_pc_op.clone(), new_address_pc_op.clone());
        println!("参数值为 {:?} 与原攻击相似率为 {:?}", new_param, result);
        if result > 0.95 {
            kill_range.push(new_param.clone());
        }
    }
    println!("kill_range {:?}", kill_range);
    let end = std::time::Instant::now();
    println!("符号执行：{:?}", end - start);
    Ok(kill_range)
}

pub async fn get_prepare_info(
    _rpc: &str,
    _tx_hash: &str,
) -> (BTreeMap<H160, AccountStateEx>, TransactionEnv, EvmContext) {
    // 1. set provider
    let provider = Provider::<Ws>::connect(_rpc)
        .await
        .expect("rpc connect error");
    let accounts_before_tx = get_accounts_state_tx(
        Arc::from(provider.clone()),
        H256::from_str(_tx_hash).unwrap(),
        ISDiff::default(),
    )
    .await;

    let transaction_content =
        get_transaction_content(provider, TxHash::from_str(_tx_hash).unwrap())
            .await
            .expect("get transaction hash error");

    let mut evm_context = EvmContext::new(); 

    evm_context.gas_price = transaction_content.gas_price;
    evm_context.block_number = transaction_content.block_number;
    evm_context.basefee = transaction_content.basefee;
    evm_context.coinbase = transaction_content.coinbase;
    evm_context.blockhash = transaction_content.block_hash;
    evm_context.difficulty = transaction_content.difficulty;
    evm_context.timestamp = transaction_content.timestamp;

    (accounts_before_tx, transaction_content, evm_context)
}
