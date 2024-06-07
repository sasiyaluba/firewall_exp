//接口
use crate::errors::ExecutionError;
use crate::Runner;
use ethers_providers::{Http, Provider, ProviderExt};
//第一步：根据txhash和rpc，将状态设置到EVM中
use crate::bytes::{_hex_string_to_bytes, to_h256};
use crate::core_module::context::account_state_ex_context::{
    get_accounts_state_tx, get_tx_after_accounts_state, ISDiff,
};
use crate::paper::strategy::param_strategy::get_range_temp;

use crate::core_module::context::evm_context::EvmContext;
use crate::core_module::context::transaction_context::get_transaction_content;
use crate::EvmState;
use dotenv::dotenv;
use ethers::prelude::{ProviderError, TxHash};
use hex::FromHex;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use crate::paper::tx_origin_data::get_origin_oplist::{
    self, compare_list, get_opcode_list, get_opcode_list_str, get_pc_op,
};

pub async fn get_evm_interpreter(
    rpc: &str,
    tx_hash: &'static str,
    _target_address: &'static str,
    _target_index: u8,
    _new_param: Vec<u8>,
) -> Result<Runner, ProviderError> {
    // 1. set provider
    let provider = Provider::<Http>::try_connect(rpc)
        .await
        .expect("rpc connect error");
    // 2. Obtain the pre_transaction_account_state, 需要把这个状态改为post的状态
    let accounts_before_tx = get_accounts_state_tx(
        Arc::from(provider.clone()),
        to_h256(tx_hash),
        ISDiff::default(),
    )
    .await;
    // let accounts_state_post_tx =
    //     get_tx_after_accounts_state(Arc::new(provider.clone()), to_h256(tx_hash)).await;

    // 3. Obtain the transaction context
    let transaction_content = get_transaction_content(provider, TxHash::from_str(tx_hash).unwrap())
        .await
        .expect("get transaction hash error");

    let state: EvmState;
    state = EvmState::new(None);

    // 4. Set the transaction context for the virtual machine
    let caller = transaction_content.from;
    let origin = transaction_content.from;
    let address = transaction_content.to.unwrap();
    let value = transaction_content.value;
    let data = transaction_content.calldata.heap;

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
        Some(<[u8; 20]>::from_hex(_target_address).expect("invaild address")),
        Some(_target_index),
        Some(_new_param),
    );

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
    _rpc: &'static str,
    _tx_hash: &'static str,
    _target_address: &'static str,
    _index: u8,
) -> Result<(), ExecutionError> {
    // 首先得到值范围
    // let _param_range = get_range_temp("", "", 0);
    let _param_range = vec![[0, 12].to_vec()];
    let origin_list = get_opcode_list_str(_rpc, _tx_hash).await;
    println!("{:?}", _param_range);
    // 这里执行
    for new_param in _param_range {
        let mut runner = get_evm_interpreter(_rpc, _tx_hash, _target_address, _index, new_param)
            .await
            .unwrap();
        let result = runner.interpret(runner.bytecode.clone(), false);
        assert!(result.is_ok());

        // 10. 获取执行结果
        let _op_list = runner.op_list.clone();
        compare_list(origin_list.clone(), _op_list);
        // 计算
    }
    Ok(())
}

#[tokio::test]
async fn test_sym_exec() {
    let rpc = "https://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/";
    let tx_hash = "0x0d51c6fbc9182bf90bcb1f24323bf18aebcce02521023789ce8e58a23a2c6ada";
    let target_address = "21fa049cecb7e9a771a4712ca05ca0501a88a48d";
    let target_index = 0;
    let _ = sym_exec(&rpc, &tx_hash, &target_address, target_index).await;
}
