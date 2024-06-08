use chrono::Local;
use ethers::prelude::*;
use std::sync::Arc;
use std::{convert::TryFrom, str::FromStr};
use tokio::time::{sleep, Duration};

use super::sym_exec::{self, sym_exec};

async fn get_storage_at(
    from: &str,
    location: H256,
    block: Option<BlockId>,
) -> Result<H256, Box<dyn std::error::Error>> {
    // 获取ws
    let provider =
        Provider::<Http>::try_connect("https://lb.nodies.app/v1/181a5ebf4c954f8496ae7cbc1ac8d03b")
            .await
            .expect("could not connect to infura");

    let slot_value = provider
        .get_storage_at(from, location, block)
        .await
        .unwrap();
    Ok(slot_value)
}

// 监控
pub async fn listening_storage(
    rpc: &'static str,
    from: &'static str,
    location: Vec<H256>,
    block: Option<BlockId>,
    exp: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // 连接到以太坊节点
    let provider = Provider::<Ws>::connect(rpc).await?;
    let provider = Arc::new(provider);

    // 需要得到要监控的合约地址，slot以及不变量表达式

    // 订阅区块
    let mut block_stream = provider.subscribe_blocks().await?;

    // 监听区块
    while let Some(block) = block_stream.next().await {
        // 得到每个slot的值
        let mut values: Vec<H256> = vec![];
        for local in &location {
            let slot_value = get_storage_at(from, *local, None).await?;
            values.push(slot_value);
        }
        // 计算表达式的值
        let result = caculate_expression_value(exp.clone(), values.clone());
        if result {
            // 得到当前与项目合约交互的交易hash
            let now_block = provider
                .get_block_with_txs(block.number.unwrap())
                .await?
                .unwrap();
            let all_tx = now_block.transactions;
            // 进行过滤，找到与from交互的交易
            let interact_tx: Vec<Transaction> = all_tx
                .into_iter()
                .filter(|tx| tx.to == Some(H160::from_str(from).unwrap()))
                .collect();

            // todo!到底执行哪笔交易？
            for tx in interact_tx {
                // 符号执行
                let kill_range = sym_exec(&rpc, &tx.hash().to_string(), from, 0)
                    .await
                    .unwrap();
                // todo! 最终需要将kill_range中的范围添加到防护模块中
            }
        }
    }
    Ok(())
}

// todo!表达式的解析，计算
pub fn caculate_expression_value(exp: String, values: Vec<H256>) -> bool {
    // 计算表达式的值
    true
}
