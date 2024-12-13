import type { LitentryIdentity } from '@litentry/parachain-api';
import type { JsonRpcRequest } from '../util/types';

import type { ApiPromise } from '@polkadot/api';
import type { H256 } from '@polkadot/types/interfaces';

import { enclave } from '../enclave';
import { safelyDecodeOption } from '../util/safely-decode-option';

/**
 * Retrieve the idGraphHash for a given identity.
 */
export async function getIdGraphHash(
  api: ApiPromise,
  { who }: { who: LitentryIdentity }
): Promise<H256 | null> {
  const shard = await enclave.getShard(api);

  // Create a public getter for idGraphHash
  const publicGetter = api.createType('PublicGetter', {
    id_graph_hash: api.createType('(LitentryIdentity)', who),
  });
  const getter = api.createType('Getter', { public: publicGetter });

  // Prepare the request
  const request = api.createType('RsaRequest', {
    shard,
    payload: getter.toHex(),
  });

  // Format the JSON-RPC request
  const payload: JsonRpcRequest = {
    jsonrpc: '2.0',
    method: 'state_executeGetter',
    params: [request.toHex()],
  };

  // Send the request and receive the response
  try {
    const [workerResponse] = await enclave.send(api, payload);

    // Decode the response to get the idGraphHash
    return safelyDecodeOption(api.registry, {
      value: workerResponse.value.toHex(),
      type: 'H256',
      throw: false,
    });
  } catch (e) {
    // swallow - empty id_graph
    return null;
  }
}
