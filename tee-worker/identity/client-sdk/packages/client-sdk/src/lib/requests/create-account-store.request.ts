import { hexToU8a } from '@polkadot/util';

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
 * Creates an account store on the Litentry Parachain.
 *
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} [payloadToSign] - The payload to sign if who is not an email identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {string} send.args.authentication - The authentication string. If who is
 * an email identity, this is the email verification code. If the who is not an email identity, this is the
 * signed payload.
 */
export async function createAccountStore(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The user's OmniAccount.  Use `createLitentryIdentityType` helper to create this struct */
    omniAccount: LitentryIdentity;
    /** The user's account.  Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
  }
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: string }) => Promise<{
    response: WorkerRpcReturnValue;
    blockHash: string;
    extrinsicHash: string;
  }>;
}> {
  const { who, omniAccount } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);

  const { call } = await createTrustedCallType(api.registry, {
    method: 'create_account_store',
    params: {
      who,
    },
  });

  const nonce = await api.rpc.system.accountNextIndex(
    omniAccount.asSubstrate.toHex()
  );

  const send = async (args: {
    authentication: string;
  }): Promise<{
    response: WorkerRpcReturnValue;
    blockHash: string;
    extrinsicHash: string;
  }> => {
    // prepare and encrypt request

    const request = await createRequestType(api, {
      sender: who,
      authentication: args.authentication,
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

  if (who.isEmail) {
    return { send };
  }

  const payloadToSign = createPayloadToSign({
    who,
    call,
    nonce,
    shard: shardU8,
  });

  return {
    payloadToSign,
    send,
  };
}