// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

//test
contract FireWall {
    address public owner;
    address[] public users;
    uint256 public balance;

    constructor() payable {
        owner = msg.sender;
    }

    ///@notice 注册成功之后的用户可以从当前合约中提取一定的资金，提取资金是存在时间消耗的。
    ///@dev initial state: 100 ether, new_balance = (balanceOf(this.address) - fundsAmount) >= 80 ether.
    function getFunds(uint256 fundsAmount, address payable withdrawer, uint256 deadline) public payable {
        (bool success,) = withdrawer.call{value: fundsAmount * 1 ether}("");
    }

    function register(address payable _user) public payable {
        users.push(_user);
    }

    function getBalance() public {
        balance = address(this).balance;
    }

    receive() external payable {}
}
