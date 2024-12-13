import { enclave } from '../enclave';
import { blake2AsHex } from '@polkadot/util-crypto';

import type {
  LitentryIdentity,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';
import type { JsonRpcRequest } from '../util/types';
import type { ApiPromise } from '@polkadot/api';
import { type IdGraph, ID_GRAPH_STRUCT } from '../type-creators/id-graph';
import { safelyDecodeOption } from '../util/safely-decode-option';
import { createLitentryMultiSignature } from '../type-creators/litentry-multi-signature';

export async function getIdGraph(
  api: ApiPromise,
  data: {
    /** The user's account. Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
  }
): Promise<{
  payloadToSign: string;
  send: (args: { signedPayload: string }) => Promise<{
    idGraph: IdGraph;
    response: WorkerRpcReturnValue;
  }>;
}> {
  const { who } = data;

  const trustedGetter = api.createType('TrustedGetter', {
    id_graph: api.createType('(LitentryIdentity)', who),
  });

  const payload = blake2AsHex(trustedGetter.toU8a());
  const prefix =
    'Our team is ready to support you in retrieving your on-chain identity securely. Please be assured, this process is safe and involves no transactions of your assets. Token: ';
  const msg: string = prefix + payload;

  // Keep the 0x prefix removal check for Bitcoin in case the message later changes to hex
  const payloadToSign =
    who.isBitcoin && msg.startsWith('0x') ? msg.slice(2) : msg;

  const send = async (args: {
    signedPayload: string;
  }): Promise<{
    response: WorkerRpcReturnValue;
    idGraph: IdGraph;
  }> => {
    const signature = createLitentryMultiSignature(api.registry, {
      who,
      signature: args.signedPayload,
    });

    const trustedGetterSigned = api.createType('TrustedGetterSigned', {
      getter: trustedGetter,
      signature,
    });

    const getter = api.createType('Getter', {
      trusted: trustedGetterSigned,
    });

    const shard = await enclave.getShard(api);

    const request = api.createType('RsaRequest', {
      shard,
      payload: getter.toHex(),
    });

    // send the request to the Enclave
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'state_executeGetter',
      params: [request.toHex()],
    };
    const [workerResponse] = await enclave.send(api, rpcRequest);

    const idGraph = safelyDecodeOption(api.registry, {
      value: workerResponse.value.toHex(),
      type: ID_GRAPH_STRUCT,
    }) as IdGraph;

    return {
      idGraph: idGraph,
      response: workerResponse,
    };
  };

  return {
    payloadToSign,
    send,
  };
}
