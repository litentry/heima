import { TypeRegistry } from '@polkadot/types';
import {
  blake2AsHex,
  cryptoWaitReady,
  randomAsHex,
} from '@polkadot/util-crypto';

import {
  type LitentryIdentity,
  type TrustedCall,
  identity,
} from '@litentry/parachain-api';

import { createPayloadToSign } from './create-payload-to-sign';
import { createLitentryIdentityType } from '../type-creators/litentry-identity';
import { stringToHex, u8aConcat } from '@polkadot/util';
import type { Index } from '@polkadot/types/interfaces';

const types = {
  ...identity.types, // LitentryIdentity is defined here
};

let registry: TypeRegistry;

const callU8 = () => randomAsHex(4);
const nonceU8 = () => randomAsHex(4);

const fakePayload = {
  call: {
    toU8a: () => callU8,
  } as unknown as TrustedCall,
  nonce: {
    toU8a: () => nonceU8,
  } as unknown as Index,
  shard: randomAsHex(4),
};

const fakePayloadHash = blake2AsHex(
  u8aConcat(
    fakePayload.call.toU8a(),
    fakePayload.nonce.toU8a(),
    fakePayload.shard,
    fakePayload.shard
  ),
  256
);

beforeAll(async () => {
  await cryptoWaitReady();

  registry = new TypeRegistry();
  registry.register(types);
});

describe('Evm', () => {
  let identity: LitentryIdentity;
  beforeAll(() => {
    identity = createLitentryIdentityType(registry, {
      type: 'Evm',
      addressOrHandle: '0x0AcE67628Bd43213C1C41ca1DAEf47E63923c75c',
    });
  });

  // hex encoded prettified message
  test('link_identity payload', () => {
    const expected = `By linking your identity to our platform, you're taking a step towards a more integrated experience. Please be assured, this process is safe and involves no transactions of your assets. Token: ${fakePayloadHash}`;

    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isLinkIdentity: true,
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    expect(result).toBe(stringToHex(expected));
  });

  // hex encoded prettified message
  test('request-batch-vc payload', () => {
    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isRequestBatchVc: true,
        asRequestBatchVc: [null, null, [1, 2, 3]],
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    const expected = `We are going to help you generate 3 secure credentials. Please be assured, this process is safe and involves no transactions of your assets. Token: ${fakePayloadHash}`;

    expect(result).toBe(stringToHex(expected));
  });

  // hex encoded prettified message
  test('set-identity-network payload', () => {
    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isSetIdentityNetworks: true,
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    const expected = `Token: ${fakePayloadHash}`;

    expect(result).toBe(stringToHex(expected));
  });
});

describe('Bitcoin', () => {
  let identity: LitentryIdentity;
  beforeAll(() => {
    identity = createLitentryIdentityType(registry, {
      type: 'Bitcoin',
      addressOrHandle:
        '0385db7c98b8c1239a397e5776571b31788e77ce3a46c265dbe82754dc9cf1f466',
    });
  });

  // prettified message
  test('link_identity payload', () => {
    const expected = `By linking your identity to our platform, you're taking a step towards a more integrated experience. Please be assured, this process is safe and involves no transactions of your assets. Token: ${fakePayloadHash}`;

    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isLinkIdentity: true,
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    expect(result).toBe(expected);
  });

  // prettified message
  test('request-batch-vc payload', () => {
    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isRequestBatchVc: true,
        asRequestBatchVc: [null, null, [1, 2, 3]],
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    const expected = `We are going to help you generate 3 secure credentials. Please be assured, this process is safe and involves no transactions of your assets. Token: ${fakePayloadHash}`;

    expect(result).toBe(expected);
  });

  // prettified message
  test('set-identity-network payload', () => {
    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isSetIdentityNetworks: true,
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    const expected = `Token: ${fakePayloadHash}`;

    expect(result).toBe(expected);
  });
});

describe('Substrate', () => {
  let identity: LitentryIdentity;
  beforeAll(() => {
    identity = createLitentryIdentityType(registry, {
      type: 'Substrate',
      addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    });
  });

  // prettified message
  test('link_identity payload', () => {
    const expected = `By linking your identity to our platform, you're taking a step towards a more integrated experience. Please be assured, this process is safe and involves no transactions of your assets. Token: ${fakePayloadHash}`;

    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isLinkIdentity: true,
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    expect(result).toBe(expected);
  });

  // prettified message
  test('request-batch-vc payload', () => {
    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isRequestBatchVc: true,
        asRequestBatchVc: [null, null, [1, 2, 3]],
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    const expected = `We are going to help you generate 3 secure credentials. Please be assured, this process is safe and involves no transactions of your assets. Token: ${fakePayloadHash}`;

    expect(result).toBe(expected);
  });

  // prettified message
  test('set-identity-network payload', () => {
    const result = createPayloadToSign({
      who: identity,
      call: {
        ...fakePayload.call,
        isSetIdentityNetworks: true,
      } as unknown as TrustedCall,
      nonce: fakePayload.nonce,
      shard: fakePayload.shard,
    });

    const expected = `Token: ${fakePayloadHash}`;

    expect(result).toBe(expected);
  });
});

// describe.skip('Bitcoin', () => {
//   test('it adds a prettified message', () => {
//     const identity = createLitentryIdentityType(registry, {
//       type: 'Bitcoin',
//       addressOrHandle:
//         '03dea1fa79da3457bcb4dfd72357a2a8e932fd48e00cc29c5f1b6363d13828ad0e',
//     });

//     const payload = [randomAsHex(4), randomAsHex(4)];

//     const output = createPayloadToSign(identity, ...payload);

//     const expected = blake2AsHex(
//       `0x${payload.map((p) => p.substring(2)).join('')}`,
//       256
//     );

//     expect(output).toBe(`Litentry authorization token: ${expected}`);
//   });
// });

// describe('Evm', () => {
//   test('it adds a prettified message', () => {
//     const identity = createLitentryIdentityType(registry, {
//       type: 'Evm',
//       addressOrHandle: '0x0AcE67628Bd43213C1C41ca1DAEf47E63923c75c',
//     });

//     const payload = [randomAsHex(4), randomAsHex(4)];

//     const output = createPayloadToSign(identity, ...payload);

//     const expected = blake2AsHex(
//       `0x${payload.map((p) => p.substring(2)).join('')}`,
//       256
//     );

//     expect(output).toBe(
//       stringToHex(`Litentry authorization token: ${expected}`)
//     );
//   });
// });
