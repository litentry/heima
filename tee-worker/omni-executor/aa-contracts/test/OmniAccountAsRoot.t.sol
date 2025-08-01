// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccountV1} from "../src/accounts/OmniAccountV1.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_SUCCESS, SIG_VALIDATION_FAILED} from "../src//core/Helpers.sol";
import {Passkey} from "../src/interfaces/Passkey.sol";

contract OmniAccountAsRoot is Test {
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
        vm.expectRevert("account: not Owner or EntryPoint");
        vm.prank(rootAddress);
        account.execute(address(counter), 0, abi.encodeWithSignature("increment()"));
    }

    function test_ExecuteBatch() public {
        vm.expectRevert("account: not Owner or EntryPoint");
        vm.prank(rootAddress);
        BaseAccount.Call[] memory calls = new BaseAccount.Call[](1);
        calls[0] = BaseAccount.Call({target: address(counter), value: 0, data: abi.encodeWithSignature("increment()")});
        account.executeBatch(calls);
    }

    function test_validateOp() public {
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

    function test_validateOpFailsWithWrongSigner() public {
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

    function test_validateOpSessionKey() public {
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

    function test_validateOpExpiredSessionKey() public {
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

    function test_validateOpSessionKeyFailsIfProofNotSignedByRoot() public {
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

    function test_RootCannotCallAddRootSignerViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address newRoot = 0x0000000000000000000000000000000000000002;

        // Prepare UserOp that calls addRootSigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addRootSigner(address)", newRoot);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail because root is trying to call a restricted function
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCannotCallRemoveRootSignerViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        // Prepare UserOp that calls removeRootSigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("removeRootSigner(address)", root);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_SessionKeyCannotCallAddRootSignerViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");
        uint256 sessionExpiration = block.timestamp + 1000;

        bytes memory sessionProof = prepareSession(session, sessionExpiration, rootPk);

        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address newRoot = 0x0000000000000000000000000000000000000002;

        // Prepare UserOp that calls addRootSigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("addRootSigner(address)", newRoot);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with session key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.SessionKey), r, s, v, sessionExpiration, sessionProof);

        // Validation should fail because session key is trying to call a restricted function
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCannotExecuteBatchViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        // Prepare UserOp that calls executeBatch
        address sender = address(account);
        bytes memory initCode = "";

        BaseAccount.Call[] memory calls = new BaseAccount.Call[](1);
        calls[0] = BaseAccount.Call({target: address(counter), value: 0, data: abi.encodeWithSignature("increment()")});
        bytes memory callData = abi.encodeWithSignature("executeBatch((address,uint256,bytes)[])", calls);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail because root cannot use executeBatch
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCannotCallAddRootSignerViaExecute() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address newRoot = 0x0000000000000000000000000000000000000002;

        // Prepare UserOp that calls execute() with addRootSigner as inner call
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory innerCallData = abi.encodeWithSignature("addRootSigner(address)", newRoot);
        bytes memory callData =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(account), 0, innerCallData);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail because root is trying to call a restricted function via execute
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_SessionKeyCannotCallAddRootSignerViaExecute() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (address session, uint256 sessionPk) = makeAddrAndKey("session");
        uint256 sessionExpiration = block.timestamp + 1000;

        bytes memory sessionProof = prepareSession(session, sessionExpiration, rootPk);

        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address newRoot = 0x0000000000000000000000000000000000000002;

        // Prepare UserOp that calls execute() with addRootSigner as inner call
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory innerCallData = abi.encodeWithSignature("addRootSigner(address)", newRoot);
        bytes memory callData =
            abi.encodeWithSignature("execute(address,uint256,bytes)", address(account), 0, innerCallData);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with session key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sessionPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.SessionKey), r, s, v, sessionExpiration, sessionProof);

        // Validation should fail because session key is trying to call a restricted function via execute
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCannotCallAddPasskeySignerViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});

        // Prepare UserOp that calls addPasskeySigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSelector(account.addPasskeySigner.selector, pk);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail because root is trying to call a restricted function
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCannotCallRemovePasskeySignerViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});

        // Prepare UserOp that calls removePasskeySigner
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSelector(account.removePasskeySigner.selector, pk);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCannotCallWithdrawDepositToViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        // Add some deposit first
        vm.deal(address(account), 1 ether);
        vm.prank(address(account));
        account.addDeposit{value: 0.5 ether}();

        address payable withdrawTo = payable(0x0000000000000000000000000000000000000003);
        uint256 withdrawAmount = 0.1 ether;

        // Prepare UserOp that calls withdrawDepositTo
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData =
            abi.encodeWithSignature("withdrawDepositTo(address,uint256)", withdrawTo, withdrawAmount);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail because root is trying to call a restricted function
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_RootCannotCallUpgradeToAndCallViaUserOp() public {
        (address root, uint256 rootPk) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountTestUtils.setUp(ownerAddress, clientId, root);

        address newImplementation = address(0x1234567890123456789012345678901234567890);
        bytes memory data = "";

        // Prepare UserOp that calls upgradeToAndCall
        address sender = address(account);
        bytes memory initCode = "";
        bytes memory callData = abi.encodeWithSignature("upgradeToAndCall(address,bytes)", newImplementation, data);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        packedOp.callData = callData;

        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Sign with root key
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(rootPk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.RootKey), r, s, v);

        // Validation should fail because root is trying to call a restricted function
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function test_AddPasskeySigner_As_Not_Allowed() public {
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});
        vm.expectRevert("only owner");
        account.addPasskeySigner(pk);
    }

    function test_RemovePasskeySigner_As_Not_Allowed() public {
        Passkey.PublicKey memory pk = Passkey.PublicKey({x: 12345, y: 67890});
        vm.expectRevert("only owner");
        account.removePasskeySigner(pk);
    }

    function test_WithdrawDepositTo_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        account.withdrawDepositTo(payable(address(0x1)), 100);
    }

    function test_UpgradeToAndCall_As_Not_Allowed() public {
        vm.expectRevert("only owner");
        account.upgradeToAndCall(address(0x1), "");
    }
}
