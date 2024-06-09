use crate::core_module::runner;
//接口
use crate::errors::ExecutionError;
use crate::Runner;
//第一步：根据txhash和rpc，将状态设置到EVM中
use crate::bytes::{_hex_string_to_bytes, pad_left, to_h256};
use crate::core_module::context::account_state_ex_context::{
    get_accounts_state_tx, get_tx_after_accounts_state, ISDiff,
};
use crate::core_module::context::evm_context::EvmContext;
use crate::core_module::context::transaction_context::get_transaction_content;
use crate::paper::strategy::param_strategy::get_range_temp;
use crate::EvmState;
use dotenv::dotenv;
use ethers::prelude::{Http, Provider, ProviderError, ProviderExt, TxHash, Ws};
use ethers::types::H256;
use hex::FromHex;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use crate::paper::tx_origin_data::get_origin_oplist::{
    self, compare_list, get_opcode_list, get_opcode_list_str, get_pc_op,
};

pub async fn get_evm_interpreter(
    rpc: &str,
    tx_hash: &str,
    _target_address: &str,
    _target_index: u8,
    _new_param: Vec<u8>,
) -> Result<Runner, ProviderError> {
    // 1. set provider
    let provider = Provider::<Ws>::connect(rpc)
        .await
        .expect("rpc connect error");
    // 2. Obtain the pre_transaction_account_state, 需要把这个状态改为post的状态
    let accounts_before_tx = get_accounts_state_tx(
        Arc::from(provider.clone()),
        H256::from_str(tx_hash).unwrap(),
        ISDiff::default(),
    )
    .await;
    // let accounts_state_post_tx =
    //     get_tx_after_accounts_state(Arc::new(provider.clone()), to_h256(tx_hash)).await;

    // 3. Obtain the transaction context
    let mut transaction_content =
        get_transaction_content(provider, TxHash::from_str(tx_hash).unwrap())
            .await
            .expect("get transaction hash error");

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
        transaction_content.calldata.heap.splice(
            4 + _target_index as usize * 32..36 + _target_index as usize * 32,
            pad_left(_new_param.clone().as_slice()),
        );
        _simulate = true;
        transaction_content.calldata.heap
    } else {
        transaction_content.calldata.heap
    };
    println!("data {:?}", &data);
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

    // 7. set evm state NULL env
    let mut evm_context = EvmContext::new(); // Adjust this based on your actual implementation

    // 8. update evm state env
    evm_context.gas_price = transaction_content.gas_price;
    evm_context.block_number = transaction_content.block_number;
    evm_context.basefee = transaction_content.basefee;
    evm_context.coinbase = transaction_content.coinbase;
    evm_context.blockhash = transaction_content.block_hash;
    evm_context.difficulty = transaction_content.difficulty;
    evm_context.timestamp = transaction_content.timestamp;

    interpreter.evm_context = Some(evm_context);
    interpreter.bytecode = interpreter
        .state
        .get_code_at(interpreter.address)
        .unwrap()
        .clone();
    // 9. 在更新上下文的时候，是否选择将拉取最新的交易上下文更新到要执行的模块中
    // todo! 拉取最新的区块链状态进行更新
    Ok(interpreter)
}

pub async fn sym_exec(
    _rpc: &str,
    _tx_hash: &str,
    _target_address: &str,
    _index: u8,
) -> Result<Vec<Vec<u8>>, ExecutionError> {
    let mut kill_range: Vec<Vec<u8>> = vec![];
    // todo! 需要得到值的范围
    // ?下面只是假设。。。
    let _param_range = vec![[0, 12].to_vec()];
    let origin_list = get_opcode_list_str(_rpc, _tx_hash).await;
    // 这里执行
    for new_param in _param_range {
        let mut runner =
            get_evm_interpreter(_rpc, _tx_hash, _target_address, _index, new_param.clone())
                .await
                .unwrap();
        let _ = runner.interpret(runner.bytecode.clone(), false);

        // 计算与原始的相似度
        let _op_list = runner.op_list.clone();
        let result = compare_list(origin_list.clone(), _op_list);
        // todo!相似率大于多少，就添加到kill_range中
        if result > 0.9 {
            kill_range.push(new_param.clone());
        }
    }
    Ok(kill_range)
}

// #[tokio::test]
async fn test_sym_exec() {
    let rpc = "https://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/";
    // let rpc = "https://lb.nodies.app/v1/181a5ebf4c954f8496ae7cbc1ac8d03b";
    let tx_hash = "0x0d51c6fbc9182bf90bcb1f24323bf18aebcce02521023789ce8e58a23a2c6ada";
    let target_address = "9e3b917755889b27266d5483e001754e1be4fc5c";
    let target_index = 0;
    let _ = sym_exec(&rpc, &tx_hash, &target_address, target_index).await;
}
