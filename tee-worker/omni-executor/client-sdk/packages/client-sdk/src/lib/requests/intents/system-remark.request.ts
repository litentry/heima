import type { ApiPromise } from '@polkadot/api';
import { stringToHex } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';

import type { Intent, Identity } from '@heima-network/parachain-api';

import { AuthenticationData } from '@type-creators/authentication';

import { intent } from './intent.request';

/**
 * Intent Request: Sends a system remark to the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {string} data.message - The message to be sent.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign
 * (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function systemRemark(
  api: ApiPromise,
  data: {
    member: Identity;
    message: string;
  },
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  return intent(api, {
    member: data.member,
    intent: api.createType<Intent>('Intent', {
      SystemRemark: stringToHex(data.message),
    }),
  });
}
