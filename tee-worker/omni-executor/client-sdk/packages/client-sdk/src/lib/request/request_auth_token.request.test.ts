import { ApiPromise, Keyring } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { u8aToHex } from '@polkadot/util';
import { identity, omniAccount, omniExecutor, sidechain } from '@litentry/parachain-api';
import { requestAuthToken } from './request_auth_token.request';
import { createLitentryIdentityType } from '../type-creators/litentry-identity';

const types = {
  ...identity.types, // LitentryIdentity is defined here
  ...omniAccount.types, // AuthOptions is defined here
  ...omniExecutor.types, // NativeCall is defined here
  ...sidechain.types, // AesOutput is defined here
};

describe('request_auth_token', () => {
  let api: ApiPromise;

  beforeAll(async () => {
    api = new ApiPromise({
      provider: new WsProvider('ws://localhost:9944'),
      types,
    });

    await api.isReady;
  });

  it('web3', async () => {
    const substrateAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

    const omniAccount = createLitentryIdentityType(api.registry, {
      type: 'Substrate',
      addressOrHandle: substrateAddress,
    });

    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const who = createLitentryIdentityType(api.registry, {
      addressOrHandle: alice.address,
      type: 'Substrate',
    });

    const { send, payloadToSign } = await requestAuthToken(
      api,
      {
        omniAccount,
        who,
        expiresAt: 99999999,
      },
      true,
    );

    const signatureHex = u8aToHex(alice.sign(payloadToSign!));

    const result = await send({
      authentication: {
        type: 'Web3',
        signer: who,
        signature: signatureHex,
      },
    });

    expect(result.token.length).toBeGreaterThan(0);
  });
});
