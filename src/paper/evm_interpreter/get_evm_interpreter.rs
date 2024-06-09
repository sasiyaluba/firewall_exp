use crate::bytes::to_h256;
use crate::core_module::context::account_state_ex_context::{
    get_accounts_state_tx, get_tx_after_accounts_state, ISDiff,
};
use crate::core_module::context::evm_context::EvmContext;
use crate::core_module::context::transaction_context::get_transaction_content;
use crate::{EvmState, Runner};
use dotenv::dotenv;
use ethers::prelude::{Provider, ProviderError, ProviderExt, TxHash, Ws};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

pub async fn get_evm_interpreter(
    rpc: &str,
    tx_hash: &'static str,
) -> Result<Runner, ProviderError> {
    // 1. set provider
    let provider = Provider::<Ws>::connect(rpc)
        .await
        .expect("rpc connect error");
    // 2. Obtain the pre_transaction_account_state, 需要把这个状态改为post的状态
    let accounts_state_post_tx =
        get_tx_after_accounts_state(Arc::new(provider.clone()), to_h256(tx_hash)).await;

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
        None,
        None,
        None,
    );

    // 6. insert account_state to evm
    accounts_state_post_tx
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
