// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SmartAccount} from "../src/accounts/SmartAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {Counter} from "../src/Counter.sol";
import {SmartAccountTestUtils} from "./SmartAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_SUCCESS, SIG_VALIDATION_FAILED} from "../src//core/Helpers.sol";

contract SmartAccountAsRoot is Test {
    SmartAccount public account;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes32 clientId = 0x0000000000000000000000000000000000000000000000000000000000000000;

    function setUp() public {
        (counter, entryPoint, account) = SmartAccountTestUtils.setUp(ownerAddress, clientId, rootAddress);
    }

    function test_Execute() public {
        SmartAccountTestUtils.performExecuteTestAs(vm, rootAddress, account, counter);
    }

    function test_ExecuteBatch() public {
        SmartAccountTestUtils.performExecuteBatchTestAs(vm, rootAddress, account, counter);
    }

    function test_validateOp() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = SmartAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";

        address session_account = 0x0000000000000000000000000000000000000000;
        bytes memory session_account_proof = "";
        PackedUserOperation memory packedOp =
            TestUtils.preparePackedOp(sender, initCode, session_account, session_account_proof);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_validateOpSessionKey() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");

        bytes memory session_proof = prepareSession(session, rootPk);

        (counter, entryPoint, account) = SmartAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";
        bytes memory session_account_proof = "";
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode, session, session_proof);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_validateOpSessionKeyFailsIfProofNotSignedByRoot() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");

        bytes memory session_proof = prepareSession(session, alicePk);

        (counter, entryPoint, account) = SmartAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";
        bytes memory session_account_proof = "";
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode, session, session_proof);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function prepareSession(address session, uint256 proofSigner) internal returns (bytes memory) {
        bytes32 session_digest = sha256(abi.encodePacked(session));
        (uint8 sv, bytes32 sr, bytes32 ss) = vm.sign(proofSigner, session_digest);
        bytes memory session_proof = abi.encodePacked(sr, ss, sv);
        return session_proof;
    }
}
