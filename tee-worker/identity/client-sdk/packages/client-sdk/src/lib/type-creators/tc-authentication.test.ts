import { cryptoWaitReady } from '@polkadot/util-crypto';
import { TypeRegistry } from '@polkadot/types';
import { trusted_operations, identity } from '@litentry/parachain-api';
import { createTCAuthenticationType } from './tc-authentication';
import { createLitentryIdentityType } from './litentry-identity';
import { createLitentryMultiSignature } from './litentry-multi-signature';

describe('createTCAuthenticationType', () => {
  const types = {
    ...trusted_operations.types,
    ...identity.types,
  };
  let registry: TypeRegistry;

  beforeAll(async () => {
    await cryptoWaitReady();

    registry = new TypeRegistry();
    registry.register(types);
  });

  it('creates Email authentication', () => {
    const emailTCAuthentication = createTCAuthenticationType(registry, {
      type: 'Email',
      verificationCode: '123456',
    });

    expect(emailTCAuthentication).toBeDefined();
    expect(emailTCAuthentication.isEmail).toEqual(true);
    expect(emailTCAuthentication.asEmail.toHuman()).toEqual('123456');
  });

  it('creates Web3 authentication', () => {
    const who = createLitentryIdentityType(registry, {
      type: 'Substrate',
      addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    });
    const signatureString = '0x' + '12'.repeat(64);
    const signature = createLitentryMultiSignature(registry, {
      who,
      signature: signatureString,
    });

    const web3TCAuthentication = createTCAuthenticationType(registry, {
      type: 'Web3',
      signature: signature.toHex(),
    });

    expect(web3TCAuthentication).toBeDefined();
    expect(web3TCAuthentication.isWeb3).toEqual(true);
    expect(web3TCAuthentication.asWeb3.toHuman()).toEqual({
      Sr25519: signatureString,
    });
  });
});
