use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use ethers::prelude::{Block, Transaction};
use ethers_providers::{Provider, Ws};
use crate::paper::my_filed::Handler::ProtectInfoCache;

#[derive(Debug)]
// 测试模块可行
pub struct Test{
    rpc: &'static str,
    sql_url: &'static str,
    rpc_connect: Arc<Provider<Ws>>,
    sql_connect: mysql::Pool,
    protect_addresses: Vec<String>,
    protect_infos: HashMap<String, ProtectInfoCache>,
    // 区块信息队列
    block_info:VecDeque<Block<Transaction>>
}