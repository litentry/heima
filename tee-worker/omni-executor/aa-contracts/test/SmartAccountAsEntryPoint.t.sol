// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SmartAccount} from "../src/accounts/SmartAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {Counter} from "../src/Counter.sol";
import {SmartAccountTestUtils} from "./SmartAccountTestUtils.sol";

contract SmartAccountAsEntryPoint is Test {
    SmartAccount public account;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;

    function setUp() public {
        (counter, entryPoint, account) = SmartAccountTestUtils.setUp(ownerAddress, rootAddress);
    }

    function test_Execute() public {
        SmartAccountTestUtils.performExecuteTestAs(vm, address(entryPoint), account, counter);
    }

    function test_ExecuteBatch() public {
        SmartAccountTestUtils.performExecuteBatchTestAs(vm, address(entryPoint), account, counter);
    }
}
