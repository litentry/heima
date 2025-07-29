// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {OmniAccountTestable} from "./OmniAccountTestable.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";
import {SIG_VALIDATION_SUCCESS, SIG_VALIDATION_FAILED} from "../src/core/Helpers.sol";

contract OmniAccountAsSessionKey is Test {
    OmniAccount public account;
    OmniAccountTestable public testableAccount;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    function setUp() public {
        (counter, entryPoint, account, testableAccount) = OmniAccountTestUtils.setUpTestable(ownerAddress, clientId, rootAddress);
    }

    // ============================================================
    // SESSION KEY VALIDATION TESTS
    // ============================================================

    function test_ValidateSessionKeyDirectValidSignature() public {
        (address sessionKey, uint256 sessionKeyPk) = makeAddrAndKey("sessionKey");
        (address validRoot, uint256 validRootPk) = makeAddrAndKey("validRoot");

        // Add root signer
        vm.prank(ownerAddress);
        testableAccount.addRootSigner(validRoot);

        bytes32 userOpHash = keccak256("test");
        uint256 sessionExpiration = block.timestamp + 1 hours;

        // Create session signature (signed by session key)
        (uint8 sessionV, bytes32 sessionR, bytes32 sessionS) = vm.sign(sessionKeyPk, userOpHash);
        bytes memory sessionSig = abi.encodePacked(sessionR, sessionS, sessionV);

        // Create session proof (root signer signs sessionKey + expiration)
        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        (uint8 proofV, bytes32 proofR, bytes32 proofS) = vm.sign(validRootPk, sessionDigest);
        bytes memory sessionProof = abi.encodePacked(proofR, proofS, proofV);

        // Combine into full signature: sessionSig (65) + expiration (32) + proof (65) = 162 bytes
        bytes memory fullSig = abi.encodePacked(sessionSig, sessionExpiration, sessionProof);

        uint256 validationData = testableAccount.validateSessionKeyPublic(userOpHash, fullSig);

        assertEq(validationData, SIG_VALIDATION_SUCCESS);
    }

    function test_ValidateSessionKeyDirectExpiredSession() public {
        (address sessionKey, uint256 sessionKeyPk) = makeAddrAndKey("sessionKey");
        (address validRoot, uint256 validRootPk) = makeAddrAndKey("validRoot");

        // Add root signer
        vm.prank(ownerAddress);
        testableAccount.addRootSigner(validRoot);

        bytes32 userOpHash = keccak256("test");
        uint256 sessionExpiration = block.timestamp - 1; // Already expired

        // Create session signature
        (uint8 sessionV, bytes32 sessionR, bytes32 sessionS) = vm.sign(sessionKeyPk, userOpHash);
        bytes memory sessionSig = abi.encodePacked(sessionR, sessionS, sessionV);

        // Create session proof
        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        (uint8 proofV, bytes32 proofR, bytes32 proofS) = vm.sign(validRootPk, sessionDigest);
        bytes memory sessionProof = abi.encodePacked(proofR, proofS, proofV);

        bytes memory fullSig = abi.encodePacked(sessionSig, sessionExpiration, sessionProof);

        uint256 validationData = testableAccount.validateSessionKeyPublic(userOpHash, fullSig);

        assertEq(validationData, SIG_VALIDATION_FAILED);
    }

    function test_ValidateSessionKeyDirectInvalidProofSigner() public {
        (address sessionKey, uint256 sessionKeyPk) = makeAddrAndKey("sessionKey");
        (, uint256 invalidProofSignerPk) = makeAddrAndKey("invalidProofSigner");

        bytes32 userOpHash = keccak256("test");
        uint256 sessionExpiration = block.timestamp + 1 hours;

        // Create session signature
        (uint8 sessionV, bytes32 sessionR, bytes32 sessionS) = vm.sign(sessionKeyPk, userOpHash);
        bytes memory sessionSig = abi.encodePacked(sessionR, sessionS, sessionV);

        // Create session proof signed by invalid signer (not a root)
        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        (uint8 proofV, bytes32 proofR, bytes32 proofS) = vm.sign(invalidProofSignerPk, sessionDigest);
        bytes memory sessionProof = abi.encodePacked(proofR, proofS, proofV);

        bytes memory fullSig = abi.encodePacked(sessionSig, sessionExpiration, sessionProof);

        uint256 validationData = testableAccount.validateSessionKeyPublic(userOpHash, fullSig);

        assertEq(validationData, SIG_VALIDATION_FAILED);
    }

    function test_ValidateSessionKeyDirectInvalidSignatureLength() public {
        bytes32 userOpHash = keccak256("test");
        bytes memory shortSig = abi.encodePacked(bytes32(0), bytes32(0), bytes32(0)); // 96 bytes instead of 162

        vm.expectRevert("SessionKey signature length invalid");
        testableAccount.validateSessionKeyPublic(userOpHash, shortSig);
    }

    function test_ValidateSessionKeyDirectEmptySignature() public {
        bytes32 userOpHash = keccak256("test");
        bytes memory emptySig = "";

        vm.expectRevert("SessionKey signature length invalid");
        testableAccount.validateSessionKeyPublic(userOpHash, emptySig);
    }

    function test_ValidateSessionKeyDirectFutureExpiration() public {
        (address sessionKey, uint256 sessionKeyPk) = makeAddrAndKey("sessionKey");
        (address validRoot, uint256 validRootPk) = makeAddrAndKey("validRoot");

        // Add root signer
        vm.prank(ownerAddress);
        testableAccount.addRootSigner(validRoot);

        bytes32 userOpHash = keccak256("test");
        uint256 sessionExpiration = block.timestamp + 1 days; // Far future

        // Create session signature
        (uint8 sessionV, bytes32 sessionR, bytes32 sessionS) = vm.sign(sessionKeyPk, userOpHash);
        bytes memory sessionSig = abi.encodePacked(sessionR, sessionS, sessionV);

        // Create session proof
        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        (uint8 proofV, bytes32 proofR, bytes32 proofS) = vm.sign(validRootPk, sessionDigest);
        bytes memory sessionProof = abi.encodePacked(proofR, proofS, proofV);

        bytes memory fullSig = abi.encodePacked(sessionSig, sessionExpiration, sessionProof);

        uint256 validationData = testableAccount.validateSessionKeyPublic(userOpHash, fullSig);

        assertEq(validationData, SIG_VALIDATION_SUCCESS);
    }

    // Test time manipulation
    function test_ValidateSessionKeyDirectTimeWarp() public {
        (address sessionKey, uint256 sessionKeyPk) = makeAddrAndKey("sessionKey");
        (address validRoot, uint256 validRootPk) = makeAddrAndKey("validRoot");

        // Add root signer
        vm.prank(ownerAddress);
        testableAccount.addRootSigner(validRoot);

        bytes32 userOpHash = keccak256("test");
        uint256 sessionExpiration = block.timestamp + 1 hours;

        // Create signatures
        (uint8 sessionV, bytes32 sessionR, bytes32 sessionS) = vm.sign(sessionKeyPk, userOpHash);
        bytes memory sessionSig = abi.encodePacked(sessionR, sessionS, sessionV);

        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        (uint8 proofV, bytes32 proofR, bytes32 proofS) = vm.sign(validRootPk, sessionDigest);
        bytes memory sessionProof = abi.encodePacked(proofR, proofS, proofV);

        bytes memory fullSig = abi.encodePacked(sessionSig, sessionExpiration, sessionProof);

        // Should work now
        uint256 validationData = testableAccount.validateSessionKeyPublic(userOpHash, fullSig);
        assertEq(validationData, SIG_VALIDATION_SUCCESS);

        // Warp time past expiration
        vm.warp(block.timestamp + 2 hours);

        // Should fail now
        validationData = testableAccount.validateSessionKeyPublic(userOpHash, fullSig);
        assertEq(validationData, SIG_VALIDATION_FAILED);
    }
}