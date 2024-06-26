use super::database_mod::DatabaseManager;
use super::expression::evaluate_exp_with_unknown;
use crate::paper::my_filed::parser::parse_expression;
use ethers::types::Transaction;
use ethers::types::H160;
use ethers::types::H256;
use ethers_providers::Middleware;
use ethers_providers::Provider;
use ethers_providers::StreamExt;
use ethers_providers::Ws;
use primitive_types::U256;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time;
#[derive(Debug, Clone)]
pub struct Block {
    pub number: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug)]
pub struct HandlerTest {
    // Arc,因为在多个线程之中共享，为了安全考虑
    rpc_provider: Arc<Provider<Ws>>,
    // 受保护合约的地址
    protect_addresses: Vec<String>,
    // 区块生产者
    block_sender: Sender<u64>,
    // 区块互斥消费者
    block_receiver: Mutex<Receiver<u64>>,
    // 不变量打破时的生产者
    broke_sender: Sender<(u64, String)>,
    // 不变量打破时的消费者
    broke_receiver: Mutex<Receiver<(u64, String)>>,
    // 信号量：限制并发数量
    semaphore: Arc<Semaphore>,
    database: Arc<DatabaseManager>,
    state_var_cache: Mutex<Vec<(String, U256)>>,
}

impl HandlerTest {
    pub async fn new(
        _rpc: &'static str,
        _sql_url: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let _share_queue = Arc::new(Mutex::new(VecDeque::<Block>::new()));
        let _rpc_provider = Arc::new(
            Provider::<Ws>::connect(_rpc)
                .await
                .expect("func_new:error rpc url"),
        );
        let (_sender, _receiver): (Sender<u64>, Receiver<u64>) = mpsc::channel(100);
        let (_broke_sender, _broke_receiver): (Sender<(u64, String)>, Receiver<(u64, String)>) =
            mpsc::channel(100);
        let cpu = num_cpus::get();
        println!("当前cpu可支持的线程数为：{}", cpu);
        let _semaphore = Arc::new(Semaphore::new(cpu));
        let mut _database = DatabaseManager::new(&_sql_url).expect("func_new:error sql url ");
        // 加载数据库缓存
        let _ = _database.load_data_for_cache();
        Ok(Self {
            rpc_provider: _rpc_provider,
            protect_addresses: _database.protect_addresses.clone(),
            block_sender: _sender,
            block_receiver: Mutex::new(_receiver),
            semaphore: _semaphore,
            database: Arc::new(_database),
            broke_receiver: Mutex::new(_broke_receiver),
            broke_sender: _broke_sender,
            state_var_cache: Mutex::new(Vec::<(String, U256)>::new()),
        })
    }

    pub async fn get_block(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始获取区块信息");
        // 进行监听
        let mut block_stream = self.rpc_provider.subscribe_blocks().await?;
        // 监听区块
        while let Some(block) = block_stream.next().await {
            println!("已出现新的区块 {:?}", block.number.unwrap());
            let _ = self
                .block_sender
                .send(block.number.unwrap().as_u64())
                .await
                .expect("func_get_block:send error");
        }
        Ok(())
    }

    pub async fn check_invariant(self: Arc<Self>) {
        loop {
            let block_number = {
                let mut receiver = self.block_receiver.lock().await;
                receiver.recv().await
            };
            // 开启线程进行处理
            if let Some(block_number) = block_number {
                for address in self.protect_addresses.clone().into_iter() {
                    let self_clone = self.clone();
                    // 消耗一个线程
                    let permit = self_clone.semaphore.clone().acquire_owned().await.unwrap();
                    println!(
                        "剩余可用线程数：{:?}",
                        self_clone.semaphore.available_permits()
                    );
                    // 检查不变量
                    tokio::spawn(async move {
                        let start = time::Instant::now();
                        // 首先得到计算前的不变量
                        let invar = self_clone
                            .get_invariant(&address)
                            .await
                            .expect("handle invariant error");
                        println!("待计算表达式为：{:?}", invar);
                        // 将表达式根据&&分割开
                        let _expressions: Vec<&str> = invar.split("&&").collect();
                        for exp in _expressions {
                            // 一旦某个式子解析结果为false，则触发符号执行线程
                            if parse_expression(exp, None) == 0 {
                                // 发送错误信息
                                let _ = self_clone
                                    .broke_sender
                                    .send((block_number, address.clone()))
                                    .await
                                    .expect("broke sender send mes error");
                            }
                        }
                        let end = time::Instant::now();
                        println!("time {:?}", end - start);
                        drop(permit); // 任务完成后释放 permit
                    });
                }
            }
        }
    }

    pub async fn get_invariant(
        &self,
        _address: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 根据address读取信息
        let data = self
            .database
            .protect_infos
            .get(_address)
            .expect("地址不存在！");
        // 读取不变量
        let invar = data.invariant.clone();
        // 读取所有变量
        let vars = data.variables.clone();
        let slot_map = data.slot_map.clone();
        // 创建一个新的表达式
        let mut values: Vec<(String, U256)> = vec![];
        // 遍历所有变量名
        for name in vars.iter() {
            // 获取slot
            if let Some(slot_str) = slot_map.get(name) {
                // 如果是balance
                if name == "address(this).balance" {
                    let balance = self.rpc_provider.get_balance(_address, None).await?;
                    values.push((name.to_string(), balance));
                } else {
                    // 根据slot获取值
                    let slot = H256::from_str(slot_str)?;
                    let _value = self
                        .rpc_provider
                        .get_storage_at(_address, slot, None)
                        .await?;

                    let value = U256::from_big_endian(_value.as_bytes());
                    values.push((name.to_string(), value));
                }
            } else {
                return Err(format!("该变量对应的slot未找到: {}", name).into());
            }
        }
        // 集中进行值替换
        let mut new_expression = invar.to_string();
        for (name, value) in values.iter() {
            new_expression = new_expression.replace(name, &value.to_string());
        }
        // 做值缓存
        let mut data_cache = self.state_var_cache.lock().await;
        *data_cache = values;
        Ok(new_expression)
    }

    pub async fn sysm_exec(self: Arc<Self>) {
        loop {
            // 接收到不变量被打破
            let _block_address = {
                let mut receiver = self.broke_receiver.lock().await;
                receiver.recv().await
            };
            let self_clone1 = self.clone();
            let self_clone2 = self.clone();
            // 消费一个线程
            let permit = self_clone1.semaphore.clone().acquire_owned().await.unwrap();
            println!(
                "剩余可用线程数：{:?}",
                self_clone1.semaphore.available_permits()
            );
            // 开启符号执行线程
            tokio::spawn(async move {
                if let Some(block_address) = _block_address {
                    let block_number = block_address.0;
                    let address = block_address.1;
                    // 根据block得到与to地址为address的交易
                    let now_block = self_clone1
                        .rpc_provider
                        .get_block_with_txs(block_number)
                        .await
                        .expect("can't get block")
                        .unwrap();
                    let all_tx: Vec<Transaction> = now_block.transactions;
                    let interact_tx: Vec<Transaction> = all_tx
                        .into_iter()
                        .filter(|tx| tx.to == Some(H160::from_str(&address).unwrap()))
                        .collect();
                    if interact_tx.len() > 0 {
                        println!("出现恶意交易");
                        let permit = self_clone1.semaphore.clone().acquire_owned().await.unwrap();
                        println!(
                            "剩余可用线程数：{:?}",
                            self_clone1.semaphore.available_permits()
                        );
                        tokio::spawn(async move {
                            for tx in interact_tx {
                                // 获得selector
                                let selector = get_selector(&tx.input[..4]);
                                println!("selector {:?}", selector);
                                // 获取selector对应的expression
                                let pi = self_clone2.database.protect_infos.get(&address).unwrap();
                                // 得到值缓存
                                let state_var_cache =
                                    self_clone2.state_var_cache.lock().await.clone();
                                for (index, expression) in
                                    pi.expression_map.get(&selector).unwrap().iter()
                                {
                                    // 替换值
                                    let mut new_expression = expression.to_string();
                                    for (name, value) in state_var_cache.iter() {
                                        new_expression =
                                            new_expression.replace(name, &value.to_string());
                                    }

                                    // 解方程得到范围
                                    let (min, max) =
                                        evaluate_exp_with_unknown(&new_expression).unwrap();
                                    // todo 符号执行
                                    // todo 发送交易
                                    // todo 更新不变量，保证程序能够正常执行
                                }
                            }
                            drop(permit);
                        });
                    }
                }
                drop(permit);
            });
        }
    }
}

pub fn get_selector(bytes: &[u8]) -> String {
    // 将字节数组转换为十六进制字符串（小写）
    let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

    // 加上 '0x' 前缀
    let hex_with_prefix = format!("0x{}", hex_string);
    hex_with_prefix
}
