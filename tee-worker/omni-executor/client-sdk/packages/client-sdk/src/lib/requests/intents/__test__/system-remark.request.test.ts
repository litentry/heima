import { ApiPromise, Keyring } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { u8aToHex } from '@polkadot/util';

import { getChain } from '@heima/chaindata';
import { identity, omniAccount, omniExecutor } from '@heima/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { requestAuthToken } from '@requests/request-auth-token.request';
import { systemRemark } from '@requests/intents/system-remark.request';

const types = {
  ...identity.types, // Identity is defined here
  ...omniAccount.types, // AuthOptions is defined here
  ...omniExecutor.types, // NativeCall is defined here
};

describe.skip('system-remark', () => {
  let api: ApiPromise;

  beforeAll(async () => {
    api = new ApiPromise({
      provider: new WsProvider(getChain('heima-local').rpcs[0].url),
      types,
    });

    await api.isReady;
  });

  it('auth token authentication', async () => {
    const keyring = new Keyring({ type: 'sr25519' });
    const memberSigner = keyring.addFromUri('//Dave');
    const member = createIdentityType(api.registry, {
      addressOrHandle: memberSigner.address,
      type: 'Substrate',
    });

    // Step 1: request auth token
    console.log('Step 1: request auth token');
    const { send, payloadToSign = '' } = await requestAuthToken(api, {
      member,
      expiresAt: 99999999,
    });

    const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

    const result = await send({
      authentication: {
        type: 'Web3',
        signer: member,
        signature: signatureHex,
      },
    });

    const token = result.token;
    expect(token).toBeDefined();

    // Step 2: system remark
    console.log('Step 2: system remark');
    await (async () => {
      const { send } = await systemRemark(api, {
        member,
        message: 'Hello, world!',
      });

      const result = await send({ authentication: { type: 'AuthToken', token } });

      expect(result.extrinsicHash.length).toBe(66);
      expect(result.blockHash.length).toBe(66);
      expect(result.status).toBeDefined();
    })();
  });
});
