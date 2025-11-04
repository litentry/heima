// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OmniAccountV2} from "../../src/accounts/OmniAccountV2.sol";
import {EntryPointV1} from "../../src/core/EntryPointV1.sol";
import {OwnerType} from "../../src/interfaces/OwnerType.sol";
import {Passkey} from "../../src/interfaces/Passkey.sol";
import {OmniAccountV2TestUtils} from "./OmniAccountV2TestUtils.sol";
import {TestUtils} from "../TestUtils.sol";

contract OmniAccountV3 is OmniAccountV2 {
    constructor(EntryPointV1 anEntryPoint) OmniAccountV2(anEntryPoint) {}

    function version() public pure override returns (string memory) {
        return "3.0.0";
    }
}

contract OmniAccountV2Upgradability is Test {
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
        assertEq(version, "3.0.0", "Should be upgraded to version 3.0.0");
    }

    function testOaOwnerCanUpgradeOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV2 account) = OmniAccountV2TestUtils.setUp(owner, clientId, rootSigner);
        OmniAccountV3 accountV3 = new OmniAccountV3(entryPoint);

        vm.prank(owner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV3), "");
        assertAccountVersion(address(account));
    }

    function testEntryPointCanUpgradeOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV2 account) = OmniAccountV2TestUtils.setUp(owner, clientId, rootSigner);
        OmniAccountV3 accountV3 = new OmniAccountV3(entryPoint);

        vm.prank(address(entryPoint));
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV3), "");
        assertAccountVersion(address(account));
    }

    function testUnauthorizedSenderCannotUpgradeOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV2 account) =
            OmniAccountV2TestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV3 accountV3 = new OmniAccountV3(entryPoint);

        vm.expectRevert("only owner");
        vm.prank(unauthorizedUser);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV3), "");
    }

    function testRootSignerCannotUpgradeEvmOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV2 account) =
            OmniAccountV2TestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Evm);
        OmniAccountV3 accountV3 = new OmniAccountV3(entryPoint);

        vm.expectRevert("only owner");
        vm.prank(rootSigner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV3), "");
    }

    function testRootSignerCannotUpgradeNonEvmOAWithPassKey() public {
        (, EntryPointV1 entryPoint, OmniAccountV2 account) =
            OmniAccountV2TestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV3 accountV3 = new OmniAccountV3(entryPoint);

        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 1, y: 2});
        vm.prank(owner);
        account.addPasskeySigner(pk);
        assertEq(account.passkeySignerCount(), 1);

        vm.expectRevert("only owner");
        vm.prank(rootSigner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV3), "");
    }

    function testRootSignerCanUpgradeNonEvmOAWithoutPassKey() public {
        (, EntryPointV1 entryPoint, OmniAccountV2 account) =
            OmniAccountV2TestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV3 accountV3 = new OmniAccountV3(entryPoint);

        assertEq(account.passkeySignerCount(), 0);

        vm.prank(rootSigner);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV3), "");
        assertAccountVersion(address(account));
    }

    function testUnauthorizedSenderCannotUpgradeNonEvmOA() public {
        (, EntryPointV1 entryPoint, OmniAccountV2 account) =
            OmniAccountV2TestUtils.setUpWithOwnerType(owner, clientId, rootSigner, OwnerType.Substrate);
        OmniAccountV3 accountV3 = new OmniAccountV3(entryPoint);

        vm.expectRevert("only owner");
        vm.prank(unauthorizedUser);
        UUPSUpgradeable(account).upgradeToAndCall(address(accountV3), "");
    }
}
