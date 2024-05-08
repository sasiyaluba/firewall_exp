use super::path_strategy::PathStrategy;
use std::{
    fs::File,
    io::{self, Read},
};

pub async fn read_op_from_file(
    _path_strategy: PathStrategy,
    filename: &str,
) -> io::Result<Vec<String>> {
    // Open the file
    let mut file = File::open(filename)?;

    // Read the contents of the file into a String
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Split the contents into lines and collect them into a Vec<String>
    let lines = contents
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<String>>();
    Ok(lines)
}

pub async fn get_similarity(
    compare_op1: Vec<&str>,
    compare_op2: Vec<&str>,
    _path_strategy: PathStrategy,
    pass_similarity: usize,
) {
    let similarity_rate = match _path_strategy {
        PathStrategy::FullPathMatch => {
            let lcs_length = full_path_algorithm(&compare_op1, &compare_op2);
            println!("lcs_length: {}", lcs_length);
            // let similarity = lcs_length / compare_op1.len();
            let similarity = 0;
            println!("full similarity : {}", similarity);
            if (similarity >= pass_similarity) {
                println!("successful，same attack logic！")
            } else {
                println!("failed！")
            }
        }
        PathStrategy::ControlFlowMatch => {
            let lcs_length = full_path_algorithm(&compare_op1, &compare_op2);
            println!("lcs_length: {}", lcs_length);
            let similarity = lcs_length / compare_op1.len();
            println!("control similarity : {}", similarity);
            if (similarity >= pass_similarity) {
                println!("successful，same attack logic！")
            } else {
                println!("failed！")
            }
        }
    };
}

pub fn full_path_algorithm(v1: &Vec<&str>, v2: &Vec<&str>) -> usize {
    let m: usize = v1.len();
    let n: usize = v2.len();

    let mut previous = vec![0; n + 1];
    let mut current = vec![0; n + 1];

    for i in 1..=m {
        std::mem::swap(&mut previous, &mut current);
        for j in 1..=n {
            if v1[i - 1] == v2[j - 1] {
                current[j] = previous[j - 1] + 1;
            } else {
                current[j] = std::cmp::max(previous[j], current[j - 1]);
            }
        }
    }

    current[n]
}

#[tokio::test]
async fn test_read_op_from_file() {
    let ret = read_op_from_file(
        PathStrategy::ControlFlowMatch,
        "src/paper/tx_oplist/HeavenGate/attack1_control.txt",
    )
    .await
    .unwrap();
    println!("{:?}", ret);
}

#[tokio::test]
async fn test_get_similarity() {
    let attack1_control_list = read_op_from_file(
        PathStrategy::ControlFlowMatch,
        "src/paper/tx_oplist/HeavenGate/attack1_control.txt",
    )
    .await
    .unwrap();
    let attack2_control_list = read_op_from_file(
        PathStrategy::ControlFlowMatch,
        "src/paper/tx_oplist/HeavenGate/attack2_control.txt",
    )
    .await
    .unwrap();
    let attack1_control_list: Vec<&str> = attack1_control_list.iter().map(|s| s.as_str()).collect();
    let attack2_control_list: Vec<&str> = attack2_control_list.iter().map(|s| s.as_str()).collect();

    get_similarity(
        attack1_control_list.clone(),
        attack2_control_list,
        PathStrategy::ControlFlowMatch,
        0,
    )
    .await;

    // let attack1_full_list = read_op_from_file(
    //     PathStrategy::ControlFlowMatch,
    //     "src/paper/tx_oplist/HeavenGate/attack1_full.txt",
    // )
    // .await
    // .unwrap();
    // let attack2_full_list = read_op_from_file(
    //     PathStrategy::ControlFlowMatch,
    //     "src/paper/tx_oplist/HeavenGate/attack2_full.txt",
    // )
    // .await
    // .unwrap();

    // let attack1_full_list: Vec<&str> = attack1_full_list.iter().map(|s| s.as_str()).collect();
    // let attack2_full_list: Vec<&str> = attack2_full_list.iter().map(|s| s.as_str()).collect();

    // get_similarity(
    //     attack1_full_list.clone(),
    //     attack2_full_list,
    //     PathStrategy::FullPathMatch,
    //     0,
    // )
    // .await;

    // ======================================== Test ================================
    let test1 = read_op_from_file(
        PathStrategy::ControlFlowMatch,
        "src/paper/tx_oplist/HeavenGate/test1.txt",
    )
    .await
    .unwrap();
    let test2 = read_op_from_file(
        PathStrategy::ControlFlowMatch,
        "src/paper/tx_oplist/HeavenGate/test2.txt",
    )
    .await
    .unwrap();
    let test1: Vec<&str> = test1.iter().map(|s| s.as_str()).collect();
    let test2: Vec<&str> = test2.iter().map(|s| s.as_str()).collect();

    get_similarity(test1, test2, PathStrategy::ControlFlowMatch, 0).await;
}
