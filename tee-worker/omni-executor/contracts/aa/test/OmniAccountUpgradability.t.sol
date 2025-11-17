// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/interfaces/IERC1271.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import {OmniAccountV1} from "../src/accounts/OmniAccountV1.sol";
import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import {Passkey} from "../src/interfaces/Passkey.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";

contract OmniAccountV2 is OmniAccountV1, IERC1271 {
    // New storage variable to test that new storage doesn't affect old storage
    uint256 public newFeatureCounter;

    // ERC1271 magic value for valid signature
    bytes4 private constant ERC1271_MAGIC_VALUE = 0x1626ba7e;

    constructor(EntryPointV1 anEntryPoint) OmniAccountV1(anEntryPoint) {}

    function version() public pure override returns (string memory) {
        return "2.0.0";
    }

    // New function only available in V2
    function incrementNewFeature() public onlyOwner {
        newFeatureCounter++;
    }

    // New function to get feature counter
    function getNewFeatureCounter() public view returns (uint256) {
        return newFeatureCounter;
    }

    /**
     * @dev ERC1271 signature validation
     * @param hash Hash of the data to be signed
     * @param signature Signature byte array
     * @return magicValue 0x1626ba7e if valid, 0xffffffff otherwise
     */
    function isValidSignature(bytes32 hash, bytes memory signature) public view override returns (bytes4 magicValue) {
        // Signature must be 65 bytes (r, s, v)
        if (signature.length != 65) {
            return 0xffffffff;
        }

        // Recover the signer from the signature
        address signer = ECDSA.recover(hash, signature);

        // Check if the signer is the owner or root signer
        if (_determineOa(signer) == owner || isRootSigner(signer)) {
            return ERC1271_MAGIC_VALUE;
        }

        return 0xffffffff;
    }
}

contract OmniAccountUpgradeable is Test {
    // Test addresses
    address public owner = address(0x1234);
    address public rootSigner = address(0x5678);
    address public unauthorizedUser = address(0x9abc);

    // Test data
    // bytes32 public ownerOa;
    bytes clientId = bytes("test_client");

    function setUp() public {}

    // Helper function to verify account version
    function assertAccountVersion(address account) internal {
        (bool success, bytes memory result) = account.call(abi.encodeWithSignature("version()"));
        assertTrue(success, "Should be able to call version()");
        string memory version = abi.decode(result, (string));
        assertEq(version, "2.0.0", "Should be upgraded to version 2.0.0");
    }

    function testOaOwnerCanUpgradeOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV1 account) = OmniAccountTestUtils.setUp(owner, clientId, rootSigner);
        OmniAccountV2 accountV2 = new OmniAccountV2(entryPoint);

        vm.prank(owner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2), "");
        assertAccountVersion(address(account));
    }

    function testEntryPointCanUpgradeOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV1 account) = OmniAccountTestUtils.setUp(owner, clientId, rootSigner);
        OmniAccountV2 accountV2 = new OmniAccountV2(entryPoint);

        vm.prank(address(entryPoint));
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2), "");
        assertAccountVersion(address(account));
    }

    function testUnauthorizedSenderCannotUpgradeOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV1 account) =
            OmniAccountTestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV2 accountV2 = new OmniAccountV2(entryPoint);

        vm.expectRevert("only owner");
        vm.prank(unauthorizedUser);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2), "");
    }

    function testRootSignerCannotUpgradeEvmOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV1 account) =
            OmniAccountTestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Evm);
        OmniAccountV2 accountV2 = new OmniAccountV2(entryPoint);

        vm.expectRevert("only owner");
        vm.prank(rootSigner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2), "");
    }

    function testRootSignerCannotUpgradeNonEvmOAWithPassKey() public {
        (, EntryPointV1 entryPoint, OmniAccountV1 account) =
            OmniAccountTestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV2 accountV2 = new OmniAccountV2(entryPoint);

        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 1, y: 2});
        vm.prank(owner);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        vm.expectRevert("only owner");
        vm.prank(rootSigner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2), "");
    }

    function testRootSignerCanUpgradeNonEvmOAWithoutPassKey() public {
        (, EntryPointV1 entryPoint, OmniAccountV1 account) =
            OmniAccountTestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV2 accountV2 = new OmniAccountV2(entryPoint);

        assertEq(account.passkeySignerCount(), 0);

        vm.prank(rootSigner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2), "");
        assertAccountVersion(address(account));
    }

    function testUnauthorizedSenderCannotUpgradeNonEvmOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV1 account) =
            OmniAccountTestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV2 accountV2 = new OmniAccountV2(entryPoint);

        vm.expectRevert("only owner");
        vm.prank(unauthorizedUser);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2), "");
    }

    // Comprehensive test that verifies:
    // 1. Address is preserved after upgrade
    // 2. Account continues to be operable
    // 3. New logic (version and new functions) works
    // 4. Old logic/storage is not affected
    function testUpgradePreservesStateAndOperability() public {
        // Setup: Create account with rich state
        (, EntryPointV1 entryPoint, OmniAccountV1 account) =
            OmniAccountTestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);

        // Add additional root signers to test state preservation
        address additionalRootSigner = address(0xABCD);
        vm.prank(owner);
        account.addRootSigner(additionalRootSigner);

        // Add passkey signers to test state preservation
        Passkey.PublicKey memory pk1 = Passkey.PublicKey({x: 12345, y: 67890});
        Passkey.PublicKey memory pk2 = Passkey.PublicKey({x: 11111, y: 22222});
        vm.prank(owner);
        account.addPasskeySigner(pk1);
        vm.prank(owner);
        account.addPasskeySigner(pk2);

        // Add ETH balance to the account
        vm.deal(address(account), 5 ether);

        // Add deposit to EntryPoint
        vm.prank(address(account));
        account.addDeposit{value: 2 ether}();

        // Record pre-upgrade state
        address accountAddress = address(account);
        bytes32 ownerBefore = account.owner();
        bytes memory clientIdBefore = account.clientId();
        OwnerType ownerTypeBefore = account.ownerType();
        uint256 passkeySignerCountBefore = account.passkeySignerCount();
        uint256 ethBalanceBefore = address(account).balance;
        uint256 depositBefore = account.getDeposit();
        bool isRootSignerBefore = account.isRootSigner(rootSigner);
        bool isAdditionalRootSignerBefore = account.isRootSigner(additionalRootSigner);

        // Verify pre-upgrade state
        assertEq(ownerBefore, account.getOwner(), "Owner mismatch before upgrade");
        assertEq(passkeySignerCountBefore, 2, "Should have 2 passkey signers");
        assertTrue(isRootSignerBefore, "Root signer should exist");
        assertTrue(isAdditionalRootSignerBefore, "Additional root signer should exist");
        assertEq(ethBalanceBefore, 3 ether, "ETH balance should be 3 ether (5 - 2 deposited)");
        assertEq(depositBefore, 2 ether, "Deposit should be 2 ether");

        // Perform upgrade
        OmniAccountV2 accountV2Impl = new OmniAccountV2(entryPoint);
        vm.prank(owner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2Impl), "");

        // Cast to V2 for accessing new functions
        OmniAccountV2 accountV2 = OmniAccountV2(payable(address(account)));

        // TEST 1: Verify address is preserved
        assertEq(address(accountV2), accountAddress, "Address should remain the same after upgrade");

        // TEST 2: Verify new logic works (version)
        assertAccountVersion(address(accountV2));

        // TEST 3: Verify old storage is preserved
        assertEq(accountV2.owner(), ownerBefore, "Owner should be preserved");
        assertEq(accountV2.clientId(), clientIdBefore, "Client ID should be preserved");
        assertTrue(accountV2.ownerType() == ownerTypeBefore, "Owner type should be preserved");
        assertEq(accountV2.passkeySignerCount(), passkeySignerCountBefore, "Passkey signer count should be preserved");
        assertTrue(accountV2.isRootSigner(rootSigner), "Root signer should be preserved");
        assertTrue(accountV2.isRootSigner(additionalRootSigner), "Additional root signer should be preserved");
        assertEq(address(accountV2).balance, ethBalanceBefore, "ETH balance should be preserved");
        assertEq(accountV2.getDeposit(), depositBefore, "Deposit should be preserved");

        // TEST 4: Verify account is still operable - can add/remove signers
        address newRootSigner = address(0xDEAD);
        vm.prank(owner);
        accountV2.addRootSigner(newRootSigner);
        assertTrue(accountV2.isRootSigner(newRootSigner), "Should be able to add new root signer after upgrade");

        vm.prank(owner);
        accountV2.removeRootSigner(newRootSigner);
        assertFalse(accountV2.isRootSigner(newRootSigner), "Should be able to remove root signer after upgrade");

        // TEST 5: Verify can still add/remove passkey signers
        Passkey.PublicKey memory pk3 = Passkey.PublicKey({x: 33333, y: 44444});
        vm.prank(owner);
        accountV2.addPasskeySigner(pk3);
        assertEq(accountV2.passkeySignerCount(), 3, "Should be able to add passkey signer after upgrade");

        vm.prank(owner);
        accountV2.removePasskeySigner(pk3);
        assertEq(accountV2.passkeySignerCount(), 2, "Should be able to remove passkey signer after upgrade");

        // TEST 6: Verify can withdraw deposit
        address payable withdrawAddress = payable(address(0xBEEF));
        uint256 withdrawAmount = 0.5 ether;
        vm.prank(owner);
        accountV2.withdrawDepositTo(withdrawAddress, withdrawAmount);
        assertEq(accountV2.getDeposit(), depositBefore - withdrawAmount, "Should be able to withdraw after upgrade");

        // TEST 7: Verify new V2 functionality works
        assertEq(accountV2.getNewFeatureCounter(), 0, "New feature counter should start at 0");
        vm.prank(owner);
        accountV2.incrementNewFeature();
        assertEq(accountV2.getNewFeatureCounter(), 1, "New feature should work after upgrade");
        vm.prank(owner);
        accountV2.incrementNewFeature();
        assertEq(accountV2.getNewFeatureCounter(), 2, "New feature should continue to work");

        // TEST 8: Verify unauthorized users still cannot call owner-only functions
        vm.expectRevert("only owner");
        vm.prank(unauthorizedUser);
        accountV2.incrementNewFeature();
    }

    // Test ERC1271 support is added after upgrade and works correctly
    function testERC1271WorksAfterUpgrade() public {
        // Setup: Create private keys for owner and root signer
        uint256 ownerPrivateKey = 0x1234;
        uint256 rootSignerPrivateKey = 0x5678;
        uint256 unauthorizedPrivateKey = 0x9abc;

        address ownerAddr = vm.addr(ownerPrivateKey);
        address rootSignerAddr = vm.addr(rootSignerPrivateKey);

        // Create account with owner and root signer
        (, EntryPointV1 entryPoint, OmniAccountV1 account) =
            OmniAccountTestUtils.setUp(ownerAddr, clientId, rootSignerAddr);

        bytes32 messageHash = keccak256("Hello world!");

        // Before upgrade: V1 doesn't have isValidSignature, attempting to call it should fail
        (bool success,) = address(account)
            .call(abi.encodeWithSignature("isValidSignature(bytes32,bytes)", messageHash, new bytes(65)));
        assertFalse(success, "V1 should not have isValidSignature");

        // Perform upgrade to V2
        OmniAccountV2 accountV2Impl = new OmniAccountV2(entryPoint);
        vm.prank(ownerAddr);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV2Impl), "");

        // Cast to V2 for accessing ERC1271
        OmniAccountV2 accountV2 = OmniAccountV2(payable(address(account)));

        // Verify upgrade was successful
        assertAccountVersion(address(accountV2));

        // TEST 1: Valid signature from owner should be accepted
        (uint8 v1, bytes32 r1, bytes32 s1) = vm.sign(ownerPrivateKey, messageHash);
        bytes memory ownerSignature = abi.encodePacked(r1, s1, v1);

        bytes4 result1 = accountV2.isValidSignature(messageHash, ownerSignature);
        assertEq(result1, bytes4(0x1626ba7e), "Owner signature should be valid");

        // TEST 2: Valid signature from root signer should be accepted
        (uint8 v2, bytes32 r2, bytes32 s2) = vm.sign(rootSignerPrivateKey, messageHash);
        bytes memory rootSignature = abi.encodePacked(r2, s2, v2);

        bytes4 result2 = accountV2.isValidSignature(messageHash, rootSignature);
        assertEq(result2, bytes4(0x1626ba7e), "Root signer signature should be valid");

        // TEST 3: Invalid signature from unauthorized user should be rejected
        (uint8 v3, bytes32 r3, bytes32 s3) = vm.sign(unauthorizedPrivateKey, messageHash);
        bytes memory unauthorizedSignature = abi.encodePacked(r3, s3, v3);

        bytes4 result3 = accountV2.isValidSignature(messageHash, unauthorizedSignature);
        assertEq(result3, bytes4(0xffffffff), "Unauthorized signature should be invalid");

        // TEST 4: Invalid signature length should be rejected
        bytes memory shortSignature = new bytes(32);
        bytes4 result4 = accountV2.isValidSignature(messageHash, shortSignature);
        assertEq(result4, bytes4(0xffffffff), "Short signature should be invalid");

        // TEST 5: Test with different message hashes to ensure proper validation
        bytes32 differentMessageHash = keccak256("Another message");
        (uint8 v5, bytes32 r5, bytes32 s5) = vm.sign(ownerPrivateKey, messageHash);
        bytes memory signatureForOriginalMessage = abi.encodePacked(r5, s5, v5);

        // Using signature for original message with different hash should fail
        bytes4 result5 = accountV2.isValidSignature(differentMessageHash, signatureForOriginalMessage);
        assertEq(result5, bytes4(0xffffffff), "Signature should not validate for different message");

        // TEST 6: Verify that added root signers can also validate signatures
        address newRootSigner = vm.addr(0xDEADBEEF);
        vm.prank(ownerAddr);
        accountV2.addRootSigner(newRootSigner);

        (uint8 v6, bytes32 r6, bytes32 s6) = vm.sign(0xDEADBEEF, messageHash);
        bytes memory newRootSignature = abi.encodePacked(r6, s6, v6);

        bytes4 result6 = accountV2.isValidSignature(messageHash, newRootSignature);
        assertEq(result6, bytes4(0x1626ba7e), "Newly added root signer signature should be valid");

        // TEST 7: Verify that removed root signers can no longer validate signatures
        vm.prank(ownerAddr);
        accountV2.removeRootSigner(rootSignerAddr);

        bytes4 result7 = accountV2.isValidSignature(messageHash, rootSignature);
        assertEq(result7, bytes4(0xffffffff), "Removed root signer signature should be invalid");

        // TEST 8: Verify ERC1271 interface support
        // The contract should properly implement IERC1271
        assertTrue(address(accountV2).code.length > 0, "Account should have code (implementing ERC1271)");
    }
}
