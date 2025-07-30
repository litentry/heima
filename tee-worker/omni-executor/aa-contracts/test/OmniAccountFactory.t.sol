// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {EntryPoint} from "../src/core/EntryPoint.sol";
import {OmniAccountFactory} from "../src/accounts/OmniAccountFactory.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestUtils} from "./TestUtils.sol";

contract OmniAccountFactoryTest is Test {
    EntryPoint public entryPoint;
    OmniAccountFactory public omniAccountFactory;
    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");
    bytes32 oa;

    function setUp() public {
        entryPoint = new EntryPoint();
        oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
        omniAccountFactory = new OmniAccountFactory(entryPoint);
    }

    function test_CreateAccountReturnsSameAddress() public {
        address senderCreator = address(entryPoint.senderCreator());
        vm.prank(senderCreator);
        OmniAccount account1 = omniAccountFactory.createAccount(oa, OwnerType.Evm, clientId, rootAddress);
        vm.prank(senderCreator);
        OmniAccount account2 = omniAccountFactory.createAccount(oa, OwnerType.Evm, clientId, rootAddress);

        assertEq(address(account1), address(account2));
    }
}
