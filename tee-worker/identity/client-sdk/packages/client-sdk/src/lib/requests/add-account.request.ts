import { hexToU8a } from '@polkadot/util';

import type { ApiPromise } from '@polkadot/api';
import type {
  LitentryIdentity,
  LitentryValidationData,
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

/**
 * Adds an account to the Litentry Parachain.
 *
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} [payloadToSign] - The payload to sign if who is not an email identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {string} send.args.authentication - The authentication string. If who is
 * an email identity, this is the email verification code. If who is not an email identity, this is the
 * signed payload.
 */
export async function addAccount(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The user's account. Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** Account to be added. Use `createCorePrimitivesIdentityType` helper to create this struct */
    identity: LitentryIdentity;
    /** The ownership proof. Use `createLitentryValidationDataType` helper to create this struct */
    validation: LitentryValidationData;
    /** Whether the account is public */
    isPublic: boolean;
  }
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: string }) => Promise<{
    response: WorkerRpcReturnValue;
    blockHash: string;
    extrinsicHash: string;
  }>;
}> {
  const { who, identity, validation, isPublic } = data;

  const shard = await enclave.getShard(api);
  const shardU8 = hexToU8a(shard);
  const nonce = await getEnclaveNonce(api, { who });

  const { call } = await createTrustedCallType(api.registry, {
    method: 'add_account',
    params: {
      who,
      identity,
      validation,
      isPublic,
    },
  });

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