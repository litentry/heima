import { sha256 } from "js-sha256";
import {
	encodeFunctionData,
	encodePacked,
	encodeAbiParameters,
	keccak256,
	pad,
	toHex,
	type Address,
	type Hash,
} from "viem";
import { CONTRACTS, DEFAULT_CLIENT_ID } from "./constants";

/**
 * Based on the OmniAccount.sol _determineOa function:
 * bytes memory oaType = bytes("evm");
 * return sha256(abi.encodePacked(clientId, oaType, sender));
 */
export function calculateOmniAccount(
	address: Address | string,
	clientId: string = DEFAULT_CLIENT_ID,
	identityType: "evm" | "solana" = "evm",
): `0x${string}` {
	// Create input array for hashing - order is important: clientId, oaType, sender
	const inputs: Uint8Array[] = [];

	// First: clientId as raw bytes (not padded to 32)
	const clientIdBytes = new TextEncoder().encode(clientId);
	inputs.push(clientIdBytes);

	// Second: identity type ("evm" or "solana")
	inputs.push(new TextEncoder().encode(identityType));

	// Third: address
	if (identityType === "evm") {
		// For EVM addresses, remove 0x prefix and convert from hex
		const addressHex = address.slice(2).toLowerCase();
		const addressBytes = new Uint8Array(20); // EVM addresses are 20 bytes
		for (let i = 0; i < addressHex.length; i += 2) {
			addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
		}
		inputs.push(addressBytes);
	} else {
		// For Solana, handle as appropriate
		// This is a simplified version - real implementation would handle base58 for Solana
		// TODO
		inputs.push(new TextEncoder().encode(address));
	}

	const totalLength = inputs.reduce((sum, arr) => sum + arr.length, 0);
	const combined = new Uint8Array(totalLength);
	let offset = 0;
	for (const input of inputs) {
		combined.set(input, offset);
		offset += input.length;
	}

	// Calculate SHA256 hash
	const hash = sha256.array(combined);

	// Convert to hex string
	const result = `0x${hash.map((b) => b.toString(16).padStart(2, "0")).join("")}`;

	console.log("OmniAccount calculation:", {
		address,
		clientId,
		identityType,
		clientIdBytes: Array.from(inputs[0])
			.map((b) => b.toString(16).padStart(2, "0"))
			.join(""),
		identityTypeBytes: Array.from(inputs[1])
			.map((b) => b.toString(16).padStart(2, "0"))
			.join(""),
		addressBytes: Array.from(inputs[2])
			.map((b) => b.toString(16).padStart(2, "0"))
			.join(""),
		combined: Array.from(combined)
			.map((b) => b.toString(16).padStart(2, "0"))
			.join(""),
		result,
	});

	return result as `0x${string}`;
}

/**
 * Convert a string to bytes32 format for contract calls
 */
export function stringToBytes32(str: string): `0x${string}` {
	const encoder = new TextEncoder();
	const bytes = encoder.encode(str);
	const padded = new Uint8Array(32);
	padded.set(bytes.slice(0, 32)); // Take first 32 bytes if longer
	return `0x${Array.from(padded)
		.map((b) => b.toString(16).padStart(2, "0"))
		.join("")}`;
}

/**
 * Convert a string to bytes format for contract calls (variable length)
 */
export function stringToBytes(str: string): `0x${string}` {
	const encoder = new TextEncoder();
	const bytes = encoder.encode(str);
	return `0x${Array.from(bytes)
		.map((b) => b.toString(16).padStart(2, "0"))
		.join("")}`;
}

/**
 * Encode the getAddress function call for the OmniAccountFactory
 * Note: This should be used with a contract read call to get the actual address
 */
export function encodeGetAddress(
	omniAccount: `0x${string}`,
	clientId: `0x${string}`,
	rootSigner: Address,
): `0x${string}` {
	return encodeFunctionData({
		abi: CONTRACTS.OmniAccountFactory.abi,
		functionName: "getAddress",
		args: [omniAccount, clientId, rootSigner],
	});
}

/**
 * Encode initialization data for Omni Account
 */
export function encodeOmniAccountInitData(
	omniAccount: `0x${string}`,
	clientId: `0x${string}`,
	rootSigner: Address,
): `0x${string}` {
	return encodeFunctionData({
		abi: CONTRACTS.OmniAccountFactory.abi,
		functionName: "createAccount",
		args: [omniAccount, clientId, rootSigner],
	});
}

/**
 * Build a UserOperation structure for ERC-4337
 */
export interface UserOperation {
	sender: Address;
	nonce: bigint;
	initCode: `0x${string}`;
	callData: `0x${string}`;
	callGasLimit: bigint;
	verificationGasLimit: bigint;
	preVerificationGas: bigint;
	maxFeePerGas: bigint;
	maxPriorityFeePerGas: bigint;
	paymasterAndData: `0x${string}`;
	signature: `0x${string}`;
}

/**
 * PackedUserOperation structure for EntryPoint v0.7
 * Includes packed gas fields and session-related fields for future extensibility
 * Session fields are currently set to default values (zero address/empty proof)
 */
export interface PackedUserOperation {
	sender: Address;
	nonce: bigint;
	initCode: `0x${string}`;
	callData: `0x${string}`;
	accountGasLimits: `0x${string}`; // packed callGasLimit and verificationGasLimit
	preVerificationGas: bigint;
	gasFees: `0x${string}`; // packed maxPriorityFeePerGas and maxFeePerGas
	paymasterAndData: `0x${string}`;
	sessionAccount: Address;
	sessionExpiration: bigint;
	sessionAccountProof: `0x${string}`;
	signature: `0x${string}`;
}

/**
 * Create a basic UserOperation
 */
export function createUserOperation(params: {
	sender: Address;
	nonce?: bigint;
	callData?: `0x${string}`;
	initCode?: `0x${string}`;
}): UserOperation {
	return {
		sender: params.sender,
		nonce: params.nonce || BigInt(0),
		initCode: params.initCode || "0x",
		callData: params.callData || "0x",
		callGasLimit: BigInt(2000000), // Increased for deployment
		verificationGasLimit: BigInt(3000000), // Increased for deployment
		preVerificationGas: BigInt(100000),
		maxFeePerGas: BigInt(20000000000), // 20 gwei
		maxPriorityFeePerGas: BigInt(1000000000), // 1 gwei
		paymasterAndData: "0x",
		signature: "0x",
	};
}

/**
 * Get the hash of a UserOperation for signing
 * This follows the ERC-4337 specification for PackedUserOperation hash calculation
 * Uses EIP-712 typed data hashing with EntryPoint v0.7 domain separator
 * Includes support for session fields (sessionAccount, sessionExpiration, sessionAccountProof)
 */
export function getUserOpHash(
	userOp: UserOperation,
	entryPointAddress: Address,
	chainId: number,
): Hash {
	// First, convert to PackedUserOperation format
	const packedOp = packUserOperation(userOp);
	
	// Type hash for PackedUserOperation
	const PACKED_USEROP_TYPEHASH = keccak256(
		toHex("PackedUserOperation(address sender,uint256 nonce,bytes initCode,bytes callData,bytes32 accountGasLimits,uint256 preVerificationGas,bytes32 gasFees,bytes paymasterAndData,address sessionAccount,uint256 sessionExpiration,bytes sessionAccountProof)")
	);
	
	// Encode according to the contract's UserOperationLib.encode
	// Note: The contract uses abi.encode, not encodePacked
	const encoded = encodeAbiParameters(
		[
			{ name: 'typehash', type: 'bytes32' },
			{ name: 'sender', type: 'address' },
			{ name: 'nonce', type: 'uint256' },
			{ name: 'hashInitCode', type: 'bytes32' },
			{ name: 'hashCallData', type: 'bytes32' },
			{ name: 'accountGasLimits', type: 'bytes32' },
			{ name: 'preVerificationGas', type: 'uint256' },
			{ name: 'gasFees', type: 'bytes32' },
			{ name: 'hashPaymasterAndData', type: 'bytes32' },
			{ name: 'sessionAccount', type: 'address' },
			{ name: 'sessionExpiration', type: 'uint256' },
			{ name: 'hashSessionAccountProof', type: 'bytes32' },
		],
		[
			PACKED_USEROP_TYPEHASH,
			packedOp.sender,
			packedOp.nonce,
			keccak256(packedOp.initCode),
			keccak256(packedOp.callData),
			packedOp.accountGasLimits,
			packedOp.preVerificationGas,
			packedOp.gasFees,
			keccak256(packedOp.paymasterAndData),
			packedOp.sessionAccount,
			packedOp.sessionExpiration,
			keccak256(packedOp.sessionAccountProof),
		],
	);

	const userOpHashInner = keccak256(encoded);
	console.log("UserOp hash inner:", userOpHashInner);

	// EIP-712 domain separator
	const domainSeparator = keccak256(
		encodeAbiParameters(
			[
				{ name: 'typeHash', type: 'bytes32' },
				{ name: 'name', type: 'bytes32' },
				{ name: 'version', type: 'bytes32' },
				{ name: 'chainId', type: 'uint256' },
				{ name: 'verifyingContract', type: 'address' },
			],
			[
				keccak256(toHex('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)')),
				keccak256(toHex('ERC4337')),
				keccak256(toHex('1')),
				BigInt(chainId),
				entryPointAddress,
			]
		)
	);

	// Final hash using EIP-712 format
	const finalHash = keccak256(
		encodePacked(
			['bytes1', 'bytes1', 'bytes32', 'bytes32'],
			['0x19', '0x01', domainSeparator, userOpHashInner]
		)
	);
	console.log("UserOp hash final:", finalHash);

	return finalHash;
}

/**
 * Generate initCode for deploying an Omni Account
 * The initCode contains factory address + createAccount calldata
 * During deployment, OmniAccount extracts these params to validate the deployer's signature
 * This enables the AA24 signature fix where deployment can be authorized by either:
 * - The account whose omniAccount matches the expected OA
 * - The designated root signer
 */
export function generateInitCode(
	factoryAddress: Address,
	omniAccount: `0x${string}`,
	clientId: `0x${string}`,
	rootSigner: Address,
): `0x${string}` {
	// For v0.7, we need to use a different approach
	// The EntryPoint will handle the deployment through SenderCreator
	// For now, return the factory data that will be used by SenderCreator

	// Encode the factory address and init data together
	const initCalldata = encodeFunctionData({
		abi: CONTRACTS.OmniAccountFactory.abi,
		functionName: "createAccount",
		args: [omniAccount, clientId, rootSigner],
	});

	// Combine factory address and calldata
	return `${factoryAddress}${initCalldata.slice(2)}` as `0x${string}`;
}

/**
 * Pack gas limits into a single bytes32 value
 * Upper 128 bits: verificationGasLimit, Lower 128 bits: callGasLimit
 */
export function packAccountGasLimits(
	callGasLimit: bigint,
	verificationGasLimit: bigint,
): `0x${string}` {
	const packed = (verificationGasLimit << BigInt(128)) | callGasLimit;
	return pad(toHex(packed), { size: 32 });
}

/**
 * Pack gas fees into a single bytes32 value
 * Upper 128 bits: maxPriorityFeePerGas, Lower 128 bits: maxFeePerGas
 */
export function packGasFees(
	maxFeePerGas: bigint,
	maxPriorityFeePerGas: bigint,
): `0x${string}` {
	const packed = (maxPriorityFeePerGas << BigInt(128)) | maxFeePerGas;
	return pad(toHex(packed), { size: 32 });
}

/**
 * Convert UserOperation to PackedUserOperation for EntryPoint v0.7
 * Session fields (sessionAccount, sessionExpiration, sessionAccountProof) are set to default values
 * as they're not yet implemented but required for the PackedUserOperation structure
 */
export function packUserOperation(userOp: UserOperation): PackedUserOperation {
	const packed = {
		sender: userOp.sender,
		nonce: userOp.nonce,
		initCode: userOp.initCode,
		callData: userOp.callData,
		accountGasLimits: packAccountGasLimits(
			userOp.callGasLimit,
			userOp.verificationGasLimit,
		),
		preVerificationGas: userOp.preVerificationGas,
		gasFees: packGasFees(userOp.maxFeePerGas, userOp.maxPriorityFeePerGas),
		paymasterAndData: userOp.paymasterAndData,
		sessionAccount: "0x0000000000000000000000000000000000000000" as Address, // No session account for now
		sessionExpiration: BigInt(0), // No session expiration
		sessionAccountProof: "0x" as `0x${string}`, // Empty proof
		signature: userOp.signature,
	};

	// Log each field to debug
	console.log("Packing UserOp - individual fields:", {
		sender: packed.sender,
		nonce: packed.nonce,
		initCode: packed.initCode,
		callData: packed.callData,
		accountGasLimits: packed.accountGasLimits,
		preVerificationGas: packed.preVerificationGas,
		gasFees: packed.gasFees,
		paymasterAndData: packed.paymasterAndData,
		sessionAccount: packed.sessionAccount,
		sessionExpiration: packed.sessionExpiration,
		sessionAccountProof: packed.sessionAccountProof,
		signature: packed.signature,
	});

	return packed;
}

/**
 * Sign a UserOperation using EIP-712 typed data signing
 */
export async function signUserOperation(
	walletClient: any,
	address: Address,
	userOp: UserOperation,
	entryPointAddress: Address,
	chainId: bigint,
): Promise<`0x${string}`> {
	// Convert UserOperation to PackedUserOperation for signing
	const packedOp = packUserOperation(userOp);
	
	// EIP-712 domain
	const domain = {
		name: 'ERC4337',
		version: '1', 
		chainId: Number(chainId),
		verifyingContract: entryPointAddress,
	};

	// EIP-712 types for PackedUserOperation
	const types = {
		PackedUserOperation: [
			{ name: 'sender', type: 'address' },
			{ name: 'nonce', type: 'uint256' },
			{ name: 'initCode', type: 'bytes' },
			{ name: 'callData', type: 'bytes' },
			{ name: 'accountGasLimits', type: 'bytes32' },
			{ name: 'preVerificationGas', type: 'uint256' },
			{ name: 'gasFees', type: 'bytes32' },
			{ name: 'paymasterAndData', type: 'bytes' },
			{ name: 'sessionAccount', type: 'address' },
			{ name: 'sessionExpiration', type: 'uint256' },
			{ name: 'sessionAccountProof', type: 'bytes' },
		],
	};

	// Message to sign (without signature field)
	const message = {
		sender: packedOp.sender,
		nonce: packedOp.nonce,
		initCode: packedOp.initCode,
		callData: packedOp.callData,
		accountGasLimits: packedOp.accountGasLimits,
		preVerificationGas: packedOp.preVerificationGas,
		gasFees: packedOp.gasFees,
		paymasterAndData: packedOp.paymasterAndData,
		sessionAccount: packedOp.sessionAccount,
		sessionExpiration: packedOp.sessionExpiration,
		sessionAccountProof: packedOp.sessionAccountProof,
	};

	try {
		console.log("Signing PackedUserOperation with EIP-712...");
		const signature = await walletClient.signTypedData({
			account: address,
			domain,
			types,
			primaryType: 'PackedUserOperation',
			message,
		});
		console.log("Successfully signed with EIP-712");
		return signature;
	} catch (e) {
		console.error("EIP-712 signing failed:", e);
		
		// Fallback: Use personal_sign (adds message prefix)
		const userOpHash = getUserOpHash(userOp, entryPointAddress, Number(chainId));
		const signature = await walletClient.signMessage({
			account: address,
			message: { raw: userOpHash },
		});
		
		console.warn("WARNING: Using personal_sign which adds message prefix");
		return signature;
	}
}

