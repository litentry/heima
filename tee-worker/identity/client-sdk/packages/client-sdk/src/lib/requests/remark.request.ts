import { assert, hexToU8a, stringToHex } from '@polkadot/util';

import type { ApiPromise } from '@polkadot/api';
import type {
  LitentryIdentity,
  TrustedCallResult,
  WorkerRpcReturnValue,
} from '@litentry/parachain-api';

import { enclave } from '../enclave';
import { codecToString } from '../util/codec-to-string';
import { createPayloadToSign } from '../util/create-payload-to-sign';
import { createTrustedCallType } from '../type-creators/trusted-call';
import { createRequestType } from '../type-creators/request';

import type { JsonRpcRequest } from '../util/types';

/**
 * OmniAccount: Make a remark
 */
export async function remark(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The user's omniAccount.  Use `createLitentryIdentityType` helper to create this struct */
    omniAccount: LitentryIdentity;
    /** The user's account.  Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** the message to be sent */
    message: string;
  }
): Promise<{
  payloadToSign: string;
  send: (args: { signedPayload: string }) => Promise<{
    response: WorkerRpcReturnValue;
    blockHash: string;
    extrinsicHash: string;
  }>;
}> {
  const { who, message, omniAccount } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);

  const { call } = await createTrustedCallType(api.registry, {
    method: 'request_intent',
    params: {
      who,
      intent: api.createType('Intent', {
        SystemRemark: stringToHex(message),
      }),
    },
  });

  assert(omniAccount.isSubstrate, 'OmniAccount must be a Substrate identity');

  const nonce = await api.rpc.system.accountNextIndex(
    omniAccount.asSubstrate.toHex()
  );

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
    blockHash: string;
    extrinsicHash: string;
  }> => {
    // prepare and encrypt request

    const request = await createRequestType(api, {
      signer: who,
      signature: args.signedPayload,
      call,
      nonce,
      shard: shardU8,
    });

    // send the request to the Enclave
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'author_submitNativeRequest',
      params: [request.toHex()],
    };

    const [response] = await enclave.send(api, rpcRequest); // we expect 1 response only

    const result: TrustedCallResult = api.createType(
      'TrustedCallResult',
      response.value
    );

    if (result.isErr) {
      throw new Error(codecToString(result.asErr));
    }

    const { extrinsic_hash, block_hash } = result.asOk;

    return {
      response,
      extrinsicHash: extrinsic_hash.toString(),
      blockHash: block_hash.toString(),
    };
  };

  return {
    payloadToSign,
    send,
  };
}
