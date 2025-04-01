import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Identity, OmniAccountPermission } from '@heima-network/parachain-api';

import { OmniAuthData } from '@type-creators/omni-auth';
import { createNativeTaskType } from '@type-creators/native-task';

import { aesTask } from './aes-task.request';

/**
 * Set the permissions for a specified account within the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {Identity} data.memberToSetPermissions - The account to be updated. Use the `createCorePrimitivesIdentityType` helper to create this structure.
 * @param {Array<OmniAccountPermission>} data.permissions - The permissions to be assigned to the account.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a function to send the request.
 * @returns {Function} getPayloadToSign - A function to get the payload that needs to be signed (only for Web3 identities)
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {OmniAuthData} send.args.authData - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function setPermissions(
  api: ApiPromise,
  data: {
    member: Identity;
    memberToSetPermissions: Identity;
    permissions: Array<OmniAccountPermission>;
  },
): Promise<{
  getPayloadToSign?: () => Promise<string>;
  send: (args: { authData: OmniAuthData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, memberToSetPermissions, permissions } = data;

  const { task } = createNativeTaskType(api.registry, {
    method: 'SetPermissions',
    params: {
      member,
      memberToSetPermissions,
      permissions,
    },
  });

  return aesTask(api, { member, task });
}
