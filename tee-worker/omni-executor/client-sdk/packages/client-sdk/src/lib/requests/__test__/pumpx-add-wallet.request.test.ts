import { enclave } from '@lib/enclave';
import { PumpxRpcMethod } from '@lib/utils/rpc-method';
import { JsonRpcRequest } from '@utils/types';

// This test is just an example. It requires receiving email verification codes, which cannot be done by running this unit test.
describe.skip('add-export-wallet', () => {
  // step 1
  it('request email verification code', async () => {
    const rpcRequest = {
      jsonrpc: '2.0',
      method: 'omni_requestEmailVerificationCode',
      params: {
        user_email: 'test@google.com',
      },
    };
    await enclave.send(rpcRequest);
  });

  // step 2
  it('request email verification code', async () => {
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: 'pumpx_requestJwt',
      params: {
        user_email: 'test@google.com',
        invite_code: '', // Optional
        google_code: '', // Optional
        language: 'en', // Optional
        email_code: '98121',
      },
    };

    const response = await enclave.send(rpcRequest);
    // console.log(response.auth_token);
  });

  // step 3
  it('add wallet should works', async () => {
    const rpcRequest: JsonRpcRequest = {
      jsonrpc: '2.0',
      method: PumpxRpcMethod.AddWallet,
      id: 1,
      params: {
        user_email: 'test@google.com',
        auth_token:
          '"eyJOeXAi0iJKV1QiLCJhbGci0iJSUzI1NiJ9.ey...6FcLY4XByKuT3a5Ax3L0En4LFbOLdGwrsNZDMOEU...iJhFisr4KHFFdjhphK_G9qn45vQ"',  //response.auth_token from step 2
      },
    };

    await enclave.send(rpcRequest);
  });
});
