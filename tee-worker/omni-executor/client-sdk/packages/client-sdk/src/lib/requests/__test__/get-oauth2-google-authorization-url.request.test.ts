import { getOAuth2GoogleAuthorizationUrl } from '@requests/get-oauth2-google-authorization-url.request';

describe('get-oauth2-google-authorization-url', () => {
  it('should works', async () => {
    const authorizeUrl = await getOAuth2GoogleAuthorizationUrl({
      googleAccount: 'test@google.com',
      redirectUri: 'https://test.com',
    });

    expect(authorizeUrl).toContain('https://accounts.google.com/o/oauth2/v2/auth');
  });
});
