import type { ApiPromise } from '@polkadot/api';
import type { LitentryIdentity, NativeCall, NativeCallResponse } from '@litentry/parachain-api';
import type { JsonRpcRequest } from '../util/types';

import { hexToU8a, assert } from '@polkadot/util';
import { createRequestType } from '../type-creators/request';
import { AuthenticationData } from '../type-creators/authentication';
import { createNativeCallType } from '../type-creators/native-call';
import { enclave } from '../enclave';
import { Index } from '@polkadot/types/interfaces';

/**
 * Requests an authentication token from the omni.
 *
 * @returns {Promise<Object>} A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} [payloadToSign] The payload to sign if the identity is not an email.
 * @returns {Function} send A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args The arguments required to send the request.
 * @returns {string} send.args.authentication The authentication string. For email identities, this is the verification code. For non-email identities, this is the signed payload.
 */
export async function requestAuthToken(
  /** Litentry Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    /** The user's omniAccount.  Use `createLitentryIdentityType` helper to create this struct */
    omniAccount: LitentryIdentity;
    /** The user's account. Use `createLitentryIdentityType` helper to create this struct */
    who: LitentryIdentity;
    /** The block number at which the token expires */
    expiresAt: number;
  },
): Promise<{
  send: (args: { authentication: AuthenticationData }) => Promise<{
    token: string;
  }>;
  call: NativeCall;
  nonce: Index;
  shard: `0x${string}`;
}> {
  const { who, expiresAt, omniAccount } = data;

  assert(omniAccount.isSubstrate, 'OmniAccount must be a Substrate identity');

  const [nonce, shard] = await Promise.all([enclave.getNonce(api, omniAccount), enclave.getShard(api)]);

  const call = createNativeCallType(api.registry, {
    method: 'request_auth_token',
    params: {
      identity: who,
      authOptions: {
        expiresAt,
      },
    },
  });

  const shardU8 = hexToU8a(shard);

  const send = async (args: {
    authentication: AuthenticationData;
  }): Promise<{
    token: string;
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
      method: 'native_submitAesRequest',
      params: [request.toHex()],
    };

    const response = await enclave.send<string>(rpcRequest); // we expect 1 response only

    const result = api.createType('NativeCallResponse', hexToU8a(response)) as unknown as NativeCallResponse;

    if (result.isErr) {
      throw new Error(result.asErr.toString());
    }

    if (!result.asOk.isAuthToken) {
      throw new Error('Unexpected response type');
    }

    const token = result.asOk.asAuthToken.toHuman();

    return { token };
  };

  return { send, call, nonce, shard };
}
