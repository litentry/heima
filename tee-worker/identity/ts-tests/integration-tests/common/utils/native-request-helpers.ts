import { ApiPromise } from '@polkadot/api';
import { u8aToHex, hexToU8a, u8aConcat } from '@polkadot/util';
import { Keyring } from '@polkadot/keyring';
import { Codec } from '@polkadot/types/types';
import { encodeAddress } from '@polkadot/util-crypto';
import { IntegrationTestContext, JsonRpcRequest } from '../common-types';
import {
    WorkerRpcReturnValue,
    CorePrimitivesIdentity,
    TrustedCall,
    TrustedCallAuthenticated,
    TCAuthentication,
    Intent,
    LitentryValidationData,
    OmniAccountPermission,
} from 'parachain-api';
import { Signer, createLitentryMultiSignature } from '../utils';
import { aesKey } from '../call';
import { KeyObject } from 'crypto';
import { Index } from '@polkadot/types/interfaces';
import { blake2AsHex } from '@polkadot/util-crypto';
import { createJsonRpcRequest, nextRequestId } from '../helpers';
import { createAesRequest, sendRequest, getSignatureMessagePrefix } from '../di-utils';
import { generateVerificationMessage } from './identity-helper';
import { ethers } from 'ethers';
import type { HexString } from '@polkadot/util/types';

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

    const authentication: TCAuthentication = parachainApi.createType('TCAuthentication', {
        Web3: parachainApi.createType('(LitentryMultiSignature)', signature),
    });

    return parachainApi.createType('TrustedCallAuthenticated', {
        call,
        nonce,
        authentication,
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

export function createAuthenticatedTrustedCallTransferNativeIntent(
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

export async function createAuthenticatedTrustedCallAddAccount(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    sender: Signer,
    senderIdentity: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    validationData: string,
    publicAccount = false,
    permissions: OmniAccountPermission[]
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        [
            'add_account',
            '(LitentryIdentity, LitentryIdentity, LitentryValidationData, bool, Option<Vec<OmniAccountPermission>>)',
        ],
        sender,
        mrenclave,
        nonce,
        [senderIdentity, identity, validationData, publicAccount, permissions]
    );
}

export async function createAuthenticatedTrustedCallSetPermissions(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    sender: Signer,
    senderIdentity: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    permissions: OmniAccountPermission[]
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['set_permissions', '(LitentryIdentity, LitentryIdentity, Vec<OmniAccountPermission>)'],
        sender,
        mrenclave,
        nonce,
        [senderIdentity, identity, permissions]
    );
}

export async function createAuthenticatedTrustedCallRemoveAccounts(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    sender: Signer,
    senderIdentity: CorePrimitivesIdentity,
    identities: CorePrimitivesIdentity[]
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['remove_accounts', '(LitentryIdentity, Vec<LitentryIdentity>)'],
        sender,
        mrenclave,
        nonce,
        [senderIdentity, identities]
    );
}

export async function createAuthenticatedTrustedCallPublicizeAccount(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    sender: Signer,
    senderIdentity: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['publicize_account', '(LitentryIdentity, LitentryIdentity)'],
        sender,
        mrenclave,
        nonce,
        [senderIdentity, identity]
    );
}

export async function createAuthenticatedTrustedCallRequestBatchVc(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    sender: Signer,
    senderIdentity: CorePrimitivesIdentity,
    assertions: string,
    aesKey: string,
    hash: string
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        [
            'request_batch_vc',
            '(LitentryIdentity, LitentryIdentity, BoundedVec<Assertion, ConstU32<32>>, Option<RequestAesKey>, H256)',
        ],
        sender,
        mrenclave,
        nonce,
        [senderIdentity, senderIdentity, assertions, aesKey, hash]
    );
}

export async function createAuthenticatedTrustedCallRequestVc(
    parachainApi: ApiPromise,
    mrenclave: string,
    nonce: Codec,
    sender: Signer,
    senderIdentity: CorePrimitivesIdentity,
    assertion: string,
    aesKey: string,
    hash: string
) {
    return createAuthenticatedTrustedCall(
        parachainApi,
        ['request_vc', '(LitentryIdentity, LitentryIdentity, Assertion, Option<RequestAesKey>, H256)'],
        sender,
        mrenclave,
        nonce,
        [senderIdentity, senderIdentity, assertion, aesKey, hash]
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
    const trustedOperation = context.api.createType('TrustedOperationAuthenticated', { direct_call: call });
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
    console.log(`Funding account ${account} with ${amount}`);
    const keyring = new Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const transfer = api.tx.balances.transferAllowDeath(account, amount);

    return new Promise<void>((resolve, reject) => {
        transfer
            .signAndSend(alice, ({ isFinalized }) => {
                if (isFinalized) {
                    resolve();
                }
            })
            .catch(reject);
    });
}

export async function buildWeb3ValidationData(
    context: IntegrationTestContext,
    sender: CorePrimitivesIdentity,
    accountToAdd: CorePrimitivesIdentity,
    nonce: number,
    network: 'evm' | 'substrate' | 'bitcoin' | 'solana',
    signer: Signer
): Promise<LitentryValidationData> {
    const msg = generateVerificationMessage(context, sender, accountToAdd, nonce);

    if (network === 'evm') {
        const evmValidationData = {
            Web3Validation: {
                Evm: {
                    message: '',
                    signature: {
                        Ethereum: '' as HexString,
                    },
                },
            },
        };
        evmValidationData.Web3Validation.Evm.message = msg;
        const msgHash = ethers.utils.arrayify(msg);
        const evmSignature = u8aToHex(await signer.sign(msgHash));

        evmValidationData!.Web3Validation.Evm.signature.Ethereum = evmSignature;

        return context.api.createType('LitentryValidationData', evmValidationData);
    }

    if (network === 'substrate') {
        const substrateValidationData = {
            Web3Validation: {
                Substrate: {
                    message: '',
                    signature: {
                        Sr25519: '' as HexString,
                    },
                },
            },
        };
        console.log('post verification msg to substrate: ', msg);
        substrateValidationData.Web3Validation.Substrate.message = msg;
        const substrateSignature = await signer.sign(msg);
        substrateValidationData!.Web3Validation.Substrate.signature.Sr25519 = u8aToHex(substrateSignature);

        return context.api.createType('LitentryValidationData', substrateValidationData);
    }

    if (network === 'bitcoin') {
        const bitcoinValidationData = {
            Web3Validation: {
                Bitcoin: {
                    message: '',
                    signature: {
                        Bitcoin: '' as HexString,
                    },
                },
            },
        };
        bitcoinValidationData.Web3Validation.Bitcoin.message = msg;
        // we need to sign the hex string without `0x` prefix, the signature is base64-encoded string
        const bitcoinSignature = await signer.sign(msg.substring(2));
        bitcoinValidationData!.Web3Validation.Bitcoin.signature.Bitcoin = u8aToHex(bitcoinSignature);

        return context.api.createType('LitentryValidationData', bitcoinValidationData);
    }

    if (network === 'solana') {
        const solanaValidationData = {
            Web3Validation: {
                Solana: {
                    message: '',
                    signature: {
                        Ed25519: '' as HexString,
                    },
                },
            },
        };
        console.log('post verification msg to solana: ', msg);
        solanaValidationData.Web3Validation.Solana.message = msg;
        const solanaSignature = await signer.sign(msg);
        solanaValidationData!.Web3Validation.Solana.signature.Ed25519 = u8aToHex(solanaSignature);

        return context.api.createType('LitentryValidationData', solanaValidationData);
    }

    throw new Error(`[buildValidation]: Unsupported network ${network}.`);
}

type OmniAccountPermissionString =
    | 'All'
    | 'AccountManagement'
    | 'RequestNativeIntent'
    | 'RequestEthereumIntent'
    | 'RequestSolanaIntent';

export function createOmniAccountPermission(
    api: ApiPromise,
    permission: OmniAccountPermissionString
): OmniAccountPermission {
    return api.createType('OmniAccountPermission', permission);
}
