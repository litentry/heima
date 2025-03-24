import { assert } from '@polkadot/util';

import { JsonRpcRequest } from '@lib/utils/types';

import { enclave } from '@lib/enclave';

export async function requestEmailVerificationCode(args: { email: string }): Promise<void> {
  const { email } = args;
  assert(email.length > 0, 'Email is required');

  // send the request to the Enclave
  const rpcRequest: JsonRpcRequest = {
    jsonrpc: '2.0',
    method: 'omni_requestEmailVerificationCode',
    params: [email],
  };

  await enclave.send(rpcRequest);
}
