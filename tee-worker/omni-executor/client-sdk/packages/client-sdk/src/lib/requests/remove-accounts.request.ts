import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Identity } from '@heima-network/parachain-api';

import { OmniAuthData } from '@type-creators/omni-auth';
import { createNativeTaskType } from '@type-creators/native-task';
import { enclave, Enclave } from '@lib/enclave';

import { aesTask } from './aes-task.request';

/**
 * Removes accounts from the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {Array<Identity>} data.membersToRemove - The member accounts for removing from the OmniAccount. Use the `createCorePrimitivesIdentityType` helper to create this structure.
 * @param {Enclave} enclaveInstance - The enclave instance use to interact with Enclave.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {Function} getPayloadToSign - A function to get the payload that needs to be signed (only for Web3 identities)
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {OmniAuthData} send.args.authData - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function removeAccounts(
  api: ApiPromise,
  data: {
    member: Identity;
    membersToRemove: Array<Identity>;
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
  const { member, membersToRemove } = data;

  const { task } = createNativeTaskType(api.registry, {
    method: 'RemoveAccounts',
    params: {
      member,
      membersToRemove,
    },
  });

  return aesTask(api, { member, task }, enclaveInstance);
}
