import { Index } from '@polkadot/types/interfaces';
import { Registry } from '@polkadot/types-codec/types';
import { stringToHex, u8aConcat } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import { HexString } from '@polkadot/util/types';

import { Identity } from '@heima/parachain-api';

/**
 * Creates a verification message for adding a new member to an Omni account
 *
 * @param registry - The Polkadot registry instance used for type encoding
 * @param args - The arguments object containing:
 * @param args.member - The identity of the existing member
 * @param args.memberToAdd - The identity of the new member to be added
 * @param args.omniAccountNonce - The current nonce of the Omni account
 * @returns A hex-encoded string containing the verification message prefixed with "Token: "
 *
 * The verification message is created by:
 * 1. Converting the nonce to U64 format
 * 2. Concatenating the encoded nonce, member identity, and new member identity
 * 3. Computing the BLAKE2 hash of the concatenated data
 * 4. Prefixing the hash with "Token: " and converting to hex
 */
export function createVerificationMessage(
  registry: Registry,
  args: {
    member: Identity;
    memberToAdd: Identity;
    omniAccountNonce: Index;
  },
): HexString {
  const { member, memberToAdd, omniAccountNonce } = args;
  // Type of nonce is u64 in https://github.com/litentry/heima/blob/dev/tee-worker/omni-executor/heima/identity-verification/src/lib.rs
  const nonce = registry.createType('U64', omniAccountNonce);
  const payload = u8aConcat(nonce.toU8a(), member.toU8a(), memberToAdd.toU8a());
  return stringToHex(`Token: ${blake2AsHex(payload, 256)}`);
}
