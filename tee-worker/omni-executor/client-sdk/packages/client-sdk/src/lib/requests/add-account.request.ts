import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Identity, ValidationData, OmniAccountPermission } from '@heima-network/parachain-api';

import { OmniAuthData } from '@type-creators/omni-auth';
import { createNativeTaskType } from '@type-creators/native-task';

import { aesTask } from './aes-task.request';
import { enclave, Enclave } from '@lib/enclave';

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
export async function addAccount(
  api: ApiPromise,
  data: {
    member: Identity;
    memberToAdd: Identity;
    validation: ValidationData;
    isPublic: boolean;
    permissions?: Array<OmniAccountPermission>;
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
  const { member, memberToAdd, validation, isPublic, permissions } = data;

  const { task } = createNativeTaskType(api.registry, {
    method: 'AddAccount',
    params: {
      member,
      memberToAdd,
      validation,
      isPublic,
      permissions,
    },
  });

  return aesTask(api, { member, task }, enclaveInstance);
}
