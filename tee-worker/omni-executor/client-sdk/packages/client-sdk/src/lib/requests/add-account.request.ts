import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Identity, ValidationData, OmniAccountPermission } from '@heima/parachain-api';

import { AuthenticationData } from '@type-creators/authentication';
import { createNativeCallType } from '@type-creators/native-call';

import { aesCall } from './aes-call.request';

/**
 * Adds an account to the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {Identity} data.memberToAdd - The member account for adding to the OmniAccount. Use the `createCorePrimitivesIdentityType` helper to create this structure.
 * @param {ValidationData} data.validation - The ownership proof. Use the `createValidationDataType` helper to create this structure.
 * @param {boolean} data.isPublic - Whether the account is public.
 * @param {Array<OmniAccountPermission>} data.permissions - The permissions for the account.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function addAccount(
  api: ApiPromise,
  data: {
    member: Identity;
    memberToAdd: Identity;
    validation: ValidationData;
    isPublic: boolean;
    permissions?: Array<OmniAccountPermission>;
  },
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, memberToAdd, validation, isPublic, permissions } = data;

  const { operation } = createNativeCallType(api.registry, {
    method: 'add_account',
    params: {
      member,
      memberToAdd,
      validation,
      isPublic,
      permissions,
    },
  });

  return aesCall(api, { member, operation });
}
