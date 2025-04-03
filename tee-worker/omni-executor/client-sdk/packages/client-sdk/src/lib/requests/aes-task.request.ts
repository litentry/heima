import type { ApiPromise } from '@polkadot/api';
import { hexToU8a } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';

import type { AesOutput, Identity, NativeTask, NativeTaskResponse } from '@heima-network/parachain-api';

import { getOmniAccountNonceWithIdentity } from '@requests/get-nonce.request';
import { OmniAuthData } from '@type-creators/omni-auth';
import { createRawTaskType } from '@type-creators/raw-task';

import { createPayloadToSign } from '@utils/create-payload-to-sign';
import { isWeb3 } from '@utils/identity';
import type { JsonRpcRequest } from '@utils/types';

import { enclave } from '@lib/enclave';
import { decrypt } from '@lib/utils';

/**
 * Sends an AES task to Enclave.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {NativeTask} data.task - The specific task to be sent to the server.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {Function} getPayloadToSign - A function to get the payload that needs to be signed (only for Web3 identities)
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {OmniAuthData} send.args.authData - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function aesTask(
  api: ApiPromise,
  data: {
    member: Identity;
    task: NativeTask;
  },
): Promise<{
  getPayloadToSign?: () => Promise<string>;
  send: (args: { authData: OmniAuthData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, task } = data;

  const nonce = await getOmniAccountNonceWithIdentity(api, member);

  const send = async (args: {
    authData: OmniAuthData;
  }): Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }> => {
    // prepare and encrypt task
    const { rawTask, encryptionKey } = await createRawTaskType(api, {
      task,
      nonce,
      authData: args.authData,
    });

    // send the request to the Enclave
    const request: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'omni_submitNativeTask',
      params: [rawTask.toHex()],
    };

    const data = await enclave.send(request);
    const aesOutput = api.createType<AesOutput>('AesOutput', data)
    const { cleartext } = await decrypt({ ciphertext: aesOutput.ciphertext, nonce: aesOutput.nonce }, encryptionKey);

    const response = api.createType<NativeTaskResponse>('NativeTaskResponse', cleartext);

    if (response.isErr) {
      throw new Error(response.asErr.toString());
    }

    const okResponse = response.asOk;
    if (!okResponse.isExtrinsicReport) {
      throw new Error('Unexpected response type');
    }

    const { block_hash, extrinsic_hash, status } = okResponse.asExtrinsicReport;

    return {
      blockHash: block_hash.toHex(),
      extrinsicHash: extrinsic_hash.toHex(),
      status: status.toHex(),
    };
  };

  if (isWeb3(member)) {
    const getPayloadToSign = async () => {
      const mrEnclave = await enclave.getMrEnclave(api);
      return createPayloadToSign({
        who: member,
        task,
        nonce,
        mrEnclave: hexToU8a(mrEnclave),
      });
    }

    return {
      getPayloadToSign,
      send,
    };
  }

  return { send };
}
