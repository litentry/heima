import { enclave } from '@lib/enclave';
import { decryptWithAes } from '@lib/utils';
import { OmniRpcMethod, PumpxRpcMethod } from '@lib/utils/rpc-method';
import { JsonRpcRequest } from '@utils/types';
import { u8aToHex } from "@polkadot/util";
import {AesOutput} from "@heima-network/parachain-api"

const aesRandomKey = globalThis.crypto.getRandomValues(new Uint8Array(32));

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

  // step 2
  it('export wallet should work', async () => {
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

  // step 3
  it('decrypt response', async () => {
    const response = {
      ciphertext: "0xb0b6126dd7acae085cd4da94185d0707bb19257f6f95cac84c2c4b0ffc0e23995ac656c96f08abd007b32bafae5477f7",
      aad: "0x",
      nonce: "0xaad559c090dc5195ace3e3eb",
    };
    const decryptedKey = await decryptWithAes(u8aToHex(aesRandomKey), response as unknown as AesOutput, "hex");
    console.log("decryptedKey", decryptedKey);
  });
});