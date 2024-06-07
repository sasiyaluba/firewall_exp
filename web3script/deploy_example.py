from web3 import Web3

rpc_url = "https://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/"

web3 = Web3(Web3.HTTPProvider(rpc_url))


def test():
    abi = [{
        "inputs": [],
        "stateMutability": "payable",
        "type": "constructor"
    }, {
        "inputs": [],
        "name":
        "balance",
        "outputs": [{
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "stateMutability":
        "view",
        "type":
        "function"
    }, {
        "inputs": [],
        "name": "getBalance",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "fundsAmount",
            "type": "uint256"
        }, {
            "internalType": "address payable",
            "name": "withdrawer",
            "type": "address"
        }, {
            "internalType": "uint256",
            "name": "deadline",
            "type": "uint256"
        }],
        "name":
        "getFunds",
        "outputs": [],
        "stateMutability":
        "payable",
        "type":
        "function"
    }, {
        "inputs": [],
        "name":
        "owner",
        "outputs": [{
            "internalType": "address",
            "name": "",
            "type": "address"
        }],
        "stateMutability":
        "view",
        "type":
        "function"
    }, {
        "inputs": [{
            "internalType": "address payable",
            "name": "_user",
            "type": "address"
        }],
        "name":
        "register",
        "outputs": [],
        "stateMutability":
        "payable",
        "type":
        "function"
    }, {
        "inputs": [{
            "internalType": "uint256",
            "name": "",
            "type": "uint256"
        }],
        "name":
        "users",
        "outputs": [{
            "internalType": "address",
            "name": "",
            "type": "address"
        }],
        "stateMutability":
        "view",
        "type":
        "function"
    }, {
        "stateMutability": "payable",
        "type": "receive"
    }]
    # 从合约地址获得合约对象
    contract_addr = "0x42C1c8Bf2C0244BBe7755E592252992F580DaaF4"

    contract = web3.eth.contract(address=contract_addr, abi=abi)
    print(contract)
    # 调用合约方法
    result = contract.functions.getBalance().call()
    print(result)


test()
