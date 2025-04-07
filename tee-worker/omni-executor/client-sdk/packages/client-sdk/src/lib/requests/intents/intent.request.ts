import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Intent, Identity } from '@heima-network/parachain-api';

import { OmniAuthData } from '@type-creators/omni-auth';
import { createNativeTaskType } from '@type-creators/native-task';

import { aesTask } from '@requests/aes-task.request';
import { enclave, Enclave } from '@lib/enclave';

/**
 * Sends a common intent request to the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {Intent} data.intent - The intent to be sent.
 * @param {U8aLike} data.input - The contract input data.
 * @param {Enclave} enclaveInstance - The enclave instance use to interact with Enclave.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to signature
 * (if applicable) and a send function.
 * @returns {Function} getPayloadToSign - A function to get the payload that needs to be signed (only for Web3 identities)
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {OmniAuthData} send.args.authData - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function intent(
  api: ApiPromise,
  data: {
    member: Identity;
    intent: Intent;
  },
  enclaveInstance: Enclave = enclave,
): Promise<{
  getPayloadToSign?: () => Promise<string>;
  send: (args: { authData: OmniAuthData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, intent } = data;

  const { task } = createNativeTaskType(api.registry, {
    method: 'RequestIntent',
    params: { member, intent },
  });

  return aesTask(api, { member, task }, enclaveInstance);
}
