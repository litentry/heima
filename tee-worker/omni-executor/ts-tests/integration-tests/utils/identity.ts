import { ApiPromise } from '@polkadot/api';
import type { HexString } from '@polkadot/util/types';
import type { Identity, LitentryValidationData } from 'parachain-api';
import { u8aToHex } from '@polkadot/util';
import { blake2AsHex } from '@polkadot/util-crypto';
import { ethers } from 'ethers';
import { Signer } from './signer';

// blake2_256(<OmniAccount nonce> + <MemberIdentity AccountId> + <identity-to-be-linked>)
export function generateVerificationMessage(
    api: ApiPromise,
    memberIdentity: Identity,
    identityToAdd: Identity,
    omniAccountNonce: number,
    options?: { prettifiedMessage?: boolean }
): string {
    const opts = { prettifiedMessage: false, ...options };
    const encodedMemberIdentity = api.createType('Identity', memberIdentity).toU8a();
    const encodedIdentityToAdd = api.createType('Identity', identityToAdd).toU8a();
    const encodedOmniAccountNonce = api.createType('u64', omniAccountNonce);
    const msg = Buffer.concat([encodedOmniAccountNonce.toU8a(), encodedMemberIdentity, encodedIdentityToAdd]);
    const hash = blake2AsHex(msg, 256);

    if (opts.prettifiedMessage) {
        return `Token: ${hash}`;
    }

    return hash;
}

export type Web2ValidationConfig =
    | {
          identityType: 'Discord';
          api: ApiPromise;
          signerIdentitity: Identity;
          linkIdentity: Identity;
          verificationType: 'PublicMessage' | 'OAuth2';
          validationNonce: number;
      }
    | {
          identityType: 'Twitter';
          api: ApiPromise;
          signerIdentitity: Identity;
          linkIdentity: Identity;
          verificationType: 'PublicTweet';
          validationNonce: number;
      }
    | {
          identityType: 'Twitter';
          api: ApiPromise;
          signerIdentitity: Identity;
          linkIdentity: Identity;
          verificationType: 'OAuth2';
          validationNonce: number;
          oauthState: string;
      };

export async function buildWeb2Validation(config: Web2ValidationConfig): Promise<LitentryValidationData> {
    const { api, signerIdentitity, linkIdentity, validationNonce } = config;
    const msg = generateVerificationMessage(api, signerIdentitity, linkIdentity, validationNonce);
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

        return api.createType('LitentryValidationData', discordValidationData);
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

        return api.createType('LitentryValidationData', twitterValidationData);
    }
}

export async function buildValidations(
    api: ApiPromise,
    signerIdentitity: Identity,
    linkIdentity: Identity,
    startingSidechainNonce: number,
    network: 'evm' | 'substrate' | 'bitcoin' | 'solana',
    signer?: Signer
): Promise<LitentryValidationData> {
    const validationNonce = startingSidechainNonce++;

    const msg = generateVerificationMessage(api, signerIdentitity, linkIdentity, validationNonce);
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

        return api.createType('LitentryValidationData', evmValidationData);
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

        return api.createType('LitentryValidationData', substrateValidationData);
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

        return api.createType('LitentryValidationData', bitcoinValidationData);
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

        return api.createType('LitentryValidationData', solanaValidationData);
    }

    throw new Error(`[buildValidation]: Unsupported network ${network}.`);
}

export async function buildWeb3ValidationData(
    api: ApiPromise,
    sender: Identity,
    accountToAdd: Identity,
    nonce: number,
    network: 'evm' | 'substrate' | 'bitcoin' | 'solana',
    signer: Signer
): Promise<LitentryValidationData> {
    const msg = generateVerificationMessage(api, sender, accountToAdd, nonce);

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

        return api.createType('LitentryValidationData', evmValidationData);
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

        return api.createType('LitentryValidationData', substrateValidationData);
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

        return api.createType('LitentryValidationData', bitcoinValidationData);
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

        return api.createType('LitentryValidationData', solanaValidationData);
    }

    throw new Error(`[buildValidation]: Unsupported network ${network}.`);
}
