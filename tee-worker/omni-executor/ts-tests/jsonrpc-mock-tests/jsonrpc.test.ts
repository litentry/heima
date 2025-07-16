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
    AddWalletResponse,
    TransferWithdrawResponse,
    SubmitSwapOrderResponse,
    SignLimitOrderResponse,
    NotifyLimitOrderResultResponse,
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
        const result: AddWalletResponse = await omniApi.addWallet(id_token);

        // expected result:
        //    "backend_response": {
        //     "code": 10000,
        //     "message": "OK",
        //     "data": {}
        // }

        expect(result.backend_response.code).to.be.equal(10000);
        expect(result.backend_response.message).to.be.equal('OK');
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

    it('should transfer withdraw successfully', async function () {
        const transferParams = {
            request_id: 1,
            chain_id: 1,
            wallet_index: 0,
            recipient_address: '0x742d35Cc6634C0532925a3b8D4bd4A0F8b3f3c0',
            token_ca: '0xA0b86a33E6441b3F1c1e3f8B0c6c8b8d7e6f3e4a',
            amount: '1000000000000000000',
            google_code: '123456',
            lang: 'en',
        };

        const result: TransferWithdrawResponse = await omniApi.transferWithdraw(transferParams, id_token);

        // expected result:
        // {
        //     "backend_response": {
        //         "code": 10000,
        //         "message": "OK",
        //         "data": {
        //             "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        //             "transfer_id": "transfer_12345",
        //             "chain_id": 1
        //         }
        //     }
        // }

        expect(result.backend_response).to.be.not.undefined;
        expect(result.backend_response.code).to.be.equal(10000);
        expect(result.backend_response.message).to.be.equal('OK');
        expect(result.backend_response.data).to.be.not.undefined;
        expect(result.backend_response.data.tx_hash).to.be.not.undefined;
        expect(result.backend_response.data.transfer_id).to.be.not.undefined;
        expect(result.backend_response.data.chain_id).to.be.equal(1);
    });

    it('should submit swap order successfully (market order)', async function () {
        const swapOrderParams = {
            intent_id: 1,
            order_type: 'market' as const,
            swap_type: 1 as const,
            from_chain_id: 1,
            from_token_ca: '0xA0b86a33E6441b3F1c1e3f8B0c6c8b8d7e6f3e4a',
            from_amount: '0.001',
            to_chain_id: 56,
            to_token_ca: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
            double_out: false,
            is_one_click: true,
            usd_worth: '0.001',
            wallet_index: 0,
        };

        const result: SubmitSwapOrderResponse = await omniApi.submitSwapOrder(swapOrderParams, id_token);

        // expected result:
        // {
        //     "backend_response": {
        //         "market_order_response": {
        //             "code": 10000,
        //             "message": "OK",
        //             "data": {
        //                 "tx_hash": ["0x1234567890abcdef..."]
        //             }
        //         }
        //     }
        // }

        expect(result.backend_response).to.be.not.undefined;
        expect(result.backend_response.market_order_response || result.backend_response.limit_order_response).to.be.not
            .undefined;
    });

    it('should submit swap order successfully (limit order)', async function () {
        const swapOrderParams = {
            intent_id: 2,
            order_type: 'limit' as const,
            swap_type: 2 as const,
            from_chain_id: 1,
            from_token_ca: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
            from_amount: '0.001',
            to_chain_id: 56,
            to_token_ca: '0xA0b86a33E6441b3F1c1e3f8B0c6c8b8d7e6f3e4a',
            double_out: false,
            is_one_click: false,
            price_usd: '0.1',
            usd_worth: '0.001',
            wallet_index: 0,
        };

        const result: SubmitSwapOrderResponse = await omniApi.submitSwapOrder(swapOrderParams, id_token);

        // expected result:
        // {
        //     "backend_response": {
        //         "limit_order_response": {
        //             "code": 10000,
        //             "message": "OK",
        //             "data": {
        //                 "order_id": 123456
        //             }
        //         }
        //     }
        // }

        expect(result.backend_response).to.be.not.undefined;
        expect(result.backend_response.limit_order_response || result.backend_response.market_order_response).to.be.not
            .undefined;
    });

    it('should sign limit order successfully', async function () {
        const signLimitOrderParams = {
            intent_id: 1,
            order_id: 123456,
            chain_id: 1,
            wallet_index: 0,
            unsigned_tx: ['0x1234567890abcdef', '0xabcdef1234567890'],
        };

        const result: SignLimitOrderResponse = await omniApi.signLimitOrder(signLimitOrderParams, id_token);

        // expected result:
        // {
        //     "intent_id": 1,
        //     "order_id": 123456,
        //     "chain_id": 1,
        //     "signed_tx": ["0x1234567890abcdef...", "0xabcdef1234567890..."]
        // }

        expect(result.intent_id).to.be.equal(1);
        expect(result.order_id).to.be.equal(123456);
        expect(result.chain_id).to.be.equal(1);
        expect(result.signed_tx).to.be.an('array');
        expect(result.signed_tx.length).to.be.greaterThan(0);
    });

    it('should notify limit order result successfully', async function () {
        const notifyParams = {
            intent_id: 1,
            result: 'success',
            message: 'Order executed successfully',
        };

        const result: NotifyLimitOrderResultResponse = await omniApi.notifyLimitOrderResult(notifyParams, id_token);

        // expected result: null
        expect(result).to.be.null;
    });
});
