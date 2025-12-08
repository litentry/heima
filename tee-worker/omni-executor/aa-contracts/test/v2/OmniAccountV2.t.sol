// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {OmniAccountV2} from "../../src/accounts/OmniAccountV2.sol";
import {BaseAccount} from "../../src/core/BaseAccount.sol";
import {EntryPointV1} from "../../src/core/EntryPointV1.sol";
import {UserOpSigner} from "../../src/interfaces/UserOpSigner.sol";
import {Counter} from "../../src/Counter.sol";
import {OmniAccountV2TestUtils} from "./OmniAccountV2TestUtils.sol";
import {TestUtils} from "../TestUtils.sol";
import {PackedUserOperation} from "../../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_FAILED} from "../../src//core/Helpers.sol";

// add test cases for revert if called by non authorized address

contract OmniAccountV2Test is Test {
    OmniAccountV2 public account;
    EntryPointV1 public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    bytes4 constant ERC1271_MAGIC_VALUE = 0x1626ba7e;
    bytes4 constant ERC1271_INVALID_SIGNATURE = 0xffffffff;

    function setUp() public {
        (counter, entryPoint, account) = OmniAccountV2TestUtils.setUp(ownerAddress, clientId, rootAddress);
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
        vm.expectRevert("account: not from EntryPoint");
        account.execute(address(counter), 0, abi.encodeWithSignature("increment()"));
    }

    function test_ExecuteBatch_As_Not_Allowed() public {
        vm.expectRevert("account: not from EntryPoint");
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
        (counter, entryPoint, account) = OmniAccountV2TestUtils.setUp(ownerAddress, clientId, rootAddress);
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

    // ============ ERC-1271 Edge Cases and Invalid Input Tests ============

    function test_ERC1271_EmptySignature() public view {
        bytes32 messageHash = keccak256("Test message");
        bytes memory signature = "";

        bytes4 result = account.isValidSignature(messageHash, signature);
        assertEq(result, ERC1271_INVALID_SIGNATURE);
    }

    function test_ERC1271_InvalidSignerType() public {
        bytes32 messageHash = keccak256("Test message");
        (, uint256 pk) = makeAddrAndKey("signer");

        // Use invalid signer type (99) - this will cause a panic in enum conversion
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, messageHash);
        bytes memory signature = abi.encodePacked(uint8(99), r, s, v);

        // Expect a panic (0x21 is the panic code for invalid enum conversion)
        vm.expectRevert();
        account.isValidSignature(messageHash, signature);
    }

    function test_ERC1271_OnlySignerTypeByte() public view {
        bytes32 messageHash = keccak256("Test message");
        bytes memory signature = abi.encodePacked(uint8(UserOpSigner.Owner));

        bytes4 result = account.isValidSignature(messageHash, signature);
        assertEq(result, ERC1271_INVALID_SIGNATURE);
    }

    function test_ERC1271_MultipleSignatureTypesForSameMessage() public {
        (address owner, uint256 ownerPk) = makeAddrAndKey("owner");
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountV2TestUtils.setUp(owner, clientId, root);

        bytes32 messageHash = keccak256("Shared message");

        // Owner signature
        (uint8 v1, bytes32 r1, bytes32 s1) = vm.sign(ownerPk, messageHash);
        bytes memory ownerSig = abi.encodePacked(uint8(UserOpSigner.Owner), r1, s1, v1);
        assertEq(account.isValidSignature(messageHash, ownerSig), ERC1271_MAGIC_VALUE);

        // Root key signature
        (uint8 v2, bytes32 r2, bytes32 s2) = vm.sign(rootPk, messageHash);
        bytes memory rootSig = abi.encodePacked(uint8(UserOpSigner.RootKey), r2, s2, v2);
        assertEq(account.isValidSignature(messageHash, rootSig), ERC1271_MAGIC_VALUE);
    }
}
