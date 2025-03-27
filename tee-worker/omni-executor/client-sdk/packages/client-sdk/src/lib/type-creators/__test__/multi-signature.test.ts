import { TypeRegistry } from '@polkadot/types';
import { base64Decode, cryptoWaitReady, randomAsHex } from '@polkadot/util-crypto';

import { identity } from '@heima-network/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { createMultiSignature } from '@type-creators/multi-signature';

const types = {
  ...identity.types, // Identity is defined here
};

let registry: TypeRegistry;

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);
});

test('Substrate: it works', () => {
  const substrateAddress = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

  const identity = createIdentityType(registry, {
    type: 'Substrate',
    addressOrHandle: substrateAddress,
  });

  // signature isn't validated
  const signature = randomAsHex(64);

  const multiSignature = createMultiSignature(registry, {
    who: identity,
    signature: signature,
  });

  expect(multiSignature).toBeDefined();
  expect(multiSignature.isSr25519).toBeTruthy();
  expect(multiSignature.asSr25519.eq(signature)).toBeTruthy();
});

test('it uses Ethereum for EVM', () => {
  const evmAddress = '0x0AcE67628Bd43213C1C41ca1DAEf47E63923c75c';

  const identity = createIdentityType(registry, {
    type: 'Evm',
    addressOrHandle: evmAddress,
  });

  // signature isn't validated
  const signature = randomAsHex(65);

  const multiSignature = createMultiSignature(registry, {
    who: identity,
    signature: signature,
  });

  expect(multiSignature).toBeDefined();
  expect(multiSignature.isEthereum).toBeTruthy();
  expect(multiSignature.asEthereum.eq(signature)).toBeTruthy();
});

test('it decodes Bitcoin signatures', () => {
  const bitcoinAddress = '03dea1fa79da3457bcb4dfd72357a2a8e932fd48e00cc29c5f1b6363d13828ad0e';

  const identity = createIdentityType(registry, {
    type: 'Bitcoin',
    addressOrHandle: bitcoinAddress,
  });

  const base64Signature = 'G16tPZuJIk4iZT9LB9L3EDOJMJ/yMCzKuowJDNar4op/c8MUfC2MAULTsadtQ3eiW3Q0yh1P0VWCNA7lf7F8Xro=';

  const multiSignature = createMultiSignature(registry, {
    who: identity,
    signature: base64Signature,
  });

  const expectedSignature = base64Decode(base64Signature);

  expect(multiSignature).toBeDefined();
  expect(multiSignature.isBitcoin).toBeTruthy();
  expect(multiSignature.asBitcoin.eq(base64Signature)).toBeFalsy();
  expect(multiSignature.asBitcoin.eq(expectedSignature)).toBeTruthy();
});
