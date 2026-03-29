// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract ReentrancyVulnerable {
    mapping(address => uint256) public balances;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }

    function withdraw() external {
        uint256 amount = balances[msg.sender];
        (bool sent, ) = msg.sender.call{value: amount}("");
        require(sent, "Transfer failed");
        balances[msg.sender] = 0;
    }
}
