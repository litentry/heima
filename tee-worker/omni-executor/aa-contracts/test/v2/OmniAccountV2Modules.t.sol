// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccountV2} from "../../src/accounts/OmniAccountV2.sol";
import {EntryPointV1} from "../../src/core/EntryPointV1.sol";
import {Counter} from "../../src/Counter.sol";
import {OmniAccountV2TestUtils} from "./OmniAccountV2TestUtils.sol";
import {MockModule} from "./MockModule.sol";

/**
 * General module tests that don't fit into actor-specific test files.
 * Tests for specific actors (Owner, Root, EntryPoint) are in their respective files:
 * - OmniAccountV2AsOwner.t.sol
 * - OmniAccountV2AsRoot.t.sol
 * - OmniAccountV2AsEntryPoint.t.sol
 */
contract OmniAccountV2Modules is Test {
    OmniAccountV2 public account;
    EntryPointV1 public entryPoint;
    Counter public counter;
    MockModule public module;
    MockModule public module2;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    function setUp() public {
        (counter, entryPoint, account) = OmniAccountV2TestUtils.setUp(ownerAddress, clientId, rootAddress);
        module = new MockModule();
        module2 = new MockModule();
    }

    // ============ Module Registration Validation Tests ============

    function test_CannotRegisterZeroAddressAsModule() public {
        vm.expectRevert("Invalid module address");
        vm.prank(ownerAddress);
        account.registerModule(address(0));
    }

    function test_CannotRegisterNonContractAsModule() public {
        address eoa = address(0x1234);
        vm.expectRevert("Module must be a contract");
        vm.prank(ownerAddress);
        account.registerModule(eoa);
    }

    function test_CannotRegisterModuleTwice() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        vm.expectRevert("Module already registered");
        vm.prank(ownerAddress);
        account.registerModule(address(module));
    }

    function test_CannotUnregisterNonRegisteredModule() public {
        vm.expectRevert("Module not registered");
        vm.prank(ownerAddress);
        account.unregisterModule(address(module));
    }

    // ============ Module Execution Tests ============

    function test_CannotExecuteUnregisteredModule() public {
        bytes memory callData = abi.encodeWithSignature("increment()");

        vm.expectRevert("Module not registered");
        vm.prank(address(entryPoint));
        account.executeModuleCall(address(module), callData);
    }

    function test_ModuleExecutionAccessesAccountContext() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        // Set a value in the account's storage at slot 0 (where MockModule.counter is stored)
        // This proves delegatecall reads from account's storage, not module's storage
        uint256 expectedValue = 123;
        vm.store(address(account), bytes32(uint256(0)), bytes32(expectedValue));

        bytes memory callData = abi.encodeWithSignature("getCounter()");

        vm.prank(address(entryPoint));
        bytes memory returnData = account.executeModuleCall(address(module), callData);

        // Delegatecall executes module code in account's context
        // The module function should access account's storage and return the value we set
        uint256 result = abi.decode(returnData, (uint256));
        assertEq(result, expectedValue, "Module should read from account's storage via delegatecall");

        // Verify module's own storage is still 0 (unaffected)
        assertEq(module.counter(), 0, "Module's own storage should remain unchanged");
    }

    // ============ Multiple Module Tests ============

    function test_MultipleModulesCanBeRegistered() public {
        vm.prank(ownerAddress);
        account.registerModule(address(module));

        vm.prank(ownerAddress);
        account.registerModule(address(module2));

        assertTrue(account.isModuleRegistered(address(module)));
        assertTrue(account.isModuleRegistered(address(module2)));
    }

}
