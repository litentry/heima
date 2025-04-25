import { ApiPromise, Keyring } from '@polkadot/api';
import { WsProvider } from '@polkadot/rpc-provider';
import { u8aToHex } from '@polkadot/util';

import { identity, omniExecutor, sidechain } from '@heima-network/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { callEthereum } from '@requests/intents/call-ethereum.request';

const types = {
  ...identity.types, // Identity is defined here
  ...omniExecutor.types, // NativeCall is defined here
  ...sidechain.types, // AesOutput is defined here
};

describe.skip('call-ethereum', () => {
  let api: ApiPromise;

  beforeAll(async () => {
    api = new ApiPromise({
      provider: new WsProvider(process.env.PARACHAIN_NETWORK),
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

    const { send, getPayloadToSign = () => '' } = await callEthereum(api, {
      member,
      address: '0x0000000000000000000000000000000000000000',
      input: '0x',
    });

    const payloadToSign = await getPayloadToSign();
    const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

    const result = await send({
      authData: {
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
