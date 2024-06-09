use abi::AbiEncode;
use chrono::Local;
use ethers::prelude::*;
use std::sync::Arc;
use std::{convert::TryFrom, str::FromStr};
use tokio::time::{sleep, Duration};

use super::sym_exec::{self, sym_exec};

// 监控
pub async fn listening_storage(
    rpc: &str,
    from: &str,
    location: Vec<H256>,
    block: Option<BlockId>,
    exp: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // 连接到以太坊节点
    let provider = Provider::<Ws>::connect(&rpc).await?;
    let provider = Arc::new(provider);
    println!("Connected to the Ethereum network");
    // 需要得到要监控的合约地址，slot以及不变量表达式

    // 订阅区块
    let mut block_stream = provider.subscribe_blocks().await?;

    // 监听区块
    while let Some(block) = block_stream.next().await {
        println!("new block number {:?}", block.number.unwrap());
        // 得到每个slot的值
        let mut values: Vec<H256> = vec![];
        for local in &location {
            let slot_value = provider.get_storage_at(from, *local, None).await?;
            values.push(slot_value);
        }
        println!("values: {:?}", values);
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
                .filter(|tx| tx.to == Some(H160::from_str(&from).unwrap()))
                .collect();
            // todo!到底执行哪笔交易？
            for tx in interact_tx {
                println!("interact tx hash: {:?}", tx.hash.encode_hex());
                // 符号执行
                let kill_range = sym_exec(&rpc, &tx.hash.encode_hex()[2..], &from, 0)
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

#[tokio::test]
async fn test1() {
    let rpc = "wss://wiser-stylish-isle.quiknode.pro/fe971117365d555490242e38972893351f3bcd6a/";
    let http_rpc = "https://lb.nodies.app/v1/181a5ebf4c954f8496ae7cbc1ac8d03b";
    // 连接到以太坊节点
    let provider = Provider::<Ws>::connect(&rpc)
        .await
        .expect("can't connect to the network");
    let provider = Arc::new(provider);
    println!("Connected to the Ethereum network");
    // 需要得到要监控的合约地址，slot以及不变量表达式

    // 订阅区块
    let mut block_stream = provider
        .subscribe_blocks()
        .await
        .expect("can't subscribe block");

    // 监听区块
    while let Some(block) = block_stream.next().await {
        println!("new block number {:?}", block.number.unwrap());

        // 得到当前与项目合约交互的交易hash
        let now_block = provider
            .get_block_with_txs(block.number.unwrap())
            .await
            .expect("can't get block")
            .unwrap();
        let all_tx = now_block.transactions;
        let first_tx = all_tx.first().unwrap();
        let tx_hash = first_tx.hash.encode_hex();
        println!("interact tx hash: {:?}", tx_hash);
        break;
    }
}
