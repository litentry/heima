import type { Registry } from '@polkadot/types-codec/types';
import { OAuth2Data, OAuth2Provider } from '@heima/parachain-api';

export type OAuth2DataType = {
  provider: 'Google';
  code: string;
  state: string;
  redirectUri: string;
};

export function createOAuth2Data(registry: Registry, data: OAuth2DataType): OAuth2Data {
  const { provider, code, state, redirectUri } = data;

  let oAuth2Provider: OAuth2Provider | undefined;
  if (provider === 'Google') {
    oAuth2Provider = registry.createType('OAuth2Provider', provider) as unknown as OAuth2Provider;
  }

  if (oAuth2Provider) {
    return registry.createType('OAuth2Data', {
      provider: oAuth2Provider,
      code,
      state,
      redirect_uri: redirectUri,
    }) as unknown as OAuth2Data;
  }

  throw new Error('Unsupported OAuth2 provider');
}
