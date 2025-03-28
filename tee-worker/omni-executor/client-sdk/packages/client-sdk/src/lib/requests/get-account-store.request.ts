import type { ApiPromise } from '@polkadot/api';

import type { Identity } from '@heima-network/parachain-api';

import { AuthenticationData } from '@type-creators/authentication';
import { createNativeQueryType } from '@type-creators/native-query';

import { aesQuery } from './aes-query.request';

/**
 * Gets an account store from the Enclave.
 *
 * @param {ApiPromise} api - The Heima Parachain API instance from Polkadot.js.
 * @param {Object} data - The data object containing the following properties:
 * @param {Identity} data.member - The member account of the OmniAccount. Use the `createIdentityType` helper to create this structure.
 * @returns {Promise<Object>} A promise that resolves to an object containing the payload to sign (if applicable) and a send function.
 * @returns {string} payloadToSign - The payload to sign if the identity is a Web3 identity.
 * @returns {Function} send - A function to send the request to the Enclave.
 * @returns {Promise<Object>} send.args The arguments required to send the request.
 * @returns {AuthenticationData} send.args.authentication - The authentication data.
 * @returns {Array<Identity>} send.return - The account store.
 */
export async function getAccountStore(
  api: ApiPromise,
  data: {
    member: Identity;
  },
): Promise<{
  payloadToSign?: string;
  send: (args: { authentication: AuthenticationData }) => Promise<Identity[]>;
}> {
  const { member } = data;

  const { operation } = createNativeQueryType(api.registry, {
    method: 'get_account_store',
    params: {
      member,
    },
  });

  const { payloadToSign, send } = await aesQuery(api, { member, operation });

  const wrappedSend = async (args: { authentication: AuthenticationData }): Promise<Identity[]> => {
    const response = await send(args);
    if (!response.isAccountStore) {
      throw new Error('Unexpected query response type');
    }

    return response.asAccountStore;
  };

  return {
    payloadToSign,
    send: wrappedSend,
  };
}
