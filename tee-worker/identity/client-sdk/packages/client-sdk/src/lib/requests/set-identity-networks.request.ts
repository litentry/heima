import { hexToU8a } from '@polkadot/util';
import { randomAsHex } from '@polkadot/util-crypto';

import { enclave } from '../enclave';
import { createTrustedCallType } from '../type-creators/trusted-call';
import { createRequestType } from '../type-creators/request';
import { createPayloadToSign } from '../util/create-payload-to-sign';
import { type IdGraph, createIdGraphType } from '../type-creators/id-graph';

import * as shieldingKeyUtils from '../util/shielding-key';
import { getEnclaveNonce } from './get-enclave-nonce';

import type {
  LitentryIdentity,
  Web3Network,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';
import type { JsonRpcRequest } from '../util/types';
import type { ApiPromise } from '@polkadot/api';

/**
 * Set the networks for a Web3 Identity.
 *
 * It allows to change the list of `networks` for an already linked web3 identity.
 */
export async function setIdentityNetworks(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The user's account. Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** Identity to be modified. Use `createLitentryIdentityType` helper to create this struct */
    identity: LitentryIdentity;
    /** Networks to be set */
    networks: Array<Web3Network['type']>;
  }
): Promise<{
  payloadToSign: string;
  txHash: string;
  send: (args: { signedPayload: string }) => Promise<{
    mutatedIdentities: IdGraph;
    idGraphHash: `0x${string}`;
    response: WorkerRpcReturnValue;
    txHash: string;
  }>;
}> {
  const { who, identity, networks } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);
  const nonce = await getEnclaveNonce(api, { who });

  const txHash = randomAsHex();

  const { call, key } = await createTrustedCallType(api.registry, {
    method: 'set_identity_networks',
    params: {
      who,
      identity,
      networks,
      hash: txHash,
    },
  });

  const payloadToSign = createPayloadToSign({
    who,
    call,
    nonce,
    shard: shardU8,
  });

  const send = async (args: {
    signedPayload: string;
  }): Promise<{
    mutatedIdentities: IdGraph;
    idGraphHash: `0x${string}`;
    response: WorkerRpcReturnValue;
    txHash: string;
  }> => {
    // prepare and encrypt request
    const requestPayload = await createRequestType(api, {
      authentication: {
        type: 'Web3',
        signer: who,
        signature: args.signedPayload,
      },
      call,
      nonce,
      shard: shardU8,
    });

    // send the request to the Enclave
    const request: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'author_submitAndWatchAesRequest',
      params: [requestPayload.toHex()],
    };

    const [workerResponse] = await enclave.send(api, request);

    // process the encrypted response
    const { mutated_id_graph, id_graph_hash } = api.createType(
      'SetIdentityNetworksResult',
      workerResponse.value
    );

    const { cleartext: decryptedIdentities } = await shieldingKeyUtils.decrypt(
      {
        ciphertext: mutated_id_graph.ciphertext,
        nonce: mutated_id_graph.nonce,
      },
      key
    );

    const mutatedIdentities = createIdGraphType(
      api.registry,
      decryptedIdentities
    );

    return {
      mutatedIdentities,
      idGraphHash: id_graph_hash.toHex(),
      txHash,
      response: workerResponse,
    };
  };

  return {
    payloadToSign,
    send,
    txHash,
  };
}
