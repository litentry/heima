// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

/**
 * Mock module for testing module registration and execution functionality
 */
contract MockModule {
    uint256 public counter;
    address public lastCaller;
    bytes public lastCallData;

    event ModuleFunctionCalled(address caller, uint256 value);
    event ModuleCounterIncremented(uint256 newValue);

    function increment() external {
        counter++;
        lastCaller = msg.sender;
        emit ModuleCounterIncremented(counter);
    }

    function incrementByValue(uint256 value) external {
        counter += value;
        lastCaller = msg.sender;
        emit ModuleCounterIncremented(counter);
    }

    function setCounter(uint256 newValue) external {
        counter = newValue;
        lastCaller = msg.sender;
    }

    function getCounter() external view returns (uint256) {
        return counter;
    }

    function functionWithReturn() external pure returns (uint256) {
        return 42;
    }

    function functionWithMultipleReturns() external pure returns (uint256, address, bool) {
        return (123, address(0x1234), true);
    }

    function functionThatReverts() external pure {
        revert("Module function reverted");
    }

    function recordCall(bytes calldata data) external {
        lastCaller = msg.sender;
        lastCallData = data;
        emit ModuleFunctionCalled(msg.sender, 0);
    }
}
