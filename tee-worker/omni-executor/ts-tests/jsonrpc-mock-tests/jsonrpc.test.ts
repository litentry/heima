import { describe, it } from 'mocha';
import { expect } from 'chai';
import { omniApi, RequestEmailVerificationCodeResponse } from './utils';

enum ClientId {
  Wildmeta = 'wildmeta',
  Heima = 'heima',
  Pumpx = 'pumpx',
}

describe('Request Email Verification Code', () => {

  it('should request email verification code successfully', async function() {
    this.timeout(10000);
    
      const result:RequestEmailVerificationCodeResponse = await omniApi.requestEmailVerificationCode({client_id: ClientId.Wildmeta,user_email: 'verin@liteng.io'});
      
      expect(result).to.be.null;
  });
});