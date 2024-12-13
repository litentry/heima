import { TypeRegistry } from '@polkadot/types';
import {
  base64Decode,
  cryptoWaitReady,
  randomAsHex,
} from '@polkadot/util-crypto';

import { identity } from '@litentry/parachain-api';

import { createLitentryIdentityType } from './litentry-identity';
import { createLitentryMultiSignature } from './litentry-multi-signature';

const types = {
  ...identity.types, // LitentryIdentity is defined here
};

let registry: TypeRegistry;

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);
});

test('Substrate: it works', () => {
  const substrateAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

  const identity = createLitentryIdentityType(registry, {
    type: 'Substrate',
    addressOrHandle: substrateAddress,
  });

  // signature isn't validated
  const signature = randomAsHex(64);

  const litentrySignature = createLitentryMultiSignature(registry, {
    who: identity,
    signature: signature,
  });

  expect(litentrySignature).toBeDefined();
  expect(litentrySignature.isSr25519).toBeTruthy();
  expect(litentrySignature.asSr25519.eq(signature)).toBeTruthy();
});

test('it uses Ethereum for EVM', () => {
  const evmAddress = '0x0AcE67628Bd43213C1C41ca1DAEf47E63923c75c';

  const identity = createLitentryIdentityType(registry, {
    type: 'Evm',
    addressOrHandle: evmAddress,
  });

  // signature isn't validated
  const signature = randomAsHex(65);

  const litentrySignature = createLitentryMultiSignature(registry, {
    who: identity,
    signature: signature,
  });

  expect(litentrySignature).toBeDefined();
  expect(litentrySignature.isEthereum).toBeTruthy();
  expect(litentrySignature.asEthereum.eq(signature)).toBeTruthy();
});

test('it decodes Bitcoin signatures', () => {
  const bitcoinAddress =
    '03dea1fa79da3457bcb4dfd72357a2a8e932fd48e00cc29c5f1b6363d13828ad0e';

  const identity = createLitentryIdentityType(registry, {
    type: 'Bitcoin',
    addressOrHandle: bitcoinAddress,
  });

  const base64Signature =
    'G16tPZuJIk4iZT9LB9L3EDOJMJ/yMCzKuowJDNar4op/c8MUfC2MAULTsadtQ3eiW3Q0yh1P0VWCNA7lf7F8Xro=';

  const litentrySignature = createLitentryMultiSignature(registry, {
    who: identity,
    signature: base64Signature,
  });

  const expectedSignature = base64Decode(base64Signature);

  expect(litentrySignature).toBeDefined();
  expect(litentrySignature.isBitcoin).toBeTruthy();
  expect(litentrySignature.asBitcoin.eq(base64Signature)).toBeFalsy();
  expect(litentrySignature.asBitcoin.eq(expectedSignature)).toBeTruthy();
});
