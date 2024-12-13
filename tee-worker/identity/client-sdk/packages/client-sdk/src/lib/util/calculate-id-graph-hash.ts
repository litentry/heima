import { compactToU8a, u8aConcat, u8aConcatStrict } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';

import { type IdGraph } from '../type-creators/id-graph';

/**
 * Returns the hash of the given id graph. It matches the hash used in the Litentry Parachain.
 */
export function calculateIdGraphHash(idGraph: IdGraph): `0x${string}` {
  const sorted: Uint8Array[] = idGraph
    .filter(Boolean) // shallow copy
    // Sorting
    // Reference: https://github.com/litentry/litentry-parachain/blob/dev/tee-worker/litentry/pallets/identity-management/src/identity_context.rs#L75-L85
    .sort((entry1, entry2) => {
      return entry1[1].link_block.toNumber() - entry2[1].link_block.toNumber();
    })
    // Convert entries to U8a
    .reduce((acc, [identity, context]) => {
      const inner = u8aConcat(identity.toU8a(), context.toU8a());

      return [...acc, inner];
    }, [] as Uint8Array[]);

  // Encode the array as u8a and concatenate all entries
  const sortedU8a = u8aConcatStrict([compactToU8a(sorted.length), ...sorted]);

  const hash = blake2AsHex(sortedU8a);

  return hash;
}
