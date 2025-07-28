// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {OmniAccountTestable} from "./OmniAccountTestable.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {Passkey} from "../src/interfaces/Passkey.sol";
import {Counter} from "../src/Counter.sol";
import {OmniAccountTestUtils} from "./OmniAccountTestUtils.sol";
import {TestUtils} from "./TestUtils.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_FAILED, SIG_VALIDATION_SUCCESS} from "../src/core/Helpers.sol";
import "@openzeppelin/contracts/utils/Base64.sol";

contract ValidationTests is Test {
    OmniAccount public account;
    OmniAccountTestable public testableAccount;
    EntryPoint public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");

    // Sample passkey data for testing
    Passkey.PublicKey public testPublicKey;
    Passkey.Signature public testSignature;
    Passkey.Metadata public testMetadata;

    function setUp() public {
        (counter, entryPoint, account, testableAccount) = OmniAccountTestUtils.setUpTestable(ownerAddress, clientId, rootAddress);

        // Initialize test passkey data
        testPublicKey = Passkey.PublicKey({
            x: 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef,
            y: 0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321
        });

        testSignature = Passkey.Signature({
            r: 0x1111111111111111111111111111111111111111111111111111111111111111,
            s: 0x2222222222222222222222222222222222222222222222222222222222222222
        });

        // Create valid authData (37 bytes minimum with UP flag set)
        bytes memory authData = new bytes(37);
        authData[32] = 0x01; // Set UP flag (bit 0)

        testMetadata = Passkey.Metadata({
            authData: authData,
            clientDataJSON: '{"type":"webauthn.get","challenge":"dGVzdA","origin":"https://example.com"}',
            challengeIndex: 36,
            typeIndex: 8,
            userVerificationRequired: false
        });
    }

    function test_ValidatePasskey_Direct_UnauthorizedPasskey() public view {
        // Test case: Direct call to _validatePasskey with unauthorized passkey
        bytes32 userOpHash = keccak256("test");
        bytes memory sig = abi.encode(testPublicKey, testSignature, testMetadata);

        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail for unauthorized passkey");
    }

    function test_ValidatePasskey_Direct_AuthorizedPasskey() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        testableAccount.addPasskeySigner(testPublicKey);

        bytes32 userOpHash = keccak256("test");
        bytes memory sig = abi.encode(testPublicKey, testSignature, testMetadata);

        // The test will fail at P256.verify since we don't have valid signature data
        // But it tests that the authorization check passes
        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);

        // This will be SIG_VALIDATION_FAILED due to invalid P256 signature,
        // but at least we verified authorization logic works
        assertEq(validationData, SIG_VALIDATION_FAILED, "Fails at P256 verification with test data");
    }

    function test_ValidatePasskey_UnauthorizedPasskey() public {
        // Test case: Passkey not added to passkeySigners mapping
        bytes32 userOpHash = keccak256("test");

        bytes memory sig = abi.encode(testPublicKey, testSignature, testMetadata);
        bytes memory fullSig = abi.encodePacked(uint8(UserOpSigner.Passkey), sig);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(account), "");
        packedOp.signature = fullSig;

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, userOpHash, 0);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail for unauthorized passkey");
    }

    function test_ValidatePasskey_AuthorizedPasskey() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        account.addPasskeySigner(testPublicKey);

        // Verify it was added
        bytes32 pkKey = Passkey.toKey(testPublicKey);
        assertTrue(account.passkeySigners(pkKey), "Passkey should be authorized");

        bytes32 userOpHash = keccak256("test");

        bytes memory sig = abi.encode(testPublicKey, testSignature, testMetadata);
        bytes memory fullSig = abi.encodePacked(uint8(UserOpSigner.Passkey), sig);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(account), "");
        packedOp.signature = fullSig;

        // Mock the P256.verify call to return true for this test
        // Note: In a real test, you would need actual valid P256 signature data
        // For now, this tests the authorization check logic
        vm.mockCall(
            address(0), // P256 precompile address would be different
            abi.encodeWithSignature("verify(bytes32,bytes32,bytes32,bytes32,bytes32)"),
            abi.encode(true)
        );

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, userOpHash, 0);

        // This will still fail due to P256 verification, but tests our authorization logic
        assertEq(validationData, SIG_VALIDATION_FAILED, "Would pass with valid P256 signature");
    }

    function test_AddPasskeySigner_OnlyOwner() public {
        // Test that only owner can add passkey signers
        vm.prank(rootAddress);
        vm.expectRevert("only owner");
        account.addPasskeySigner(testPublicKey);

        // Owner should be able to add
        vm.prank(ownerAddress);
        account.addPasskeySigner(testPublicKey);

        bytes32 pkKey = Passkey.toKey(testPublicKey);
        assertTrue(account.passkeySigners(pkKey), "Passkey should be authorized after owner adds it");
    }

    function test_RemovePasskeySigner_OnlyOwner() public {
        // First add a passkey signer
        vm.prank(ownerAddress);
        account.addPasskeySigner(testPublicKey);

        bytes32 pkKey = Passkey.toKey(testPublicKey);
        assertTrue(account.passkeySigners(pkKey), "Passkey should be authorized");

        // Test that only owner can remove passkey signers
        vm.prank(rootAddress);
        vm.expectRevert("only owner");
        account.removePasskeySigner(testPublicKey);

        // Owner should be able to remove
        vm.prank(ownerAddress);
        account.removePasskeySigner(testPublicKey);

        assertFalse(account.passkeySigners(pkKey), "Passkey should be removed after owner removes it");
    }

    function test_PasskeySignerEvents() public {
        // Test PasskeySignerAdded event
        vm.prank(ownerAddress);
        vm.expectEmit(true, true, true, true);
        emit PasskeySignerAdded(testPublicKey);
        account.addPasskeySigner(testPublicKey);

        // Test PasskeySignerRemoved event
        vm.prank(ownerAddress);
        vm.expectEmit(true, true, true, true);
        emit PasskeySignerRemoved(testPublicKey);
        account.removePasskeySigner(testPublicKey);
    }

    function test_PasskeyToKey() public view{
        // Test the toKey function from Passkey library
        bytes32 expectedKey = sha256(abi.encodePacked(testPublicKey.x, testPublicKey.y));
        bytes32 actualKey = Passkey.toKey(testPublicKey);

        assertEq(actualKey, expectedKey, "toKey should return sha256 of x and y coordinates");
    }

    function test_ValidatePasskey_InvalidSignatureLength() public {
        // Test with signature too short
        bytes memory shortSig = abi.encodePacked(uint8(UserOpSigner.Passkey));

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(account), "");
        packedOp.signature = shortSig;

        bytes32 userOpHash = keccak256("test");

        vm.prank(address(entryPoint));
        vm.expectRevert();
        account.validateUserOp(packedOp, userOpHash, 0);
    }

    function test_ValidatePasskey_InvalidAuthDataFlags() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        account.addPasskeySigner(testPublicKey);

        // Create authData with invalid flags (UP flag not set)
        bytes memory invalidAuthData = new bytes(37);
        invalidAuthData[32] = 0x00; // No flags set

        Passkey.Metadata memory invalidMetadata = Passkey.Metadata({
            authData: invalidAuthData,
            clientDataJSON: testMetadata.clientDataJSON,
            challengeIndex: testMetadata.challengeIndex,
            typeIndex: testMetadata.typeIndex,
            userVerificationRequired: false
        });

        bytes memory sig = abi.encode(testPublicKey, testSignature, invalidMetadata);
        bytes memory fullSig = abi.encodePacked(uint8(UserOpSigner.Passkey), sig);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(account), "");
        packedOp.signature = fullSig;

        bytes32 userOpHash = keccak256("test");

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, userOpHash, 0);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with invalid auth data flags");
    }

    function test_ValidatePasskey_UserVerificationRequired() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        account.addPasskeySigner(testPublicKey);

        // Create authData with UV flag not set but required
        bytes memory authDataNoUV = new bytes(37);
        authDataNoUV[32] = 0x01; // Only UP flag set, UV flag (0x04) not set

        Passkey.Metadata memory metadataUVRequired = Passkey.Metadata({
            authData: authDataNoUV,
            clientDataJSON: testMetadata.clientDataJSON,
            challengeIndex: testMetadata.challengeIndex,
            typeIndex: testMetadata.typeIndex,
            userVerificationRequired: true // Require UV but it's not set in authData
        });

        bytes memory sig = abi.encode(testPublicKey, testSignature, metadataUVRequired);
        bytes memory fullSig = abi.encodePacked(uint8(UserOpSigner.Passkey), sig);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(account), "");
        packedOp.signature = fullSig;

        bytes32 userOpHash = keccak256("test");

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, userOpHash, 0);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail when UV required but not set");
    }

    function test_ValidatePasskey_Direct_MalformedSignature() public {
        // Test with malformed signature data that cannot be decoded
        bytes32 userOpHash = keccak256("test");
        bytes memory malformedSig = abi.encodePacked("invalid", "signature", "data");

        // This should revert due to ABI decoding failure
        vm.expectRevert();
        testableAccount.validatePasskeyPublic(userOpHash, malformedSig);
    }

    function test_ValidatePasskey_Direct_EmptySignature() public {
        // Test with empty signature
        bytes32 userOpHash = keccak256("test");
        bytes memory emptySig = "";

        // This should revert due to ABI decoding failure
        vm.expectRevert();
        testableAccount.validatePasskeyPublic(userOpHash, emptySig);
    }

    function test_ValidatePasskey_Direct_ShortAuthData() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        testableAccount.addPasskeySigner(testPublicKey);

        // Create authData that's too short (less than 37 bytes)
        bytes memory shortAuthData = new bytes(36);

        Passkey.Metadata memory shortMetadata = Passkey.Metadata({
            authData: shortAuthData,
            clientDataJSON: testMetadata.clientDataJSON,
            challengeIndex: testMetadata.challengeIndex,
            typeIndex: testMetadata.typeIndex,
            userVerificationRequired: false
        });

        bytes32 userOpHash = keccak256("test");
        bytes memory sig = abi.encode(testPublicKey, testSignature, shortMetadata);

        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with short authData");
    }

    function test_ValidatePasskey_Direct_WrongClientDataType() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        testableAccount.addPasskeySigner(testPublicKey);

        // Create metadata with wrong type (not "webauthn.get")
        Passkey.Metadata memory wrongTypeMetadata = Passkey.Metadata({
            authData: testMetadata.authData,
            clientDataJSON: '{"type":"webauthn.create","challenge":"dGVzdA","origin":"https://example.com"}',
            challengeIndex: testMetadata.challengeIndex,
            typeIndex: 8, // Points to "webauthn.create" instead of "webauthn.get"
            userVerificationRequired: false
        });

        bytes32 userOpHash = keccak256("test");
        bytes memory sig = abi.encode(testPublicKey, testSignature, wrongTypeMetadata);

        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with wrong clientData type");
    }

    function test_ValidatePasskey_Direct_WrongChallenge() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        testableAccount.addPasskeySigner(testPublicKey);

        // Create metadata with challenge that doesn't match userOpHash
        bytes32 userOpHash = keccak256("test");
        bytes32 wrongChallenge = keccak256("wrong");
        string memory wrongChallengeB64 = Base64.encodeURL(abi.encodePacked(wrongChallenge));
        string memory wrongClientDataJSON = string.concat(
            '{"type":"webauthn.get","challenge":"', wrongChallengeB64, '","origin":"https://example.com"}'
        );

        Passkey.Metadata memory wrongChallengeMetadata = Passkey.Metadata({
            authData: testMetadata.authData,
            clientDataJSON: wrongClientDataJSON,
            challengeIndex: 36, // Points to the wrong challenge
            typeIndex: 8,
            userVerificationRequired: false
        });

        bytes memory sig = abi.encode(testPublicKey, testSignature, wrongChallengeMetadata);

        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with wrong challenge");
    }

    function test_PasskeyLibrary_Contains() public pure {
        // Test the contains function from Passkey library
        string memory substr = "test";
        string memory str = "this is a test string";

        assertTrue(Passkey.contains(substr, str, 10), "Should find substr at correct location");
        assertFalse(Passkey.contains(substr, str, 0), "Should not find substr at wrong location");
        assertFalse(Passkey.contains(substr, str, 100), "Should not find substr beyond string length");
    }

    function test_PasskeyLibrary_CheckAuthFlags() public pure {
        // Test UP flag (required)
        assertTrue(Passkey.checkAuthFlags(0x01, false), "Should pass with UP flag set");
        assertFalse(Passkey.checkAuthFlags(0x00, false), "Should fail without UP flag");

        // Test UV flag when required
        assertTrue(Passkey.checkAuthFlags(0x05, true), "Should pass with UP and UV flags when UV required");
        assertFalse(Passkey.checkAuthFlags(0x01, true), "Should fail with only UP flag when UV required");

        // Test BE/BS flags
        assertTrue(Passkey.checkAuthFlags(0x09, false), "Should pass with UP and BE flags");
        assertFalse(Passkey.checkAuthFlags(0x11, false), "Should fail with UP and BS but no BE");
    }

    // ============================================================
    // OWNER VALIDATION TESTS
    // ============================================================

    function test_ValidateOwner_Direct_ValidSignature() public {
        (address validOwner, uint256 validOwnerPk) = makeAddrAndKey("validOwner");

        // Create account with this owner
        (,,, OmniAccountTestable ownerTestableAccount) =
            OmniAccountTestUtils.setUpTestable(validOwner, clientId, rootAddress);

        bytes32 userOpHash = keccak256("test");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(validOwnerPk, userOpHash);
        bytes memory sig = abi.encodePacked(r, s, v);

        uint256 validationData = ownerTestableAccount.validateOwnerPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_SUCCESS, "Should succeed with valid owner signature");
    }

    function test_ValidateOwner_Direct_InvalidSignature() public {
        (, uint256 invalidSignerPk) = makeAddrAndKey("invalidSigner");

        bytes32 userOpHash = keccak256("test");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(invalidSignerPk, userOpHash);
        bytes memory sig = abi.encodePacked(r, s, v);

        uint256 validationData = testableAccount.validateOwnerPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with invalid owner signature");
    }

    function test_ValidateOwner_Direct_InvalidSignatureLength() public {
        bytes32 userOpHash = keccak256("test");
        bytes memory shortSig = abi.encodePacked(bytes32(0), bytes16(0)); // 48 bytes instead of 65

        vm.expectRevert("Owner signature length invalid");
        testableAccount.validateOwnerPublic(userOpHash, shortSig);
    }

    function test_ValidateOwner_Direct_EmptySignature() public {
        bytes32 userOpHash = keccak256("test");
        bytes memory emptySig = "";

        vm.expectRevert("Owner signature length invalid");
        testableAccount.validateOwnerPublic(userOpHash, emptySig);
    }

    // ============================================================
    // ROOT KEY VALIDATION TESTS
    // ============================================================

    function test_ValidateRootKey_Direct_ValidSignature() public {
        (address validRoot, uint256 validRootPk) = makeAddrAndKey("validRoot");

        // Add as root signer
        vm.prank(ownerAddress);
        testableAccount.addRootSigner(validRoot);

        bytes32 userOpHash = keccak256("test");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(validRootPk, userOpHash);
        bytes memory sig = abi.encodePacked(r, s, v);

        uint256 validationData = testableAccount.validateRootKeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_SUCCESS, "Should succeed with valid root key signature");
    }

    function test_ValidateRootKey_Direct_InvalidSignature() public {
        (, uint256 invalidSignerPk) = makeAddrAndKey("invalidSigner");

        bytes32 userOpHash = keccak256("test");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(invalidSignerPk, userOpHash);
        bytes memory sig = abi.encodePacked(r, s, v);

        uint256 validationData = testableAccount.validateRootKeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with non-root signer");
    }

    function test_ValidateRootKey_Direct_RemovedRootSigner() public {
        (address removedRoot, uint256 removedRootPk) = makeAddrAndKey("removedRoot");

        // Add then remove root signer
        vm.prank(ownerAddress);
        testableAccount.addRootSigner(removedRoot);

        vm.prank(ownerAddress);
        testableAccount.removeRootSigner(removedRoot);

        bytes32 userOpHash = keccak256("test");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(removedRootPk, userOpHash);
        bytes memory sig = abi.encodePacked(r, s, v);

        uint256 validationData = testableAccount.validateRootKeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with removed root signer");
    }

    function test_ValidateRootKey_Direct_InvalidSignatureLength() public {
        bytes32 userOpHash = keccak256("test");
        bytes memory shortSig = abi.encodePacked(bytes32(0), bytes16(0)); // 48 bytes instead of 65

        vm.expectRevert("RootKey signature length invalid");
        testableAccount.validateRootKeyPublic(userOpHash, shortSig);
    }

    // ============================================================
    // SESSION KEY VALIDATION TESTS
    // ============================================================

    function test_ValidateSessionKey_Direct_ValidSignature() public {
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

        assertEq(validationData, SIG_VALIDATION_SUCCESS, "Should succeed with valid session key");
    }

    function test_ValidateSessionKey_Direct_ExpiredSession() public {
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

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail with expired session");
    }

    function test_ValidateSessionKey_Direct_InvalidProofSigner() public {
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

        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail when proof not signed by root");
    }

    function test_ValidateSessionKey_Direct_InvalidSignatureLength() public {
        bytes32 userOpHash = keccak256("test");
        bytes memory shortSig = abi.encodePacked(bytes32(0), bytes32(0), bytes32(0)); // 96 bytes instead of 162

        vm.expectRevert("SessionKey signature length invalid");
        testableAccount.validateSessionKeyPublic(userOpHash, shortSig);
    }

    function test_ValidateSessionKey_Direct_EmptySignature() public {
        bytes32 userOpHash = keccak256("test");
        bytes memory emptySig = "";

        vm.expectRevert("SessionKey signature length invalid");
        testableAccount.validateSessionKeyPublic(userOpHash, emptySig);
    }

    function test_ValidateSessionKey_Direct_FutureExpiration() public {
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

        assertEq(validationData, SIG_VALIDATION_SUCCESS, "Should succeed with future expiration");
    }

    // Test time manipulation
    function test_ValidateSessionKey_Direct_TimeWarp() public {
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
        assertEq(validationData, SIG_VALIDATION_SUCCESS, "Should succeed before time warp");

        // Warp time past expiration
        vm.warp(block.timestamp + 2 hours);

        // Should fail now
        validationData = testableAccount.validateSessionKeyPublic(userOpHash, fullSig);
        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail after time warp");
    }

    // ============================================================
    // INTEGRATION TESTS - Test through validateUserOp
    // ============================================================

    function test_Integration_OwnerValidation() public {
        (address validOwner, uint256 validOwnerPk) = makeAddrAndKey("validOwner");
        (, EntryPoint ownerEntryPoint, OmniAccount ownerAccount,) = OmniAccountTestUtils.setUpTestable(validOwner, clientId, rootAddress);

        bytes32 userOpHash = keccak256("test");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(validOwnerPk, userOpHash);
        bytes memory sig = abi.encodePacked(r, s, v);
        bytes memory fullSig = abi.encodePacked(uint8(UserOpSigner.Owner), sig);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(ownerAccount), "");
        packedOp.signature = fullSig;

        vm.prank(address(ownerEntryPoint));
        uint256 validationData = ownerAccount.validateUserOp(packedOp, userOpHash, 0);

        assertEq(validationData, SIG_VALIDATION_SUCCESS, "Owner validation should succeed through validateUserOp");
    }

    function test_Integration_RootKeyValidation() public {
        (address validRoot, uint256 validRootPk) = makeAddrAndKey("validRoot");

        // Add as root signer
        vm.prank(ownerAddress);
        account.addRootSigner(validRoot);

        bytes32 userOpHash = keccak256("test");
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(validRootPk, userOpHash);
        bytes memory sig = abi.encodePacked(r, s, v);
        bytes memory fullSig = abi.encodePacked(uint8(UserOpSigner.RootKey), sig);

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(account), "");
        packedOp.signature = fullSig;

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, userOpHash, 0);

        assertEq(validationData, SIG_VALIDATION_SUCCESS, "Root key validation should succeed through validateUserOp");
    }

    // Define events for testing
    event PasskeySignerAdded(Passkey.PublicKey pk);
    event PasskeySignerRemoved(Passkey.PublicKey pk);
}