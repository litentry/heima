// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {EntryPoint} from "../src/core/EntryPoint.sol";
import {SmartAccountFactory} from "../src/accounts/SmartAccountFactory.sol";
import {SmartAccount} from "../src/accounts/SmartAccount.sol";
import {Test, console} from "forge-std/Test.sol";
import {TestUtils} from "./TestUtils.sol";

contract SmartAccountFactoryTest is Test {
    EntryPoint public entryPoint;
    SmartAccountFactory public smartAccountFactory;
    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes32 clientId = 0x0000000000000000000000000000000000000000000000000000000000000000;
    bytes32 oa;

    function setUp() public {
        entryPoint = new EntryPoint();
        oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
        smartAccountFactory = new SmartAccountFactory(entryPoint);
    }

    function test_CreateAccountReturnsSameAddress() public {
        address senderCreator = address(entryPoint.senderCreator());
        vm.prank(senderCreator);
        SmartAccount account1 = smartAccountFactory.createAccount(oa, clientId, rootAddress);
        vm.prank(senderCreator);
        SmartAccount account2 = smartAccountFactory.createAccount(oa, clientId, rootAddress);

        assertEq(address(account1), address(account2));
    }
}
