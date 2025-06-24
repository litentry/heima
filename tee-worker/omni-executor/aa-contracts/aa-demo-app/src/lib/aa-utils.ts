import { sha256 } from "js-sha256";
import {
	encodeFunctionData,
	encodePacked,
	keccak256,
	pad,
	toHex,
	type Address,
	type Hash,
} from "viem";
import { CONTRACTS, DEFAULT_CLIENT_ID } from "./constants";

/**
 * Calculate the omni account for a given identity and client ID
 * Following the same logic as Identity::to_omni_account in Rust
 * Formula: sha256(identityType + clientId + address)
 *
 * Based on the SmartAccount.sol _determineOa function:
 * bytes memory oaType = bytes("evm");
 * return sha256(abi.encodePacked(oaType, clientId, sender));
 */
export function calculateOmniAccount(
	address: Address | string,
	clientId: string = DEFAULT_CLIENT_ID,
	identityType:
		| "evm"
		| "substrate"
		| "bitcoin"
		| "solana"
		| "twitter"
		| "discord"
		| "github"
		| "email"
		| "google" = "evm",
): `0x${string}` {
	// Create input array for hashing
	const inputs: Uint8Array[] = [];

	// Add identity type FIRST (matching SmartAccount.sol)
	inputs.push(new TextEncoder().encode(identityType));

	// Add client ID (as bytes32)
	const clientIdBytes = stringToBytes32(clientId);
	const clientIdArray = new Uint8Array(32);
	for (let i = 0; i < 32; i++) {
		clientIdArray[i] = parseInt(clientIdBytes.slice(2 + i * 2, 4 + i * 2), 16);
	}
	inputs.push(clientIdArray);

	// Add address/handle based on type
	if (identityType === "evm") {
		// For EVM addresses, remove 0x prefix and convert from hex
		const addressHex = address.slice(2).toLowerCase();
		const addressBytes = new Uint8Array(20); // EVM addresses are 20 bytes
		for (let i = 0; i < addressHex.length; i += 2) {
			addressBytes[i / 2] = parseInt(addressHex.substring(i, i + 2), 16);
		}
		inputs.push(addressBytes);
	} else if (identityType === "solana" || identityType === "substrate") {
		// For Solana/Substrate, handle as appropriate
		// This is a simplified version - real implementation would handle base58 for Solana
		inputs.push(new TextEncoder().encode(address));
	} else {
		// For Web2 identities (twitter, discord, etc.), use the string directly
		inputs.push(new TextEncoder().encode(address));
	}

	// Combine all inputs
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
		identityTypeBytes: Array.from(inputs[0])
			.map((b) => b.toString(16).padStart(2, "0"))
			.join(""),
		clientIdBytes: Array.from(inputs[1])
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
 * Encode the getAddress function call for the SmartAccountFactory
 * Note: This should be used with a contract read call to get the actual address
 */
export function encodeGetAddress(
	omniAccount: `0x${string}`,
	clientId: `0x${string}`,
	rootSigner: Address,
): `0x${string}` {
	return encodeFunctionData({
		abi: CONTRACTS.SmartAccountFactory.abi,
		functionName: "getAddress",
		args: [omniAccount, clientId, rootSigner],
	});
}

/**
 * Encode initialization data for Smart Account
 */
export function encodeSmartAccountInitData(
	omniAccount: `0x${string}`,
	clientId: `0x${string}`,
	rootSigner: Address,
): `0x${string}` {
	return encodeFunctionData({
		abi: CONTRACTS.SmartAccountFactory.abi,
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
}): Partial<UserOperation> {
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
 * This follows the ERC-4337 specification for UserOperation hash calculation
 */
export function getUserOpHash(
	userOp: UserOperation,
	entryPointAddress: Address,
	chainId: number,
): Hash {
	// Pack the UserOperation struct hash
	const packed = encodePacked(
		[
			"address",
			"uint256",
			"bytes32",
			"bytes32",
			"uint256",
			"uint256",
			"uint256",
			"uint256",
			"uint256",
			"bytes32",
		],
		[
			userOp.sender,
			userOp.nonce,
			keccak256(userOp.initCode),
			keccak256(userOp.callData),
			userOp.callGasLimit,
			userOp.verificationGasLimit,
			userOp.preVerificationGas,
			userOp.maxFeePerGas,
			userOp.maxPriorityFeePerGas,
			keccak256(userOp.paymasterAndData),
		],
	);

	const userOpHashInner = keccak256(packed);
	console.log("UserOp hash inner:", userOpHashInner);

	// Calculate the final hash with EntryPoint address and chain ID
	const encoded = encodePacked(
		["bytes32", "address", "uint256"],
		[userOpHashInner, entryPointAddress, BigInt(chainId)],
	);

	const finalHash = keccak256(encoded);
	console.log("UserOp hash final:", finalHash);

	return finalHash;
}

/**
 * Generate initCode for deploying a Smart Account
 * For EntryPoint v0.7, initCode should be empty if using SenderCreator
 * The actual deployment happens via EntryPoint -> SenderCreator -> Factory
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
		abi: CONTRACTS.SmartAccountFactory.abi,
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

