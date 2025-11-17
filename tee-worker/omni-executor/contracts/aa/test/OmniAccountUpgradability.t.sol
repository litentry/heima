// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OmniAccountV1} from "../src/accounts/OmniAccountV1.sol";
import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import {Passkey} from "../src/interfaces/Passkey.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";

contract OmniAccountV2 is OmniAccountV1 {
    // New storage variable to test that new storage doesn't affect old storage
    uint256 public newFeatureCounter;

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
    function testComprehensiveUpgradePreservesStateAndOperability() public {
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
}
