import type { ApiPromise } from '@polkadot/api';
import { hexToU8a } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';

import type { Identity, NativeCall, NativeOperationResponse } from '@heima-network/parachain-api';

import { getOmniAccountNonceWithIdentity } from '@requests/get-nonce.request';
import { AuthenticationData } from '@type-creators/authentication';
import { createCallRequestType } from '@type-creators/request';

import { createPayloadToSign } from '@utils/create-payload-to-sign';
import { isWeb3 } from '@utils/identity';
import type { JsonRpcRequest } from '@utils/types';

import { enclave } from '@lib/enclave';

/**
 * Sends an AES call to Enclave.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {NativeCall} data.operation - The specific call operation to be sent to the server.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function aesCall(
  api: ApiPromise,
  data: {
    member: Identity;
    operation: NativeCall;
  },
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, operation } = data;

  const [nonce, mrEnclave] = await Promise.all([
    getOmniAccountNonceWithIdentity(api, member),
    enclave.getMrEnclave(api),
  ]);
  const mrEnclaveU8 = hexToU8a(mrEnclave);

  const send = async (args: {
    authentication: AuthenticationData;
  }): Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }> => {
    // prepare and encrypt request
    const request = await createCallRequestType(api, {
      authentication: args.authentication,
      operation,
      nonce,
      mrEnclave: mrEnclaveU8,
    });

    // send the request to the Enclave
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'native_submitCallAesRequest',
      params: [request.toHex()],
    };

    const data = await enclave.send(rpcRequest);

    const result = api.createType<NativeOperationResponse>('NativeOperationResponse', data);

    if (result.isErr) {
      throw new Error(result.asErr.toString());
    }

    if (!result.asOk.isCallResponse) {
      throw new Error('Unexpected response type');
    }

    const callResponse = result.asOk.asCallResponse;
    if (!callResponse.isExtrinsicReport) {
      throw new Error('Unexpected call response type');
    }

    const { block_hash, extrinsic_hash, status } = callResponse.asExtrinsicReport;

    return {
      blockHash: block_hash.toHex(),
      extrinsicHash: extrinsic_hash.toHex(),
      status: status.toHex(),
    };
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
