import type { Registry } from '@polkadot/types-codec/types';
import { assert, isHex, stringToHex } from '@polkadot/util';

import type { Identity, ValidationData } from '@heima-network/parachain-api';

import { createIdentityType } from '@lib/type-creators/identity';
import { decodeSignature } from '@utils/decode-signature';
import { getSignatureCryptoType } from '@utils/get-signature-crypto-type';

/**
 * Ownership proof for Web3 accounts (Substrate, EVM, Bitcoin).
 *
 * Bitcoin signatures are base64-encoded strings. Substrate and EVM signatures are hex-encoded strings.
 *
 * @see createIdentityType
 */
export type Web3Proof = {
  signature: `0x${string}` | string;
  message: string;
};

/**
 * Ownership proof for Twitter accounts
 *
 * @see createValidationDataType
 */
export type TwitterProof = { tweetId: string };

/**
 * Ownership proof for Twitter accounts using oAuth2
 *
 * @see createValidationDataType
 */
export type TwitterOAuth2Proof = {
  code: string;
  state: string;
  redirectUri: string;
};

/**
 * Ownership proof for Discord accounts
 *
 * @see createValidationDataType
 */
export type DiscordProof = {
  channelId: string;
  messageId: string;
  guildId: string;
};
/**
 * Ownership proof for Discord accounts using oAuth2
 *
 * @see createValidationDataType
 */
export type DiscordOAuth2Proof = {
  code: string;
  redirectUri: string;
};

/**
 * Ownership proof for Email
 *
 * @see createValidationDataType
 */
export type EmailProof = {
  email: string;
  verificationCode: string;
};

/**
 * Creates the ValidationData given the identity network and its type.
 *
 * The proof to pass depends on the identity network (IdentityType):
 * - Web3: Web3Proof
 * - Twitter: TwitterProof
 * - Discord: DiscordProof
 *
 * @example Web3
 * ```ts
 * import { createValidationDataType } from '@heima-network/client-sdk';
 * import type { Web3Proof } from '@heima-network/client-sdk';
 *
 * const userAddress = '0x123';
 *
 * const proof: Web3Proof = {
 *   signature: '0x123',
 *   message: '0x123',
 * }
 *
 * const validationData = createValidationDataType(
 *   registry,
 *   {
 *     addressOrHandle: userAddress,
 *     type: 'Evm',
 *   },
 *   proof,
 * );
 * ```
 *
 * @example Twitter
 * ```ts
 * import { createValidationDataType } from '@heima-network/client-sdk';
 * import type { TwitterProof } from '@heima-network/client-sdk';
 *
 * const userHandle = '@heima';
 *
 * const proof: TwitterProof = {
 *   // Both twitter.com and x.com are valid
 *   tweetId: 'https://twitter.com/0x123/status/123',
 * };
 *
 * const validationData = createValidationDataType(
 *   registry,
 *   {
 *     addressOrHandle: userHandle,
 *     type: 'Twitter',
 *   },
 *   proof,
 * );
 * ```
 *
 */
export function createValidationDataType<IIdentityType extends Identity['type']>(
  /**  Parachain API's type registry */
  registry: Registry,
  identityDescriptor: {
    /** The address or handle of the identity */
    addressOrHandle: string;
    /** The identity type */
    type: IIdentityType;
  },
  /** The ownership proof */
  proof: IIdentityType extends 'Discord'
    ? DiscordProof | DiscordOAuth2Proof
    : IIdentityType extends 'Twitter'
      ? TwitterProof | TwitterOAuth2Proof
      : IIdentityType extends 'Email'
        ? EmailProof
        : Web3Proof,
): ValidationData {
  const identity = createIdentityType(registry, identityDescriptor);

  if (isProofWeb3(identity, proof)) {
    const signature: Uint8Array = decodeSignature(proof.signature, identity);
    let message = proof.message;

    let web3Type: 'Evm' | 'Substrate' | 'Bitcoin' | 'Solana' = 'Evm';
    let cryptoType: 'Ethereum' | 'None' | 'Ed25519' | 'Sr25519' | 'Ecdsa' | 'Bitcoin' = 'Ethereum';

    // derive the substrate crypto type from the signature
    if (identity.isSubstrate) {
      web3Type = 'Substrate';
      cryptoType = getSignatureCryptoType({
        message: proof.message,
        signature,
        address: identity.asSubstrate,
      });

      if (cryptoType !== 'Sr25519' && cryptoType !== 'Ed25519') {
        throw new Error(`[vault] Unsupported Substrate signature crypto type "${cryptoType}"`);
      }
    }

    if (identity.isBitcoin) {
      web3Type = 'Bitcoin';
      cryptoType = 'Bitcoin';

      // restore the 0x prefix that was removed for bitcoin challenge codes for raw messages
      message = proof.message.startsWith('Token: ')
        ? proof.message
        : isHex(proof.message)
          ? proof.message
          : `0x${proof.message}`;
    }

    if (identity.isSolana) {
      web3Type = 'Solana';
      cryptoType = 'Ed25519';
    }

    return registry.createType<ValidationData>('ValidationData', {
      Web3Validation: {
        [web3Type]: {
          message,
          signature: {
            [cryptoType]: signature,
          },
        },
      },
    });
  }

  if (isProofEmail(identity, proof)) {
    return registry.createType<ValidationData>('ValidationData', {
      Web2Validation: {
        Email: {
          email: stringToHex(proof.email),
          verification_code: stringToHex(proof.verificationCode),
        },
      },
    });
  }

  if (isProofTwitter(identity, proof)) {
    assert(proof.tweetId, '[vault::link_identity] Missing tweetId');

    return registry.createType<ValidationData>('ValidationData', {
      Web2Validation: {
        Twitter: {
          PublicTweet: {
            tweet_id: stringToHex(proof.tweetId),
          },
        },
      },
    });
  }

  if (isProofTwitterOAuth2(identity, proof)) {
    return registry.createType<ValidationData>('ValidationData', {
      Web2Validation: {
        Twitter: {
          OAuth2: {
            code: stringToHex(proof.code),
            state: stringToHex(proof.state),
            redirect_uri: stringToHex(proof.redirectUri),
          },
        },
      },
    });
  }

  if (isProofDiscord(identity, proof)) {
    assert(proof.guildId, '[vault::link_identity] Missing guildId');
    assert(proof.channelId, '[vault::link_identity] Missing channelId');
    assert(proof.messageId, '[vault::link_identity] Missing messageId');

    return registry.createType<ValidationData>('ValidationData', {
      Web2Validation: {
        Discord: {
          PublicMessage: {
            channel_id: stringToHex(proof.channelId),
            message_id: stringToHex(proof.messageId),
            guild_id: stringToHex(proof.guildId),
          },
        },
      },
    });
  }

  if (isProofDiscordOAuth2(identity, proof)) {
    return registry.createType<ValidationData>('ValidationData', {
      Web2Validation: {
        Discord: {
          OAuth2: {
            code: stringToHex(proof.code),
            redirect_uri: stringToHex(proof.redirectUri),
          },
        },
      },
    });
  }

  throw new Error(`[vault] Unsupported identity network "${identity.type}"`);
}

function isProofWeb3(
  identity: Identity,
  proof: Web3Proof | TwitterProof | TwitterOAuth2Proof | DiscordProof | DiscordOAuth2Proof | EmailProof,
): proof is Web3Proof {
  const isWeb3 = identity.isEvm || identity.isSubstrate || identity.isBitcoin || identity.isSolana;

  if (!isWeb3) {
    return false;
  }

  const maybeWeb3Proof = proof as Web3Proof;
  assert(maybeWeb3Proof.message, 'Missing message property for web3 proof');
  assert(maybeWeb3Proof.signature, 'Missing signature property for web3 proof');

  return true;
}

function isProofTwitter(
  identity: Identity,
  proof: Web3Proof | TwitterProof | TwitterOAuth2Proof | DiscordProof | DiscordOAuth2Proof,
): proof is TwitterProof {
  const isTwitter = identity.isTwitter;

  if (!isTwitter) {
    return false;
  }

  if (!(proof as TwitterProof).tweetId) {
    return false;
  }

  return true;
}

function isProofTwitterOAuth2(
  identity: Identity,
  proof: Web3Proof | TwitterProof | TwitterOAuth2Proof | DiscordProof | DiscordOAuth2Proof,
): proof is TwitterOAuth2Proof {
  const isTwitter = identity.isTwitter;

  if (!isTwitter) {
    return false;
  }

  const maybeTwitterProof = proof as TwitterOAuth2Proof;

  if (!maybeTwitterProof.code || !maybeTwitterProof.state || !maybeTwitterProof.redirectUri) {
    return false;
  }

  return true;
}

function isProofEmail(
  identity: Identity,
  proof: Web3Proof | TwitterProof | DiscordProof | DiscordOAuth2Proof | EmailProof,
): proof is EmailProof {
  const isEmail = identity.isEmail;

  if (!isEmail) {
    return false;
  }

  const maybeEmailProof = proof as EmailProof;

  if (!maybeEmailProof.email || !maybeEmailProof.verificationCode) {
    return false;
  }

  return true;
}

function isProofDiscord(
  identity: Identity,
  proof: Web3Proof | TwitterProof | DiscordProof | DiscordOAuth2Proof,
): proof is DiscordProof {
  const isDiscord = identity.isDiscord;

  if (!isDiscord) {
    return false;
  }

  const maybeDiscordProof = proof as DiscordProof;

  if (!maybeDiscordProof.channelId || !maybeDiscordProof.guildId || !maybeDiscordProof.messageId) {
    return false;
  }

  return true;
}

function isProofDiscordOAuth2(
  identity: Identity,
  proof: Web3Proof | TwitterProof | DiscordProof | DiscordOAuth2Proof,
): proof is DiscordOAuth2Proof {
  const isDiscord = identity.isDiscord;

  if (!isDiscord) {
    return false;
  }

  const maybeDiscordProof = proof as DiscordOAuth2Proof;

  if (!maybeDiscordProof.code || !maybeDiscordProof.redirectUri) {
    return false;
  }

  return true;
}
