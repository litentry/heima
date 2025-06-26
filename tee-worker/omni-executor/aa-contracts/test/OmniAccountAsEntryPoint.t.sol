// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";

contract OmniAccountAsEntryPoint is Test {
    OmniAccount public account;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    function setUp() public {
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, rootAddress);
    }

    function test_Execute() public {
        OmniAccountTestUtils.performExecuteTestAs(vm, address(entryPoint), account, counter);
    }

    function test_ExecuteBatch() public {
        OmniAccountTestUtils.performExecuteBatchTestAs(vm, address(entryPoint), account, counter);
    }
}
