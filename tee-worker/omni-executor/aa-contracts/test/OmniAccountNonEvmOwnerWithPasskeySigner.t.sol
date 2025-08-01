// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccountV1 as OmniAccount} from "../src/accounts/OmniAccountV1.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPointV1 as EntryPoint} from "../src/core/EntryPointV1.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import {Passkey} from "../src/interfaces/Passkey.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_SUCCESS, SIG_VALIDATION_FAILED} from "../src/core/Helpers.sol";

contract OmniAccountNonEvmOwnerWithPasskeySigner is Test {
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

    function test_RootCannotCallOnlyOwnerWhenPasskeySignersExist() public {
        // First add a passkey signer as owner
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 1, y: 2});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        // Now root should not be able to call onlyOwner functions
        address newRoot = 0x0000000000000000000000000000000000000002;
        vm.prank(rootAddress);
        vm.expectRevert("only owner");
        account.addRootSigner(newRoot);
    }

    function test_RootCanCallOnlyOwnerAgainAfterLastPasskeyRemoved() public {
        // Add a passkey signer
        Passkey.PublicKey memory pk1 = Passkey.PublicKey({x: 1, y: 2});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk1);
        assertEq(account.passkeySignerCount(), 1);

        // Root cannot call onlyOwner
        address newRoot = 0x0000000000000000000000000000000000000002;
        vm.prank(rootAddress);
        vm.expectRevert("only owner");
        account.addRootSigner(newRoot);

        // Remove the passkey signer
        vm.prank(ownerAddress);
        account.removePasskeySigner(pk1);
        assertEq(account.passkeySignerCount(), 0);

        // Now root should be able to call onlyOwner again
        vm.prank(rootAddress);
        account.addRootSigner(newRoot);
        assertTrue(account.rootSigners(newRoot));
    }

    function test_MultiplePasskeySignersCount() public {
        // Add multiple passkey signers
        Passkey.PublicKey memory pk1 = Passkey.PublicKey({x: 1, y: 2});
        Passkey.PublicKey memory pk2 = Passkey.PublicKey({x: 3, y: 4});
        Passkey.PublicKey memory pk3 = Passkey.PublicKey({x: 5, y: 6});

        vm.startPrank(ownerAddress);
        account.addPasskeySigner(pk1);
        assertEq(account.passkeySignerCount(), 1);

        account.addPasskeySigner(pk2);
        assertEq(account.passkeySignerCount(), 2);

        account.addPasskeySigner(pk3);
        assertEq(account.passkeySignerCount(), 3);

        // Remove one
        account.removePasskeySigner(pk2);
        assertEq(account.passkeySignerCount(), 2);

        // Try to add the same key again (should not increase count)
        account.addPasskeySigner(pk1);
        assertEq(account.passkeySignerCount(), 2);

        // Try to remove non-existent key (should not decrease count)
        account.removePasskeySigner(pk2);
        assertEq(account.passkeySignerCount(), 2);
        vm.stopPrank();
    }

    function test_RootCannotCallRestrictedFunctionsViaUserOpWhenPasskeyExists() public {
        // Add a passkey signer first
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 100, y: 200});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        // Setup root signer
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        vm.prank(ownerAddress);
        account.addRootSigner(root);

        address newRoot = 0x0000000000000000000000000000000000000003;

        // Prepare UserOp that calls addRootSigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addRootSigner(address)", newRoot);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail because passkey signers exist
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_PasskeyValidationForAddRootSignerWhenPasskeyExists() public {
        // Add a passkey signer
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 111, y: 222});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        address newRoot = 0x0000000000000000000000000000000000000004;

        // Prepare UserOp that calls addRootSigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addRootSigner(address)", newRoot);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Create passkey signature (simplified - actual implementation will be done by colleague)
        bytes memory passkeySignature = abi.encodePacked("passkey_signature_placeholder");
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Passkey), passkeySignature);

        // For non-EVM accounts with passkey signers, validation should pass for passkeys
        // (though _validatePasskey currently returns FAILED - will be implemented by colleague)
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        // Currently returns FAILED because _validatePasskey is not implemented
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_PasskeyValidationForRemoveRootSignerWhenPasskeyExists() public {
        // Add a passkey signer
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 111, y: 222});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        // Prepare UserOp that calls removeRootSigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("removeRootSigner(address)", rootAddress);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Create passkey signature
        bytes memory passkeySignature = abi.encodePacked("passkey_signature_placeholder");
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Passkey), passkeySignature);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        // Currently returns FAILED because _validatePasskey is not implemented
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_PasskeyValidationForAddPasskeySignerWhenPasskeyExists() public {
        // Add a passkey signer
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 111, y: 222});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        // Prepare UserOp that calls addPasskeySigner
        Passkey.PublicKey memory newPk = Passkey.PublicKey({x: 333, y: 444});
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addPasskeySigner((uint256,uint256))", newPk);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Create passkey signature
        bytes memory passkeySignature = abi.encodePacked("passkey_signature_placeholder");
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Passkey), passkeySignature);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        // Currently returns FAILED because _validatePasskey is not implemented
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_PasskeyValidationForRemovePasskeySignerWhenPasskeyExists() public {
        // Add a passkey signer
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 111, y: 222});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        // Prepare UserOp that calls removePasskeySigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("removePasskeySigner((uint256,uint256))", pk);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Create passkey signature
        bytes memory passkeySignature = abi.encodePacked("passkey_signature_placeholder");
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Passkey), passkeySignature);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        // Currently returns FAILED because _validatePasskey is not implemented
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_PasskeyValidationForWithdrawDepositWhenPasskeyExists() public {
        // Add a passkey signer
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 111, y: 222});
        vm.prank(ownerAddress);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        // Add some deposit first
        vm.deal(address(account), 1 ether);
        vm.prank(address(account));
        account.addDeposit{value: 0.5 ether}();

        // Prepare UserOp that calls withdrawDepositTo
        address payable withdrawTo = payable(0x0000000000000000000000000000000000000005);
        uint256 withdrawAmount = 0.1 ether;
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData =
            abi.encodeWithSignature("withdrawDepositTo(address,uint256)", withdrawTo, withdrawAmount);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Create passkey signature
        bytes memory passkeySignature = abi.encodePacked("passkey_signature_placeholder");
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Passkey), passkeySignature);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        // Currently returns FAILED because _validatePasskey is not implemented
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }
}
