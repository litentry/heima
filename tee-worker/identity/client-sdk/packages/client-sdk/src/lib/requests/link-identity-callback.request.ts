import { hexToU8a } from '@polkadot/util';
import { randomAsHex } from '@polkadot/util-crypto';

import { enclave } from '../enclave';
import { createTrustedCallType } from '../type-creators/trusted-call';
import { createRequestType } from '../type-creators/request';
import { createPayloadToSign } from '../util/create-payload-to-sign';
import { type IdGraph, createIdGraphType } from '../type-creators/id-graph';

import * as shieldingKeyUtils from '../util/shielding-key';

import type {
  LitentryIdentity,
  Web3Network,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';
import type { JsonRpcRequest } from '../util/types';
import type { ApiPromise } from '@polkadot/api';

import { getEnclaveNonce } from './get-enclave-nonce';

/**
 * (internal) Link an identity to the user's account.
 *
 * This function is only meant to be used in development networks where root or enclave_signer_account
 * are used as the signer.
 */
export async function linkIdentityCallback(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The signer. Use `createCorePrimitivesIdentityType` helper to create this struct */
    signer: LitentryIdentity;
    /** The prime identity. Use `createCorePrimitivesIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** Identity to be linked. Use `createCorePrimitivesIdentityType` helper to create this struct */
    identity: LitentryIdentity;
    /** The networks to link the identity to, for web3 accounts */
    networks?: Array<Web3Network['type']>;
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
  const { signer, who, identity, networks } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);
  const nonce = await getEnclaveNonce(api, { who: signer });

  const txHash = randomAsHex();

  const { call, key } = await createTrustedCallType(api.registry, {
    method: 'link_identity_callback',
    params: {
      signer,
      who,
      identity,
      networks: networks || [],
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
    response: WorkerRpcReturnValue;
    mutatedIdentities: IdGraph;
    idGraphHash: `0x${string}`;
    txHash: string;
  }> => {
    // prepare and encrypt request
    const request = await createRequestType(api, {
      authentication: {
        type: 'Web3',
        signer,
        signature: args.signedPayload,
      },
      call,
      nonce,
      shard: shardU8,
    });

    // send the request to the Enclave
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'author_submitAndWatchAesRequest',
      params: [request.toHex()],
      // note: it should be number and less than 10
      id: 1,
    };
    const [workerResponse] = await enclave.send(api, rpcRequest);

    // process the encrypted response
    const { mutated_id_graph, id_graph_hash } = api.createType(
      'LinkIdentityResult',
      workerResponse.value
    );

    const { cleartext: decryptedIdGraphRaw } = await shieldingKeyUtils.decrypt(
      {
        ciphertext: mutated_id_graph.ciphertext,
        nonce: mutated_id_graph.nonce,
      },
      key
    );

    const mutatedIdentities = createIdGraphType(
      api.registry,
      decryptedIdGraphRaw
    );

    return {
      mutatedIdentities,
      idGraphHash: id_graph_hash.toHex(),
      txHash,
      response: workerResponse,
    };
  };

  return {
    txHash,
    payloadToSign,
    send,
  };
}
