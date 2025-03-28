import { ApiPromise, WsProvider } from '@polkadot/api';

import { getChain } from '@heima-network/chaindata';
import { identity } from '@heima-network/parachain-api';

import { getOmniAccountNonceWithIdentity } from '@requests/get-nonce.request';
import { createIdentityType } from '@type-creators/identity';

const types = {
  ...identity.types, // Identity is defined here
};

describe('get-nonce', () => {
  let api: ApiPromise;

  beforeAll(async () => {
    api = new ApiPromise({
      provider: new WsProvider(getChain('heima-local').rpcs[0].url),
      types,
    });

    await api.isReady;
  });

  it('should works', async () => {
    // ensure the omni account is created for this identity
    const member = createIdentityType(api.registry, {
      // //Dave
      addressOrHandle: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
      type: 'Substrate',
    });

    const nonce = await getOmniAccountNonceWithIdentity(api, member);

    expect(nonce.toNumber()).toBeGreaterThanOrEqual(0);
  });
});
