// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";
import {SIG_VALIDATION_SUCCESS} from "../src//core/Helpers.sol";

contract OmniAccountAsOwner is Test {
    OmniAccount public account;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    function setUp() public {
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, rootAddress);
    }

    function test_Execute() public {
        OmniAccountTestUtils.performExecuteTestAs(vm, ownerAddress, account, counter);
    }

    function test_ExecuteBatch() public {
        OmniAccountTestUtils.performExecuteBatchTestAs(vm, ownerAddress, account, counter);
    }

    function test_AddRemoveRootSigner() public {
        address root = 0x0000000000000000000000000000000000000002;
        vm.prank(ownerAddress);
        account.addRootSigner(root);
        assert(account.rootSigners(root));
        vm.prank(ownerAddress);
        account.removeRootSigner(root);
        assert(!account.rootSigners(root));
    }

    function test_validateOp() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(alice, clientId, rootAddress);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;

        bytes memory initCode = "";

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_OwnerCanCallAddRootSignerViaUserOp() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(owner, clientId, rootAddress);

        address newRoot = 0x0000000000000000000000000000000000000002;

        // Prepare UserOp that calls execute() with addRootSigner as inner call
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory innerCallData = abi.encodeWithSignature("addRootSigner(address)", newRoot);
        bytes memory callData =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(account), 0, innerCallData);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        // Validation should succeed because owner can call restricted functions
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Execute the UserOp to verify it actually works
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the root signer was actually added
        assertTrue(account.rootSigners(newRoot));
    }

    function test_OwnerCanCallRemoveRootSignerViaUserOp() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(owner, clientId, rootAddress);

        // First add a root signer directly
        vm.prank(owner);
        account.addRootSigner(rootAddress);
        assertTrue(account.rootSigners(rootAddress));

        // Prepare UserOp that calls execute() with removeRootSigner as inner call
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory innerCallData = abi.encodeWithSignature("removeRootSigner(address)", rootAddress);
        bytes memory callData =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(account), 0, innerCallData);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        // Validation should succeed
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Execute the UserOp
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the root signer was actually removed
        assertFalse(account.rootSigners(rootAddress));
    }

    function test_OwnerCanCallWithdrawDepositViaUserOp() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(owner, clientId, rootAddress);

        // Add some deposit first
        vm.deal(address(account), 1 ether);
        vm.prank(address(account));
        account.addDeposit{value: 0.5 ether}();

        address payable withdrawTo = payable(0x0000000000000000000000000000000000000003);
        uint256 withdrawAmount = 0.1 ether;

        // Prepare UserOp that calls withdrawDepositTo
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData =
            abi.encodeWithSignature("withdrawDepositTo(address,uint256)", withdrawTo, withdrawAmount);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        // Validation should succeed
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_OwnerCanExecuteBatchViaUserOp() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(owner, clientId, rootAddress);

        // Prepare UserOp that calls executeBatch
        address sender = address(account);
        bytes memory initCode = "";

        BaseAccount.Call[] memory calls = new BaseAccount.Call[](2);
        calls[0] = BaseAccount.Call({target: address(counter), value: 0, data: abi.encodeWithSignature("increment()")});
        calls[1] = BaseAccount.Call({target: address(counter), value: 0, data: abi.encodeWithSignature("increment()")});
        bytes memory callData = abi.encodeWithSignature("executeBatch((address,uint256,bytes)[])", calls);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        // Validation should succeed because owner can use executeBatch
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Execute the UserOp to verify it works
        uint256 initialCount = counter.number();
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the counter was incremented twice
        assertEq(counter.number(), initialCount + 2);
    }

    function test_OwnerCanExecuteWithRestrictedInnerCallViaUserOp() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(owner, clientId, rootAddress);

        address newRoot = 0x0000000000000000000000000000000000000004;

        // Prepare UserOp that calls execute() with addRootSigner as inner call
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory innerCallData = abi.encodeWithSignature("addRootSigner(address)", newRoot);
        bytes memory callData =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(account), 0, innerCallData);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        // Validation should succeed because owner can call restricted functions even through execute()
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Execute the UserOp to verify it works
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the root signer was actually added
        assertTrue(account.rootSigners(newRoot));
    }

    function test_OwnerCanCallAddRootSignerDirectlyViaUserOp() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(owner, clientId, rootAddress);

        address newRoot = 0x0000000000000000000000000000000000000005;

        // Prepare UserOp that calls addRootSigner directly (without execute wrapper)
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addRootSigner(address)", newRoot);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        // Validation should succeed for owner
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Now execution will succeed because EntryPoint is allowed in _onlyOwner
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the root signer was actually added
        assertTrue(account.rootSigners(newRoot));
    }

    function test_OwnerCanCallWithdrawDepositDirectlyViaUserOp() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(owner, clientId, rootAddress);

        // Add some deposit first
        vm.deal(address(account), 1 ether);
        vm.prank(address(account));
        account.addDeposit{value: 0.5 ether}();

        address payable withdrawTo = payable(0x0000000000000000000000000000000000000006);
        uint256 withdrawAmount = 0.1 ether;

        // Prepare UserOp that calls withdrawDepositTo directly
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData =
            abi.encodeWithSignature("withdrawDepositTo(address,uint256)", withdrawTo, withdrawAmount);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with owner key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        // Validation should succeed for owner
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Now execution will succeed because EntryPoint is allowed in _onlyOwner
        uint256 balanceBefore = withdrawTo.balance;
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the withdrawal actually happened
        assertEq(withdrawTo.balance, balanceBefore + withdrawAmount);
    }
}
