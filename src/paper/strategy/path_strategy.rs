use ethers::core::k256::sha2::digest::typenum::op;
use crate::paper::tx_origin_data::get_origin_oplist::get_opcode_list;

pub enum PathStrategy {
    FullPathMatch,
    ControlFlowMatch,
}

pub struct ControlFlowMatchOpcode {
    contain_opcode: Vec<&'static str>
}

impl ControlFlowMatchOpcode {
    pub fn default() -> Self {
        Self {
            contain_opcode: vec!["CALL", "DELEGATECALL", "JUMP", "JUMPI", "RETURN", "CREATE", "CREATE2"],
        }
    }
}

pub async fn get_path_strategy(_rpc: &str, _tx_hash: &str, _path_strategy: PathStrategy) -> Vec<String> {
    let opcode_list = get_opcode_list(_rpc, _tx_hash).await;

    let ret_path_strategy = match _path_strategy {
        PathStrategy::FullPathMatch => {
            opcode_list
        }
        PathStrategy::ControlFlowMatch => {
            let control_flow_opcode = ControlFlowMatchOpcode::default().contain_opcode;
            let ret= opcode_list.into_iter().filter(|opcode| {
                control_flow_opcode.contains(&opcode.as_str()) == true
            }).collect();
            // ret
            ret
        }
    };
    ret_path_strategy
}




