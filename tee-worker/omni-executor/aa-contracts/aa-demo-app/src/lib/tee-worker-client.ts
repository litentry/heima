import { type WalletClient } from "viem";
import { TEE_WORKER_CONFIG } from "./constants";

interface Web3SignInMessageResponse {
    message_code: string;
    omni_account: string;
    client_id: string;
}

interface UserLoginParams {
    user_id: {
        type: "evm" | "substrate" | "solana" | "bitcoin" | "email" | "twitter" | "discord" | "github" | "google";
        value: string;
    };
    user_auth: {
        type: "evm" | "substrate" | "solana" | "bitcoin" | "email" | "auth_token" | "oauth2";
        value: string | any;
    };
    client_id: string;
    client_auth?: any;
}

interface UserLoginResponse {
    access_token: string;
    id_token: string;
    backend_response: any;
}

interface GetSmartWalletRootSignerParams {
    omni_account: string;
    chain_type: "evm" | "solana" | "tron";
    wallet_index: number;
}

// JSON-RPC request helper
async function makeRpcRequest<T>(
    method: string,
    params: any,
    authToken?: string
): Promise<T> {
    const headers: HeadersInit = {
        "Content-Type": "application/json",
    };

    if (authToken) {
        headers["Authorization"] = `Bearer ${authToken}`;
    }

    // The TEE worker RPC server expects params as a direct object (not wrapped in array)
    // If params is already an array (for methods like omni_getWeb3SignInMessage), it's the actual params
    // If params is an object or null/undefined, use it directly
    const rpcParams = params ?? null;

    const requestBody = {
        jsonrpc: "2.0",
        method,
        params: rpcParams,
        id: Date.now(),
    };

    console.log(`[TEE Worker RPC] Request to ${method}:`, requestBody);

    const response = await fetch(TEE_WORKER_CONFIG.rpcUrl, {
        method: "POST",
        headers,
        body: JSON.stringify(requestBody),
    });

    const responseText = await response.text();
    console.log(`[TEE Worker RPC] Response from ${method}:`, responseText);

    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}, body: ${responseText}`);
    }

    let data;
    try {
        data = JSON.parse(responseText);
    } catch (e) {
        throw new Error(`Failed to parse response: ${responseText}`);
    }

    if (data.error) {
        console.error(`[TEE Worker RPC] Error from ${method}:`, data.error);
        // Include error code and data in the error message for better debugging
        const errorData = typeof data.error.data === 'object' 
            ? JSON.stringify(data.error.data)
            : data.error.data;
        const errorMessage = data.error.data
            ? `${data.error.message} (${errorData})`
            : data.error.message || "RPC error";
        throw new Error(errorMessage);
    }

    return data.result;
}

// Get the message to sign for Web3 authentication
export async function getWeb3SignInMessage(
    clientId: string,
    omniAccount: string
): Promise<Web3SignInMessageResponse> {
    // The omniAccount should already be a properly formatted 32-byte hex string (64 chars) with 0x prefix
    // from calculateOmniAccount function
    console.log('[TEE Worker] Using omni account:', omniAccount);

    // Pass parameters as an array for positional arguments
    return makeRpcRequest<Web3SignInMessageResponse>(
        "omni_getWeb3SignInMessage",
        [clientId, omniAccount]
    );
}

// Authenticate with the TEE worker using EVM signature
export async function loginWithEvm(
    walletClient: WalletClient,
    evmAddress: string,
    clientId: string,
    messagePayload: Web3SignInMessageResponse
): Promise<UserLoginResponse> {
    // Sign the message with the wallet
    // The server expects the message to be a JSON string of the payload
    const message = JSON.stringify(messagePayload);
    console.log("[TEE Worker] Message to sign:", message);

    const signature = await walletClient.signMessage({
        account: evmAddress as `0x${string}`,
        message,
    });

    // Prepare the login parameters with tagged enum format
    // Note: The enum types must use snake_case ("evm" not "Evm")
    const params: UserLoginParams = {
        user_id: {
            type: "evm",
            value: evmAddress
        },
        user_auth: {
            type: "evm",
            value: signature
        },
        client_id: clientId,
        client_auth: null,
    };

    console.log("[TEE Worker] UserLogin params:", JSON.stringify(params, null, 2));

    // Pass the params object directly
    return makeRpcRequest<UserLoginResponse>("omni_userLogin", params);
}

// Get the TEE worker's smart wallet root signer address
export async function getSmartWalletRootSigner(
    omniAccount: string,
    chainType: "evm" | "solana" | "tron" = TEE_WORKER_CONFIG.chainType,
    index: number = TEE_WORKER_CONFIG.signerIndex
): Promise<string> {
    const params: GetSmartWalletRootSignerParams = {
        omni_account: omniAccount,
        chain_type: chainType,
        wallet_index: index,
    };

    return makeRpcRequest<string>("omni_getSmartWalletRootSigner", params);
}

// Simplified flow to get TEE worker address without authentication
export async function getTEEWorkerAddress(
    omniAccount: string
): Promise<string> {
    console.log("[TEE Worker] Getting TEE worker address", {
        omniAccount,
        rpcUrl: TEE_WORKER_CONFIG.rpcUrl
    });

    try {
        // Directly get the TEE worker's address without authentication
        console.log("[TEE Worker] Getting smart wallet root signer");
        const workerAddress = await getSmartWalletRootSigner(omniAccount);
        console.log("[TEE Worker] Received worker address:", workerAddress);

        return workerAddress;
    } catch (error) {
        console.error("[TEE Worker] Failed to get worker address:", error);
        throw error;
    }
}

// SerializablePackedUserOperation interface matching the Rust struct
interface SerializablePackedUserOperation {
    sender: string;             // Address as hex string (e.g., "0x1234...")
    nonce: number;              // U256 as u128 integer
    init_code: string;          // Bytes as hex string (e.g., "0xabc...")
    call_data: string;          // Bytes as hex string (e.g., "0xdef...")
    account_gas_limits: string; // FixedBytes<32> as hex string (e.g., "0x123...")
    pre_verification_gas: number; // U256 as u128 integer
    gas_fees: string;           // FixedBytes<32> as hex string (e.g., "0x456...")
    paymaster_and_data: string; // Bytes as hex string (e.g., "0x789...")
    signature?: string;         // Optional signature: None = unsigned, Some("0xabc...") = signed
}

interface SubmitUserOpTestParams {
    user_operations: SerializablePackedUserOperation[];
    chain_id: number;
    wallet_index: number;
    omni_account: string;
    client_id: string;
}

interface SubmitUserOpTestResponse {
    transaction_hash: string | null;
}

// Submit user operations through the TEE worker
export async function submitUserOpTest(
    userOperations: SerializablePackedUserOperation[],
    chainId: number,
    walletIndex: number,
    omniAccount: string,
    clientId: string
): Promise<SubmitUserOpTestResponse> {
    const params: SubmitUserOpTestParams = {
        user_operations: userOperations,
        chain_id: chainId,
        wallet_index: walletIndex,
        omni_account: omniAccount,
        client_id: clientId,
    };

    console.log("[TEE Worker] Submitting user operation test", params);

    return makeRpcRequest<SubmitUserOpTestResponse>("omni_submitUserOpTest", params);
}

// Interfaces for gas estimation
interface EstimateUserOpGasParams {
    user_operation: SerializablePackedUserOperation;
    chain_id: number;
    wallet_index: number;
    omni_account: string;
    client_id: string;
}

interface EstimateUserOpGasResponse {
    callGasLimit: string;
    verificationGasLimit: string;
    preVerificationGas: string;
    paymasterVerificationGasLimit: string;
    paymasterPostOpGasLimit: string;
    maxFeePerGas: string;
    maxPriorityFeePerGas: string;
}

// Get OmniAccount hash from email
export interface GetOmniAccountParams {
    client_id: string;
    user_email: string;
}

export async function getOmniAccount(
    clientId: string,
    email: string
): Promise<string> {
    const params: GetOmniAccountParams = {
        client_id: clientId,
        user_email: email,
    };

    console.log("[TEE Worker] Getting OmniAccount for email:", email);

    return makeRpcRequest<string>("omni_getOmniAccount", params);
}

// Estimate gas for a UserOperation through the TEE worker
export async function estimateUserOpGas(
    userOperation: SerializablePackedUserOperation,
    chainId: number,
    walletIndex: number,
    omniAccount: string,
    clientId: string
): Promise<EstimateUserOpGasResponse> {
    const params: EstimateUserOpGasParams = {
        user_operation: userOperation,
        chain_id: chainId,
        wallet_index: walletIndex,
        omni_account: omniAccount,
        client_id: clientId,
    };

    console.log("[TEE Worker] Estimating gas for user operation", params);

    try {
        const response = await makeRpcRequest<EstimateUserOpGasResponse>(
            "omni_estimateUserOpGas",
            params
        );

        console.log("[TEE Worker] Gas estimation response:", response);
        return response;
    } catch (error) {
        console.error("[TEE Worker] Gas estimation failed:", error);
        throw error;
    }
}

// Interfaces for loan request test
interface RequestLoanTestParams {
    user_operation: SerializablePackedUserOperation;
    chain_id: number;
    wallet_index: number;
    omni_account: string;
    client_id: string;
    collateral_ticker: string;
    collateral_size: string;
    lending_ratio: number;
}

interface RequestLoanTestResponse {
    spot_sell_cloid: string;
    hedge_open_cloid: string;
    usdc_received: string;
    spot_sell_tx_hash: string | null;
    hedge_open_tx_hash: string | null;
}

// Request a loan through the TEE worker (test version with UserOperation)
export async function requestLoanTest(
    userOperation: SerializablePackedUserOperation,
    chainId: number,
    walletIndex: number,
    omniAccount: string,
    clientId: string,
    collateralTicker: string,
    collateralSize: string,
    lendingRatio: number
): Promise<RequestLoanTestResponse> {
    const params: RequestLoanTestParams = {
        user_operation: userOperation,
        chain_id: chainId,
        wallet_index: walletIndex,
        omni_account: omniAccount,
        client_id: clientId,
        collateral_ticker: collateralTicker,
        collateral_size: collateralSize,
        lending_ratio: lendingRatio,
    };

    console.log("[TEE Worker] Requesting loan test", params);

    try {
        const response = await makeRpcRequest<RequestLoanTestResponse>(
            "omni_requestLoanTest",
            params
        );

        console.log("[TEE Worker] Loan request test response:", response);
        return response;
    } catch (error) {
        console.error("[TEE Worker] Loan request test failed:", error);
        throw error;
    }
}

// Interfaces for loan query
export interface LoanRecord {
    collateral_ticker: string;
    collateral_size: string;
    usdc_sold: string;
    usdc_loaned: string;
    spot_sell_cloid: string;
    hedge_open_cloid: string;
    position_size: string;
}

interface QueryLoanTestParams {
    omni_account: string;
    nonce: number | null;
}

export interface QueryLoanTestResponse {
    records: Record<string, LoanRecord>;
}

// Query loan records for a user
export async function queryLoanTest(
    omniAccount: string,
    nonce?: number
): Promise<QueryLoanTestResponse> {
    const params: QueryLoanTestParams = {
        omni_account: omniAccount,
        nonce: nonce ?? null,
    };

    console.log("[TEE Worker] Querying loan records", params);

    try {
        const response = await makeRpcRequest<QueryLoanTestResponse>(
            "omni_queryLoanTest",
            params
        );

        console.log("[TEE Worker] Loan query response:", response);
        return response;
    } catch (error) {
        console.error("[TEE Worker] Loan query failed:", error);
        throw error;
    }
}

// Interfaces for loan payback test
interface PaybackLoanTestParams {
    user_operation: SerializablePackedUserOperation;
    chain_id: number;
    wallet_index: number;
    omni_account: string;
    client_id: string;
    loan_nonce: number;
    min_expected_account_value: string;
}

interface PaybackLoanTestResponse {
    collateral_ticker: string;
    collateral_size: string;
    hedge_cancel_tx_hash: string | null;
    hedge_close_cloid: string;
    hedge_close_tx_hash: string | null;
    usd_transfer_tx_hash: string | null;
    spot_buy_cloid: string;
    spot_buy_tx_hash: string | null;
}

// Payback a loan through the TEE worker (test version with UserOperation)
export async function paybackLoanTest(
    userOperation: SerializablePackedUserOperation,
    chainId: number,
    walletIndex: number,
    omniAccount: string,
    clientId: string,
    loanNonce: number,
    minExpectedAccountValue: string
): Promise<PaybackLoanTestResponse> {
    const params: PaybackLoanTestParams = {
        user_operation: userOperation,
        chain_id: chainId,
        wallet_index: walletIndex,
        omni_account: omniAccount,
        client_id: clientId,
        loan_nonce: loanNonce,
        min_expected_account_value: minExpectedAccountValue,
    };

    console.log("[TEE Worker] Paying back loan test", params);

    try {
        const response = await makeRpcRequest<PaybackLoanTestResponse>(
            "omni_paybackLoanTest",
            params
        );

        console.log("[TEE Worker] Loan payback test response:", response);
        return response;
    } catch (error) {
        console.error("[TEE Worker] Loan payback test failed:", error);
        throw error;
    }
}
