// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_FAILED} from "../src//core/Helpers.sol";

// add test cases for revert if called by non authorized address

contract OmniAccountTest is Test {
    OmniAccount public account;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    function setUp() public {
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, rootAddress);
    }

    function test_Owner() public view {
        bytes32 expectedOwner = 0xe502d639feeb199ae332376b050307d76d11b32e93b9fd1310127d2af64923fe;
        assertEq(account.owner(), expectedOwner);
    }

    function test_Root() public view {
        address expectedRoot = 0x0000000000000000000000000000000000000001;
        assert(account.isRootSigner(expectedRoot));
    }

    function test_EntryPoint() public view {
        assertEq(address(account.entryPoint()), address(entryPoint));
    }

    function test_Execute_As_Not_Allowed() public {
        vm.expectRevert("account: not Owner or EntryPoint or root");
        account.execute(address(counter), 0, abi.encodeWithSignature("increment()"));
    }

    function test_ExecuteBatch_As_Not_Allowed() public {
        vm.expectRevert("account: not Owner or EntryPoint or root");
        BaseAccount.Call[] memory calls = new BaseAccount.Call[](0);
        account.executeBatch(calls);
    }

    function test_AddRootSigner_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        account.addRootSigner(0x0000000000000000000000000000000000000000);
    }

    function test_RemoveRootSigner_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        account.removeRootSigner(0x0000000000000000000000000000000000000000);
    }

    function test_ValidateOp() public {
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, rootAddress);
        (, uint256 bobPk) = makeAddrAndKey("bob");

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;

        bytes memory initCode = "";

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(bobPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }
}
