// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title VulnerableToken
 * @notice A deliberately vulnerable token for security testing
 * @dev Contains: Reentrancy, Integer Overflow, Access Control issues
 */

contract VulnerableToken {
    string public name = "VulnerableToken";
    string public symbol = "VULN";
    uint8 public decimals = 18;
    uint256 public totalSupply;
    
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    
    // Reentrancy vulnerable: no reentrancy guard
    function transfer(address _to, uint256 _value) public returns (bool) {
        require(balanceOf[msg.sender] >= _value, "Insufficient balance");
        
        // Vulnerable: external call before state update
        (bool success, ) = _to.call{value: _value}("");
        
        balanceOf[msg.sender] -= _value;
        balanceOf[_to] += _value;
        
        emit Transfer(msg.sender, _to, _value);
        return true;
    }
    
    // Reentrancy vulnerable: withdraw function
    function withdraw(uint256 _amount) public {
        require(balanceOf[msg.sender] >= _amount, "Insufficient balance");
        
        // Vulnerable: external call before state update (checks-effects-interactions violated)
        (bool success, ) = msg.sender.call{value: _amount}("");
        require(success, "Transfer failed");
        
        balanceOf[msg.sender] -= _amount; // State update after external call
    }
    
    // Integer overflow vulnerable: no SafeMath, using solidity <0.8 behavior
    function mint(address _to, uint256 _amount) public {
        // Vulnerable: no access control (anyone can mint)
        totalSupply += _amount;
        balanceOf[_to] += _amount;
        emit Transfer(address(0), _to, _amount);
    }
    
    // Integer underflow vulnerable
    function burn(uint256 _amount) public {
        // Vulnerable: no underflow check in older solidity
        balanceOf[msg.sender] -= _amount;
        totalSupply -= _amount;
    }
    
    // Approval race condition vulnerable
    function approve(address _spender, uint256 _value) public returns (bool) {
        allowance[msg.sender][_spender] = _value;
        emit Approval(msg.sender, _spender, _value);
        return true;
    }
    
    // TransferFrom with no approval check vulnerability
    function transferFrom(address _from, address _to, uint256 _value) public returns (bool) {
        // Missing: allowance check
        balanceOf[_from] -= _value;
        balanceOf[_to] += _value;
        emit Transfer(_from, _to, _value);
        return true;
    }
    
    // Fallback to receive ETH
    receive() external payable {}
    fallback() external payable {}
    
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}
