import { hexToU8a } from '@polkadot/util';
import { base58Encode } from '@polkadot/util-crypto';

import { createNonceType } from '../type-creators/nonce';
import { enclave } from '../enclave';

import type { ApiPromise } from '@polkadot/api';
import type { Index } from '@polkadot/types/interfaces';
import type { LitentryIdentity } from '@litentry/parachain-api';
import type { JsonRpcRequest } from '../util/types';

/**
 * Retrieve the Sidechain's nonce.
 */
export async function getEnclaveNonce(
  api: ApiPromise,
  { who }: { who: LitentryIdentity }
): Promise<Index> {
  const shard = await enclave.getShard(api);

  const payload: JsonRpcRequest = {
    jsonrpc: '2.0',
    method: 'author_getNextNonce',
    params: [base58Encode(hexToU8a(shard)), who.toHex()],
  };

  const [workerResponse] = await enclave.send(api, payload);

  const nonce = createNonceType(api.registry, {
    workerRpcReturnValue: workerResponse,
  });

  return nonce;
}
