import { ApiPromise, Keyring } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { u8aToHex } from '@polkadot/util';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import { identity, omniAccount, omniExecutor } from '@heima/parachain-api';

import { requestAuthToken } from '@requests/request-auth-token.request';
import { createIdentityType } from '@type-creators/identity';

const types = {
  ...identity.types, // Identity is defined here
  ...omniAccount.types, // AuthOptions is defined here
  ...omniExecutor.types, // NativeCall is defined here
};

describe('request-auth-token', () => {
  let api: ApiPromise;

  beforeAll(async () => {
    api = new ApiPromise({
      provider: new WsProvider('ws://localhost:9944'),
      types,
    });

    await api.isReady;
    await cryptoWaitReady();
  });

  it('web3', async () => {
    const keyring = new Keyring({ type: 'sr25519' });
    const memberSigner = keyring.addFromUri('//Dave');
    const member = createIdentityType(api.registry, {
      addressOrHandle: memberSigner.address,
      type: 'Substrate',
    });

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

    expect(result.token.length).toBeGreaterThan(0);
  });
});
