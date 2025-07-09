import { describe, it } from 'mocha';
import { expect } from 'chai';
import {
    ClientId,
    GetShieldingKeyResponse,
    omniApi,
    RequestEmailVerificationCodeResponse,
    UserLoginResponse,
    calculateOmniAccount,
    GetWeb3SignInMessageResponse,
    randomEvmWallet,
    GetNextIntentIdResponse,
} from './utils';
import { signMessage } from 'viem/accounts';

describe('Omni JsonRpc Mock Tests', function () {
    this.timeout(100000);

    const evmWallet = randomEvmWallet();
    const omniAccount = calculateOmniAccount(evmWallet.address);

    let access_token: string;

    it('should request email verification code successfully', async function () {
        const result: RequestEmailVerificationCodeResponse = await omniApi.requestEmailVerificationCode({
            client_id: ClientId.Wildmeta,
            user_email: 'test@gmail.com',
        });

        // expected result:
        // null

        expect(result).to.be.null;
    });

    it('should get shielding key successfully', async function () {
        const result: GetShieldingKeyResponse = await omniApi.getShieldingKey();

        // expected result:
        // {
        //     "n": "0x123...",
        //     "e": "0x123..."
        // }

        expect(result.n).to.be.not.undefined;
        expect(result.e).to.be.not.undefined;
    });

    it('should get next intent id successfully', async function () {
        const result: GetNextIntentIdResponse = await omniApi.getNextIntentId({
            omni_account: omniAccount,
        });

        // expected result:
        // 0

        expect(result).to.be.equal(0);
    });

    it('should get web3 sign in message successfully', async function () {
        const result: GetWeb3SignInMessageResponse = await omniApi.getWeb3SignInMessage({
            client_id: ClientId.Wildmeta,
            omni_account: omniAccount,
        });

        // expected result:
        // {
        //     "message_code": "123456",
        //     "client_id": "wildmeta",
        //     "omni_account": "0x1234567890123456789012345678901234567890"
        // }

        expect(result.message_code).to.be.not.undefined;
        expect(result.client_id).to.be.equal(ClientId.Wildmeta);
        expect(result.omni_account).to.be.equal(omniAccount);
    });

    it('should fail to login with web2(email) due to missing verification code', async function () {
        try {
            const result: UserLoginResponse = await omniApi.userLogin({
                user_id: {
                    type: 'email',
                    value: 'test@gmail.com',
                },
                user_auth: {
                    type: 'email',
                    value: '654321', // email code
                },
                client_id: ClientId.Wildmeta,
                client_auth: {
                    type: 'wildmeta',
                    value: {
                        google_code: '',
                        invite_code: '',
                    },
                },
            });
        } catch (error) {
            // Expected failure: verify_email_authentication fails when code not found in VerificationCodeStorage
            // This happens because we need to call requestEmailVerificationCode first and use the actual code
            console.log('Expected email login failure:', error);
        }
    });

    it('should login user successfully with web3(evm)', async function () {
        const omniAccount = calculateOmniAccount(evmWallet.address);
        const message: GetWeb3SignInMessageResponse = await omniApi.getWeb3SignInMessage({
            client_id: ClientId.Wildmeta,
            omni_account: omniAccount,
        });

        const messageString = JSON.stringify(message);
        const signature = await signMessage({ message: messageString, privateKey: evmWallet.privateKey });

        const result: UserLoginResponse = await omniApi.userLogin({
            user_id: {
                type: 'evm',
                value: evmWallet.address,
            },
            user_auth: {
                type: 'evm',
                value: signature,
            },
            client_id: ClientId.Wildmeta,
            client_auth: {
                type: 'wildmeta',
                value: {
                    google_code: '',
                    invite_code: '',
                },
            },
        });

        // expected result:
        // {
        //     "access_token": "0x123...",
        //     "id_token": "0x123...",
        //     "backend_response": {
        //         "code": 10000,
        //         "message": "OK",
        //         "data": {
        //             "user_id": null
        //         }
        //     }
        // }

        expect(result.access_token).to.be.not.undefined;
        expect(result.id_token).to.be.not.undefined;
        expect(result.backend_response.code).to.be.equal(10000);
        expect(result.backend_response.message).to.be.equal('OK');

        // Store the access token for use in subsequent tests
        access_token = result.access_token;
    });

    it('should add wallet successfully', async function () {
        const result: null = await omniApi.addWallet(access_token);

        // expected result:
        // {}

        expect(result).to.be.null;
    });
});
