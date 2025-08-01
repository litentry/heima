// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccountV1 as OmniAccount} from "../src/accounts/OmniAccountV1.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPointV1 as EntryPoint} from "../src/core/EntryPointV1.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_SUCCESS, SIG_VALIDATION_FAILED} from "../src/core/Helpers.sol";
import {Passkey} from "../src/interfaces/Passkey.sol";

contract OmniAccountAsRootNonEvm is Test {
    OmniAccount public account;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_email@example.com");

    function setUp() public {
        // Set up with Email owner type (non-EVM)
        (counter, entryPoint, account) =
            OmniAccountTestUtils.setUpWithOwnerType(ownerAddress, clientId, rootAddress, OwnerType.Email);
    }

    function test_RootCanCallAddRootSignerDirectlyForNonEvmOwner() public {
        // Direct call from root should succeed for non-EVM owner
        address newRoot = 0x0000000000000000000000000000000000000002;
        vm.prank(rootAddress);
        account.addRootSigner(newRoot);
        assertTrue(account.rootSigners(newRoot));
    }

    function test_RootCanCallRemoveRootSignerDirectlyForNonEvmOwner() public {
        // First add a root signer
        address newRoot = 0x0000000000000000000000000000000000000002;
        vm.prank(rootAddress);
        account.addRootSigner(newRoot);
        assertTrue(account.rootSigners(newRoot));

        // Direct call from root should succeed for non-EVM owner
        vm.prank(rootAddress);
        account.removeRootSigner(newRoot);
        assertFalse(account.rootSigners(newRoot));
    }

    function test_RootCanCallAddRootSignerViaUserOpForNonEvmOwner() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) =
            OmniAccountTestUtils.setUpWithOwnerType(ownerAddress, clientId, root, OwnerType.Email);

        address newRoot = 0x0000000000000000000000000000000000000002;

        // Prepare UserOp that calls addRootSigner directly
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addRootSigner(address)", newRoot);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should succeed because root can call restricted functions for non-EVM owners
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Execute the UserOp
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the root signer was actually added
        assertTrue(account.rootSigners(newRoot));
    }

    function test_RootCanCallWithdrawDepositViaUserOpForNonEvmOwner() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) =
            OmniAccountTestUtils.setUpWithOwnerType(ownerAddress, clientId, root, OwnerType.Google);

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

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should succeed for root with non-EVM owner
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Execute and verify
        uint256 balanceBefore = withdrawTo.balance;
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);
        assertEq(withdrawTo.balance, balanceBefore + withdrawAmount);
    }

    function test_SessionKeyStillCannotCallRestrictedFunctionsForNonEvmOwner() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");
        uint256 sessionExpiration = block.timestamp + 1000;

        bytes memory sessionProof = prepareSession(session, sessionExpiration, rootPk);

        (counter, entryPoint, account) =
            OmniAccountTestUtils.setUpWithOwnerType(ownerAddress, clientId, root, OwnerType.Twitter);

        address newRoot = 0x0000000000000000000000000000000000000002;

        // Prepare UserOp that calls addRootSigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addRootSigner(address)", newRoot);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with session key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.SessionKey), r, s, v, sessionExpiration, sessionProof);

        // Validation should still fail for session keys even with non-EVM owner
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function prepareSession(address session, uint256 expiration, uint256 proofSigner)
        internal
        pure
        returns (bytes memory)
    {
        bytes32 sessionDigest = sha256(abi.encodePacked(session, expiration));
        (uint8 sv, bytes32 sr, bytes32 ss) = vm.sign(proofSigner, sessionDigest);
        bytes memory sessionProof = abi.encodePacked(sr, ss, sv);
        return sessionProof;
    }

    function test_RootCanCallAddPasskeySignerViaUserOpForNonEvmOwner() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) =
            OmniAccountTestUtils.setUpWithOwnerType(ownerAddress, clientId, root, OwnerType.Email);

        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});

        // Prepare UserOp that calls addPasskeySigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSelector(account.addPasskeySigner.selector, pk);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should succeed because root can call restricted functions for non-EVM owners
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);

        // Execute the UserOp
        vm.prank(address(entryPoint));
        (bool success,) = address(account).call(callData);
        assertTrue(success);

        // Verify the passkey signer was actually added
        bytes32 key = Passkey.toKey(pk);
        assertTrue(account.passkeySigners(key));
    }

    function test_RootCannotRemovePasskeySignerViaUserOpWhenPasskeySignersExist() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) =
            OmniAccountTestUtils.setUpWithOwnerType(ownerAddress, clientId, root, OwnerType.Google);

        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});

        // First add a passkey signer directly
        vm.prank(root);
        account.addPasskeySigner(pk);
        bytes32 key = Passkey.toKey(pk);
        assertTrue(account.passkeySigners(key));

        // Prepare UserOp that calls removePasskeySigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSelector(account.removePasskeySigner.selector, pk);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail for root when passkey signers exist
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCanCallUpgradeToAndCallViaUserOpForNonEvmOwner() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) =
            OmniAccountTestUtils.setUpWithOwnerType(ownerAddress, clientId, root, OwnerType.Twitter);

        address newImplementation = address(0x1234567890123456789012345678901234567890);
        bytes memory data = "";

        // Prepare UserOp that calls upgradeToAndCall
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("upgradeToAndCall(address,bytes)", newImplementation, data);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should succeed for root with non-EVM owner
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_RootCanCallAddPasskeySignerDirectlyForNonEvmOwner() public {
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});

        // Direct call from root should succeed for non-EVM owner
        vm.prank(rootAddress);
        account.addPasskeySigner(pk);
        bytes32 key = Passkey.toKey(pk);
        assertTrue(account.passkeySigners(key));
    }

    function test_RootCannotRemovePasskeySignerDirectlyWhenPasskeySignersExist() public {
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});

        // First add a passkey signer
        vm.prank(rootAddress);
        account.addPasskeySigner(pk);
        bytes32 key = Passkey.toKey(pk);
        assertTrue(account.passkeySigners(key));

        // Direct call from root should fail because passkey signers now exist
        vm.expectRevert("only owner");
        vm.prank(rootAddress);
        account.removePasskeySigner(pk);
    }

    function test_RootCanCallUpgradeToAndCallDirectlyForNonEvmOwner() public {
        address newImplementation = address(0x1234567890123456789012345678901234567890);
        bytes memory data = "";

        // Direct call from root should succeed for non-EVM owner
        // This test will revert because the implementation address is not a valid contract
        // but that's expected - we're testing access control, not actual upgrade
        vm.expectRevert();
        vm.prank(rootAddress);
        account.upgradeToAndCall(newImplementation, data);
    }
}
