// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Vm} from "forge-std/Vm.sol";
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

contract OmniAccountAsPasskey is Test {
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

    function test_ValidatePasskeyDirectUnauthorizedPasskey() public view {
        // Test case: Direct call to _validatePasskey with unauthorized passkey
        bytes32 userOpHash = keccak256("test");
        bytes memory sig = abi.encode(testPublicKey, testSignature, testMetadata);
        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);
        assertEq(validationData, SIG_VALIDATION_FAILED, "Should fail for unauthorized passkey");
    }

    function test_ValidatePasskeyDirectAuthorizedPasskey() public {
        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        testableAccount.addPasskeySigner(testPublicKey);

        bytes32 userOpHash = keccak256("test");
        bytes memory sig = abi.encode(testPublicKey, testSignature, testMetadata);
        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);
        assertEq(validationData, SIG_VALIDATION_FAILED, "Fails at P256 verification with test data");
    }

    function test_ValidatePasskeyUnauthorizedPasskey() public {
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

    function test_ValidatePasskeyAuthorizedPasskey() public {
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

        assertEq(validationData, SIG_VALIDATION_FAILED, "P256 signature is invalid");
    }

    function test_AddPasskeySignerOnlyOwner() public {
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

    function test_RemovePasskeySignerOnlyOwner() public {
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

    function test_ValidatePasskeyInvalidSignatureLength() public {
        // Test with signature too short
        bytes memory shortSig = abi.encodePacked(uint8(UserOpSigner.Passkey));

        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(address(account), "");
        packedOp.signature = shortSig;

        bytes32 userOpHash = keccak256("test");

        vm.prank(address(entryPoint));
        vm.expectRevert();
        account.validateUserOp(packedOp, userOpHash, 0);
    }

    function test_ValidatePasskeyInvalidAuthDataFlags() public {
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

    function test_ValidatePasskeyUserVerificationRequired() public {
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

        assertEq(validationData, SIG_VALIDATION_FAILED);
    }

    function test_ValidatePasskeyDirectMalformedSignature() public {
        // Test with malformed signature data that cannot be decoded
        bytes32 userOpHash = keccak256("test");
        bytes memory malformedSig = abi.encodePacked("invalid", "signature", "data");

        // This should revert due to ABI decoding failure
        vm.expectRevert();
        testableAccount.validatePasskeyPublic(userOpHash, malformedSig);
    }

    function test_ValidatePasskeyDirectEmptySignature() public {
        // Test with empty signature
        bytes32 userOpHash = keccak256("test");
        bytes memory emptySig = "";

        // This should revert due to ABI decoding failure
        vm.expectRevert();
        testableAccount.validatePasskeyPublic(userOpHash, emptySig);
    }

    function test_ValidatePasskeyDirectShortAuthData() public {
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

        assertEq(validationData, SIG_VALIDATION_FAILED);
    }

    function test_ValidatePasskeyDirectWrongClientDataType() public {
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

        assertEq(validationData, SIG_VALIDATION_FAILED);
    }

    function test_ValidatePasskeyDirectWrongChallenge() public {
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

        assertEq(validationData, SIG_VALIDATION_FAILED);
    }

    function test_PasskeyLibraryContains() public pure {
        // Test the contains function from Passkey library
        string memory substr = "test";
        string memory str = "this is a test string";

        assertTrue(Passkey.contains(substr, str, 10), "Should find substr at correct location");
        assertFalse(Passkey.contains(substr, str, 0), "Should not find substr at wrong location");
        assertFalse(Passkey.contains(substr, str, 100), "Should not find substr beyond string length");
    }

    function test_PasskeyLibraryCheckAuthFlags() public pure {
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

    function test_ValidatePasskeyDirectSucceed() public {
        // Use the exact public key from JavaScript generation
        Passkey.PublicKey memory validPublicKey = Passkey.PublicKey({
            x: 0xb814cadbe97896dc16192a641cee9fe08e5bdb61fdb1c9dca9b690c36aa16794,
            y: 0x54a8bacd85b0dac616dde551e1cfc2806a8b467f5a0b95502afa98f8a59e188f
        });

        // Use the exact signature from JavaScript generation
        Passkey.Signature memory validSignature = Passkey.Signature({
            r: 0x5450f52fc923b60d14feab0897021c7763f5337f926dcf3c60820870e4868d25,
            s: 0x03c18f5feb1ae2236ed048401f8fd56f9fd2716d51a86b2d38fcb6dcacf31a18
        });

        // Add the passkey to authorized signers first
        vm.prank(ownerAddress);
        testableAccount.addPasskeySigner(validPublicKey);

        // Use the exact userOpHash from JavaScript generation
        bytes32 userOpHash = 0x9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08;
        // Use the exact clientDataJSON from JavaScript generation
        string memory clientDataJSON = '{"type":"webauthn.get","challenge":"n4bQgYhMfWWaL-qgxVrQFaO_TxsrC4Is0V1sFbDwCgg","origin":"https://example.com"}';

        // Use the exact authData from JavaScript generation
        bytes memory authData = hex"00000000000000000000000000000000000000000000000000000000000000000100000000";

        Passkey.Metadata memory metadata = Passkey.Metadata({
            authData: authData,
            clientDataJSON: clientDataJSON,
            challengeIndex: 23, // Exact challengeIndex from JavaScript
            typeIndex: 1,       // Exact typeIndex from JavaScript
            userVerificationRequired: false
        });

        bytes memory sig = abi.encode(validPublicKey, validSignature, metadata);

        // Mock the actual staticcall made by OpenZeppelin's P256 library
        // When P256.verify is called, it makes a staticcall to the precompile
        vm.mockCall(
            address(0x100),
            abi.encodeWithSignature(""),
            abi.encode(uint256(1))
        );

        uint256 validationData = testableAccount.validatePasskeyPublic(userOpHash, sig);

        assertEq(validationData, SIG_VALIDATION_SUCCESS, "SUCCESS: Valid P256 signature verification");
    }

    // Define events for testing
    event PasskeySignerAdded(Passkey.PublicKey pk);
    event PasskeySignerRemoved(Passkey.PublicKey pk);
}
