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
    constructor(EntryPointV1 anEntryPoint) OmniAccountV1(anEntryPoint) {}

    function version() public pure override returns (string memory) {
        return "2.0.0";
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
}
