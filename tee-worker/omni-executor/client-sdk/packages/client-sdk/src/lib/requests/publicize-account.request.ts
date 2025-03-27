import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Identity } from '@heima-network/parachain-api';

import { AuthenticationData } from '@type-creators/authentication';
import { createNativeCallType } from '@type-creators/native-call';

import { aesCall } from './aes-call.request';

/**
 * Publicizes a member account in the AccountStore on the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {Identity} data.memberToPublicize - The member account for publicizing. Use the `createIdentityType` helper to create this structure.
 * @returns {Promise<Object>} A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function publicizeAccount(
  api: ApiPromise,
  data: {
    member: Identity;
    memberToPublicize: Identity;
  },
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, memberToPublicize } = data;

  const { operation } = createNativeCallType(api.registry, {
    method: 'publicize_account',
    params: {
      member,
      memberToPublicize,
    },
  });

  return aesCall(api, { member, operation });
}
