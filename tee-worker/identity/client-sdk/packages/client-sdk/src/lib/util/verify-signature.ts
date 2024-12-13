import type { VerifyResult } from '@polkadot/util-crypto/types';
import { decodeAddress, signatureVerify } from '@polkadot/util-crypto';
import { u8aToHex } from '@polkadot/util';

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
