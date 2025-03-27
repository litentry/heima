import type { ApiPromise } from '@polkadot/api';
import { hexToU8a } from '@polkadot/util';

import type { Identity, NativeOperationResponse, NativeQuery, QueryResponse } from '@heima-network/parachain-api';

import { getOmniAccountNonceWithIdentity } from '@requests/get-nonce.request';
import { AuthenticationData } from '@type-creators/authentication';
import { createQueryRequestType } from '@type-creators/request';

import { createPayloadToSign } from '@utils/create-payload-to-sign';
import { isWeb3 } from '@utils/identity';
import type { JsonRpcRequest } from '@utils/types';

import { enclave } from '@lib/enclave';

/**
 * Queries data from Enclave.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {NativeQuery} data.operation - The specific query operation to be sent to the server.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {QueryResponse} send.return - The query response.
 */
export async function aesQuery(
  api: ApiPromise,
  data: {
    member: Identity;
    operation: NativeQuery;
  },
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<QueryResponse>;
}> {
  const { member, operation } = data;

  const [nonce, mrEnclave] = await Promise.all([
    getOmniAccountNonceWithIdentity(api, member),
    enclave.getMrEnclave(api),
  ]);
  const mrEnclaveU8 = hexToU8a(mrEnclave);

  const send = async (args: { authentication: AuthenticationData }): Promise<QueryResponse> => {
    // prepare and encrypt request
    const request = await createQueryRequestType(api, {
      authentication: args.authentication,
      operation,
      nonce,
      mrEnclave: mrEnclaveU8,
    });

    // send the request to the Enclave
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'native_submitQueryAesRequest',
      params: [request.toHex()],
    };

    const data = await enclave.send(rpcRequest);

    const result = api.createType<NativeOperationResponse>('NativeOperationResponse', data);

    if (result.isErr) {
      throw new Error(result.asErr.toString());
    }

    if (!result.asOk.isQueryResponse) {
      throw new Error('Unexpected response type');
    }

    return result.asOk.asQueryResponse;
  };

  if (isWeb3(member)) {
    const payloadToSign = createPayloadToSign({
      who: member,
      operation,
      nonce,
      mrEnclave: mrEnclaveU8,
    });

    return {
      payloadToSign,
      send,
    };
  }

  return { send };
}
