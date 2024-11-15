import { LitentryIdentity, TCAuthentication } from '@litentry/parachain-api';
import { Registry } from '@polkadot/types-codec/types';
import { createLitentryMultiSignature } from './litentry-multi-signature';

type AuthenticationData =
  | {
      type: 'Email';
      verificationCode: string;
    }
  | {
      type: 'Web3';
      signer: LitentryIdentity;
      signature: string;
    };

export function createTCAuthenticationType(
  registry: Registry,
  data: AuthenticationData
): TCAuthentication {
  return registry.createType('TCAuthentication', {
    [data.type]:
      data.type === 'Email'
        ? data.verificationCode
        : createLitentryMultiSignature(registry, {
            who: data.signer,
            signature: data.signature,
          }),
  });
}
