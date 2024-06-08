import z3
from web3 import Web3

url = "https://lb.nodies.app/v1/181a5ebf4c954f8496ae7cbc1ac8d03b"
web3 = Web3(Web3.HTTPProvider(url))


def get_storage(contract_address, storage_index):
    return web3.eth.get_storage_at(contract_address, storage_index)


def main():
    # 根据给定的表达式，得到一个求解器
    ...


main()
