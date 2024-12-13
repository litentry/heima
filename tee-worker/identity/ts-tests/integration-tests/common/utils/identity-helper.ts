import { u8aToHex } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import type { IntegrationTestContext } from '../common-types';
import { AesOutput } from 'parachain-api';
import { decryptWithAes, Signer } from './crypto';
import { ethers } from 'ethers';
import type { TypeRegistry } from '@polkadot/types';
import type { PalletIdentityManagementTeeIdentityContext } from 'sidechain-api';
import type { LitentryValidationData, CorePrimitivesIdentity } from 'parachain-api';
import type { HexString } from '@polkadot/util/types';

// blake2_256(<sidechain nonce> + <primary AccountId> + <identity-to-be-linked>)
export function generateVerificationMessage(
    context: IntegrationTestContext,
    signer: CorePrimitivesIdentity,
    identity: CorePrimitivesIdentity,
    sidechainNonce: number,
    options?: { prettifiedMessage?: boolean }
): string {
    const _options = { prettifiedMessage: false, ...options };
    const encodedIdentity = context.api.createType('CorePrimitivesIdentity', identity).toU8a();
    const encodedWho = context.api.createType('CorePrimitivesIdentity', signer).toU8a();
    const encodedSidechainNonce = context.api.createType('Index', sidechainNonce);
    const msg = Buffer.concat([encodedSidechainNonce.toU8a(), encodedWho, encodedIdentity]);
    const hash = blake2AsHex(msg, 256);

    if (_options.prettifiedMessage) {
        return `Token: ${hash}`;
    }

    return hash;
}

export async function buildIdentityHelper(
    address: HexString | string,
    type: CorePrimitivesIdentity['type'],
    context: IntegrationTestContext
): Promise<CorePrimitivesIdentity> {
    const identity = {
        [type]: address,
    };
    return context.api.createType('CorePrimitivesIdentity', identity);
}

export function parseIdGraph(
    sidechainRegistry: TypeRegistry,
    idGraphOutput: AesOutput,
    aesKey: HexString
): [CorePrimitivesIdentity, PalletIdentityManagementTeeIdentityContext][] {
    const decryptedIdGraph = decryptWithAes(aesKey, idGraphOutput, 'hex');
    const idGraph: [CorePrimitivesIdentity, PalletIdentityManagementTeeIdentityContext][] =
        sidechainRegistry.createType(
            'Vec<(CorePrimitivesIdentity, PalletIdentityManagementTeeIdentityContext)>',
            decryptedIdGraph
        );

    return idGraph;
}

export type Web2ValidationConfig =
    | {
          identityType: 'Discord';
          context: IntegrationTestContext;
          signerIdentitity: CorePrimitivesIdentity;
          linkIdentity: CorePrimitivesIdentity;
          verificationType: 'PublicMessage' | 'OAuth2';
          validationNonce: number;
      }
    | {
          identityType: 'Twitter';
          context: IntegrationTestContext;
          signerIdentitity: CorePrimitivesIdentity;
          linkIdentity: CorePrimitivesIdentity;
          verificationType: 'PublicTweet';
          validationNonce: number;
      }
    | {
          identityType: 'Twitter';
          context: IntegrationTestContext;
          signerIdentitity: CorePrimitivesIdentity;
          linkIdentity: CorePrimitivesIdentity;
          verificationType: 'OAuth2';
          validationNonce: number;
          oauthState: string;
      };

export async function buildWeb2Validation(config: Web2ValidationConfig): Promise<LitentryValidationData> {
    const { context, signerIdentitity, linkIdentity, validationNonce } = config;
    const msg = generateVerificationMessage(context, signerIdentitity, linkIdentity, validationNonce);
    console.log(`post verification msg to ${config.identityType}:`, msg);

    if (config.identityType === 'Discord') {
        const discordValidationData = {
            Web2Validation: {
                Discord: {},
            },
        };

        if (config.verificationType === 'PublicMessage') {
            discordValidationData.Web2Validation.Discord = {
                PublicMessage: {
                    channel_id: `0x${Buffer.from('919848392035794945', 'utf8').toString('hex')}`,
                    message_id: `0x${Buffer.from('1', 'utf8').toString('hex')}`,
                    guild_id: `0x${Buffer.from(validationNonce.toString(), 'utf8').toString('hex')}`,
                },
            };
        } else {
            discordValidationData.Web2Validation.Discord = {
                OAuth2: {
                    code: `0x${Buffer.from('test-oauth-code', 'utf8').toString('hex')}`,
                    redirect_uri: `0x${Buffer.from('http://test-redirect-uri', 'utf8').toString('hex')}`,
                },
            };
        }

        return context.api.createType('LitentryValidationData', discordValidationData);
    } else {
        const twitterValidationData = {
            Web2Validation: {
                Twitter: {},
            },
        };

        if (config.verificationType === 'PublicTweet') {
            twitterValidationData.Web2Validation.Twitter = {
                PublicTweet: {
                    tweet_id: `0x${Buffer.from(validationNonce.toString(), 'utf8').toString('hex')}`,
                },
            };
        } else {
            twitterValidationData.Web2Validation.Twitter = {
                OAuth2: {
                    code: `0x${Buffer.from('test-oauth-code', 'utf8').toString('hex')}`,
                    state: config.oauthState,
                    redirect_uri: `0x${Buffer.from('http://test-redirect-uri', 'utf8').toString('hex')}`,
                },
            };
        }

        return context.api.createType('LitentryValidationData', twitterValidationData);
    }
}

export async function buildValidations(
    context: IntegrationTestContext,
    signerIdentitity: CorePrimitivesIdentity,
    linkIdentity: CorePrimitivesIdentity,
    startingSidechainNonce: number,
    network: 'evm' | 'substrate' | 'bitcoin' | 'solana',
    signer?: Signer,
    options?: { prettifiedMessage?: boolean }
): Promise<LitentryValidationData> {
    const _options = { prettifiedMessage: false, ...options };
    const validationNonce = startingSidechainNonce++;

    const msg = generateVerificationMessage(context, signerIdentitity, linkIdentity, validationNonce);
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
        const evmSignature = u8aToHex(await signer!.sign(msgHash));

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
        const substrateSignature = await signer!.sign(msg);
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
        const bitcoinSignature = await signer!.sign(msg.substring(2));
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
        const solanaSignature = await signer!.sign(msg);
        solanaValidationData!.Web3Validation.Solana.signature.Ed25519 = u8aToHex(solanaSignature);

        return context.api.createType('LitentryValidationData', solanaValidationData);
    }

    throw new Error(`[buildValidation]: Unsupported network ${network}.`);
}
