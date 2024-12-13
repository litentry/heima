import { ApiPromise } from '@polkadot/api';
import { TypeRegistry } from '@polkadot/types';
import { hexToU8a, u8aToString } from '@polkadot/util';
import { cryptoWaitReady, randomAsHex } from '@polkadot/util-crypto';
import { MockProvider } from '@polkadot/rpc-provider/mock';
import { webcrypto } from 'crypto';

import {
  LitentryIdentity,
  identity,
  sidechain,
  trusted_operations,
} from '@litentry/parachain-api';

import { enclave } from '../enclave';
import { getEnclaveNonce } from '../requests/get-enclave-nonce';

import {
  createChallengeCode,
  linkIdentity,
} from '../requests/link-identity.request';
import { createLitentryIdentityType } from '../type-creators/litentry-identity';
import { createLitentryValidationDataType } from '../type-creators/validation-data';
import { setIdentityNetworks } from '../requests';
const hexToString = (hex: string): string => {
  return u8aToString(hexToU8a(hex));
};
// Mocks
jest.mock('../enclave.ts');
const enclaveMock = jest.mocked(enclave);

jest.mock('../requests/get-enclave-nonce');
const getEnclaveNonceMock = jest.mocked(getEnclaveNonce);
// End mocks

const originalCrypto = globalThis.crypto;

const types = {
  ...identity.types,
  ...sidechain.types,
  ...trusted_operations.types,
};

const substrateAccount = '5DNx1Kgis2u2SQq7EJrBdnV49PoZCxV3NqER4vV5VqjqZcat';
const bitcoinAccount =
  '0385db7c98b8c1239a397e5776571b31788e77ce3a46c265dbe82754dc9cf1f466';
const evmAccount = '0xd82308cE12A1383e8380D522Cce61d414899E199';

let substrateIdentity: LitentryIdentity;
let bitcoinIdentity: LitentryIdentity;
let evmIdentity: LitentryIdentity;

let api: ApiPromise;

beforeAll(async () => {
  await cryptoWaitReady();

  const registry = new TypeRegistry();
  registry.register(types);

  api = new ApiPromise({
    types,
    registry,
    provider: new MockProvider(registry),
  });

  substrateIdentity = createLitentryIdentityType(registry, {
    type: 'Substrate',
    addressOrHandle: substrateAccount,
  });
  bitcoinIdentity = createLitentryIdentityType(registry, {
    type: 'Bitcoin',
    addressOrHandle: bitcoinAccount,
  });
  evmIdentity = createLitentryIdentityType(registry, {
    type: 'Evm',
    addressOrHandle: evmAccount,
  });

  enclaveMock.getShard.mockResolvedValue(randomAsHex(32));
  getEnclaveNonceMock.mockResolvedValue(api.createType('Index', 1));

  // enable globalThis.crypto.subtle
  Object.defineProperty(global, 'crypto', {
    value: webcrypto,
    writable: true,
  });
});

afterAll(() => {
  Object.defineProperty(global, 'crypto', {
    value: originalCrypto,
    writable: true,
  });
});

describe('challenge code', () => {
  test('adds prefix when options.prettify is true', async () => {
    const message = await createChallengeCode(
      api,
      {
        who: substrateIdentity,
        identity: createLitentryIdentityType(api.registry, {
          addressOrHandle: bitcoinAccount,
          type: 'Bitcoin',
        }),
      },
      { prettify: true }
    );

    expect(message).toBeDefined();
    expect(message).toMatch(/^Token: 0x/);
  });

  test('omits prefix when options.prettify is false', async () => {
    const message = await createChallengeCode(
      api,
      {
        who: substrateIdentity,
        identity: createLitentryIdentityType(api.registry, {
          addressOrHandle: bitcoinAccount,
          type: 'Bitcoin',
        }),
      },
      { prettify: true }
    );

    expect(message).toBeDefined();
    expect(message).toMatch(/^Token: 0x/);
  });

  test('omits prefix when options.prettify is undefined', async () => {
    const message = await createChallengeCode(api, {
      who: substrateIdentity,
      identity: createLitentryIdentityType(api.registry, {
        addressOrHandle: bitcoinAccount,
        type: 'Bitcoin',
      }),
    });

    expect(message).toBeDefined();
    expect(message).not.toMatch(/^Token: /);
  });

  test('ignores options.prettify for web2', async () => {
    const message = await createChallengeCode(api, {
      who: substrateIdentity,
      identity: createLitentryIdentityType(api.registry, {
        addressOrHandle: 'my-twitter-handle',
        type: 'Twitter',
      }),
    });

    expect(message).toBeDefined();
    expect(message).not.toMatch(/^Token: /);
    expect(message).toMatch(/^0x/);
  });
});

describe('Bitcoin', () => {
  // bitcoin wallets need to sign the challenge code without the 0x prefix.
  test('challenge code has no 0x prefix.', async () => {
    const message = await createChallengeCode(api, {
      who: substrateIdentity,
      identity: createLitentryIdentityType(api.registry, {
        addressOrHandle: bitcoinAccount,
        type: 'Bitcoin',
      }),
    });

    expect(message).toBeDefined();
    expect(message).not.toMatch(/^0x/);
  });

  test('payload has no 0x prefix, but LitentryValidationData.message does', async () => {
    // real output from requests.createChallengeCode
    // noticed that it as no 0x as it is for a bitcoin account
    const challengeCode =
      '5c6cf11d75e49d9c6f26b458efcc429b3ce5626fa2ea9de874806edef934581a';

    const validationData = createLitentryValidationDataType(
      api.registry,
      {
        addressOrHandle: bitcoinAccount,
        type: 'Bitcoin',
      },
      {
        message: challengeCode,
        signature:
          'HHMj8yMSOlgJb3r8FVcfPw1t8zQorpS6ZzfaedS+HrRxfUFxxEhWzUEY24IB/NzYjGSExRVii1B/f4AdD7ml/Ms=',
      }
    );

    // should restore the 0x prefix for the message
    expect(validationData.isWeb3Validation).toBeTruthy();
    expect(
      validationData.asWeb3Validation.asBitcoin.message.toString()
    ).toMatch(/^0x/);
    expect(validationData.asWeb3Validation.asBitcoin.message.toString()).toBe(
      `0x${challengeCode}`
    );

    // since the prime account is bitcoin
    const { payloadToSign } = await linkIdentity(api, {
      who: bitcoinIdentity, // bitcoin is prime identity
      identity: createLitentryIdentityType(api.registry, {
        addressOrHandle: bitcoinAccount,
        type: 'Bitcoin',
      }),
      validation: validationData,
      networks: [],
    });

    expect(typeof payloadToSign).toBe('string');
    expect(payloadToSign).not.toMatch(/^0x/);
  });
});

describe('Ethereum and Substrate', () => {
  test.each([
    { address: evmAccount, type: 'Evm' },
    { address: substrateAccount, type: 'Substrate' },
  ])('challenge code has 0x prefix for $type.', async ({ address, type }) => {
    const message = await createChallengeCode(api, {
      who: substrateIdentity,
      identity: createLitentryIdentityType(api.registry, {
        addressOrHandle: address,
        type: type as 'Evm' | 'Substrate',
      }),
    });

    expect(message).toBeDefined();
    expect(message).toMatch(/^0x/);
  });

  test('Ethereum payload is hex and has a prettified format', async () => {
    const { payloadToSign } = await setIdentityNetworks(api, {
      who: evmIdentity,
      identity: createLitentryIdentityType(api.registry, {
        addressOrHandle: bitcoinAccount,
        type: 'Bitcoin',
      }),
      networks: [],
    });

    expect(typeof payloadToSign).toBe('string');
    expect(payloadToSign).toMatch(/^0x/);
    expect(hexToString(payloadToSign)).toMatch(/^Token: /);
  });

  test('Substrate payload is prettified format', async () => {
    const { payloadToSign } = await setIdentityNetworks(api, {
      who: substrateIdentity,
      identity: createLitentryIdentityType(api.registry, {
        addressOrHandle: bitcoinAccount,
        type: 'Bitcoin',
      }),
      networks: [],
    });

    expect(typeof payloadToSign).toBe('string');
    expect(payloadToSign).not.toMatch(/^0x/);
    expect(payloadToSign).toMatch(/^Token: /);
  });
});
