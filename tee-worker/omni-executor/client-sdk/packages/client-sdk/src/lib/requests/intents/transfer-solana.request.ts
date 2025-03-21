import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';
import { base58 } from '@scure/base';

import type { Intent, IntentTransferSolana, Identity } from '@heima/parachain-api';

import { AuthenticationData } from '@type-creators/authentication';

import { intent } from './intent.request';

/**
 * Intent Request: Transfers SOL to another account on Solana.
 *
 * @param {ApiPromise} api - The  Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {string} data.to - The Solana address destination.
 * @param {bigint} data.amount - The amount to send in lamports.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to signature
 * (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function transferSolana(
  api: ApiPromise,
  data: {
    member: Identity;
    to: string;
    amount: bigint;
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
      TransferSolana: api.createType<IntentTransferSolana>('IntentTransferSolana', {
        to: base58.decode(data.to),
        value: data.amount,
      }),
    }),
  });
}
