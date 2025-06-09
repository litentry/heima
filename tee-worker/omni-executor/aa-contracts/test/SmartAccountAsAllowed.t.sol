// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SmartAccount} from "../src/accounts/SmartAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {Counter} from "../src/Counter.sol";
import {SmartAccountTestUtils} from "./SmartAccountTestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";
import {SIG_VALIDATION_SUCCESS} from "../src//core/Helpers.sol";

contract SmartAccountAsAllowed is Test {
    SmartAccount public account;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes32 clientId = 0x0000000000000000000000000000000000000000000000000000000000000000;
    address allowedAddress;
    uint256 allowedPk;

    function setUp() public {
        (counter, entryPoint, account) = SmartAccountTestUtils.setUp(ownerAddress, clientId, rootAddress);
        (allowedAddress, allowedPk) = makeAddrAndKey("allowed");
    }

    function test_Execute() public {
        SmartAccountTestUtils.performExecuteTestAs(vm, ownerAddress, account, counter);
    }

    function test_ExecuteBatch() public {
        SmartAccountTestUtils.performExecuteBatchTestAs(vm, ownerAddress, account, counter);
    }

    function test_AddAllowedSigner_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        vm.prank(allowedAddress);
        account.addAllowedSigner(0x0000000000000000000000000000000000000000);
    }

    function test_RemoveAllowedSigner_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        vm.prank(allowedAddress);
        account.removeAllowedSigner(0x0000000000000000000000000000000000000000);
    }

    function test_validateOp() public {
        (counter, entryPoint, account) = SmartAccountTestUtils.setUp(ownerAddress, clientId, rootAddress);

        vm.prank(ownerAddress);
        account.addAllowedSigner(allowedAddress);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;

        bytes memory initCode = "";

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(allowedPk, packedOpHash);
        packedOp.signature = abi.encodePacked(r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }
}
