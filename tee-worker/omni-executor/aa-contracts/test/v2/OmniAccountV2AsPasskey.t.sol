// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {OmniAccountV2} from "../../src/accounts/OmniAccountV2.sol";
import {BaseAccount} from "../../src/core/BaseAccount.sol";
import {EntryPointV1} from "../../src/core/EntryPointV1.sol";
import {UserOpSigner} from "../../src/interfaces/UserOpSigner.sol";
import {Counter} from "../../src/Counter.sol";
import {OmniAccountV2TestUtils} from "./OmniAccountV2TestUtils.sol";
import {TestUtils} from "../TestUtils.sol";
import {PackedUserOperation} from "../../src/interfaces/PackedUserOperation.sol";
import {SIG_VALIDATION_SUCCESS, SIG_VALIDATION_FAILED} from "../../src//core/Helpers.sol";
import {Passkey} from "../../src/interfaces/Passkey.sol";
import "@openzeppelin/contracts/utils/Base64.sol";

contract OmniAccountV2AsPasskey is Test {
    OmniAccountV2 public account;
    EntryPointV1 public entryPoint;
    Counter public counter;

    address ownerAddress = 0x0000000000000000000000000000000000000000;
    bytes clientId = bytes("test_client");

    function test_ValidateOpPasskey() public {
        (address root,) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountV2TestUtils.setUp(ownerAddress, clientId, root);

        // Generate a real P256 private key and derive the public key
        uint256 passkeyPrivateKey = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        (uint256 publicKeyX, uint256 publicKeyY) = vm.publicKeyP256(passkeyPrivateKey);

        Passkey.PublicKey memory passkeyPubKey = Passkey.PublicKey({x: publicKeyX, y: publicKeyY});

        // Add passkey as a signer (must be called by owner)
        vm.prank(ownerAddress);
        account.addPasskeySigner(passkeyPubKey);

        // Prepare PackedUserOperation
        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Create WebAuthn clientDataJSON with the challenge
        string memory challengeB64url = Base64.encodeURL(abi.encodePacked(packedOpHash));
        string memory clientDataJSON = string.concat(
            '{"type":"webauthn.get","challenge":"',
            challengeB64url,
            '","origin":"https://example.com","crossOrigin":false}'
        );

        // Create authentic authenticatorData (37 bytes minimum)
        // rpIdHash (32 bytes) + flags (1 byte) + signCount (4 bytes)
        bytes32 rpIdHash = sha256("example.com");
        bytes1 flags = 0x05; // UP=1, UV=1 (user present and verified)
        bytes4 signCount = bytes4(uint32(1));
        bytes memory authData = abi.encodePacked(rpIdHash, flags, signCount);

        // Calculate the message hash that will be signed
        bytes32 clientDataJSONHash = sha256(bytes(clientDataJSON));
        bytes32 messageHash = sha256(abi.encodePacked(authData, clientDataJSONHash));

        // Sign the message hash with P256
        (bytes32 r, bytes32 s) = vm.signP256(passkeyPrivateKey, messageHash);

        Passkey.Signature memory passkeySignature = Passkey.Signature({r: uint256(r), s: uint256(s)});

        // Find the correct positions for type and challenge in clientDataJSON
        bytes memory clientDataJSONBytes = bytes(clientDataJSON);
        uint16 typeIndex = _findSubstring(clientDataJSONBytes, bytes('"type":"webauthn.get"'));
        uint16 challengeIndex =
            _findSubstring(clientDataJSONBytes, bytes(string.concat('"challenge":"', challengeB64url, '"')));

        Passkey.Metadata memory metadata = Passkey.Metadata({
            authData: authData,
            clientDataJSON: clientDataJSON,
            challengeIndex: challengeIndex,
            typeIndex: typeIndex,
            userVerificationRequired: true // UV flag is set
        });

        // Encode signature according to Passkey format
        bytes memory passkeySignatureData = abi.encode(passkeyPubKey, passkeySignature, metadata);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Passkey), passkeySignatureData);

        // Validate the user operation
        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);

        // Should pass with real P256 signature
        assertEq(SIG_VALIDATION_SUCCESS, validationData);
    }

    function test_ValidateOpPasskeyUnauthorized() public {
        (address root,) = makeAddrAndKey("root");
        (counter, entryPoint, account) = OmniAccountV2TestUtils.setUp(ownerAddress, clientId, root);

        // Create passkey public key but don't add it as authorized signer
        Passkey.PublicKey memory passkeyPubKey = Passkey.PublicKey({
            x: 0x65eda5a12577c2bae829437fe338701a10aaa375e1bb5b5de108de439c08551d,
            y: 0x1e52ed75701163f7f9e40ddf9f341b3dc9ba860af7e0ca7ca7e9eecd0084d19c
        });

        // Prepare PackedUserOperation
        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;
        bytes memory initCode = "";
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);

        // Create mock signature components
        Passkey.Signature memory passkeySignature = Passkey.Signature({
            r: 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef,
            s: 0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321
        });

        bytes memory mockAuthData = hex"49960de5880e8c687434170f6476605b8fe4aeb9a28632c7995cf3ba831d97634500000001";
        string memory mockClientDataJSON = '{"type":"webauthn.get","challenge":"test","origin":"https://example.com"}';

        Passkey.Metadata memory metadata = Passkey.Metadata({
            authData: mockAuthData,
            clientDataJSON: mockClientDataJSON,
            challengeIndex: 32,
            typeIndex: 1,
            userVerificationRequired: false
        });

        bytes memory passkeySignatureData = abi.encode(passkeyPubKey, passkeySignature, metadata);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Passkey), passkeySignatureData);

        vm.prank(address(entryPoint));
        uint256 validationData = account.validateUserOp(packedOp, packedOpHash, 0);

        // Should fail because passkey isn't authorized
        assertEq(SIG_VALIDATION_FAILED, validationData);
    }

    function _findSubstring(bytes memory haystack, bytes memory needle) internal pure returns (uint16) {
        require(needle.length > 0, "Empty needle");
        require(haystack.length >= needle.length, "Needle longer than haystack");

        for (uint256 i = 0; i <= haystack.length - needle.length; i++) {
            bool found = true;
            for (uint256 j = 0; j < needle.length; j++) {
                if (haystack[i + j] != needle[j]) {
                    found = false;
                    break;
                }
            }
            if (found) {
                return uint16(i);
            }
        }
        revert("Substring not found");
    }
}
