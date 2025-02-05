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
import { getEnclaveNonce } from './get-enclave-nonce';
import { AuthenticationData } from '../type-creators/tc-authentication';

/**
 * Publicizes a member account in the AccountStore on the Litentry Parachain.
 *
 * @returns {Promise<Object>} A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} [payloadToSign] The payload to sign if the identity is not an email.
 * @returns {Function} send A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args The arguments required to send the request.
 * @returns {string} send.args.authentication The authentication string. For email identities, this is the verification code. For non-email identities, this is the signed payload.
 */
export async function publicizeAccount(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The user's account. Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** The member account for publicizing. Use `createLitentryIdentityType` helper to create this struct */
    identity: LitentryIdentity;
  },
  /** Whether the user is using Web3 authentication */
  isWeb3Auth: boolean
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<{
    response: WorkerRpcReturnValue;
    blockHash: string;
    extrinsicHash: string;
  }>;
}> {
  const { who, identity } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);
  const nonce = await getEnclaveNonce(api, { who });

  const { call } = await createTrustedCallType(api.registry, {
    method: 'publicize_account',
    params: {
      who,
      identity,
    },
  });

  const send = async (args: {
    authentication: AuthenticationData;
  }): Promise<{
    response: WorkerRpcReturnValue;
    blockHash: string;
    extrinsicHash: string;
  }> => {
    // prepare and encrypt request
    const request = await createRequestType(api, {
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

    if (!result.asOk.isExtrinsicReport) {
      throw new Error('Unexpected response type');
    }

    const { extrinsic_hash, block_hash } = result.asOk.asExtrinsicReport;

    return {
      response,
      extrinsicHash: extrinsic_hash.toString(),
      blockHash: block_hash.toString(),
    };
  };

  if (!isWeb3Auth) {
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
