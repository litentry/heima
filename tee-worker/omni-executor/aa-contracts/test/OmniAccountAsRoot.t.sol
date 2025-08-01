// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_SUCCESS, SIG_VALIDATION_FAILED} from "../src//core/Helpers.sol";

contract OmniAccountAsRoot is Test {
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
        OmniAccountTestUtils.performExecuteTestAs(vm, rootAddress, account, counter);
    }

    function test_ExecuteBatch() public {
        OmniAccountTestUtils.performExecuteBatchTestAs(vm, rootAddress, account, counter);
    }

    function test_ValidateOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_ValidateOpFailsWithWrongSigner() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v); // The signer is wrong here

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_ValidateOpSessionKey() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");
        uint256 sessionExpiration = 2;

        bytes memory sessionProof = prepareSession(session, sessionExpiration, rootPk);

        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.SessionKey), r, s, v, sessionExpiration, sessionProof);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_ValidateOpExpiredSessionKey() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");
        uint256 sessionExpiration = 0;

        bytes memory sessionProof = prepareSession(session, sessionExpiration, rootPk);

        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.SessionKey), r, s, v, sessionExpiration, sessionProof);

        vm.prank(address(entryPoint));

        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_ValidateOpSessionKeyFailsIfProofNotSignedByRoot() public {
        (, uint256 alicePk) = makeAddrAndKey("alice");
        (address root,) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");
        uint256 sessionExpiration = 2;

        bytes memory sessionProof = prepareSession(session, sessionExpiration, alicePk);

        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.SessionKey), r, s, v, sessionExpiration, sessionProof);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_AddRootSigner_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        account.addRootSigner(0x0000000000000000000000000000000000000000);
    }

    function test_RemoveRootSigner_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        account.removeRootSigner(0x0000000000000000000000000000000000000000);
    }


    function prepareSession(address session, uint256 expiration, uint256 proofSigner)
        internal
        pure
        returns (bytes memory)
    {
        bytes32 sessionDigest = sha256(abi.encodePacked(session, expiration));
        (uint8 sv, bytes32 sr, bytes32 ss) = vm.sign(proofSigner, sessionDigest);
        bytes memory sessionProof = abi.encodePacked(sr, ss, sv);
        return sessionProof;
    }
}
