// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";
import {SIG_VALIDATION_SUCCESS} from "../src//core/Helpers.sol";

contract OmniAccountAsOwner is Test {
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
        OmniAccountTestUtils.performExecuteTestAs(vm, ownerAddress, account, counter);
    }

    function test_ExecuteBatch() public {
        OmniAccountTestUtils.performExecuteBatchTestAs(vm, ownerAddress, account, counter);
    }

    function test_AddRemoveRootSigner() public {
        address root = 0x0000000000000000000000000000000000000002;
        vm.prank(ownerAddress);
        account.addRootSigner(root);
        assert(account.rootSigners(root));
        vm.prank(ownerAddress);
        account.removeRootSigner(root);
        assert(!account.rootSigners(root));
    }

    function test_ValidateOp() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(alice, clientId, rootAddress);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;

        bytes memory initCode = "";

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }
}
