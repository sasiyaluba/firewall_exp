use crate::bytes::pad_left;
use crate::debug::to_hex_string;
use crate::paper::my_filed::expression::{evaluate_exp, evaluate_exp_with_unknown, find_max_min};
use crate::paper::my_filed::parser;
use crate::paper::my_filed::parser::parse_expression;
use crate::paper::my_filed::sym_exec::sym_exec;
use abi::Hash;
use ansi_term::Colour::{Black, Blue, Cyan, Fixed, Green, Purple, Red, White, Yellow};
use binlog::value;
use ethers::abi::AbiEncode;
use ethers::prelude::*;
use ethers::types::{Block, Transaction};
use mysql::prelude::Queryable;
use mysql::*;
use num_traits::ToBytes;
use regex::bytes::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io::Read;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
#[derive(Debug)]
pub struct Handler {
    rpc: &'static str,
    sql_url: &'static str,
    rpc_connect: Arc<Provider<Ws>>,
    sql_connect: mysql::Pool,
    protect_addresses: Vec<String>,
    protect_infos: HashMap<String, ProtectInfoCache>,
}
#[derive(Debug)]
pub struct ProtectInfoCache {
    // 地址
    address: String,
    address_id: i32,
    // 不变量表达式
    invariant_expression: String,
    invariant_expression_id: i32,
    // 不变量表达式涉及的变量名
    state_variables: Vec<String>,
    // 不变量表达式涉及的slot，address => variable => slot
    slots: HashMap<i32, HashMap<String, String>>,
    // 保护的函数选择器列表
    selectors: Vec<String>,
    // 函数选择器 => index
    function_selectors: HashMap<String, u8>,
    // index => expression
    function_expressions: HashMap<u8, String>,
}

impl Handler {
    pub fn insert_data_for_address(
        &mut self,
        _addresses: Vec<String>,
        _selectors: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取sql连接
        let mut connect = self.sql_connect.get_conn().unwrap();

        for _address in _addresses.into_iter() {
            let mut protect_info_cache = ProtectInfoCache::new();

            // 插入address以及address_id
            let address_id: i32 = connect
                .exec_first(
                    "SELECT id FROM addresses WHERE address =:address",
                    params! {
                        "address" => _address.as_str(),
                    },
                )?
                .ok_or("Address not found")?;
            protect_info_cache.insert_address(_address.clone(), address_id);

            // 插入不变量以及expression_id
            let expression: String = connect
                .exec_first(
                    "SELECT expression FROM expressions WHERE address_id =:address_id",
                    params! {
                        "address_id" => address_id,
                    },
                )?
                .ok_or("Address not found")?;
            // expression_id
            let expression_id: i32 = connect
                .exec_first(
                    "SELECT id FROM expressions WHERE address_id =:address_id",
                    params! {
                        "address_id" => address_id,
                    },
                )?
                .ok_or("Address not found")?;
            protect_info_cache.insert_invariant_expression(expression.clone(), expression_id);

            // 插入相关变量名
            let re = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
            let mut variables: Vec<String> = re
                .find_iter(expression.as_bytes())
                .map(|mat| String::from_utf8(mat.as_bytes().to_vec()).unwrap())
                .collect();
            // variables 去重
            let unique_vec: Vec<String> = vec_remove_duplicates(&mut variables);
            protect_info_cache.insert_more_state_variables(unique_vec.clone());
            // slots
            for name in unique_vec.into_iter() {
                if name.eq("address(this).balance") {
                    protect_info_cache.insert_slot(
                        address_id,
                        "address(this).balance".to_string(),
                        "address(this).balance".to_string(),
                    );
                    continue;
                }
                let _slot: String = connect
                    .exec_first(
                        "SELECT slot FROM variables WHERE variable_name = :variable_name AND expression_id = :expression_id",
                        params! {
                            "variable_name" => name.as_str(),
                            "expression_id" => expression_id.to_string()
                        },
                    )?
                    .ok_or("Variable name not found")?;
                protect_info_cache.insert_slot(address_id, name, _slot);
            }

            //selectors
            for selector in _selectors.iter() {
                let index:u8 = connect.exec_first(
                    "SELECT param_index FROM function_expressions WHERE address_id =:address_id AND function_selector =:function_selector",
                    params! {
                        "address_id" => address_id.to_string(),
                        "function_selector" => selector.as_str()
                    },)?.ok_or("Address not found or funcSelector not found")?;
                let param_expression:String = connect
                    .exec_first(
                        "SELECT expression FROM function_expressions WHERE address_id =:address_id AND function_selector =:function_selector",
                        params! {
                            "address_id" => address_id.to_string(),
                            "function_selector" => selector.as_str()
                        },
                    )?
                    .ok_or("Address not found")?;
                protect_info_cache.selectors.push(selector.clone());
                protect_info_cache
                    .function_expressions
                    .insert(index, param_expression);
                protect_info_cache
                    .function_selectors
                    .insert(selector.clone(), index);
            }
            protect_info_cache.print_all();
            self.protect_infos.insert(_address, protect_info_cache);
        }
        Ok(())
    }

    pub async fn new(
        _rpc: &'static str,
        _sql_url: &'static str,
        _addresses: Vec<String>,
        _selectors: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Ws>::connect(_rpc).await?;
        println!();
        println!("{}", Yellow.paint("---成功连接rpc节点"));
        let rpc_connect = Arc::new(provider);
        let sql_connect = mysql::Pool::new(_sql_url)?;
        println!("{}", Yellow.paint("---成功连接数据库"));
        println!();
        let mut instance = Self {
            rpc: _rpc,
            sql_url: _sql_url,
            rpc_connect,
            sql_connect,
            protect_addresses: _addresses.clone(),
            protect_infos: HashMap::new(),
        };
        let _ = instance.insert_data_for_address(_addresses, _selectors);
        println!("{}", Yellow.paint("---成功初始化数据库缓存"));
        println!();
        Ok(instance)
    }

    // 获取范围
    pub async fn get_range(
        &self,
        _address: &str,
        _function_selector: String,
    ) -> Result<(u128, u128, u8), Box<dyn std::error::Error>> {
        // 获得index
        let index = self
            .protect_infos
            .get(_address)
            .unwrap()
            .get_index_with_selector(&_function_selector)
            .unwrap();
        // 获得函数参数相关的表达式
        let function_expression = self
            .protect_infos
            .get(_address)
            .unwrap()
            .get_function_expression(index)
            .unwrap();
        let (min, max) = self
            .caculate_range(_address, function_expression.to_string())
            .await;
        println!();
        println!("经过z3求解后，得出的初步范围为：{} ~ {}", min, max);
        println!();
        Ok((min, max, *index))
    }

    // 检查不变量是否异常
    pub async fn check_invariant(
        &self,
        _address: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // 获得表达式
        let expression = self.get_expression(_address);
        println!("-----------当前不变量 {}", Blue.paint(&expression.clone()));
        println!();
        // 处理表达式
        let result = self.handle_exp(_address, expression).await;
        Ok(result)
    }

    // 获取表达式
    pub fn get_expression(&self, _address: &str) -> String {
        let protect_info = self.protect_infos.get(_address).unwrap();
        protect_info.invariant_expression.clone()
    }

    // 根据变量名获取值
    pub async fn get_values_with_names(
        &self,
        _address: &str,
        _state_names: Vec<String>,
    ) -> Result<Vec<U256>, Box<dyn std::error::Error>> {
        // 读取缓存
        let _protect_info = self.protect_infos.get(_address).unwrap();
        let mut values: Vec<U256> = Vec::new();
        // 遍历所有变量名
        for name in _protect_info.state_variables.iter() {
            // 获取slot
            let _slot = _protect_info
                .get_slot(_protect_info.address_id, name.clone())
                .unwrap();
            // 如果是balance
            if name.eq("address(this).balance") {
                let balance = self.rpc_connect.get_balance(_address, None).await?;
                values.push(balance);
                continue;
            }
            // 根据slot获取值
            let slot = H256::from_str(&_slot)?;
            let value = self
                .rpc_connect
                .get_storage_at(_address, slot, None)
                .await?;
            values.push(U256::from_big_endian(value.as_bytes()));
        }

        Ok(values)
    }

    // 处理表达式（不变量表达式）
    pub async fn handle_exp(&self, _address: &str, _expression: String) -> bool {
        // 将表达式分割，以&&为分隔符
        let _expression = _expression.replace(" ", "");
        let _expressions: Vec<&str> = _expression.split("&&").collect();
        let mut result = true;
        // 分别处理每个表达式
        for _expression in _expressions {
            // 获得所有变量
            let re = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
            let variables: Vec<String> = re
                .find_iter(_expression.as_bytes())
                .map(|mat| String::from_utf8(mat.as_bytes().to_vec()).unwrap())
                .collect();
            // 获得所有变量的值
            let mut _values = self
                .get_values_with_names(_address, variables.clone())
                .await
                .unwrap();
            // println!("表达式 {:?}", _expression);
            // 将原表达式中的每个变量替换为对应的值
            let mut new_expression = _expression.to_string();
            for i in 0..variables.len() {
                new_expression =
                    new_expression.replace(variables[i].as_str(), _values[i].to_string().as_str());
            }
            // println!("替换后，表达式为：{:?}", new_expression);
            // 计算表达式的值
            result = result && parse_expression(new_expression.as_str());
        }
        result
    }

    // 主要函数，入口
    pub async fn handle(self: &Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        // 进行监听
        let mut block_stream = self.rpc_connect.subscribe_blocks().await?;
        // 监听区块
        while let Some(block) = block_stream.next().await {
            println!(
                "-----当前区块为 {}",
                Yellow.paint(&block.number.unwrap().to_string())
            );
            // todo 在这里先检查不变量
            // 记录不变量检测的开始时间
            let start = std::time::Instant::now();
            for address in self.protect_addresses.iter() {
                let check_result = self.check_invariant(address.as_str()).await?;
                if !check_result {
                    // todo 接下来的一系列动作
                }
            }
            let end = std::time::Instant::now();
            println!(
                "检查不变量的时间为：{}",
                Yellow.paint(&format!("{:?}", end - start))
            );
            // 创建一个共享引用
            let self_clone: Arc<Handler> = Arc::clone(self);
            tokio::spawn(async move {
                println!("check thread: {:?})", thread::current().id());
                self_clone.block_check(block).await.unwrap();
            });
        }
        Ok(())
    }

    // 区块检查
    pub async fn block_check(
        &self,
        _now_block: Block<H256>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for address in self.protect_addresses.iter() {
            let result = self.check_invariant(address.as_str()).await?;
            if !result {
                println!("{}", Red.paint("Invariant is broken!"));
                println!();

                // todo 进行过滤，找到与from交互的交易
                let now_block = self
                    .rpc_connect
                    .get_block_with_txs(_now_block.number.unwrap())
                    .await
                    .expect("can't get block")
                    .unwrap();
                let all_tx: Vec<Transaction> = now_block.transactions;
                let interact_tx: Vec<Transaction> = all_tx
                    .into_iter()
                    .filter(|tx| tx.to == Some(H160::from_str(address).unwrap()))
                    .collect();

                //todo 这里做符号执行，放到其他线程去做...
                if interact_tx.len() > 0 {
                    println!(
                        "interact tx hash: {}",
                        Red.paint(interact_tx[0].hash.encode_hex())
                    );
                    let selector = get_selector(&interact_tx[0].input[..4]);
                    println!("attacked selector：{} ", Red.paint(selector.as_str()));
                    let (min, max, index) = self.get_range(address, selector).await.unwrap();
                    let kill_range = sym_exec(
                        &self.rpc,
                        interact_tx[0].hash.encode_hex().as_str(),
                        &address,
                        index,
                        min,
                        max,
                    )
                    .await
                    .unwrap();
                    // 得到最值
                    let (max, min) = find_max_min(&kill_range).unwrap();
                    let origin_data = "d133576a000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000001046be12c000000000000000000000000008eaD3c2F184Bf64CDAa428653A17E287aa3addb5000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a029e99f07000000000000000000000000000000000000000000000000000000000000000000000000000000004b00a35eb8cae62337f37fe561d7ff48987a4fed00000000000000000000000000000000000000000000000000000000000000001111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200000000000000000000000000000000000000000000000000000000";
                    let _new_data = origin_data.replace(
                        "1111111111111111111111111111111111111111111111111111111111111111",
                        remove_0x_prefix(min.encode_hex().as_str()),
                    );
                    let new_data = _new_data.replace(
                        "2222222222222222222222222222222222222222222222222222222222222222",
                        remove_0x_prefix(max.encode_hex().as_str()),
                    );
                    // todo发送交易到合约
                    let _ = self
                        .send_tx(
                            "ba457ad011e6f8b3efc7f2b51d3fa7db94c26903f58b6d5da8176d5fbbc7f2e4",
                            "0xdBE9D9fcD06Ab1C82815eEcb9E4b78fD805c84A7",
                            Bytes::from_str(new_data.as_str()).unwrap(),
                        )
                        .await;
                }
            } else {
                println!("{}", Green.paint("Invariant is safe!"));
                println!();
            }
        }
        Ok(())
    }

    // 计算参数范围
    pub async fn caculate_range(&self, _address: &str, _expression: String) -> (u128, u128) {
        let _expression = _expression.replace(" ", "");
        // 首先获得所有状态变量的名称
        let variable_names = self
            .protect_infos
            .get(_address)
            .unwrap()
            .state_variables
            .clone();

        // 获得所有状态变量的值
        let _values = self
            .get_values_with_names(_address, variable_names.clone())
            .await
            .unwrap();

        // 将原表达式中的每个状态变量替换为对应的值，因此得到一个包含未知数和常数的表达式
        let mut new_expression: String = _expression.to_string();
        for i in 0.._values.len() {
            new_expression =
                new_expression.replace(variable_names[i].as_str(), _values[i].to_string().as_str());
        }

        // println!("替换后，表达式为：{:?}", new_expression);
        let (min, max) = evaluate_exp_with_unknown(&new_expression).unwrap();
        (min, max)
    }

    // 发送交易的辅助函数
    pub async fn send_tx(
        &self,
        private_key: &str,
        contract_address: &str,
        data: Bytes,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let wallet: LocalWallet = private_key.parse()?;
        let wallet = wallet.with_chain_id(11155111u64);
        let client: SignerMiddleware<_, _> =
            SignerMiddleware::new(self.rpc_connect.clone(), wallet.clone());
        let client = Arc::new(client);
        let to_address: H160 = contract_address.parse()?;
        let tx = TransactionRequest::new()
            .to(to_address)
            .value(U256::from(0))
            .data(data)
            .from(wallet.address());
        let pending_tx = client.send_transaction(tx, None).await?;
        let receipt = pending_tx.await?;
        if let Some(receipt) = receipt {
            println!(
                "Transaction successful with hash: {:?}",
                receipt.transaction_hash
            );
        } else {
            println!("Transaction not confirmed yet.");
        }

        Ok(())
    }
}

impl ProtectInfoCache {
    pub fn new() -> Self {
        Self {
            address: String::new(),
            address_id: 0,
            invariant_expression: String::new(),
            invariant_expression_id: 0,
            state_variables: Vec::new(),
            slots: HashMap::new(),
            selectors: Vec::new(),
            function_selectors: HashMap::new(),
            function_expressions: HashMap::new(),
        }
    }

    pub fn insert_address(&mut self, _address: String, _address_id: i32) {
        self.address = _address;
        self.address_id = _address_id;
    }

    pub fn insert_invariant_expression(&mut self, _expression: String, _expression_id: i32) {
        self.invariant_expression = _expression;
        self.invariant_expression_id = _expression_id;
    }

    pub fn insert_one_state_variables(&mut self, _variables: String) {
        self.state_variables.push(_variables);
    }

    pub fn insert_more_state_variables(&mut self, _variables: Vec<String>) {
        for _variable in _variables {
            self.state_variables.push(_variable);
        }
    }

    pub fn insert_slot(&mut self, _address: i32, _variable: String, _slot: String) {
        // 先判断是否存在
        if self.slots.contains_key(&_address) {
            println!("insert_slot if");
            let mut _map = self.slots.get_mut(&_address).unwrap();
            _map.insert(_variable, _slot);
        } else {
            println!("insert_slot else");

            let mut _map = HashMap::new();
            _map.insert(_variable, _slot);
            self.slots.insert(_address, _map);
        }
    }
    pub fn get_slot(&self, _address: i32, _variable: String) -> Option<&String> {
        if self.slots.contains_key(&_address) {
            let _map = self.slots.get(&_address).unwrap();
            _map.get(&_variable)
        } else {
            None
        }
    }
    pub fn get_function_expression(&self, _index: &u8) -> Option<&String> {
        self.function_expressions.get(_index)
    }
    pub fn get_index_with_selector(&self, _selector: &String) -> Option<&u8> {
        self.function_selectors.get(_selector)
    }
    pub fn print_all(&self) {
        // 输出一切
        println!("{}", self);
    }
}
// 将字节数组转换为十六进制字符串
pub fn get_selector(bytes: &[u8]) -> String {
    // 将字节数组转换为十六进制字符串（小写）
    let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

    // 加上 '0x' 前缀
    let hex_with_prefix = format!("0x{}", hex_string);
    hex_with_prefix
}

// #[tokio::test]
// async fn test_send_tx() {
//     let handler = Handler::new(
//         "wss://go.getblock.io/4f364318713f46aba8d5b6de9b7e3ae6",
//         "mysql://root:1234@172.29.199.74:3306/invariantregistry",
//         vec![String::from_str("0x70ccd19d14552da0fb0712fd3920aeb1f9f65f59").unwrap()],
//     )
//     .await
//     .unwrap();
//     let private_key = "ba457ad011e6f8b3efc7f2b51d3fa7db94c26903f58b6d5da8176d5fbbc7f2e4";
//     let contract_address = "dBE9D9fcD06Ab1C82815eEcb9E4b78fD805c84A7";
//     let str_data = "d133576a000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000001046be12c00000000000000000000000000f59b5f18aabaae7ecb0b1713f07b635881e001bd000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a029e99f07000000000000000000000000000000000000000000000000000000000000000000000000000000004b00a35eb8cae62337f37fe561d7ff48987a4fed00000000000000000000000000000000000000000000000000000000000000001111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200000000000000000000000000000000000000000000000000000000";
//     println!("str_data {:?}", str_data);
//     let min = 10 as u128;
//     let max = 100 as u128;
//     let _min = pad_left(min.to_be_bytes().as_slice());
//     let _max = pad_left(max.to_be_bytes().as_slice());
//     let new_data = str_data.replace(
//         "1111111111111111111111111111111111111111111111111111111111111111",
//         remove_0x_prefix(min.encode_hex().as_str()),
//     );
//     println!("new_data {:?}", new_data);
//     let calldata = new_data.replace(
//         "2222222222222222222222222222222222222222222222222222222222222222",
//         remove_0x_prefix(max.encode_hex().as_str()),
//     );
//     println!("calldata {:?}", calldata);
//     let _calldata = Bytes::from_str(calldata.as_str()).unwrap();
//     handler
//         .send_tx(private_key, contract_address, _calldata)
//         .await
//         .unwrap();
// }

fn remove_0x_prefix(hex_string: &str) -> &str {
    if hex_string.starts_with("0x") || hex_string.starts_with("0X") {
        &hex_string[2..]
    } else {
        hex_string
    }
}

fn vec_remove_duplicates(old_vec: &mut Vec<String>) -> Vec<String> {
    let new_vec: HashSet<_> = old_vec.drain(..).collect();
    // 如果需要的话，再将HashSet转换回Vec
    new_vec.into_iter().collect()
}

impl fmt::Display for ProtectInfoCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProtectInfoCache {{\n")?;
        write!(f, "  Address: {}, ID: {}\n", self.address, self.address_id)?;
        write!(
            f,
            "  Invariant Expression: {}, ID: {}\n",
            self.invariant_expression, self.invariant_expression_id
        )?;
        write!(
            f,
            "  State Variables: [{}]\n",
            self.state_variables
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        write!(f, "  Slots:\n")?;

        for (address_id, variables) in &self.slots {
            write!(f, "    Address ID: {} => [\n", address_id)?;
            for (variable, slot) in variables {
                write!(f, "      Variable: \"{}\", Slot: \"{}\"\n", variable, slot)?;
            }
            write!(f, "    ]\n")?;
        }

        write!(
            f,
            "  Selectors: [{}]\n",
            self.selectors
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        write!(f, "  Function Selectors: [\n")?;

        for (selector, index) in &self.function_selectors {
            write!(f, "    Selector: \"{}\", Index: {}\n", selector, index)?;
        }

        write!(f, "  ]\n")?;
        write!(f, "  Function Expressions: [\n")?;

        for (index, expression) in &self.function_expressions {
            write!(f, "    Index: {}, Expression: \"{}\"\n", index, expression)?;
        }

        write!(f, "  ]\n")?;
        write!(f, "}}")
    }
}
