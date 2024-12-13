import { u8aToHex } from '@polkadot/util';
import { TypeRegistry } from '@polkadot/types';
import { cryptoWaitReady, addressEq } from '@polkadot/util-crypto';

import { identity } from '@litentry/parachain-api';

import { createLitentryIdentityType } from './litentry-identity';

const types = {
  ...identity.types, // LitentryIdentity is defined here
};

let registry: TypeRegistry;

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);
});

test('it creates Substrate identity', () => {
  const substrateAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

  const identity = createLitentryIdentityType(registry, {
    type: 'Substrate',
    addressOrHandle: substrateAddress,
  });

  expect(identity).toBeDefined();
  expect(identity.isSubstrate).toBeTruthy();
  expect(
    addressEq(identity.asSubstrate.toHex(), substrateAddress)
  ).toBeTruthy();
});

test('it creates Evm identity', () => {
  const evmAddress = '0x0AcE67628Bd43213C1C41ca1DAEf47E63923c75c';

  const identity = createLitentryIdentityType(registry, {
    type: 'Evm',
    addressOrHandle: evmAddress,
  });

  expect(identity).toBeDefined();
  expect(identity.isEvm).toBeTruthy();
  expect(identity.asEvm.toHex()).toEqual(evmAddress.toLowerCase());
});

test('it creates Bitcoin identity', () => {
  const bitcointPk =
    '03dea1fa79da3457bcb4dfd72357a2a8e932fd48e00cc29c5f1b6363d13828ad0e';

  const identity = createLitentryIdentityType(registry, {
    addressOrHandle: bitcointPk,
    type: 'Bitcoin',
  });

  expect(identity).toBeDefined();
  expect(identity.isBitcoin).toBeTruthy();
  expect(identity.asBitcoin.toHex()).toEqual(`0x${bitcointPk.toLowerCase()}`);
});

test('it creates Solana identity from base58-encoded string', () => {
  // Solana pubkey generated with solana-cli
  const publicKey = [
    62, 98, 142, 133, 73, 223, 90, 233, 203, 16, 83, 42, 79, 192, 27, 0, 78, 62,
    120, 211, 178, 86, 134, 94, 179, 13, 149, 234, 209, 247, 224, 230,
  ];
  const address = '5CXSbcqN6hS5skWhViaJs1bYzVdL6KS7X1ANSoH3Uhc1';

  const identity = createLitentryIdentityType(registry, {
    addressOrHandle: address,
    type: 'Solana',
  });

  expect(identity).toBeDefined();
  expect(identity.isSolana).toBeTruthy();
  expect(identity.asSolana.eq(publicKey)).toEqual(true);
});

test('it creates Solana identity from hex-encoded string', () => {
  // Solana pubkey generated with solana-cli
  const publicKey = [
    62, 98, 142, 133, 73, 223, 90, 233, 203, 16, 83, 42, 79, 192, 27, 0, 78, 62,
    120, 211, 178, 86, 134, 94, 179, 13, 149, 234, 209, 247, 224, 230,
  ];
  const address = u8aToHex(new Uint8Array(publicKey));

  const identity = createLitentryIdentityType(registry, {
    addressOrHandle: address,
    type: 'Solana',
  });

  expect(identity).toBeDefined();
  expect(identity.isSolana).toBeTruthy();
  expect(identity.asSolana.eq(publicKey)).toEqual(true);
});

test('it creates Twitter identity', () => {
  const handle = 'my-haNdle';

  const identity = createLitentryIdentityType(registry, {
    type: 'Twitter',
    addressOrHandle: handle,
  });

  expect(identity).toBeDefined();
  expect(identity.isTwitter).toBeTruthy();
  expect(identity.asTwitter.toHuman()).toEqual(handle);
});

test('it creates Discord identity', () => {
  const handle = 'my-haNdle';

  const identity = createLitentryIdentityType(registry, {
    type: 'Discord',
    addressOrHandle: handle,
  });

  expect(identity).toBeDefined();
  expect(identity.isDiscord).toBeTruthy();
  expect(identity.asDiscord.toHuman()).toEqual(handle);
});

test('it creates Email identity', () => {
  const handle = 'test@test.com.not.valid';

  const identity = createLitentryIdentityType(registry, {
    type: 'Email',
    addressOrHandle: handle,
  });

  expect(identity).toBeDefined();
  expect(identity.isEmail).toBeTruthy();
  expect(identity.asEmail.toHuman()).toEqual(handle);
});
