import { TCAuthentication } from '@litentry/parachain-api';
import { Registry } from '@polkadot/types-codec/types';

type AuthenticationData =
  | {
      type: 'Email';
      verificationCode: string;
    }
  | {
      type: 'Web3';
      signature: string;
    };

export function createTCAuthenticationType(
  registry: Registry,
  data: AuthenticationData
): TCAuthentication {
  return registry.createType('TCAuthentication', {
    [data.type]: data.type === 'Email' ? data.verificationCode : data.signature,
  });
}
