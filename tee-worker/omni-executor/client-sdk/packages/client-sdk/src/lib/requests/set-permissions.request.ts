import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Identity, OmniAccountPermission } from '@heima/parachain-api';

import { AuthenticationData } from '@type-creators/authentication';
import { createNativeCallType } from '@type-creators/native-call';

import { aesCall } from './aes-call.request';

/**
 * Set the permissions for a specified account within the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {Identity} data.memberToSetPermissions - The account to be updated. Use the `createCorePrimitivesIdentityType` helper to create this structure.
 * @param {Array<OmniAccountPermission>} data.permissions - The permissions to be assigned to the account.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a function to send the request.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function that sends the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments needed to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
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
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, memberToSetPermissions, permissions } = data;

  const { operation } = createNativeCallType(api.registry, {
    method: 'set_permissions',
    params: {
      member,
      memberToSetPermissions,
      permissions,
    },
  });

  return aesCall(api, { member, operation });
}
