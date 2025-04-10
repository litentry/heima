import { enclave } from '@lib/enclave';
import { OmniRpcMethod, PumpxRpcMethod } from '@lib/utils/rpc-method';
import { JsonRpcRequest } from '@utils/types';

// This test is just an example. It requires receiving email verification codes, which cannot be done by running this unit test.
describe.skip('add-export-wallet', () => {
  // step 1
  it('request email verification code', async () => {
    const rpcRequest = {
      jsonrpc: '2.0',
      method: OmniRpcMethod.RequestEmailVerificationCode,
      params: {
        user_email: 'test@google.com',
      },
    };
    await enclave.send(rpcRequest);
  });

  // step2
  it('export wallet should works', async () => {
    const aesRandomKey = globalThis.crypto.getRandomValues(new Uint8Array(32));
    const encryptedKey = await enclave.encrypt({ cleartext: aesRandomKey }); // already get shielding key
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: PumpxRpcMethod.ExportWallet,
      id: 1,
      params: {
        user_email: 'test@google.com',
        key: encryptedKey.ciphertext.toString(),
        google_code: '123',
        chain_id: '0',
        wallet_index: '1',
        wallet_address: '0xdeadbeaf',
        email_code: '98121',
      }
    };
    await enclave.send(rpcRequest);
  });
});
