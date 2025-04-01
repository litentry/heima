import { assert, compactStripLength, hexToU8a, u8aToString } from '@polkadot/util';

import { JsonRpcRequest } from '@utils/types';

import { enclave } from '@lib/enclave';

/**
 * Generates an OAuth2 authorization URL for Google authentication.
 * 
 * @param args - The arguments object
 * @param args.googleAccount - The Google account email address
 * @param args.redirectUri - The URI where Google will redirect after authentication
 * @returns A Promise that resolves to the Google OAuth2 authorization URL
 * @throws {Error} If googleAccount or redirectUri is empty or undefined
 */
export async function getOAuth2GoogleAuthorizationUrl(args: {
  googleAccount: string;
  redirectUri: string;
}): Promise<string> {
  const { googleAccount, redirectUri } = args;

  assert(googleAccount.length > 0, 'Google account is required');
  assert(redirectUri.length > 0, 'Redirect URI is required');

  // send the request to the Enclave
  const rpcRequest: JsonRpcRequest = {
    jsonrpc: '2.0',
    method: 'omni_getOAuth2GoogleAuthorizationUrl',
    params: [googleAccount, redirectUri],
  };

  const hexString = await enclave.send(rpcRequest);

  const [, data] = compactStripLength(hexToU8a(hexString));

  // authorize url
  return u8aToString(data);
}
