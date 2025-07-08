// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/* solhint-disable no-inline-assembly */

/**
 * The UserOp signature type
 * It's the first byte in PackedUserOperation.signature that stands for the signature type.
 * Different validation method should be called accordingly.
 *
 * @param EOA                - Signed by user's own EOA address
 * @param RootKey            - Signed by an authorised evm signer
 * @param SessionKey         - Signed by a short-lived session key, additional authorisation proof
 *                             (by RootKey) must be provided along. The concatenated signature length
 *                             should be 162 bytes excluding the leading UserOpSigType byte:
 *                             sessionSig (65) | sessionExpiration (32 = uint256) | sessionProof (65)
 * @param Passkey            - Signed by an authorised Passkey
 *                             Note it's Passkey not PassKey as it's an integral term
 */
enum UserOpSigType {
    EOA, // 0x00
    RootKey, // 0x01
    SessionKey, // 0x02
    Passkey // 0x03

}
