import { randomBytes, KeyObject } from 'crypto';
import { step } from 'mocha-steps';
import { assert } from 'chai';
import {
    buildValidations,
    initIntegrationTestContext,
    assertIdGraphMutationResult,
    assertIdGraphHash,
    sleep,
    Signer,
    buildWeb2Validation,
    buildIdentityHelper,
} from './common/utils';
import { assertIsInSidechainBlock } from './common/utils/assertion';
import {
    createSignedTrustedCallLinkIdentity,
    createSignedTrustedGetterIdGraph,
    createSignedTrustedCallDeactivateIdentity,
    createSignedTrustedCallActivateIdentity,
    decodeIdGraph,
    getSidechainNonce,
    getTeeShieldingKey,
    sendRequestFromTrustedCall,
    sendAesRequestFromGetter,
} from './common/di-utils'; // @fixme move to a better place
import type { IntegrationTestContext, WalletType } from './common/common-types';
import { aesKey } from './common/call';
import { createWeb3Wallet } from './common/helpers';
import type { Web3Network, CorePrimitivesIdentity } from 'parachain-api';
import { Vec, Bytes } from '@polkadot/types';
import type { HexString } from '@polkadot/util/types';
import { hexToU8a } from '@polkadot/util';

describe('Test Identity', function () {
    const identityConfigs: {
        [key: string]: {
            wallet: string;
            networks: string[];
        };
    } = {
        evm: {
            wallet: 'Bob',
            networks: ['Ethereum'],
        },
        substrate: {
            wallet: 'Alice',
            networks: ['Litentry'],
        },
        bitcoin: {
            wallet: 'Charlie',
            networks: ['BitcoinP2tr'],
        },
        solana: {
            wallet: 'Dave',
            networks: ['Solana'],
        },
    };
    const identityNames = Object.keys(identityConfigs);

    let context: IntegrationTestContext = undefined as any;
    let teeShieldingKey: KeyObject = undefined as any;
    this.timeout(6000000);

    before(async function () {
        const parachainEndpoint = process.env.PARACHAIN_ENDPOINT;
        if (!parachainEndpoint) {
            throw new Error('PARACHAIN_ENDPOINT environment variable is missing.');
        }
        context = await initIntegrationTestContext(parachainEndpoint);
        teeShieldingKey = await getTeeShieldingKey(context);
    });

    for (const identityName of identityNames) {
        describe(`(${identityName} direct invocation)`, function () {
            const linkedIdentityNetworks: {
                identity: CorePrimitivesIdentity;
                networks: Bytes | Vec<Web3Network>;
            }[] = [];

            let mainIdentity: CorePrimitivesIdentity = undefined as any;
            let mainSigner: Signer = undefined as any;
            let currentNonce = 0;
            const isSubstrate = identityName === 'substrate';
            const walletName = identityConfigs[identityName].wallet;

            const getNextNonce = () => currentNonce++;

            before(async function () {
                const wallet = (context.web3Wallets as any)[identityName] as WalletType;
                mainSigner = wallet[walletName];

                mainIdentity = await mainSigner.getIdentity(context);
                currentNonce = (await getSidechainNonce(context, mainIdentity)).toNumber();
            });

            step('check idGraph from sidechain storage before linking', async function () {
                const idGraphGetter = await createSignedTrustedGetterIdGraph(context.api, mainSigner, mainIdentity);
                const res = await sendAesRequestFromGetter(context, teeShieldingKey, hexToU8a(aesKey), idGraphGetter);

                const idGraph = decodeIdGraph(context.sidechainRegistry, res.value);

                assert.lengthOf(idGraph, 0);
            });

            step(`linking identities (${walletName} ${identityName} account)`, async function () {
                const idGraphHashResults: HexString[] = [];

                const linkAndAssert = async (
                    identityName: string,
                    identity: CorePrimitivesIdentity,
                    expectedIdGraph: [CorePrimitivesIdentity, boolean][],
                    signer?: Signer,
                    identityType?: string,
                    verificationType?: string
                ) => {
                    const nonce = getNextNonce();
                    const validationData = signer
                        ? await buildValidations(context, mainIdentity, identity, nonce, identityName as any, signer)
                        : await buildWeb2Validation({
                              identityType,
                              context,
                              signerIdentitity: mainIdentity,
                              linkIdentity: identity,
                              verificationType,
                              validationNonce: nonce,
                          } as any);

                    const networks = context.api.createType(
                        'Vec<Web3Network>',
                        identityConfigs[identityName]?.networks ?? []
                    );
                    const requestIdentifier = `0x${randomBytes(32).toString('hex')}`;
                    const linkIdentityCall = await createSignedTrustedCallLinkIdentity(
                        context.api,
                        context.mrEnclave,
                        context.api.createType('Index', nonce),
                        mainSigner,
                        mainIdentity,
                        identity.toHex(),
                        validationData.toHex(),
                        networks.toHex(),
                        context.api.createType('Option<RequestAesKey>', aesKey).toHex(),
                        requestIdentifier,
                        {
                            withWrappedBytes: false,
                            withPrefix: (idGraphHashResults.length + 1) % 2 === 0, // alternate per entry
                        }
                    );
                    const res = await sendRequestFromTrustedCall(context, teeShieldingKey, linkIdentityCall);
                    idGraphHashResults.push(
                        await assertIdGraphMutationResult(
                            context,
                            teeShieldingKey,
                            mainIdentity,
                            res,
                            'LinkIdentityResult',
                            expectedIdGraph
                        )
                    );
                    await assertIsInSidechainBlock('linkIdentityCall', res);

                    linkedIdentityNetworks.push({
                        identity,
                        networks,
                    });
                };

                // link identity
                for (let i = 0; i < identityNames.length; i++) {
                    const identityName = identityNames[i];
                    const signer = createWeb3Wallet(identityName, randomBytes(32).toString('base64'));
                    const identity = await signer.getIdentity(context);

                    const expectedIdGraph: [CorePrimitivesIdentity, boolean][] =
                        i === 0
                            ? [
                                  [mainIdentity, true],
                                  [identity, true],
                              ]
                            : [[identity, true]];

                    await linkAndAssert(identityName, identity, expectedIdGraph, signer);
                }

                // Web2
                if (isSubstrate) {
                    // discord
                    const discordIdentity = await buildIdentityHelper('bob', 'Discord', context);
                    await linkAndAssert(
                        'discord',
                        discordIdentity,
                        [[discordIdentity, true]],
                        undefined,
                        'Discord',
                        'OAuth2'
                    );

                    // twitter
                    const twitterIdentity = await buildIdentityHelper('mock_user', 'Twitter', context);
                    await linkAndAssert(
                        'twitter',
                        twitterIdentity,
                        [[twitterIdentity, true]],
                        undefined,
                        'Twitter',
                        'PublicTweet'
                    );
                }

                assert.lengthOf(idGraphHashResults, linkedIdentityNetworks.length);
            });

            step('check user sidechain storage after linking', async function () {
                const idGraphGetter = await createSignedTrustedGetterIdGraph(context.api, mainSigner, mainIdentity);
                const res = await sendAesRequestFromGetter(context, teeShieldingKey, hexToU8a(aesKey), idGraphGetter);

                const idGraph = decodeIdGraph(context.sidechainRegistry, res.value);

                for (const { identity, networks } of linkedIdentityNetworks) {
                    const identityDump = JSON.stringify(identity.toHuman(), null, 4);
                    console.debug(`checking identity: ${identityDump}`);
                    const idGraphNode = idGraph.find(([idGraphNodeIdentity]) => idGraphNodeIdentity.eq(identity));
                    assert.isDefined(idGraphNode, `identity not found in idGraph: ${identityDump}`);
                    const [, idGraphNodeContext] = idGraphNode!;

                    const web3networks = idGraphNode![1].web3networks.toHuman();
                    assert.deepEqual(web3networks, networks.toHuman());

                    assert.equal(
                        idGraphNodeContext.status.toString(),
                        'Active',
                        `status should be active for identity: ${identityDump}`
                    );
                    console.debug('active ✅');
                }

                await assertIdGraphHash(context, teeShieldingKey, mainIdentity, idGraph);
            });

            step(`deactivating identity(${walletName} ${identityName} account)`, async function () {
                const idGraphHashResults: HexString[] = [];
                for (const { identity } of linkedIdentityNetworks) {
                    const nonce = getNextNonce();
                    const requestIdentifier = `0x${randomBytes(32).toString('hex')}`;
                    const deactivateIdentityCall = await createSignedTrustedCallDeactivateIdentity(
                        context.api,
                        context.mrEnclave,
                        context.api.createType('Index', nonce),
                        mainSigner,
                        mainIdentity,
                        identity.toHex(),
                        context.api.createType('Option<RequestAesKey>', aesKey).toHex(),
                        requestIdentifier
                    );

                    const expectedIdGraph: [CorePrimitivesIdentity, boolean][] = [[identity, false]];

                    const res = await sendRequestFromTrustedCall(context, teeShieldingKey, deactivateIdentityCall);
                    idGraphHashResults.push(
                        await assertIdGraphMutationResult(
                            context,
                            teeShieldingKey,
                            mainIdentity,
                            res,
                            'DeactivateIdentityResult',
                            expectedIdGraph
                        )
                    );
                    await assertIsInSidechainBlock('deactivateIdentityCall', res);
                }

                assert.lengthOf(idGraphHashResults, linkedIdentityNetworks.length);
            });

            step('check idGraph from sidechain storage after deactivating', async function () {
                const idGraphGetter = await createSignedTrustedGetterIdGraph(context.api, mainSigner, mainIdentity);
                const res = await sendAesRequestFromGetter(context, teeShieldingKey, hexToU8a(aesKey), idGraphGetter);
                const idGraph = decodeIdGraph(context.sidechainRegistry, res.value);

                for (const { identity } of linkedIdentityNetworks) {
                    const identityDump = JSON.stringify(identity.toHuman(), null, 4);
                    console.debug(`checking identity: ${identityDump}`);
                    const idGraphNode = idGraph.find(([idGraphNodeIdentity]) => idGraphNodeIdentity.eq(identity));
                    assert.isDefined(idGraphNode, `identity not found in idGraph: ${identityDump}`);
                    const [, idGraphNodeContext] = idGraphNode!;

                    assert.equal(
                        idGraphNodeContext.status.toString(),
                        'Inactive',
                        `status should be Inactive for identity: ${identityDump}`
                    );
                    console.debug('inactive ✅');
                }

                await assertIdGraphHash(context, teeShieldingKey, mainIdentity, idGraph);
            });

            step(`activating identity(${walletName} ${identityName} account)`, async function () {
                const idGraphHashResults: HexString[] = [];
                for (const { identity } of linkedIdentityNetworks) {
                    const nonce = getNextNonce();
                    const requestIdentifier = `0x${randomBytes(32).toString('hex')}`;
                    const activateIdentityCall = await createSignedTrustedCallActivateIdentity(
                        context.api,
                        context.mrEnclave,
                        context.api.createType('Index', nonce),
                        mainSigner,
                        mainIdentity,
                        identity.toHex(),
                        context.api.createType('Option<RequestAesKey>', aesKey).toHex(),
                        requestIdentifier
                    );

                    const expectedIdGraph: [CorePrimitivesIdentity, boolean][] = [[identity, true]];

                    const res = await sendRequestFromTrustedCall(context, teeShieldingKey, activateIdentityCall);
                    idGraphHashResults.push(
                        await assertIdGraphMutationResult(
                            context,
                            teeShieldingKey,
                            mainIdentity,
                            res,
                            'ActivateIdentityResult',
                            expectedIdGraph
                        )
                    );
                    await assertIsInSidechainBlock('activateIdentityCall', res);
                }
                assert.lengthOf(idGraphHashResults, linkedIdentityNetworks.length);
            });

            step('check idGraph from sidechain storage after activating', async function () {
                const idGraphGetter = await createSignedTrustedGetterIdGraph(context.api, mainSigner, mainIdentity);
                const res = await sendAesRequestFromGetter(context, teeShieldingKey, hexToU8a(aesKey), idGraphGetter);
                const idGraph = decodeIdGraph(context.sidechainRegistry, res.value);

                for (const { identity } of linkedIdentityNetworks) {
                    const identityDump = JSON.stringify(identity.toHuman(), null, 4);
                    console.debug(`checking identity: ${identityDump}`);
                    const idGraphNode = idGraph.find(([idGraphNodeIdentity]) => idGraphNodeIdentity.eq(identity));
                    assert.isDefined(idGraphNode, `identity not found in idGraph: ${identityDump}`);
                    const [, idGraphNodeContext] = idGraphNode!;

                    assert.equal(
                        idGraphNodeContext.status.toString(),
                        'Active',
                        `status should be active for identity: ${identityDump}`
                    );
                    console.debug('active ✅');
                }

                await assertIdGraphHash(context, teeShieldingKey, mainIdentity, idGraph);
            });

            step('check sidechain nonce', async function () {
                await sleep(20);
                const nonce = await getSidechainNonce(context, mainIdentity);
                assert.equal(nonce.toNumber(), currentNonce);
            });
        });
    }
});
