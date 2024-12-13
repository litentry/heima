import { TypeRegistry } from '@polkadot/types';
import { identity } from '@litentry/parachain-api';

import { safelyDecodeOption } from './safely-decode-option';

const registry = new TypeRegistry();
registry.register({
  ...identity.types,
});

describe('IdGraph decoder', () => {
  it('correctly decodes an empty IdGraph', async () => {
    const workerResponseValue = '0x010400';

    const decoded = safelyDecodeOption(registry, {
      value: workerResponseValue,
      type: 'Vec<(LitentryIdentity, IdentityContext)>',
    });

    expect(decoded.isEmpty).toBeTruthy();
    expect(decoded.length).toBe(0);
  });

  it('correctly decodes an non-empty IdGraph', async () => {
    // real value got from workerRpcReturn.value struct
    const workerResponseValue =
      '0x01ec0804e50cf8a7fbe830dd6238aa811325f799220576ca010000000807080004d82308ce12a1383e8380d522cce61d414899e1992ef9000008070800';

    const decoded = safelyDecodeOption(registry, {
      value: workerResponseValue,
      type: 'Vec<(LitentryIdentity, IdentityContext)>',
    });

    expect(decoded.isEmpty).toBeFalsy();
    expect(decoded.length).toBe(2);

    const [identity1, identity2] = decoded;

    expect(
      identity1[0].asEvm.eq('0xE50Cf8a7fBE830dd6238aA811325f799220576Ca')
    ).toBeTruthy();

    expect(
      identity2[0].asEvm.eq('0xd82308cE12A1383e8380D522Cce61d414899E199')
    ).toBeTruthy();
    expect(identity2[1].link_block.toNumber()).toEqual(63790);
  });
});
