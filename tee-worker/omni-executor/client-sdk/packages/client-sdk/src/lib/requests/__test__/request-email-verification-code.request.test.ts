import { requestEmailVerificationCode } from '@requests/request-email-verification-code.request';

describe.skip('request-email-verification-code', () => {
  it('should works', async () => {
    await requestEmailVerificationCode({
      email: 'test@google.com',
    });
  });
});
