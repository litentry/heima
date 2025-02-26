import { Authentication, LitentryIdentity, OAuth2Data } from '@litentry/parachain-api';
import { Registry } from '@polkadot/types-codec/types';
import { createLitentryMultiSignature } from './litentry-multi-signature';
import { createOAuth2Data, type OAuth2DataType } from './oauth2';

export type AuthenticationData =
  | {
      type: 'Email';
      verificationCode: string;
    }
  | {
      type: 'Web3';
      signer: LitentryIdentity;
      signature: string;
    }
  | {
      type: 'AuthToken';
      token: string;
    }
  | {
      type: 'OAuth2';
      data: OAuth2DataType;
    };

export function createAuthentication(registry: Registry, data: AuthenticationData): Authentication {
  let authentication;
  switch (data.type) {
    case 'Email':
      authentication = {
        Email: data.verificationCode,
      };
      break;
    case 'Web3':
      authentication = {
        Web3: createLitentryMultiSignature(registry, {
          who: data.signer,
          signature: data.signature,
        }),
      };
      break;
    case 'AuthToken':
      authentication = {
        AuthToken: data.token,
      };
      break;
    case 'OAuth2':
      authentication = {
        OAuth2: createOAuth2Data(registry, data.data),
      };
      break;
    default:
      throw new Error('Unsupported authentication type');
  }
  return registry.createType('Authentication', authentication) as unknown as Authentication;
}
