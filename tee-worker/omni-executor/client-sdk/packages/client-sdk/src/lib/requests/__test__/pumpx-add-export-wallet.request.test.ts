import { requestEmailVerificationCode } from '@requests/request-email-verification-code.request';
import { enclave, Enclave } from '@lib/enclave';
import { PumpxRpcMethod } from '@lib/utils/rpc-method';
import { JsonRpcRequest } from '@utils/types';
describe.skip('add-export-wallet', () => {
  // step 1
  it('request email verification code', async () => {
    const rpcRequest = {
      jsonrpc: '2.0',
      method: 'omni_requestEmailVerificationCode',
      params: ['test@google.com'],
    };
    await enclave.send(rpcRequest);
  });

  // step 2
  it('request email verification code', async () => {
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'pumpx_requestJwt',
      params: ['test@google.com', '', '', '', '1223'],
    };

    const response = await enclave.send(rpcRequest);
    // console.log(response.auth_token);
  });
  it('add wallet should works', async () => {
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: PumpxRpcMethod.AddWallet,
      id: 1,
      params: [
        'test@google.com', // user_email
        '0xxxxxx', // auth_token already get from step 2
      ],
    };

    await enclave.send(rpcRequest);
  });

  it('export wallet should works', async () => {
    const aesRandomKey = globalThis.crypto.getRandomValues(new Uint8Array(32));
    const encryptedKey = await enclave.encrypt({ cleartext: aesRandomKey }); // already get shielding key
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: PumpxRpcMethod.ExportWallet,
      id: 1,
      params: [
        'test@google.com', // user_email
        encryptedKey.ciphertext.toString(), // key
        '123', // google_code
        '0', // chain_id
        '1', // wallet_index
        '0xdeadbeaf', // wallet_address
        '123', // email_code
      ],
    };
    await enclave.send(rpcRequest);
  });
});
