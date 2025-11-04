// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {OmniAccountV1} from "../../src/accounts/OmniAccountV1.sol";
import {BaseAccount} from "../../src/core/BaseAccount.sol";
import {EntryPointV1} from "../../src/core/EntryPointV1.sol";
import {Counter} from "../../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";

contract OmniAccountAsEntryPoint is Test {
    OmniAccountV1 public account;
    EntryPointV1 public entryPoint;
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
