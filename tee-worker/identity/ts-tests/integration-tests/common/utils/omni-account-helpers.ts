import { ApiPromise } from '@polkadot/api';
import { u8aToHex, hexToU8a, u8aConcat } from '@polkadot/util';
import { Keyring } from '@polkadot/keyring';
import { Codec } from '@polkadot/types/types';
import { encodeAddress } from '@polkadot/util-crypto';
import { IntegrationTestContext, JsonRpcRequest } from '../common-types';
import type {
    WorkerRpcReturnValue,
    CorePrimitivesIdentity,
    TrustedCall,
    TrustedCallAuthenticated,
    TCAuthentication,
    Intent,
} from 'parachain-api';
import { Signer, createLitentryMultiSignature } from '../utils';
import { aesKey } from '../call';
import { KeyObject } from 'crypto';
import { Index } from '@polkadot/types/interfaces';
import { blake2AsHex } from '@polkadot/util-crypto';
import { createJsonRpcRequest, nextRequestId } from '../helpers';
import { createAesRequest, sendRequest, getSignatureMessagePrefix } from 'common/di-utils';

export const createAuthenticatedTrustedCall = async (
    parachainApi: ApiPromise,
    trustedCall: [string, string],
    signer: Signer,
    mrenclave: string,
    nonce: Codec,
    params: any,
    withWrappedBytes = false,
    withPrefix = false
): Promise<TrustedCallAuthenticated> => {
    const [variant, argType] = trustedCall;
    const call: TrustedCall = parachainApi.createType('TrustedCall', {
        [variant]: parachainApi.createType(argType, params),
    });
    let payload: string = blake2AsHex(
        u8aConcat(
            call.toU8a(),
            nonce.toU8a(),
            hexToU8a(mrenclave),
            hexToU8a(mrenclave) // should be shard, but it's the same as MRENCLAVE in our case
        ),
        256
    );

    if (withWrappedBytes) {
        payload = `<Bytes>${payload}</Bytes>`;
    }

    if (withPrefix) {
        const prefix = getSignatureMessagePrefix(call);
        const msg = prefix + payload;
        payload = msg;
        console.log('Signing message: ', payload);
    }

    const signature = await createLitentryMultiSignature(parachainApi, {
        signer,
        payload,
    });

    const tcAuthentication: TCAuthentication = parachainApi.createType('TCAuthentication', {
        Web3: parachainApi.createType('(LitentryMultiSignature)', signature),
    });

    return parachainApi.createType('TrustedCallAuthenticated', {
        call: call,
        index: nonce,
        authentication: tcAuthentication,
    });
};

export function createAuthenticatedTrustedCallCreateAccountStore(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    identity: CorePrimitivesIdentity
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['create_account_store', '(LitentryIdentity)'],
        signer,
        mrenclave,
        nonce,
        identity
    );
}

export function createTransferNativeIntentAuthenticatedTrustedCall(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    sender: CorePrimitivesIdentity,
    dest: string,
    amount: bigint
) {
    const intent: Intent = parachainApi.createType('Intent', {
        TransferNative: parachainApi.createType('IntentTransferNative', {
            to: dest,
            value: amount,
        }),
    });
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['request_intent', '(LitentryIdentity, Intent)'],
        signer,
        mrenclave,
        nonce,
        [sender, intent]
    );
}

export function createAuthenticatedTrustedCallRequestIntent(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    signer: Signer,
    identity: CorePrimitivesIdentity
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['request_intent', '(LitentryIdentity, Intent)'],
        signer,
        mrenclave,
        nonce,
        [identity]
    );
}

export async function createAuthenticatedTrustedCallAddAccount(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    sender: Signer,
    senderIdentity: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    validationData: string,
    publicAccount = false
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['add_account', '(LitentryIdentity, LitentryIdentity, LitentryValidationData, bool)'],
        sender,
        mrenclave,
        nonce,
        [senderIdentity, identity, validationData, publicAccount]
    );
}

export const getOmniAccount = async (parachainApi: ApiPromise, identity: CorePrimitivesIdentity): Promise<string> => {
    const omniAccount = await parachainApi.rpc.state.call('OmniAccountApi_omni_account', identity.toHex());

    return encodeAddress(omniAccount.toHex());
};

export const getOmniAccountNonce = async (
    parachainApi: ApiPromise,
    memberIdentity: CorePrimitivesIdentity
): Promise<Index> => {
    const omniAccount = await getOmniAccount(parachainApi, memberIdentity);
    const nonce = await parachainApi.rpc.system.accountNextIndex(omniAccount);

    return nonce;
};

export const sendRequestFromTrustedCall = async (
    context: IntegrationTestContext,
    teeShieldingKey: KeyObject,
    call: TrustedCallAuthenticated,
    onMessageReceived?: (res: WorkerRpcReturnValue) => void
) => {
    // construct trusted operation
    const trustedOperation = context.api.createType('TrustedOperation', { direct_call: call });
    console.log('trustedOperation: ', JSON.stringify(trustedOperation.toHuman(), null, 2));
    // create the request parameter
    const requestParam = await createAesRequest(
        context.api,
        context.mrEnclave,
        teeShieldingKey,
        hexToU8a(aesKey),
        trustedOperation.toU8a()
    );

    const request: JsonRpcRequest = createJsonRpcRequest(
        'author_submitNativeRequest',
        [u8aToHex(requestParam)],
        nextRequestId(context)
    );

    return sendRequest(context.tee, request, context.api, onMessageReceived);
};

export async function fundAccount(api: ApiPromise, account: string, amount: bigint) {
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const transfer = api.tx.balances.transferAllowDeath(account, amount);
    const hash = await transfer.signAndSend(alice);
    console.log('Transfer sent with hash', hash.toHex());
    const { data } = await api.query.system.account(account);
    console.log(`Account balance: ${data.free}`);
}
