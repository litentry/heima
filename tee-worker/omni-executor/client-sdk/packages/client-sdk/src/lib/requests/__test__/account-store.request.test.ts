import { ApiPromise, Keyring } from '@polkadot/api';
import type { KeyringPair } from '@polkadot/keyring/types';
import { WsProvider } from '@polkadot/rpc-provider';
import { u8aToHex } from '@polkadot/util';
import { cryptoWaitReady, encodeAddress } from '@polkadot/util-crypto';

import {
  identity,
  ValidationData,
  OmniAccountPermission,
  omniExecutor,
  Identity,
  sidechain,
  omniAccount,
} from '@heima-network/parachain-api';

import { createIdentityType } from '@type-creators/identity';
import { addAccount } from '@requests/add-account.request';
import { createAccountStore } from '@requests/create-account-store.request';
import { getAccountNonce } from '@requests/get-nonce.request';
import { publicizeAccount } from '@requests/publicize-account.request';
import { removeAccounts } from '@requests/remove-accounts.request';
import { requestAuthToken } from '@requests/request-auth-token.request';
import { requestEmailVerificationCode } from '@requests/request-email-verification-code.request';
import { setPermissions } from '@requests/set-permissions.request';
import { transferNative } from '@requests/intents/transfer-native.request';
import { createVerificationMessage } from '@utils/create-verification-message';
import { toHash } from '@utils/identity';

import { getAndWaitForAccountStoreCreation } from '@test-utils/helpers';

const types = {
  ...identity.types, // Identity is defined here
  ...omniAccount.types, // OmniAccountPermission is defined here
  ...omniExecutor.types, // NativeCall is defined here
  ...sidechain.types, // AesOutput is defined here
};

describe('account-store', () => {
  let api: ApiPromise;
  let member: Identity;
  let memberSigner: KeyringPair;
  let memberToAdd: Identity;
  let memberToAddSigner: KeyringPair;

  beforeAll(async () => {
    api = new ApiPromise({
      provider: new WsProvider(process.env.PARACHAIN_NETWORK),
      types,
    });

    await api.isReady;
    await cryptoWaitReady();

    const keyring = new Keyring({ type: 'sr25519' });
    memberSigner = keyring.addFromUri(`//Bob`);
    member = createIdentityType(api.registry, {
      addressOrHandle: memberSigner.address,
      type: 'Substrate',
    });
    memberToAddSigner = keyring.addFromUri('//Charlie');
    memberToAdd = createIdentityType(api.registry, {
      addressOrHandle: memberToAddSigner.address,
      type: 'Substrate',
    });
  });

  it('web3 identity (substrate) & web3 authentication', async () => {
    // Step 1: create account store
    console.log('Step 1: create account store');
    await (async () => {
      const { send, getPayloadToSign = () => '' } = await createAccountStore(api, { member });
      const payloadToSign = await getPayloadToSign();
      const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

      await send({
        authData: {
          type: 'Web3',
          signer: member,
          signature: signatureHex,
        },
      });
    })();
    const omniAccount = toHash(member);
    const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
    expect(accountStore).toBeDefined();
    expect(encodeAddress(omniAccount)).toBe('5H8eg2qghG4ZRCePpqCTbj5sFa7FCGV5w9xnCkKwtQ9v6hLZ');

    // Check account store
    console.log('Step 1: check account store');
    await (async () => {
      expect(accountStore.length).toBe(1);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
      );
    })();

    // Wait 1 second for the omni_account can be retrieved from the omni_account_storage in omni-executor.
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Step 2: add account
    console.log('Step 2: add account');
    await (async () => {
      const omniAccountNonce = await getAccountNonce(api, omniAccount);
      const message = createVerificationMessage(api.registry, {
        member,
        memberToAdd,
        omniAccountNonce,
      });

      const validation = api.createType<ValidationData>('ValidationData', {
        Web3Validation: {
          Substrate: {
            message,
            signature: {
              Sr25519: u8aToHex(memberToAddSigner.sign(message)),
            },
          },
        },
      });

      const { send, getPayloadToSign = () => '' } = await addAccount(api, {
        member,
        memberToAdd,
        validation,
        isPublic: false,
      });
      const payloadToSign = await getPayloadToSign();

      await send({
        authData: {
          type: 'Web3',
          signer: member,
          signature: u8aToHex(memberSigner.sign(payloadToSign)),
        },
      });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account added
    console.log('Step 2: check account store after account added');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(2);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
      expect(accountStore[1].isPrivate).toBeTruthy(); // Charlie

      // Check permissions
      const permissions = await api.query.omniAccount.memberAccountPermissions(toHash(memberToAdd));
      expect(permissions.toJSON()).toEqual(['All']);
    })();

    // Step 3: set permissions
    console.log('Step 3: set permissions');
    await (async () => {
      const { send, getPayloadToSign = () => '' } = await setPermissions(api, {
        member,
        memberToSetPermissions: memberToAdd,
        permissions: [
          api.createType<OmniAccountPermission>('OmniAccountPermission', 'All'),
          api.createType<OmniAccountPermission>('OmniAccountPermission', 'AccountManagement'),
        ],
      });
      const payloadToSign = await getPayloadToSign();

      await send({
        authData: {
          type: 'Web3',
          signer: member,
          signature: u8aToHex(memberSigner.sign(payloadToSign)),
        },
      });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check permissions after permissions set
    console.log('Step 3: check permissions after permissions set');
    const permissions = await api.query.omniAccount.memberAccountPermissions(toHash(memberToAdd));
    expect(permissions.toJSON()).toEqual(['All', 'AccountManagement']);

    // Step 4: publicize account
    console.log('Step 4: publicize account');
    await (async () => {
      const { send, getPayloadToSign = () => '' } = await publicizeAccount(api, { member, memberToPublicize: memberToAdd });
      const payloadToSign = await getPayloadToSign();

      const result = await send({
        authData: {
          type: 'Web3',
          signer: member,
          signature: u8aToHex(memberSigner.sign(payloadToSign)),
        },
      });

      expect(result.extrinsicHash.length).toBe(66);
      expect(result.blockHash.length).toBe(66);
      expect(result.status).toBeDefined();
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account publicized
    console.log('Step 4: check account store after account publicized');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(2);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
      expect(encodeAddress(accountStore[1].asPublic.asSubstrate.toU8a())).toBe(
        '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y', // Charlie
      );
    })();

    // Step 5: remove accounts
    console.log('Step 5: remove accounts');
    await (async () => {
      const { send, getPayloadToSign = () => '' } = await removeAccounts(api, { member, membersToRemove: [memberToAdd] });

      const payloadToSign = await getPayloadToSign();
      const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

      await send({
        authData: {
          type: 'Web3',
          signer: member,
          signature: signatureHex,
        },
      });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account removed
    console.log('Step 5: check account store after account removed');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(1);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
    })();
  });

  it('web3 identity (substrate) & auth token authentication', async () => {
    // Step 1: create account store
    console.log('Step 1: create account store');
    await (async () => {
      const { send, getPayloadToSign = () => '' } = await createAccountStore(api, { member });

      const payloadToSign = await getPayloadToSign();
      const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

      await send({
        authData: {
          type: 'Web3',
          signer: member,
          signature: signatureHex,
        },
      });
    })();
    const omniAccount = toHash(member);
    const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
    expect(accountStore).toBeDefined();
    expect(encodeAddress(omniAccount)).toBe('5H8eg2qghG4ZRCePpqCTbj5sFa7FCGV5w9xnCkKwtQ9v6hLZ');

    // Check account store
    console.log('Step 1: check account store');
    await (async () => {
      expect(accountStore.length).toBe(1);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
      );
    })();

    // Wait 1 second for the omni_account can be retrieved from the omni_account_storage in omni-executor.
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Step 2: request auth token
    console.log('Step 2: request auth token');
    const { send, getPayloadToSign = () => '' } = await requestAuthToken(api, {
      member,
    });

    const payloadToSign = await getPayloadToSign();
    const signatureHex = u8aToHex(memberSigner.sign(payloadToSign));

    const result = await send({
      authData: {
        type: 'Web3',
        signer: member,
        signature: signatureHex,
      },
    });

    const token = result.token;
    expect(token.length).toBeGreaterThan(0);

    // Step 3: add account
    console.log('Step 3: add account');
    await (async () => {
      const omniAccountNonce = await getAccountNonce(api, omniAccount);
      const message = createVerificationMessage(api.registry, {
        member,
        memberToAdd,
        omniAccountNonce,
      });

      const validation = api.createType<ValidationData>('ValidationData', {
        Web3Validation: {
          Substrate: {
            message,
            signature: {
              Sr25519: u8aToHex(memberToAddSigner.sign(message)),
            },
          },
        },
      });

      const { send } = await addAccount(api, {
        member,
        memberToAdd,
        validation,
        isPublic: false,
      });

      await send({ authData: { type: 'AuthToken', token } });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account added
    console.log('Step 3: check account store after account added');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(2);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
      expect(accountStore[1].isPrivate).toBeTruthy(); // Charlie

      // Check permissions
      const permissions = await api.query.omniAccount.memberAccountPermissions(toHash(memberToAdd));
      expect(permissions.toJSON()).toEqual(['All']);
    })();

    // Step 4: set permissions
    console.log('Step 4: set permissions');
    await (async () => {
      const { send } = await setPermissions(api, {
        member,
        memberToSetPermissions: memberToAdd,
        permissions: [
          api.createType<OmniAccountPermission>('OmniAccountPermission', 'All'),
          api.createType<OmniAccountPermission>('OmniAccountPermission', 'AccountManagement'),
        ],
      });

      await send({ authData: { type: 'AuthToken', token } });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check permissions after permissions set
    console.log('Step 4: check permissions after permissions set');
    const permissions = await api.query.omniAccount.memberAccountPermissions(toHash(memberToAdd));
    expect(permissions.toJSON()).toEqual(['All', 'AccountManagement']);

    // Step 5: publicize account
    console.log('Step 5: publicize account');
    await (async () => {
      const { send } = await publicizeAccount(api, { member, memberToPublicize: memberToAdd });

      const result = await send({ authData: { type: 'AuthToken', token } });

      expect(result.extrinsicHash.length).toBe(66);
      expect(result.blockHash.length).toBe(66);
      expect(result.status).toBeDefined();
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account publicized
    console.log('Step 5: check account store after account publicized');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(2);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
      expect(encodeAddress(accountStore[1].asPublic.asSubstrate.toU8a())).toBe(
        '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y', // Charlie
      );
    })();

    // Step 6: remove accounts
    console.log('Step 6: remove accounts');
    await (async () => {
      const { send } = await removeAccounts(api, { member, membersToRemove: [memberToAdd] });
      await send({ authData: { type: 'AuthToken', token } });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account removed
    console.log('Step 6: check account store after account removed');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(1);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
    })();

    // Step 7: transfer native
    console.log('Step 7: transfer native');
    await (async () => {
      const { send } = await transferNative(api, {
        member,
        to: '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw',
        amount: BigInt(100),
      });

      const result = await send({ authData: { type: 'AuthToken', token } });

      expect(result.extrinsicHash.length).toBe(66);
      expect(result.blockHash.length).toBe(66);
      expect(result.status).toBeDefined();
    })();
  });

  // This test is just an example. It requires receiving email verification codes, which cannot be done by running this unit test.
  it.skip('web2 identity (email) & auth token authentication', async () => {
    const email = 'test@test.com';

    const member = createIdentityType(api.registry, {
      addressOrHandle: email,
      type: 'Email',
    });
    // Step 1: create account store
    console.log('Step 1: create account store');
    await (async () => {
      // Request email verification code for next account store creation
      await requestEmailVerificationCode({ email, clientId: 'account-store-test-client' });
      // Get the email verification code from the email inbox
      const verificationCode = 'emailVerificationCodeYouReceivedInEmail';

      const { send } = await createAccountStore(api, { member });

      await send({ authData: { type: 'Email', verificationCode } });
    })();
    const omniAccount = toHash(member);
    const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
    expect(accountStore).toBeDefined();
    expect(encodeAddress(omniAccount)).toBe('5H8eg2qghG4ZRCePpqCTbj5sFa7FCGV5w9xnCkKwtQ9v6hLZ');

    // Check account store
    console.log('Step 1: check account store');
    await (async () => {
      expect(accountStore.length).toBe(1);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
      );
    })();

    // Wait 1 second for the omni_account can be retrieved from the omni_account_storage in omni-executor.
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Step 2: request auth token
    console.log('Step 2: request auth token');
    // Request email verification code again for next auth token request
    await requestEmailVerificationCode({ email, clientId: 'account-store-test-client' });
    // Get the email verification code from the email inbox
    const verificationCode = 'emailVerificationCodeYouReceivedInEmail';

    const { send } = await requestAuthToken(api, { member });

    const result = await send({ authData: { type: 'Email', verificationCode } });

    const token = result.token;
    expect(token.length).toBeGreaterThan(0);

    // Step 3: add account
    console.log('Step 3: add account');
    await (async () => {
      const omniAccountNonce = await getAccountNonce(api, omniAccount);
      const message = createVerificationMessage(api.registry, {
        member,
        memberToAdd,
        omniAccountNonce,
      });

      const validation = api.createType<ValidationData>('ValidationData', {
        Web3Validation: {
          Substrate: {
            message,
            signature: {
              Sr25519: u8aToHex(memberToAddSigner.sign(message)),
            },
          },
        },
      });

      const { send } = await addAccount(api, {
        member,
        memberToAdd,
        validation,
        isPublic: false,
      });

      await send({ authData: { type: 'AuthToken', token } });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account added
    console.log('Step 3: check account store after account added');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(2);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
      expect(accountStore[1].isPrivate).toBeTruthy(); // Charlie

      // Check permissions
      const permissions = await api.query.omniAccount.memberAccountPermissions(toHash(memberToAdd));
      expect(permissions.toJSON()).toEqual(['All']);
    })();

    // Step 4: set permissions
    console.log('Step 4: set permissions');
    await (async () => {
      const { send } = await setPermissions(api, {
        member,
        memberToSetPermissions: memberToAdd,
        permissions: [
          api.createType<OmniAccountPermission>('OmniAccountPermission', 'All'),
          api.createType<OmniAccountPermission>('OmniAccountPermission', 'AccountManagement'),
        ],
      });

      await send({ authData: { type: 'AuthToken', token } });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check permissions after permissions set
    console.log('Step 4: check permissions after permissions set');
    const permissions = await api.query.omniAccount.memberAccountPermissions(toHash(memberToAdd));
    expect(permissions.toJSON()).toEqual(['All', 'AccountManagement']);

    // Step 5: publicize account
    console.log('Step 5: publicize account');
    await (async () => {
      const { send } = await publicizeAccount(api, { member, memberToPublicize: memberToAdd });

      const result = await send({ authData: { type: 'AuthToken', token } });

      expect(result.extrinsicHash.length).toBe(66);
      expect(result.blockHash.length).toBe(66);
      expect(result.status).toBeDefined();
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account publicized
    console.log('Step 5: check account store after account publicized');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(2);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
      expect(encodeAddress(accountStore[1].asPublic.asSubstrate.toU8a())).toBe(
        '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y', // Charlie
      );
    })();

    // Step 6: remove accounts
    console.log('Step 6: remove accounts');
    await (async () => {
      const { send } = await removeAccounts(api, { member, membersToRemove: [memberToAdd] });
      await send({ authData: { type: 'AuthToken', token } });
    })();

    // wait 10 seconds
    await new Promise((resolve) => setTimeout(resolve, 10000));

    // Check account store after account removed
    console.log('Step 6: check account store after account removed');
    await (async () => {
      // Check account store from parachain api
      const accountStore = await getAndWaitForAccountStoreCreation(api, omniAccount);
      expect(accountStore.length).toBe(1);
      expect(encodeAddress(accountStore[0].asPublic.asSubstrate.toU8a())).toBe(
        '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', // Bob
      );
    })();

    // Step 7: transfer native
    console.log('Step 7: transfer native');
    await (async () => {
      const { send } = await transferNative(api, {
        member,
        to: '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw',
        amount: BigInt(100),
      });

      const result = await send({ authData: { type: 'AuthToken', token } });

      expect(result.extrinsicHash.length).toBe(66);
      expect(result.blockHash.length).toBe(66);
      expect(result.status).toBeDefined();
    })();
  });
});
