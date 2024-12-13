import { verifySignature } from './verify-signature';

/**
 * Returns the crypto type of the signature.
 *
 * We return the capitalized crypto type because the Parachain API expects it to be capitalized.
 */
export function getSignatureCryptoType(args: {
  message: string | Uint8Array;
  signature: string | Uint8Array;
  address: string | Uint8Array;
}): 'None' | 'Ed25519' | 'Sr25519' | 'Ecdsa' | 'Ethereum' {
  const verifySignatureResult = verifySignature(args);
  if (!verifySignatureResult.isValid) {
    throw new Error('Invalid signature');
  }

  if (
    !verifySignatureResult.crypto ||
    typeof verifySignatureResult.crypto !== 'string'
  ) {
    return 'None';
  }

  // crypto type needs to be capitalized
  const cryptoType = (verifySignatureResult.crypto.charAt(0).toUpperCase() +
    verifySignatureResult.crypto.slice(1)) as
    | 'None'
    | 'Ed25519'
    | 'Sr25519'
    | 'Ecdsa'
    | 'Ethereum';

  return cryptoType;
}
