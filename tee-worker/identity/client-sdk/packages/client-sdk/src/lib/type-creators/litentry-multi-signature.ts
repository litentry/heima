import type { Registry } from '@polkadot/types-codec/types';
import type {
  LitentryIdentity,
  LitentryMultiSignature,
} from '@litentry/parachain-api';
import { decodeSignature } from '../util/decode-signature';

/**
 * Creates a LitentryIdentity struct type.
 *
 * Accepted signature encoding: hex-encoded string, base64-encoded string (Bitcoin only), base58-encoded string (Solana only).
 */
export function createLitentryMultiSignature(
  registry: Registry,
  data: {
    who: LitentryIdentity;
    signature: string;
  }
): LitentryMultiSignature {
  const { who, signature } = data;

  // Pick the crypto type. EthereumPrettified is a special case for EVM.
  // fallback to Sr25519 for all Substrate accounts.
  const cryptoType: LitentryMultiSignature['type'] = who.isBitcoin
    ? 'Bitcoin'
    : who.isEvm
    ? 'Ethereum' // Work with prettified signature for EVM only
    : who.isSolana
    ? 'Ed25519'
    : 'Sr25519';

  const struct = registry.createType('LitentryMultiSignature', {
    [cryptoType]: decodeSignature(signature, who),
  });

  return struct;
}
