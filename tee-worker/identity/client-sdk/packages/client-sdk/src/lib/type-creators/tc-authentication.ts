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
  return registry.createType('TCAuthentication', {
    [data.type]:
      data.type === 'Email'
        ? data.verificationCode
        : data.type === 'Web3'
        ? createLitentryMultiSignature(registry, {
            who: data.signer,
            signature: data.signature,
          })
        : data.token,
  });
}
