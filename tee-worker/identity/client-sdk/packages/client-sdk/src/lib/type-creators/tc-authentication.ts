import { LitentryIdentity, TCAuthentication } from '@litentry/parachain-api';
import { Registry } from '@polkadot/types-codec/types';
import { createLitentryMultiSignature } from './litentry-multi-signature';

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
    };

export function createTCAuthenticationType(
  registry: Registry,
  data: AuthenticationData
): TCAuthentication {
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
    default:
      throw new Error('Unsupported authentication type');
  }
  return registry.createType('TCAuthentication', authentication);
}
