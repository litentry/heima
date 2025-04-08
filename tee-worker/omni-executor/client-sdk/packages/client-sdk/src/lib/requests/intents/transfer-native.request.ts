import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Intent, IntentTransferNative, Identity } from '@heima-network/parachain-api';

import { OmniAuthData } from '@type-creators/omni-auth';
import { enclave, Enclave } from '@lib/enclave';

import { intent } from './intent.request';

/**
 * Intent Request: Transfers native tokens to another account on the  Parachain.
 *
 * @param {ApiPromise} api - The  Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {string} data.to - The account destination in hex or ss58 formatted address.
 * @param {bigint} data.amount - The amount to send.
 * @param {Enclave} enclaveInstance - The enclave instance use to interact with Enclave.
 * @returns {Promise<Object>} - A promise that resolves to an object containing the payload to sign
 * (if applicable) and a send function.
 * @returns {Function} getPayloadToSign - A function to get the payload that needs to be signed (only for Web3 identities)
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args - The arguments required to send the request.
 * @returns {OmniAuthData} send.args.authData - The authentication data.
 * @returns {HexString} send.return.blockHash - Block hash of the transaction
 * @returns {HexString} send.return.extrinsicHash - Extrinsic hash of the transaction
 * @returns {HexString} send.return.status - Status of the transaction
 */
export async function transferNative(
  /**  Parachain API instance from Polkadot.js */
  api: ApiPromise,
  data: {
    member: Identity;
    to: string;
    amount: bigint;
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
  return intent(api, {
    member: data.member,
    intent: api.createType<Intent>('Intent', {
      TransferNative: api.createType<IntentTransferNative>('IntentTransferNative', {
        to: data.to,
        value: data.amount,
      }),
    }),
  }, enclaveInstance);
}
