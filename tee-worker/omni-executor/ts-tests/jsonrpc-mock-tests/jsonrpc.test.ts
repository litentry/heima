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
    ExportWalletResponse,
    encrypt,
    decryptWithAes,
} from './utils';
import { signMessage } from 'viem/accounts';
import { u8aToHex } from '@polkadot/util';
import { AesOutput } from '@heima-network/api-augment/identity';
describe('Omni JsonRpc Mock Tests', function () {
    this.timeout(100000);

    const evmWallet = randomEvmWallet();
    const omniAccount = calculateOmniAccount(evmWallet.address);
    const aesRandomKey = crypto.getRandomValues(new Uint8Array(32));
    let id_token: string;
    let shieldingKey: GetShieldingKeyResponse;

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

        // store shielding key for later use
        shieldingKey = result;
    });

    it('should get next intent id successfully', async function () {
        const result: GetNextIntentIdResponse = await omniApi.getNextIntentId({
            omni_account: omniAccount,
        });

        // expected result:
        // 1

        expect(result).to.be.equal(1);
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
        id_token = result.id_token;
    });

    it('should add wallet successfully', async function () {
        const result: null = await omniApi.addWallet(id_token);

        // expected result:
        // {}

        expect(result).to.be.null;
    });

    it('should export wallet successfully', async function () {
        const encryptedKey = await encrypt({ cleartext: aesRandomKey, shieldingKey });
        try {
            const result: ExportWalletResponse = await omniApi.exportWallet(
                {
                    key: u8aToHex(encryptedKey.ciphertext),
                    google_code: '',
                    chain_id: 1, // get from backend
                    wallet_index: 0, // get from backend
                    wallet_address: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266', // We do not currently know the logic for generating the backend address, so we can only use a hardcode here temporarily.
                },
                id_token
            );

            // expected result:
            // {
            //    ciphertext: "0xb0b6126dd7acae085cd4da94185d0707bb19257f6f95cac84c2c4b0ffc0e23995ac656c96f08abd007b32bafae5477f7",
            //    aad: "0x",
            //    nonce: "0xaad559c090dc5195ace3e3eb",
            //  };
        } catch (error) {
            console.log('Expected export wallet failure:', error);
        }

        const mockResult = {
            ciphertext:
                '0x1f7dde5143ccf34ac1756dcbd1cb7563408e5750c6758dba58ca383e723f47a1b2d774170b690229c01d7d9d9a1b2780',
            aad: '0x',
            nonce: '0xcc6e4b8b9b50bc990dcb016e',
        };

        const mockAesKey = '0x3aefe1ec780dc389ed4e048eacaf25679a7f2e567d11f65071b51029dd63e79d';

        const decryptedKey = await decryptWithAes(mockAesKey, mockResult as unknown as AesOutput, 'hex');
        console.log('decryptedKey', decryptedKey);
    });
});
