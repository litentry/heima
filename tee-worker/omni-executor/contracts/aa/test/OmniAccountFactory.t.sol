// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {OmniAccountFactoryV1} from "../src/accounts/OmniAccountFactoryV1.sol";
import {OmniAccountV1} from "../src/accounts/OmniAccountV1.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestUtils} from "./TestUtils.sol";

contract OmniAccountFactoryTest is Test {
    EntryPointV1 public entryPoint;
    OmniAccountFactoryV1 public omniAccountFactory;
    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");
    bytes32 oa;

    function setUp() public {
        entryPoint = new EntryPointV1();
        oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
        omniAccountFactory = new OmniAccountFactoryV1(entryPoint);
    }

    function test_CreateAccountReturnsSameAddress() public {
        address senderCreator = address(entryPoint.senderCreator());
        vm.prank(senderCreator);
        OmniAccountV1 account1 = omniAccountFactory.createAccount(oa, OwnerType.Evm, clientId, rootAddress);
        vm.prank(senderCreator);
        OmniAccountV1 account2 = omniAccountFactory.createAccount(oa, OwnerType.Evm, clientId, rootAddress);

        assertEq(address(account1), address(account2));
    }
}
