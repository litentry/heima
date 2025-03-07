import { u8aToHex } from '@polkadot/util';
import { decodeAddress, signatureVerify } from '@polkadot/util-crypto';
import type { VerifyResult } from '@polkadot/util-crypto/types';

/**
 * Verifies a cryptographic signature against a message and an address.
 *
 * @param args - The verification parameters
 * @param args.message - The message to verify (either as a string or Uint8Array)
 * @param args.signature - The signature to verify (either as a string or Uint8Array)
 * @param args.address - The signer's address (either as a string or Uint8Array)
 * @returns {VerifyResult} Object containing verification result with properties:
 *                         - isValid: boolean indicating if signature is valid
 *                         - crypto: string indicating the crypto type used
 */
export function verifySignature(args: {
  message: string | Uint8Array;
  signature: string | Uint8Array;
  address: string | Uint8Array;
}): VerifyResult {
  const { message, signature, address } = args;
  const publicKey = decodeAddress(address);
  const hexPublicKey = u8aToHex(publicKey);

  return signatureVerify(message, signature, hexPublicKey);
}
