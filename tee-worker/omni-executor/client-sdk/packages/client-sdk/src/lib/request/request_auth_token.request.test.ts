import { ApiPromise, Keyring } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { requestAuthToken } from './request_auth_token.request';
import { createLitentryIdentityType } from '../type-creators/litentry-identity';
import { identity, omniAccount, omniExecutor, sidechain } from '@litentry/parachain-api';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';

const types = {
  ...identity.types, // LitentryIdentity is defined here
  ...omniAccount.types, // AuthOptions is defined here
  ...omniExecutor.types, // NativeCall is defined here
  ...sidechain.types, // AesOutput is defined here
};

describe('request_auth_token.request', () => {
  it('request_auth_token', async () => {
    const api = new ApiPromise({
      provider: new WsProvider('ws://localhost:9944'),
      types,
    });

    await api.isReady;

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

    const { send, call, nonce, shard } = await requestAuthToken(
      api,
      {
        omniAccount,
        who,
        expiresAt: 99999999,
      },
    );
    

    const payload = Buffer.concat([
      call.toU8a(),
      nonce.toU8a(),
      hexToU8a(shard),
    ]);

    const signature = alice.sign(`Token: ${blake2AsHex(payload, 256)}`);
    const signatureHex = u8aToHex(signature);

    const result = await send({
      authentication: {
        type: 'Web3',
        signer: who,
        signature: signatureHex,
      },
    });

    console.log('result', result);
  });
});
