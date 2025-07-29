// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/* solhint-disable no-inline-assembly */

/**
 * The owner type
 * Required as a single byte in the `OmniAccount.initialize` to allow for more fine-grained access control
 *
 * See P-1644 for more context
 *
 * The fields of this enum are 1 to 1 mapped to the rust `UserId` type in tee-worker/omni-executor/executor-primitives/src/auth.rs
 */
enum OwnerType {
    Pumpx, // 0x00
    Email, // 0x01
    Twitter, // 0x02
    Discord, // 0x03
    Github, // 0x04
    Substrate, // 0x05
    Evm, // 0x06
    Bitcoin, // 0x07
    Solana, // 0x08
    Google, // 0x09
    Passkey // 0x0a

}
