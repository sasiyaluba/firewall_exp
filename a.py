from web3 import Web3
import json
import requests


def main():
    payload = json.dumps({
        "method":
        "debug_traceTransaction",
        "params": [
            "0x14d5a6113fb53eb3cbeb9a6753b17c27387b0e77bd50532b6122e3cd39f47e76",
            # {
            #     "tracer": "prestateTracer"
            # }
        ],
        "id":
        1,
        "jsonrpc":
        "2.0"
    })

    headers = {'Content-Type': 'application/json'}
    response = requests.request(
        "POST",
        "https://chaotic-sly-panorama.ethereum-sepolia.quiknode.pro/b0ed5f4773268b080eaa3143de06767fcc935b8d/",
        headers=headers,
        data=payload)
    with open("./debug4.json", "w") as f:
        json.dump(response.json(), f)
        f.close()
    _opcodes = json.load(open("./debug4.json"))["result"]["structLogs"]
    opcodes = []
    for opcode in _opcodes:
        opcodes.append(opcode.get("op"))
    with open("./debug2.json", "w") as f:
        json.dump(opcodes, f)
        f.close()


main()
