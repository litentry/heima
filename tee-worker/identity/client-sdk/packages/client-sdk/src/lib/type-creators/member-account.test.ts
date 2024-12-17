import { TypeRegistry } from '@polkadot/types';
import { cryptoWaitReady } from '@polkadot/util-crypto';

import { identity, omniAccount } from '@litentry/parachain-api';

import { createMemberAccountType } from './member-account';
import { createLitentryIdentityType } from './litentry-identity';

const types = {
    ...identity.types, // LitentryIdentity is defined here
    ...omniAccount.types, // MemberAccount is defined here
};

let registry: TypeRegistry;


beforeAll(async () => {
    await cryptoWaitReady();

    registry = new TypeRegistry();
    registry.register(types);
});

it('creates a public MemberAccount', () => {
    const litentryIdentity = createLitentryIdentityType(registry, {
        type: 'Substrate',
        addressOrHandle: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
    });

    const publicAccount = createMemberAccountType(registry, {
        type: 'Public',
        identity: litentryIdentity,
    });

    expect(publicAccount.isPublic).toBe(true);
    expect(publicAccount.asPublic.toHuman()).toEqual(litentryIdentity.toHuman());
});

it('creates a private MemberAccount with correct length', () => {
    const encryptedPayload = new Uint8Array(66).fill(1);
    const hash = '0x' + '2'.repeat(64) as `0x${string}`;

    const privateAccount = createMemberAccountType(registry, {
        type: 'Private',
        encryptedPayload,
        hash,
    });

    expect(privateAccount.isPrivate).toBe(true);
    const [privateData, privateHash] = privateAccount.asPrivate;
    expect(privateData.toU8a()).toEqual(encryptedPayload);
    expect(privateHash.toHex()).toEqual(hash);
});

