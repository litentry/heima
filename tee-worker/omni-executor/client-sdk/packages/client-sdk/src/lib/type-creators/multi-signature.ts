import type { Registry } from '@polkadot/types-codec/types';

import type { Identity, MultiSignature } from '@heima/parachain-api';

import { decodeSignature } from '@utils/decode-signature';

/**
 * Creates a Identity struct type.
 *
 * Accepted signature encoding: hex-encoded string, base64-encoded string (Bitcoin only), base58-encoded string (Solana only).
 */
export function createMultiSignature(
  registry: Registry,
  data: {
    who: Identity;
    signature: string;
  },
): MultiSignature {
  const { who, signature } = data;

  // Pick the crypto type. EthereumPrettified is a special case for EVM.
  // fallback to Sr25519 for all Substrate accounts.
  const cryptoType: MultiSignature['type'] = who.isBitcoin
    ? 'Bitcoin'
    : who.isEvm
      ? 'Ethereum' // Work with prettified signature for EVM only
      : who.isSolana
        ? 'Ed25519'
        : 'Sr25519';

  return registry.createType<MultiSignature>('MultiSignature', {
    [cryptoType]: decodeSignature(signature, who),
  });
}
