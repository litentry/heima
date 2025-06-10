import { assert } from '@polkadot/util';

import { JsonRpcRequest } from '@utils/types';

import { enclave, Enclave } from '@lib/enclave';

/**
 * Requests an email verification code to be sent to the specified email address.
 *
 * @param {Object} args - The args object containing the following properties:
 * @param {string} args.email - The email address to send the verification code to.
 * @param {string} args.clientId - The client ID to associate with the verification request.
 * @param {Enclave} [enclaveInstance] - The enclave instance to use for the request.
 * @throws {Error} Throws an error if the email or clientId is empty.
 * @returns {Promise<void>} A promise that resolves when the request is sent.
 */
export async function requestEmailVerificationCode(args: { email: string; clientId: string }, enclaveInstance: Enclave = enclave): Promise<void> {
  const { email, clientId } = args;
  assert(email.length > 0, 'Email is required');
  assert(clientId.length > 0, 'Client ID is required');

  // send the request to the Enclave
  const rpcRequest: JsonRpcRequest = {
    jsonrpc: '2.0',
    method: 'omni_requestEmailVerificationCode',
    params: {
      user_email: email,
      client_id: clientId,
    },
  };

  await enclaveInstance.send(rpcRequest);
}
