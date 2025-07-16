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
	chain_type: "Evm" | "Solana" | "Tron";
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
		// Include error code in the error message for better debugging
		const errorMessage = data.error.data
			? `${data.error.message} (${data.error.data})`
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
	chainType: "Evm" | "Solana" | "Tron" = TEE_WORKER_CONFIG.chainType,
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