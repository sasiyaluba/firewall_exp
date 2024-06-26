use mysql::params;
use mysql::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
#[derive(Debug)]
// 数据库相关
pub struct DatabaseManager {
    // 数据库连接
    pub sql_pool: Arc<mysql::Pool>,
    // 所有地址
    pub protect_addresses: Vec<String>,
    // 数据
    pub protect_infos: HashMap<String, ProtectInfo>,
}
#[derive(Debug)]
pub struct ProtectInfo {
    // 不变量
    pub invariant: String,
    // 相关的变量
    pub variables: Vec<String>,
    // 保护的选择器
    pub selectors: Vec<String>,
    // 变量 => slot
    pub slot_map: HashMap<String, String>,
    // 选择器 => index => 函数表达式
    pub expression_map: HashMap<String, HashMap<u8, String>>,
}

impl DatabaseManager {
    // 初始化
    pub fn new(_sql_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            sql_pool: Arc::new(mysql::Pool::new(_sql_url).expect("error mysql url")),
            protect_addresses: vec![],
            protect_infos: HashMap::new(),
        })
    }

    // 加载数据库中的信息到本地
    pub fn load_data_for_cache(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 获得连接
        let mut sql_conn = self.sql_pool.get_conn().unwrap();
        // 首先获得所有地址以及不变量
        let address_invariant: Vec<(u64, String, String)> =
            sql_conn.query("Select * from address_invariants").unwrap();
        let mut temp_map = HashMap::new();
        for (address_id, address, invar) in address_invariant {
            self.protect_addresses.push(address.clone());
            // 根据地址获得变量
            let variable_slot: Vec<(String, String)> = sql_conn
                .exec(
                    "Select variable,slot from variables Where address_id =:address_id ",
                    params! {
                        "address_id" => address_id
                    },
                )
                .unwrap();
            // 获得var
            let variables = variable_slot
                .iter()
                .map(|(variable, _)| variable.clone())
                .collect();
            // 构造对应关系
            let mut slot_map = HashMap::new();
            for (var, slot) in variable_slot {
                slot_map.insert(var, slot);
            }
            // 根据地址获得选择器
            let selector_index_exp: Vec<(String, u8, String)> = sql_conn.exec(
                "Select selector,`index`,expression from expressions Where address_id =:address_id ",
                params! {
                    "address_id" => address_id
                },
            ).unwrap();
            // 获得选择器
            let selectors = selector_index_exp
                .iter()
                .map(|(selector, _, _)| selector.clone())
                .collect();
            // 构造对应关系
            let mut exp_map = HashMap::new();
            for (selector, index, exp) in selector_index_exp {
                let mut temp_map3 = HashMap::new();
                temp_map3.insert(index, exp);
                exp_map.insert(selector, temp_map3);
            }
            let pi = ProtectInfo {
                invariant: invar,
                variables: variables,
                selectors: selectors,
                slot_map: slot_map,
                expression_map: exp_map,
            };
            temp_map.insert(address, pi);
        }
        self.protect_infos = temp_map;
        Ok(())
    }
}

#[test]
fn test_data() {
    let sql_url = format!("mysql://root:1234@{}:3306/new_data", "172.29.218.244");
    let mut manager = DatabaseManager::new(sql_url.as_str()).unwrap();
    let _ = manager.load_data_for_cache();
    println!("{:?}", manager.protect_infos)
}
