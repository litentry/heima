import { LitentryIdentity } from '@litentry/parachain-api';
import { hexToU8a, isHex } from '@polkadot/util';
import { base58Decode, base64Decode } from '@polkadot/util-crypto';

/**
 * Decodes a signature string into a Uint8Array based on the identity type.
 */
export function decodeSignature(
  signature: string,
  identity: LitentryIdentity
): Uint8Array {
  if (!signature) {
    throw new Error(`[enclave] Signature is empty`);
  }

  if (isHex(signature)) {
    return hexToU8a(signature);
  }

  if (identity.isBitcoin) {
    // Bitcoin signature is base64-encoded string
    return base64Decode(signature);
  }

  if (identity.isSolana) {
    // Solana signature is base58-encoded string
    return base58Decode(signature);
  }

  throw new Error(
    `[enclave] Unsupported signature format for "${identity.type}". Expected hex-encoded string. Got "${signature}"`
  );
}
