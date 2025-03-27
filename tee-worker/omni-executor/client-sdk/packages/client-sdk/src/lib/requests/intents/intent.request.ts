import type { ApiPromise } from '@polkadot/api';
import { HexString } from '@polkadot/util/types';

import type { Intent, Identity } from '@heima-network/parachain-api';

import { AuthenticationData } from '@type-creators/authentication';
import { createNativeCallType } from '@type-creators/native-call';

import { aesCall } from '@requests/aes-call.request';

/**
 * Sends a common intent request to the Heima Parachain.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @param {Intent} data.intent - The intent to be sent.
 * @param {U8aLike} data.input - The contract input data.
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
export async function intent(
  api: ApiPromise,
  data: {
    member: Identity;
    intent: Intent;
  },
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<{
    blockHash: HexString;
    extrinsicHash: HexString;
    status: HexString;
  }>;
}> {
  const { member, intent } = data;

  const { operation } = createNativeCallType(api.registry, {
    method: 'request_intent',
    params: { member, intent },
  });

  return aesCall(api, { member, operation });
}
