import { Identity, OmniAuth } from '@heima-network/parachain-api';
import { Registry } from '@polkadot/types-codec/types';
import { createHeimaMultiSignature } from './heima-multi-signature';
import { createOAuth2Data, type OAuth2DataType } from './oauth2';

export type OmniAuthData =
  | {
      type: 'Email';
      verificationCode: string;
    }
  | {
      type: 'Web3';
      signer: Identity;
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

export function createOmniAuth(registry: Registry, data: OmniAuthData): OmniAuth {
  let auth;
  switch (data.type) {
    case 'Email':
      auth = {
        Email: data.verificationCode,
      };
      break;
    case 'Web3':
      auth = {
        Web3: createHeimaMultiSignature(registry, {
          who: data.signer,
          signature: data.signature,
        }),
      };
      break;
    case 'AuthToken':
      auth = {
        AuthToken: data.token,
      };
      break;
    case 'OAuth2':
      auth = {
        OAuth2: createOAuth2Data(registry, data.data),
      };
      break;
    default:
      throw new Error('Unsupported auth type');
  }
  return registry.createType<OmniAuth>('OmniAuth', auth);
}
