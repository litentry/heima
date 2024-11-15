import { enclave } from '../enclave';

import type { ApiPromise } from '@polkadot/api';
import type { JsonRpcRequest } from '../util/types';

/**
 * Request email verification code.
 */
export async function requestVerificationCode(
  api: ApiPromise,
  { omniAccount, email }: { omniAccount: `0x${string}`; email: string }
): Promise<{ success: boolean }> {
  const payload: JsonRpcRequest = {
    jsonrpc: '2.0',
    method: 'omni_requestEmailVerificationCode',
    params: [omniAccount, email],
  };

  const [workerResponse] = await enclave.send(api, payload);

  return { success: workerResponse.status.isOk };
}
