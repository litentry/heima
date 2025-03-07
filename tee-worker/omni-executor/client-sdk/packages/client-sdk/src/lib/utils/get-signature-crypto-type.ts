import { verifySignature } from './verify-signature';

/**
 * Returns the crypto type of the signature in capitalized format.
 *
 * @param args - The signature verification parameters
 * @param args.message - The message that was signed (string or Uint8Array)
 * @param args.signature - The signature to verify (string or Uint8Array)
 * @param args.address - The address that signed the message (string or Uint8Array)
 *
 * @returns The capitalized crypto type used for the signature:
 *          'None' | 'Ed25519' | 'Sr25519' | 'Ecdsa' | 'Ethereum'
 *
 * @throws {Error} If the signature is invalid
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

  if (!verifySignatureResult.crypto || typeof verifySignatureResult.crypto !== 'string') {
    return 'None';
  }

  // crypto type needs to be capitalized
  const cryptoType = (verifySignatureResult.crypto.charAt(0).toUpperCase() + verifySignatureResult.crypto.slice(1)) as
    | 'None'
    | 'Ed25519'
    | 'Sr25519'
    | 'Ecdsa'
    | 'Ethereum';

  return cryptoType;
}
