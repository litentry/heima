import { describe, it } from 'mocha';
import { expect } from 'chai';
import { signMessage } from 'viem/accounts';
import {
    ClientId,
    omniApi,
    GetHyperliquidSignatureDataResponse,
    GetWeb3SignInMessageResponse,
    UserLoginResponse,
    calculateOmniAccount,
    randomEvmWallet,
} from './utils';

const evmWallet = randomEvmWallet();

describe('Hyperliquid Signature Data Tests', function () {
    this.timeout(100000);

    function validateEIP712Signature(signature: string) {
        expect(signature).to.be.a('string');
        expect(signature).to.match(/^0x[a-fA-F0-9]{130}$/);

        const r = signature.slice(2, 66);
        const s = signature.slice(66, 130);
        const v = signature.slice(130, 132);

        expect(r).to.match(/^[a-fA-F0-9]{64}$/);
        expect(s).to.match(/^[a-fA-F0-9]{64}$/);
        expect(v).to.match(/^[a-fA-F0-9]{2}$/);

        const vValue = parseInt(v, 16);
        expect([27, 28, 0, 1].includes(vValue) || [0x1b, 0x1c].includes(vValue)).to.be.true;
    }

    function validateBasicResponse(result: GetHyperliquidSignatureDataResponse) {
        expect(result).to.have.property('main_address');
        expect(result).to.have.property('hyperliquid_signature_data');
        expect(result.main_address).to.be.a('string');
        expect(result.main_address).to.match(/^0x[a-fA-F0-9]{40}$/);

        const data = result.hyperliquid_signature_data;
        expect(data).to.have.property('action');
        expect(data).to.have.property('nonce');
        expect(data).to.have.property('signature');
        expect(data.nonce).to.be.a('number');
        expect(data.nonce).to.be.greaterThan(0);
        expect(data.nonce).to.be.lessThan(Date.now() + 60000);

        validateEIP712Signature(data.signature);
    }

    function validateHyperliquidChain(action: any, chainId: number) {
        const isTestnet = [421614, 998, 1337, 31337].includes(chainId);
        expect(action.hyperliquidChain).to.equal(isTestnet ? 'Testnet' : 'Mainnet');
    }

    function validateActionBase(action: any, chainId: number) {
        expect(action).to.have.property('signatureChainId');
        expect(action.signatureChainId).to.equal('0x' + chainId.toString(16));
        expect(action).to.have.property('hyperliquidChain');
        validateHyperliquidChain(action, chainId);
    }

    describe('Parameter Validation and Authentication Failures', function () {
        it('should fail with EVM user_id type', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'evm',
                        value: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                        agent_name: 'Test Trading Bot',
                    },
                    chain_id: 42161,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected method to throw an error for EVM user_id');
            } catch (error) {
                console.log('Expected EVM user_id error:', error);
            }
        });

        it('should fail when neither user_auth nor client_auth is provided', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                        agent_name: 'Test Bot',
                    },
                    chain_id: 42161,
                };

                await omniApi.getHyperliquidSignatureData(params as any);
                expect.fail('Expected method to throw an error');
            } catch (error) {
                console.log('Expected missing auth failure:', error);
            }
        });

        it('should fail with user_auth due to authentication verification', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                        agent_name: 'Test Trading Bot',
                    },
                    chain_id: 42161,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected authentication to fail in mock environment');
            } catch (error) {
                console.log('Expected user authentication failure:', error);
            }
        });

        it('should fail with WildMeta client_auth due to verification failure', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    client_id: ClientId.Wildmeta,
                    client_auth: {
                        type: 'wildmeta_hl',
                        value: {
                            agent_address: '0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd',
                            business_json: JSON.stringify({
                                action: 'approve_agent',
                                agent_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                                timestamp: Date.now(),
                            }),
                            main_address: '0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e',
                            signature:
                                '0x46c737250d61b60cbf0f46a6755e59815844a2f7cdb9dc16bf867b57bfed3526424343a237c15eef9089d571d1f60fd0bd7f91d5888c649216a7df147b386a681c',
                            login_type: 0,
                        },
                    },
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                        agent_name: 'WildMeta Trading Bot',
                    },
                    chain_id: 42161,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected WildMeta authentication to fail in mock environment');
            } catch (error) {
                console.log('Expected WildMeta authentication failure:', error);
            }
        });

        it('should fail with invalid client_auth type', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    client_id: ClientId.Wildmeta,
                    client_auth: {
                        type: 'invalid_type',
                        value: 'some_value',
                    },
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                        agent_name: 'Test Bot',
                    },
                    chain_id: 42161,
                };

                await omniApi.getHyperliquidSignatureData(params as any);
                expect.fail('Expected method to throw an error for invalid client_auth type');
            } catch (error) {
                console.log('Expected invalid client_auth type error:', error);
            }
        });

        it('should test approve_agent action type with testnet chain', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                        agent_name: 'Test Trading Bot',
                    },
                    chain_id: 11155111,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected authentication to fail but covers approve_agent + testnet branch');
            } catch (error) {
                console.log('Expected failure for approve_agent + testnet:', error);
            }
        });

        it('should test withdraw3 action type with mainnet chain', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'withdraw3' as const,
                        amount: '100.0',
                        destination: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                    },
                    chain_id: 1,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected authentication to fail but covers withdraw3 + mainnet branch');
            } catch (error) {
                console.log('Expected failure for withdraw3 + mainnet:', error);
            }
        });

        it('should test approve_builder_fee action type', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'approve_builder_fee' as const,
                        max_fee_rate: '0.01',
                        builder: '0x742d35Cc6634C0532925a3b844Bc9e7595f02A10',
                    },
                    chain_id: 42161,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected authentication to fail but covers approve_builder_fee branch');
            } catch (error) {
                console.log('Expected failure for approve_builder_fee:', error);
            }
        });

        it('should fail with invalid agent_address format', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: 'invalid_address',
                        agent_name: 'Test Trading Bot',
                    },
                    chain_id: 42161,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected method to throw an error for invalid agent_address');
            } catch (error: any) {
                expect(error).to.have.property('message', 'Authentication verification failed');
            }
        });

        it('should fail with invalid destination address in withdraw3', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'withdraw3' as const,
                        amount: '100.0',
                        destination: '0xinvalid',
                    },
                    chain_id: 1,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected method to throw an error for invalid destination');
            } catch (error: any) {
                expect(error).to.have.property('message', 'Authentication verification failed');
            }
        });

        it('should fail with invalid builder address in approve_builder_fee', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email',
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'email',
                        value: '123456',
                    },
                    client_id: ClientId.Wildmeta,
                    action_type: {
                        type: 'approve_builder_fee' as const,
                        max_fee_rate: '0.01',
                        builder: 'not_an_address',
                    },
                    chain_id: 11155111,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected method to throw an error for invalid builder address');
            } catch (error: any) {
                expect(error).to.have.property('message', 'Authentication verification failed');
            }
        });
    });

    describe('Signature Data Generation with Valid Authentication', function () {
        let id_token: string;
        const omniAccount = calculateOmniAccount(evmWallet.address);

        before(async function () {
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

            id_token = result.id_token;
        });

        it('should successfully generate signature data for approve_agent action on testnet', async function () {
            const params = {
                user_id: {
                    type: 'email' as const,
                    value: 'test@example.com',
                },
                user_auth: {
                    type: 'auth_token' as const,
                    value: id_token,
                },
                client_id: ClientId.Wildmeta,
                client_auth: {
                    type: 'wildmeta_hl' as const,
                    value: {
                        agent_address: '0x1234567890123456789012345678901234567890',
                        business_json: JSON.stringify({ test: 'data' }),
                        main_address: evmWallet.address,
                        signature: 'test_signature',
                        login_type: 1,
                    },
                },
                action_type: {
                    type: 'approve_agent' as const,
                    agent_address: '0x1234567890123456789012345678901234567890',
                    agent_name: 'Test Agent',
                },
                chain_id: 421614,
            };

            const result: GetHyperliquidSignatureDataResponse = await omniApi.getHyperliquidSignatureData(params);

            validateBasicResponse(result);
            expect(result.hyperliquid_signature_data.action).to.have.property('type', 'approve_agent');

            const action = result.hyperliquid_signature_data.action;
            validateActionBase(action, params.chain_id);
            expect(action).to.have.property('agentAddress', params.action_type.agent_address);
            expect(action).to.have.property('agentName', params.action_type.agent_name);
            expect(action).to.have.property('nonce');
            expect(action.nonce).to.be.a('number');
            expect(action.nonce).to.be.greaterThan(0);
        });

        it('should successfully generate signature data for approve_agent action on mainnet', async function () {
            const params = {
                user_id: {
                    type: 'email' as const,
                    value: 'test@example.com',
                },
                user_auth: {
                    type: 'auth_token' as const,
                    value: id_token,
                },
                client_id: ClientId.Wildmeta,
                client_auth: {
                    type: 'wildmeta_hl' as const,
                    value: {
                        agent_address: '0x1234567890123456789012345678901234567890',
                        business_json: JSON.stringify({ test: 'data' }),
                        main_address: evmWallet.address,
                        signature: 'test_signature',
                        login_type: 1,
                    },
                },
                action_type: {
                    type: 'approve_agent' as const,
                    agent_address: '0x1234567890123456789012345678901234567890',
                    agent_name: 'Test Agent',
                },
                chain_id: 42161,
            };

            const result: GetHyperliquidSignatureDataResponse = await omniApi.getHyperliquidSignatureData(params);

            validateBasicResponse(result);
            expect(result.hyperliquid_signature_data.action).to.have.property('type', 'approve_agent');
        });

        it('should successfully generate signature data for withdraw3 action on testnet', async function () {
            const params = {
                user_id: {
                    type: 'email' as const,
                    value: 'test@example.com',
                },
                user_auth: {
                    type: 'auth_token' as const,
                    value: id_token,
                },
                client_id: ClientId.Wildmeta,
                client_auth: {
                    type: 'wildmeta_hl' as const,
                    value: {
                        agent_address: '0x1234567890123456789012345678901234567890',
                        business_json: JSON.stringify({ test: 'data' }),
                        main_address: evmWallet.address,
                        signature: 'test_signature',
                        login_type: 1,
                    },
                },
                action_type: {
                    type: 'withdraw3' as const,
                    destination: '0x1234567890123456789012345678901234567890',
                    amount: '1000000',
                },
                chain_id: 421614,
            };

            const result: GetHyperliquidSignatureDataResponse = await omniApi.getHyperliquidSignatureData(params);

            validateBasicResponse(result);
            expect(result.hyperliquid_signature_data.action).to.have.property('type', 'withdraw3');

            const action = result.hyperliquid_signature_data.action;
            validateActionBase(action, params.chain_id);
            expect(action).to.have.property('amount', params.action_type.amount);
            expect(action).to.have.property('destination', params.action_type.destination);
            expect(action).to.have.property('time');
            expect(action.time).to.be.a('number');
        });

        it('should successfully generate signature data for withdraw3 action on mainnet', async function () {
            const params = {
                user_id: {
                    type: 'email' as const,
                    value: 'test@example.com',
                },
                user_auth: {
                    type: 'auth_token' as const,
                    value: id_token,
                },
                client_id: ClientId.Wildmeta,
                client_auth: {
                    type: 'wildmeta_hl' as const,
                    value: {
                        agent_address: '0x1234567890123456789012345678901234567890',
                        business_json: JSON.stringify({ test: 'data' }),
                        main_address: evmWallet.address,
                        signature: 'test_signature',
                        login_type: 1,
                    },
                },
                action_type: {
                    type: 'withdraw3' as const,
                    destination: '0x1234567890123456789012345678901234567890',
                    amount: '1000000',
                },
                chain_id: 42161,
            };

            const result: GetHyperliquidSignatureDataResponse = await omniApi.getHyperliquidSignatureData(params);

            validateBasicResponse(result);
            expect(result.hyperliquid_signature_data.action).to.have.property('type', 'withdraw3');

            const action = result.hyperliquid_signature_data.action;
            validateActionBase(action, params.chain_id);
            expect(action).to.have.property('amount', params.action_type.amount);
            expect(action).to.have.property('destination', params.action_type.destination);
            expect(action).to.have.property('time');
            expect(action.time).to.be.a('number');
        });

        it('should successfully generate signature data for approve_builder_fee action on testnet', async function () {
            const params = {
                user_id: {
                    type: 'email' as const,
                    value: 'test@example.com',
                },
                user_auth: {
                    type: 'auth_token' as const,
                    value: id_token,
                },
                client_id: ClientId.Wildmeta,
                client_auth: {
                    type: 'wildmeta_hl' as const,
                    value: {
                        agent_address: '0x1234567890123456789012345678901234567890',
                        business_json: JSON.stringify({ test: 'data' }),
                        main_address: evmWallet.address,
                        signature: 'test_signature',
                        login_type: 1,
                    },
                },
                action_type: {
                    type: 'approve_builder_fee' as const,
                    max_fee_rate: '100',
                    builder: '0x1234567890123456789012345678901234567890',
                },
                chain_id: 421614,
            };

            const result: GetHyperliquidSignatureDataResponse = await omniApi.getHyperliquidSignatureData(params);

            validateBasicResponse(result);
            expect(result.hyperliquid_signature_data.action).to.have.property('type', 'approve_builder_fee');

            const action = result.hyperliquid_signature_data.action;
            validateActionBase(action, params.chain_id);
            expect(action).to.have.property('maxFeeRate', params.action_type.max_fee_rate);
            expect(action).to.have.property('builder', params.action_type.builder);
            expect(action).to.have.property('nonce');
            expect(action.nonce).to.be.a('number');
            expect(action.nonce).to.be.greaterThan(0);
        });

        it('should successfully generate signature data for approve_builder_fee action on mainnet', async function () {
            const params = {
                user_id: {
                    type: 'email' as const,
                    value: 'test@example.com',
                },
                user_auth: {
                    type: 'auth_token' as const,
                    value: id_token,
                },
                client_id: ClientId.Wildmeta,
                client_auth: {
                    type: 'wildmeta_hl' as const,
                    value: {
                        agent_address: '0x1234567890123456789012345678901234567890',
                        business_json: JSON.stringify({ test: 'data' }),
                        main_address: evmWallet.address,
                        signature: 'test_signature',
                        login_type: 1,
                    },
                },
                action_type: {
                    type: 'approve_builder_fee' as const,
                    max_fee_rate: '100',
                    builder: '0x1234567890123456789012345678901234567890',
                },
                chain_id: 42161, // Arbitrum mainnet
            };

            const result: GetHyperliquidSignatureDataResponse = await omniApi.getHyperliquidSignatureData(params);

            validateBasicResponse(result);
            expect(result.hyperliquid_signature_data.action).to.have.property('type', 'approve_builder_fee');

            const action = result.hyperliquid_signature_data.action;
            validateActionBase(action, params.chain_id);
            expect(action).to.have.property('maxFeeRate', params.action_type.max_fee_rate);
            expect(action).to.have.property('builder', params.action_type.builder);
            expect(action).to.have.property('nonce');
            expect(action.nonce).to.be.a('number');
            expect(action.nonce).to.be.greaterThan(0);
        });

        it('should fail with invalid agent_address format (with valid auth)', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email' as const,
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'auth_token' as const,
                        value: id_token,
                    },
                    client_id: ClientId.Wildmeta,
                    client_auth: {
                        type: 'wildmeta_hl' as const,
                        value: {
                            agent_address: '0x1234567890123456789012345678901234567890',
                            business_json: JSON.stringify({ test: 'data' }),
                            main_address: evmWallet.address,
                            signature: 'test_signature',
                            login_type: 1,
                        },
                    },
                    action_type: {
                        type: 'approve_agent' as const,
                        agent_address: 'invalid_address',
                        agent_name: 'Test Agent',
                    },
                    chain_id: 421614,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected method to throw an error for invalid agent_address');
            } catch (error: any) {
                expect(error).to.have.property('message', 'Invalid params');
            }
        });

        it('should fail with invalid destination address in withdraw3 (with valid auth)', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email' as const,
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'auth_token' as const,
                        value: id_token,
                    },
                    client_id: ClientId.Wildmeta,
                    client_auth: {
                        type: 'wildmeta_hl' as const,
                        value: {
                            agent_address: '0x1234567890123456789012345678901234567890',
                            business_json: JSON.stringify({ test: 'data' }),
                            main_address: evmWallet.address,
                            signature: 'test_signature',
                            login_type: 1,
                        },
                    },
                    action_type: {
                        type: 'withdraw3' as const,
                        amount: '100.0',
                        destination: '0xinvalid',
                    },
                    chain_id: 1,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected method to throw an error for invalid destination');
            } catch (error: any) {
                expect(error).to.have.property('message', 'Invalid params');
            }
        });

        it('should fail with invalid builder address in approve_builder_fee (with valid auth)', async function () {
            try {
                const params = {
                    user_id: {
                        type: 'email' as const,
                        value: 'test@example.com',
                    },
                    user_auth: {
                        type: 'auth_token' as const,
                        value: id_token,
                    },
                    client_id: ClientId.Wildmeta,
                    client_auth: {
                        type: 'wildmeta_hl' as const,
                        value: {
                            agent_address: '0x1234567890123456789012345678901234567890',
                            business_json: JSON.stringify({ test: 'data' }),
                            main_address: evmWallet.address,
                            signature: 'test_signature',
                            login_type: 1,
                        },
                    },
                    action_type: {
                        type: 'approve_builder_fee' as const,
                        max_fee_rate: '0.01',
                        builder: 'not_an_address',
                    },
                    chain_id: 11155111,
                };

                await omniApi.getHyperliquidSignatureData(params);
                expect.fail('Expected method to throw an error for invalid builder address');
            } catch (error: any) {
                expect(error).to.have.property('message', 'Invalid params');
            }
        });
    });
});
