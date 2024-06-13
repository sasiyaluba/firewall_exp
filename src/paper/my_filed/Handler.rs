use crate::paper::my_filed::expression::evaluate_exp;
use crate::paper::my_filed::sym_exec::sym_exec;
use ansi_term::Colour::{Black, Blue, Cyan, Fixed, Green, Purple, Red, White, Yellow};
use ethers::abi::AbiEncode;
use ethers::prelude::*;
use ethers::types::{Block, Transaction};
use mysql::prelude::Queryable;
use mysql::*;
use regex::bytes::Regex;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
#[derive(Debug)]
pub struct Handler {
    rpc: &'static str,
    sql_url: &'static str,
    rpc_connect: Arc<Provider<Ws>>,
    sql_connect: mysql::Pool,
    protect_addresses: Vec<String>,
}

impl Handler {
    pub async fn new(
        _rpc: &'static str,
        _sql_url: &'static str,
        _addresses: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Ws>::connect(_rpc).await?;
        println!();
        println!(
            "{}",
            Yellow.paint("---初始化连接rpc节点 Connected to the Ethereum network")
        );
        let rpc_connect = Arc::new(provider);
        let sql_connect = mysql::Pool::new(_sql_url)?;
        println!(
            "{}",
            Yellow.paint("---初始化连接数据库 Connected to the MySQL database")
        );
        println!();
        Ok(Self {
            rpc: _rpc,
            sql_url: _sql_url,
            rpc_connect,
            sql_connect,
            protect_addresses: _addresses,
        })
    }

    pub async fn get_range(
        &self,
        _address: &str,
        _function_selector: &str,
    ) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut connect = self.sql_connect.get_conn()?;
        let address_id: i32 = connect
            .exec_first(
                "SELECT id FROM addresses WHERE address =:address",
                params! {
                    "address" => _address,
                },
            )?
            .ok_or("Address not found")?;

        let expression:String = connect
            .exec_first(
                "SELECT expression FROM function_expressions WHERE address_id =:address_id AND function_selector:=function_selector",
                params! {
                    "address_id" => address_id,
                    "function_selector" => _function_selector
                },
            )?
            .ok_or("Address not found")?;
        Ok(vec![])
    }

    // 检查不变量是否异常
    pub async fn check_invariant(
        &self,
        _address: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // 获得表达式
        let expression = self.get_expression(_address)?;
        println!("-----------当前不变量 {}", Blue.paint(&expression.clone()));
        println!();
        // 处理表达式
        let result = self.handle_exp(_address, expression).await;
        Ok(result)
    }

    pub fn get_expression(&self, _address: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut connect = self.sql_connect.get_conn()?;
        let address_id: i32 = connect
            .exec_first(
                "SELECT id FROM addresses WHERE address =:address",
                params! {
                    "address" => _address,
                },
            )?
            .ok_or("Address not found")?;
        let expression: String = connect
            .exec_first(
                "SELECT expression FROM expressions WHERE address_id =:address_id",
                params! {
                    "address_id" => address_id,
                },
            )?
            .ok_or("Address not found")?;
        Ok(expression)
    }

    pub async fn get_values_with_names(
        &self,
        _address: &str,
        _state_names: Vec<String>,
    ) -> Result<Vec<U256>, Box<dyn std::error::Error>> {
        let mut connect = self.sql_connect.get_conn()?;
        //
        let address_id: i32 = connect
            .exec_first(
                "SELECT id FROM addresses WHERE address =:address",
                params! {
                    "address" => _address,
                },
            )?
            .ok_or("Address not found")?;
        // println!("address_id {:?}", address_id);
        let expression_id: i32 = connect
            .exec_first(
                "SELECT id FROM expressions WHERE address_id =:address_id",
                params! {
                    "address_id" => address_id,
                },
            )?
            .ok_or("Address not found")?;
        // println!("expression_id {:?}", expression_id);
        let mut values: Vec<U256> = Vec::new();
        // println!("state_names {:?}", _state_names);
        for name in _state_names {
            if name.eq("xxx") {
                continue;
            }
            if name.eq("address(this).balance") {
                let balance = self.rpc_connect.get_balance(_address, None).await?;
                values.push(balance);
                continue;
            }
            let _slot: String = connect
                .exec_first(
                    "SELECT slot FROM variables WHERE variable_name = :variable_name AND expression_id = :expression_id",
                    params! {
                        "variable_name" => name,
                        "expression_id" => expression_id.to_string()
                    },
                )?
                .ok_or("Variable name not found")?;

            let slot = H256::from_str(&_slot)?;
            // println!("slot {:?}", slot);
            let value = self
                .rpc_connect
                .get_storage_at(_address, slot, None)
                .await?;
            // println!("value {:?}", value);
            values.push(U256::from_big_endian(value.as_bytes()));
        }

        Ok(values)
    }

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
            result = result && evaluate_exp(new_expression.as_str()).unwrap();
        }
        result
    }

    pub async fn handle(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 进行监听
        let mut block_stream = self.rpc_connect.subscribe_blocks().await?;
        // 监听区块
        while let Some(block) = block_stream.next().await {
            println!(
                "-----当前区块为 {}",
                Red.paint(&block.number.unwrap().to_string())
            );
            // todo 保证监控一直执行，需要将这些工作分到其他线程去做...
            self.block_check(block).await?;
        }
        Ok(())
    }

    pub async fn block_check(
        &self,
        _now_block: Block<H256>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for address in self.protect_addresses.iter() {
            let result = self.check_invariant(address.as_str()).await?;
            if !result {
                println!("need notice,its attack");
                let now_block = self
                    .rpc_connect
                    .get_block_with_txs(_now_block.number.unwrap())
                    .await
                    .expect("can't get block")
                    .unwrap();
                let all_tx: Vec<Transaction> = now_block.transactions;
                // 进行过滤，找到与from交互的交易
                let interact_tx: Vec<Transaction> = all_tx
                    .into_iter()
                    .filter(|tx| tx.to == Some(H160::from_str(address).unwrap()))
                    .collect();
                //todo 这里做符号执行，放到其他线程去做...
                if interact_tx.len() > 0 {
                    println!("interact tx hash: {:?}", interact_tx[0].hash.encode_hex());
                    let _ = sym_exec(
                        &self.rpc,
                        interact_tx[0].hash.encode_hex().as_str(),
                        &address,
                        0,
                    )
                    .await;
                }
            }
        }
        Ok(())
    }

    pub async fn caculate_range(&self, _address: &str, _expression: String) -> bool {
        // 将表达式分割，以&&为分隔符
        let _expression = _expression.replace(" ", "");
        let _expressions: Vec<&str> = _expression.split("&&").collect();
        let mut result = true;
        // 首先获得所有状态变量的名称
        let mut connect = self.sql_connect.get_conn().unwrap();
        let address_id: i32 = connect
            .exec_first(
                "SELECT id FROM addresses WHERE address =:address",
                params! {
                    "address" => _address,
                },
            )
            .unwrap()
            .unwrap();
        let variable_names: Vec<String> = connect
            .exec(
                "SELECT variable_name FROM variablestoslot WHERE address_id =:address_id",
                params! {
                    "address_id" => address_id,
                },
            )
            .unwrap();
        println!("variable_names {:?}", variable_names);
        // 分别处理每个表达式
        for _expression in _expressions {
            // 获得所有变量
            let re = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
            let mut variables: Vec<String> = re
                .find_iter(_expression.as_bytes())
                .map(|mat| String::from_utf8(mat.as_bytes().to_vec()).unwrap())
                .collect();

            // 只包含状态变量的变量集
            let mut filtered_variables: Vec<String> = Vec::new();
            for var in variables.iter() {
                if variable_names.contains(var) {
                    filtered_variables.push(var.clone());
                }
            }

            // 获得所有变量的值
            let mut _values = self
                .get_values_with_names(_address, filtered_variables.clone())
                .await
                .unwrap();
            // 将原表达式中的每个变量替换为对应的值
            let mut new_expression = _expression.to_string();
            for i in 0..variables.len() {
                new_expression =
                    new_expression.replace(variables[i].as_str(), _values[i].to_string().as_str());
            }
            // println!("替换后，表达式为：{:?}", new_expression);
            // 计算表达式的值
            result = result && evaluate_exp(new_expression.as_str()).unwrap();
        }
        result
    }
}

#[tokio::test]
async fn test_all() {
    let mut handler = Handler::new(
        "wss://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/",
        "mysql://root:1234@172.29.199.74:3306/invariantregistry",
        vec![String::from_str("0x433ba3f2322F6449582f60d36fa64C4EB8830fCC").unwrap()],
    )
    .await
    .unwrap();
}
