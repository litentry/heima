import { ApiPromise, Keyring } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { u8aToHex } from '@polkadot/util';

import { getChain } from '@heima/chaindata';
import { identity, omniAccount, omniExecutor } from '@heima/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { transferNative } from '@requests/intents/transfer-native.request';

const types = {
  ...identity.types, // Identity is defined here
  ...omniAccount.types, // AuthOptions is defined here
  ...omniExecutor.types, // NativeCall is defined here
};

describe.skip('transfer-native', () => {
  let api: ApiPromise;

  beforeAll(async () => {
    api = new ApiPromise({
      provider: new WsProvider(getChain('heima-local').rpcs[0].url),
      types,
    });

    await api.isReady;
  });

  it('web3 authentication', async () => {
    const keyring = new Keyring({ type: 'sr25519' });
    const memberSigner = keyring.addFromUri('//Dave');
    const member = createIdentityType(api.registry, {
      addressOrHandle: memberSigner.address,
      type: 'Substrate',
    });

    const { send, payloadToSign = '' } = await transferNative(api, {
      member,
      to: '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw',
      amount: BigInt(100),
    });

    const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

    const result = await send({
      authentication: {
        type: 'Web3',
        signer: member,
        signature: signatureHex,
      },
    });

    expect(result.extrinsicHash.length).toBe(66);
    expect(result.blockHash.length).toBe(66);
    expect(result.status).toBeDefined();
  });
});
