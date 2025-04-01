import type { ApiPromise } from '@polkadot/api';
import type { HexString, U8aLike } from '@polkadot/util/types';

import type { Intent, IntentCallEthereum, Identity } from '@heima-network/parachain-api';

import { OmniAuthData } from '@type-creators/omni-auth';

import { intent } from './intent.request';

/**
 * Intent Request: Call an Ethereum contract.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {string} data.address - The Ethereum contract address.
 * @param {U8aLike} data.input - The contract input data.
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
export async function callEthereum(
  api: ApiPromise,
  data: {
    member: Identity;
    address: string;
    input: U8aLike;
  },
): Promise<{
  getPayloadToSign?: () => Promise<string>;
  send: (args: { authData: OmniAuthData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  return intent(api, {
    member: data.member,
    intent: api.createType<Intent>('Intent', {
      CallEthereum: api.createType<IntentCallEthereum>('IntentCallEthereum', {
        address: data.address,
        input: data.input,
      }),
    }),
  });
}
