import { assert } from '@polkadot/util';

import { JsonRpcRequest } from '@utils/types';

import { enclave, Enclave } from '@lib/enclave';

/**
 * Requests an email verification code to be sent to the specified email address.
 *
 * @param {Object} args - The args object containing the following properties:
 * @param {string} args.email - The email address to send the verification code to.
 * @param {Enclave} [enclaveInstance] - The enclave instance to use for the request.
 * @throws {Error} Throws an error if the email is empty.
 * @returns {Promise<void>} A promise that resolves when the request is sent.
 */
export async function requestEmailVerificationCode(args: { email: string }, enclaveInstance: Enclave = enclave): Promise<void> {
  const { email } = args;
  assert(email.length > 0, 'Email is required');

  // send the request to the Enclave
  const rpcRequest: JsonRpcRequest = {
    jsonrpc: '2.0',
    method: 'omni_requestEmailVerificationCode',
    params: [email],
  };

  await enclaveInstance.send(rpcRequest);
}
