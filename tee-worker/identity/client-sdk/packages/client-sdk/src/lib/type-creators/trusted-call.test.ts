import { webcrypto } from 'crypto';
import { TypeRegistry } from '@polkadot/types';
import { cryptoWaitReady, randomAsHex } from '@polkadot/util-crypto';

import {
  vc,
  identity,
  sidechain,
  trusted_operations,
  type LitentryIdentity,
  type Assertion,
} from '@litentry/parachain-api';

import { createTrustedCallType } from './trusted-call';
import { createLitentryIdentityType } from './litentry-identity';
import { createLitentryValidationDataType } from './validation-data';
import { exportKey } from '../util/shielding-key';

const types = {
  ...identity.types, // LitentryIdentity is defined here
  ...sidechain.types, // RequestAesKey type is defined here
  ...trusted_operations.types, // TrustedCall types are defined here
  ...vc.types, // Assertion types are defined here
};

let registry: TypeRegistry;
let aliceIdentity: LitentryIdentity;
const txHash = randomAsHex(32);

const originalCrypto = globalThis.crypto;

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);

  // set up data
  aliceIdentity = createLitentryIdentityType(registry, {
    addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    type: 'Substrate',
  });

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

describe('LinkIdentity', () => {
  test('it works', async () => {
    const validationData = createLitentryValidationDataType(
      registry,
      {
        addressOrHandle: 'hello',
        type: 'Twitter',
      },
      { tweetId: '0x123' }
    );

    const { call, key } = await createTrustedCallType(registry, {
      method: 'link_identity',
      params: {
        who: aliceIdentity,
        identity: aliceIdentity,
        validation: validationData,
        networks: [],
        hash: txHash,
      },
    });

    const keyU8 = await exportKey(key);

    expect(key).toBeDefined();
    expect(call).toBeDefined();
    expect(call.isLinkIdentity).toBeTruthy();
    expect(call.asLinkIdentity[0].eq(aliceIdentity)).toBeTruthy(); // signer
    expect(call.asLinkIdentity[1].eq(aliceIdentity)).toBeTruthy(); // who
    expect(call.asLinkIdentity[2].eq(aliceIdentity)).toBeTruthy(); // identity
    expect(call.asLinkIdentity[3].eq(validationData)).toBeTruthy();
    expect(call.asLinkIdentity[4].eq([])).toBeTruthy();
    expect(call.asLinkIdentity[5].eq(keyU8)).toBeTruthy();
    expect(call.asLinkIdentity[6].eq(txHash)).toBeTruthy();
  });
});

describe('SetIdentityNetworks', () => {
  test('it works', async () => {
    const { call, key } = await createTrustedCallType(registry, {
      method: 'set_identity_networks',
      params: {
        who: aliceIdentity,
        identity: aliceIdentity,
        networks: [],
        hash: txHash,
      },
    });

    const keyU8 = await exportKey(key);

    expect(call).toBeDefined();
    expect(call.isSetIdentityNetworks).toBeTruthy();
    expect(call.asSetIdentityNetworks[0].eq(aliceIdentity)).toBeTruthy(); // signer
    expect(call.asSetIdentityNetworks[1].eq(aliceIdentity)).toBeTruthy(); // who
    expect(call.asSetIdentityNetworks[2].eq(aliceIdentity)).toBeTruthy(); // identity
    expect(call.asSetIdentityNetworks[3].eq([])).toBeTruthy(); // network
    expect(call.asSetIdentityNetworks[4].eq(keyU8)).toBeTruthy();
    expect(call.asSetIdentityNetworks[5].eq(txHash)).toBeTruthy(); // txhash
    // and nothing else
    expect(call.asSetIdentityNetworks[6]).toBeUndefined(); //
  });
});

describe('RequestBatchVc', () => {
  test('it works', async () => {
    const assertion = registry.createType('Assertion', {
      A20: [],
    }) as Assertion;

    const { call, key } = await createTrustedCallType(registry, {
      method: 'request_batch_vc',
      params: {
        signer: aliceIdentity,
        who: aliceIdentity,
        assertions: [assertion],
        hash: txHash,
      },
    });

    const keyU8 = await exportKey(key);

    expect(call).toBeDefined();
    expect(call.isRequestBatchVc).toBeTruthy();
    expect(call.asRequestBatchVc[0].eq(aliceIdentity)).toBeTruthy(); // signer
    expect(call.asRequestBatchVc[1].eq(aliceIdentity)).toBeTruthy(); // who
    expect(call.asRequestBatchVc[2].eq([assertion])).toBeTruthy(); // identity
    expect(call.asRequestBatchVc[3].eq(keyU8)).toBeTruthy();
    expect(call.asRequestBatchVc[4].eq(txHash)).toBeTruthy(); // txhash
    // and nothing else
    expect(call.asRequestBatchVc[5]).toBeUndefined(); //
  });
});
