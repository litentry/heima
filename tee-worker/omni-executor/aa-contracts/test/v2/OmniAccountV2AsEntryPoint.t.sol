// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {OmniAccountV2} from "../../src/accounts/OmniAccountV2.sol";
import {BaseAccount} from "../../src/core/BaseAccount.sol";
import {EntryPointV1} from "../../src/core/EntryPointV1.sol";
import {Counter} from "../../src/Counter.sol";
import {OmniAccountV2TestUtils} from "./OmniAccountV2TestUtils.sol";
import {MockModule} from "./MockModule.sol";

contract OmniAccountV2AsEntryPoint is Test {
    OmniAccountV2 public account;
    EntryPointV1 public entryPoint;
    Counter public counter;
    MockModule public module;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    function setUp() public {
        (counter, entryPoint, account) = OmniAccountV2TestUtils.setUp(ownerAddress, clientId, rootAddress);
        module = new MockModule();
    }

    function test_Execute() public {
        OmniAccountV2TestUtils.performExecuteTestAs(vm, address(entryPoint), account, counter);
    }

    function test_ExecuteBatch() public {
        OmniAccountV2TestUtils.performExecuteBatchTestAs(vm, address(entryPoint), account, counter);
    }

    // ============ Module Management Tests ============

    function test_EntryPointCanRegisterModule() public {
        vm.prank(address(entryPoint));
        account.registerModule(address(module));

        assertTrue(account.isModuleRegistered(address(module)));
    }

    function test_EntryPointCanUnregisterModule() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        vm.prank(address(entryPoint));
        account.unregisterModule(address(module));

        assertFalse(account.isModuleRegistered(address(module)));
    }

    function test_OnlyEntryPointCanExecuteModule() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        bytes memory callData = abi.encodeWithSignature("increment()");

        vm.expectRevert("account: not from EntryPoint");
        vm.prank(ownerAddress);
        account.executeModuleCall(address(module), callData);
    }

    function test_CanExecuteRegisteredModule() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        bytes memory callData = abi.encodeWithSignature("increment()");

        vm.prank(address(entryPoint));
        account.executeModuleCall(address(module), callData);

        // Module executed successfully (delegatecall modifies account's storage, not module's)
        // We verify success by checking that no revert occurred
    }

    function test_ModuleExecutionWithReturnValue() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        bytes memory callData = abi.encodeWithSignature("functionWithReturn()");

        vm.prank(address(entryPoint));
        bytes memory returnData = account.executeModuleCall(address(module), callData);

        uint256 result = abi.decode(returnData, (uint256));
        assertEq(result, 42);
    }

    function test_ModuleExecutionWithMultipleReturns() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        bytes memory callData = abi.encodeWithSignature("functionWithMultipleReturns()");

        vm.prank(address(entryPoint));
        bytes memory returnData = account.executeModuleCall(address(module), callData);

        (uint256 num, address addr, bool flag) = abi.decode(returnData, (uint256, address, bool));
        assertEq(num, 123);
        assertEq(addr, address(0x1234));
        assertTrue(flag);
    }

    function test_ModuleExecutionRevertsCorrectly() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        bytes memory callData = abi.encodeWithSignature("functionThatReverts()");

        vm.expectRevert("Module function reverted");
        vm.prank(address(entryPoint));
        account.executeModuleCall(address(module), callData);
    }

    function test_ModuleExecutionWithParameters() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        bytes memory callData = abi.encodeWithSignature("incrementByValue(uint256)", 5);

        vm.prank(address(entryPoint));
        account.executeModuleCall(address(module), callData);

        // Module executed successfully with parameters (delegatecall modifies account's storage, not module's)
        // We verify success by checking that no revert occurred
    }
}
