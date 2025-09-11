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
    type PublicClient,
} from "viem";
import { CONTRACTS, DEFAULT_CLIENT_ID } from "./constants";

// Dummy signature for gas estimation (66 bytes total for OmniAccount)
// This is a standard practice in ERC-4337 for gas estimation
const DUMMY_SIGNATURE = "0x017c4deefde7150b2176eb266f1b1d34cf9df0913944cfb0bccb939a66692c49407f7dc4a4d867bc74b3d0c5a9be72b311148c849fa404db959d156774bd48c22d1b" as `0x${string}`;

/**
 * UserOpSigner enum - matches the contract enum
 */
export enum UserOpSigner {
    Owner = 0x00,
    RootKey = 0x01,
    SessionKey = 0x02,
    Passkey = 0x03,
}

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
    ownerType: number,
    clientId: `0x${string}`,
    rootSigner: Address,
): `0x${string}` {
    return encodeFunctionData({
        abi: CONTRACTS.OmniAccountFactory.abi,
        functionName: "getAddress",
        args: [omniAccount, ownerType, clientId, rootSigner],
    });
}

/**
 * Encode initialization data for Omni Account
 */
export function encodeOmniAccountInitData(
    omniAccount: `0x${string}`,
    ownerType: number,
    clientId: `0x${string}`,
    rootSigner: Address,
): `0x${string}` {
    return encodeFunctionData({
        abi: CONTRACTS.OmniAccountFactory.abi,
        functionName: "createAccount",
        args: [omniAccount, ownerType, clientId, rootSigner],
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
 * Includes packed gas fields
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
    signature: `0x${string}`;
}

/**
 * Estimate gas parameters for a UserOperation
 */
export async function estimateUserOperationGas(
    publicClient: PublicClient,
    isDeployment: boolean = false,
): Promise<{
    callGasLimit: bigint;
    verificationGasLimit: bigint;
    preVerificationGas: bigint;
    maxFeePerGas: bigint;
    maxPriorityFeePerGas: bigint;
}> {
    try {
        // Get current gas prices from the network
        const feeData = await publicClient.estimateFeesPerGas();

        // Use the estimated values with a safety margin (1.2x for maxFeePerGas)
        const maxFeePerGas = (feeData.maxFeePerGas || BigInt(20000000000)) * BigInt(120) / BigInt(100);
        const maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || BigInt(1000000000);

        // Use higher gas limits for deployment
        const callGasLimit = isDeployment ? BigInt(2000000) : BigInt(500000);
        const verificationGasLimit = isDeployment ? BigInt(3000000) : BigInt(1000000);
        const preVerificationGas = BigInt(100000);

        return {
            callGasLimit,
            verificationGasLimit,
            preVerificationGas,
            maxFeePerGas,
            maxPriorityFeePerGas,
        };
    } catch (error) {
        console.warn("Failed to estimate gas, using fallback values:", error);
        // Fallback values appropriate for Arbitrum Sepolia
        return {
            callGasLimit: isDeployment ? BigInt(2000000) : BigInt(500000),
            verificationGasLimit: isDeployment ? BigInt(3000000) : BigInt(1000000),
            preVerificationGas: BigInt(100000),
            maxFeePerGas: BigInt(30000000000), // 30 gwei fallback
            maxPriorityFeePerGas: BigInt(1500000000), // 1.5 gwei fallback
        };
    }
}

/**
 * Create a basic UserOperation
 */
export function createUserOperation(params: {
    sender: Address;
    nonce?: bigint;
    callData?: `0x${string}`;
    initCode?: `0x${string}`;
    gasParams?: {
        callGasLimit: bigint;
        verificationGasLimit: bigint;
        preVerificationGas: bigint;
        maxFeePerGas: bigint;
        maxPriorityFeePerGas: bigint;
    };
    paymaster?: {
        address: Address;
        validationGasLimit?: bigint;
        postOpGasLimit?: bigint;
        data?: `0x${string}`;
    };
    forGasEstimation?: boolean;
}): UserOperation {
    // Encode paymasterAndData if paymaster is provided
    let paymasterAndData: `0x${string}` = "0x";
    if (params.paymaster && params.paymaster.address !== "0x0000000000000000000000000000000000000000") {
        paymasterAndData = encodePaymasterAndData(
            params.paymaster.address,
            params.paymaster.validationGasLimit,
            params.paymaster.postOpGasLimit,
            params.paymaster.data,
        );
    }

    // Use provided gas parameters or defaults
    const gasParams = params.gasParams || {
        callGasLimit: BigInt(2000000),
        verificationGasLimit: BigInt(3000000),
        preVerificationGas: BigInt(100000),
        maxFeePerGas: BigInt(30000000000), // 30 gwei default
        maxPriorityFeePerGas: BigInt(1500000000), // 1.5 gwei default
    };

    return {
        sender: params.sender,
        nonce: params.nonce || BigInt(0),
        initCode: params.initCode || "0x",
        callData: params.callData || "0x",
        callGasLimit: gasParams.callGasLimit,
        verificationGasLimit: gasParams.verificationGasLimit,
        preVerificationGas: gasParams.preVerificationGas,
        maxFeePerGas: gasParams.maxFeePerGas,
        maxPriorityFeePerGas: gasParams.maxPriorityFeePerGas,
        paymasterAndData,
        // Use dummy signature for gas estimation, empty otherwise
        signature: params.forGasEstimation ? DUMMY_SIGNATURE : "0x",
    };
}

/**
 * Get the hash of a UserOperation for signing
 * This follows the ERC-4337 specification for PackedUserOperation hash calculation
 * Uses EIP-712 typed data hashing with EntryPoint v0.7 domain separator
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
        toHex("PackedUserOperation(address sender,uint256 nonce,bytes initCode,bytes callData,bytes32 accountGasLimits,uint256 preVerificationGas,bytes32 gasFees,bytes paymasterAndData)")
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
    ownerType: number,
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
        args: [omniAccount, ownerType, clientId, rootSigner],
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
        signature: packed.signature,
    });

    return packed;
}

/**
 * Add UserOpSigner prefix to a signature
 */
export function addSignaturePrefix(
    signature: `0x${string}`,
    signerType: UserOpSigner
): `0x${string}` {
    // Remove 0x prefix from signature
    const sigWithoutPrefix = signature.slice(2);
    // Add signer type byte
    const prefixedSig = `0x${signerType.toString(16).padStart(2, '0')}${sigWithoutPrefix}`;
    return prefixedSig as `0x${string}`;
}

/**
 * Encode paymasterAndData field for UserOperation
 * Format: [20 bytes paymaster address][16 bytes validationGasLimit][16 bytes postOpGasLimit][arbitrary bytes for paymaster data]
 *
 * @param paymasterAddress - The paymaster contract address
 * @param validationGasLimit - Gas limit for paymaster validation (default: 100000)
 * @param postOpGasLimit - Gas limit for paymaster postOp (default: 50000)
 * @param paymasterData - Optional paymaster-specific data
 * @returns Encoded paymasterAndData field
 */
export function encodePaymasterAndData(
    paymasterAddress: Address,
    validationGasLimit: bigint = BigInt(100000),
    postOpGasLimit: bigint = BigInt(50000),
    paymasterData: `0x${string}` = "0x",
): `0x${string}` {
    // Validate inputs
    if (!paymasterAddress || paymasterAddress === "0x0000000000000000000000000000000000000000") {
        throw new Error("Invalid paymaster address");
    }

    // Remove 0x prefix from address
    const addressBytes = paymasterAddress.slice(2).toLowerCase();

    // Convert gas limits to hex strings (uint128 = 16 bytes)
    const validationGasHex = validationGasLimit.toString(16).padStart(32, '0');
    const postOpGasHex = postOpGasLimit.toString(16).padStart(32, '0');

    // Remove 0x prefix from paymaster data if present
    const dataBytes = paymasterData.startsWith('0x') ? paymasterData.slice(2) : paymasterData;

    // Combine all parts
    const encoded = `0x${addressBytes}${validationGasHex}${postOpGasHex}${dataBytes}`;

    console.log("Encoded paymasterAndData:", {
        paymasterAddress,
        validationGasLimit: validationGasLimit.toString(),
        postOpGasLimit: postOpGasLimit.toString(),
        paymasterData,
        encoded,
    });

    return encoded as `0x${string}`;
}

/**
 * Decode paymasterAndData field
 * @param paymasterAndData - The encoded paymaster data
 * @returns Decoded components or null if empty
 */
export function decodePaymasterAndData(paymasterAndData: `0x${string}`): {
    paymaster: Address;
    validationGasLimit: bigint;
    postOpGasLimit: bigint;
    data: `0x${string}`;
} | null {
    if (!paymasterAndData || paymasterAndData === "0x" || paymasterAndData.length < 106) {
        return null;
    }

    const data = paymasterAndData.slice(2); // Remove 0x prefix

    const paymaster = `0x${data.slice(0, 40)}` as Address;
    const validationGasLimit = BigInt(`0x${data.slice(40, 72)}`);
    const postOpGasLimit = BigInt(`0x${data.slice(72, 104)}`);
    const paymasterData = `0x${data.slice(104)}` as `0x${string}`;

    return {
        paymaster,
        validationGasLimit,
        postOpGasLimit,
        data: paymasterData,
    };
}

/**
 * Check if paymaster is available and has sufficient balance
 * @param publicClient - Viem public client
 * @param paymasterAddress - Address of the paymaster contract
 * @param entryPointAddress - Address of the EntryPoint contract
 * @param minBalance - Minimum balance required (default: 0.01 ETH)
 * @returns Object with availability status and balance info
 */
export async function checkPaymasterStatus(
    publicClient: any,
    paymasterAddress: Address,
    entryPointAddress: Address,
    minBalance: bigint = BigInt("10000000000000000"), // 0.01 ETH
): Promise<{
    isAvailable: boolean;
    isDeployed: boolean;
    isFunded: boolean;
    balance: bigint;
    error?: string;
}> {
    try {
        // Check if paymaster is deployed
        const code = await publicClient.getBytecode({ address: paymasterAddress });
        const isDeployed = !!(code && code !== "0x");

        if (!isDeployed) {
            return {
                isAvailable: false,
                isDeployed: false,
                isFunded: false,
                balance: BigInt(0),
                error: "Paymaster not deployed",
            };
        }

        // Check paymaster balance at EntryPoint
        const balance = await publicClient.readContract({
            address: entryPointAddress,
            abi: CONTRACTS.EntryPoint.abi,
            functionName: "balanceOf",
            args: [paymasterAddress],
        }) as bigint;

        const isFunded = balance >= minBalance;

        return {
            isAvailable: isDeployed && isFunded,
            isDeployed,
            isFunded,
            balance,
            error: isFunded ? undefined : `Paymaster has insufficient balance: ${balance.toString()} wei`,
        };
    } catch (error) {
        console.error("Error checking paymaster status:", error);
        return {
            isAvailable: false,
            isDeployed: false,
            isFunded: false,
            balance: BigInt(0),
            error: error instanceof Error ? error.message : "Unknown error",
        };
    }
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
    signerType: UserOpSigner = UserOpSigner.Owner,
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
        // Add the signer type prefix
        return addSignaturePrefix(signature, signerType);
    } catch (e) {
        console.error("EIP-712 signing failed:", e);

        // Fallback: Use personal_sign (adds message prefix)
        const userOpHash = getUserOpHash(userOp, entryPointAddress, Number(chainId));
        const signature = await walletClient.signMessage({
            account: address,
            message: { raw: userOpHash },
        });

        console.warn("WARNING: Using personal_sign which adds message prefix");
        // Add the signer type prefix
        return addSignaturePrefix(signature, signerType);
    }
}

/**
 * SerializablePackedUserOperation interface matching the Rust struct
 */
export interface SerializablePackedUserOperation {
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

/**
 * Convert PackedUserOperation to SerializablePackedUserOperation
 */
export function toSerializablePackedUserOperation(
    packedOp: PackedUserOperation
): SerializablePackedUserOperation {
    return {
        sender: packedOp.sender,
        nonce: Number(packedOp.nonce), // Convert bigint to number
        init_code: packedOp.initCode,
        call_data: packedOp.callData,
        account_gas_limits: packedOp.accountGasLimits,
        pre_verification_gas: Number(packedOp.preVerificationGas), // Convert bigint to number
        gas_fees: packedOp.gasFees,
        paymaster_and_data: packedOp.paymasterAndData,
        signature: packedOp.signature === "0x" ? undefined : packedOp.signature,
    };
}

/**
 * Build calldata for ERC20 transfer
 */
export function buildERC20TransferCallData(
    to: Address,
    amount: bigint
): `0x${string}` {
    return encodeFunctionData({
        abi: [{
            name: 'transfer',
            type: 'function',
            inputs: [
                { name: 'to', type: 'address' },
                { name: 'amount', type: 'uint256' }
            ],
            outputs: [{ name: '', type: 'bool' }]
        }],
        functionName: 'transfer',
        args: [to, amount]
    });
}

/**
 * Build only the ERC20 paymaster data segment used by ERC20PaymasterV1
 * Layout: token(20) | exchangeRate(32) | validUntil(32) | validAfter(32)
 */
export function buildErc20PaymasterData(
    tokenAddress: Address,
    exchangeRate: bigint,
    validUntil: bigint,
    validAfter: bigint,
): `0x${string}` {
    return encodePacked(
        ['address', 'uint256', 'uint256', 'uint256'],
        [tokenAddress, exchangeRate, validUntil, validAfter]
    ) as `0x${string}`;
}

/**
 * Encode ERC20 paymaster data for use with ERC20PaymasterV1
 * This encodes the specific format expected by ERC20PaymasterV1 contract
 * @param paymasterAddress The ERC20PaymasterV1 contract address
 * @param validationGasLimit Gas limit for paymaster validation
 * @param postOpGasLimit Gas limit for paymaster postOp
 * @param tokenAddress The ERC20 token address to use for payment
 * @param exchangeRate Exchange rate (0 to let TEE worker fill it)
 * @param validUntil Timestamp until when this rate is valid
 * @param validAfter Timestamp after which this rate is valid
 * @returns Encoded paymasterAndData field for ERC20PaymasterV1
 */
export function encodeERC20PaymasterData(
    paymasterAddress: Address,
    validationGasLimit: bigint = BigInt(100000),
    postOpGasLimit: bigint = BigInt(50000),
    tokenAddress: Address,
    exchangeRate: bigint = BigInt(0), // 0 means TEE worker will fill this
    validUntil: bigint = BigInt(Math.floor(Date.now() / 1000) + 3600), // Valid for 1 hour by default
    validAfter: bigint = BigInt(0), // Valid immediately by default
): `0x${string}` {
    if (!paymasterAddress || paymasterAddress === "0x0000000000000000000000000000000000000000") {
        throw new Error("Invalid paymaster address");
    }
    if (!tokenAddress || tokenAddress === "0x0000000000000000000000000000000000000000") {
        throw new Error("Invalid token address - ERC20PaymasterV1 requires a valid token");
    }

    const encoded = encodePacked(
        ['address', 'uint128', 'uint128', 'address', 'uint256', 'uint256', 'uint256'],
        [paymasterAddress, validationGasLimit, postOpGasLimit, tokenAddress, exchangeRate, validUntil, validAfter]
    );
    
    console.log("Encoded ERC20 paymaster data:", {
        paymasterAddress,
        tokenAddress,
        validationGasLimit: validationGasLimit.toString(),
        postOpGasLimit: postOpGasLimit.toString(),
        exchangeRate: exchangeRate.toString(),
        validUntil: validUntil.toString(),
        validAfter: validAfter.toString(),
        encoded,
    });

    return encoded;
}

/**
 * Build UserOperation for ERC20 approve transaction
 * Used to approve the ERC20PaymasterV1 to spend tokens for gas
 */
export function buildApprovalUserOp(params: {
    omniAccountAddress: Address;
    tokenAddress: Address;
    spender: Address;
    amount: bigint;
    nonce: bigint;
    gasParams?: {
        callGasLimit: bigint;
        verificationGasLimit: bigint;
        preVerificationGas: bigint;
        maxFeePerGas: bigint;
        maxPriorityFeePerGas: bigint;
    };
    paymaster?: {
        address: Address;
        validationGasLimit?: bigint;
        postOpGasLimit?: bigint;
        data?: `0x${string}`;
    };
    forGasEstimation?: boolean;
}): UserOperation {
    // Build approve calldata
    const approveCallData = encodeFunctionData({
        abi: [{
            name: 'approve',
            type: 'function',
            inputs: [
                { name: 'spender', type: 'address' },
                { name: 'amount', type: 'uint256' }
            ],
            outputs: [{ name: '', type: 'bool' }]
        }],
        functionName: 'approve',
        args: [params.spender, params.amount]
    });

    // Build execute calldata for OmniAccount
    const callData = encodeFunctionData({
        abi: CONTRACTS.OmniAccountImplementation.abi,
        functionName: "execute",
        args: [params.tokenAddress, BigInt(0), approveCallData],
    });

    return createUserOperation({
        sender: params.omniAccountAddress,
        nonce: params.nonce,
        callData,
        gasParams: params.gasParams,
        paymaster: params.paymaster,
        forGasEstimation: params.forGasEstimation,
    });
}

/**
 * Build UserOperation for token transfer through OmniAccount
 */
export function buildTokenTransferUserOp(params: {
    omniAccountAddress: Address;
    tokenAddress: Address;
    recipient: Address;
    amount: bigint;
    nonce?: bigint;
    gasParams?: {
        callGasLimit: bigint;
        verificationGasLimit: bigint;
        preVerificationGas: bigint;
        maxFeePerGas: bigint;
        maxPriorityFeePerGas: bigint;
    };
    paymaster?: {
        address: Address;
        validationGasLimit?: bigint;
        postOpGasLimit?: bigint;
        data?: `0x${string}`;
    };
    forGasEstimation?: boolean;
}): UserOperation {
    // Build the ERC20 transfer calldata
    const erc20TransferData = buildERC20TransferCallData(params.recipient, params.amount);

    // Build the OmniAccount execute calldata
    const executeCallData = encodeFunctionData({
        abi: CONTRACTS.OmniAccountImplementation.abi,
        functionName: 'execute',
        args: [
            params.tokenAddress, // target: ERC20 token contract
            BigInt(0), // value: 0 ETH
            erc20TransferData // data: ERC20 transfer function call
        ]
    });

    // Create the UserOperation
    return createUserOperation({
        sender: params.omniAccountAddress,
        nonce: params.nonce,
        callData: executeCallData,
        gasParams: params.gasParams,
        paymaster: params.paymaster,
        forGasEstimation: params.forGasEstimation,
    });
}

/**
 * Build UserOperation for native ETH transfer through OmniAccount
 */
export function buildNativeTransferUserOp(params: {
    omniAccountAddress: Address;
    recipient: Address;
    amount: bigint;
    nonce?: bigint;
    gasParams?: {
        callGasLimit: bigint;
        verificationGasLimit: bigint;
        preVerificationGas: bigint;
        maxFeePerGas: bigint;
        maxPriorityFeePerGas: bigint;
    };
    paymaster?: {
        address: Address;
        validationGasLimit?: bigint;
        postOpGasLimit?: bigint;
        data?: `0x${string}`;
    };
    forGasEstimation?: boolean;
}): UserOperation {
    // Build the OmniAccount execute calldata for native ETH transfer
    const executeCallData = encodeFunctionData({
        abi: CONTRACTS.OmniAccountImplementation.abi,
        functionName: 'execute',
        args: [
            params.recipient, // target: recipient address
            params.amount, // value: amount of ETH to send
            "0x" as `0x${string}` // data: empty bytes for native transfer
        ]
    });

    // Create the UserOperation
    return createUserOperation({
        sender: params.omniAccountAddress,
        nonce: params.nonce,
        callData: executeCallData,
        gasParams: params.gasParams,
        paymaster: params.paymaster,
        forGasEstimation: params.forGasEstimation,
    });
}

/**
 * Estimate gas parameters for a UserOperation using the TEE Worker
 * Falls back to local estimation if the worker call fails
 */
export async function estimateUserOpGasFromWorker(
    userOp: UserOperation,
    chainId: number,
    walletIndex: number = 0,
    omniAccount: string,
    clientId: string,
    publicClient?: PublicClient,
): Promise<{
    callGasLimit: bigint;
    verificationGasLimit: bigint;
    preVerificationGas: bigint;
    maxFeePerGas: bigint;
    maxPriorityFeePerGas: bigint;
    paymasterVerificationGasLimit?: bigint;
    paymasterPostOpGasLimit?: bigint;
}> {
    try {
        // Import the worker client function dynamically to avoid circular dependencies
        const { estimateUserOpGas } = await import('./tee-worker-client');

        // Convert UserOperation to PackedUserOperation then to SerializablePackedUserOperation
        const packedOp = packUserOperation(userOp);
        const serializableOp = toSerializablePackedUserOperation(packedOp);

        console.log("[Gas Estimation] Requesting gas estimation from TEE worker...");

        // Call the worker's gas estimation RPC method
        const workerEstimate = await estimateUserOpGas(
            serializableOp,
            chainId,
            walletIndex,
            omniAccount,
            clientId
        );

        console.log("[Gas Estimation] Worker response:", workerEstimate);

        // Get current network fee data for maxFeePerGas and maxPriorityFeePerGas
        let maxFeePerGas: bigint;
        let maxPriorityFeePerGas: bigint;

        if (publicClient) {
            try {
                const feeData = await publicClient.estimateFeesPerGas();
                maxFeePerGas = (feeData.maxFeePerGas || BigInt(20000000000)) * BigInt(120) / BigInt(100);
                maxPriorityFeePerGas = feeData.maxPriorityFeePerGas || BigInt(1000000000);
            } catch {
                // Fallback values if fee estimation fails
                maxFeePerGas = BigInt(30000000000);
                maxPriorityFeePerGas = BigInt(1500000000);
            }
        } else {
            // Default values if no publicClient provided
            maxFeePerGas = BigInt(30000000000);
            maxPriorityFeePerGas = BigInt(1500000000);
        }

        // Convert string values from worker response to bigints
        const gasParams = {
            callGasLimit: BigInt(workerEstimate.callGasLimit),
            verificationGasLimit: BigInt(workerEstimate.verificationGasLimit),
            preVerificationGas: BigInt(workerEstimate.preVerificationGas),
            maxFeePerGas,
            maxPriorityFeePerGas,
            paymasterVerificationGasLimit: BigInt(workerEstimate.paymasterVerificationGasLimit),
            paymasterPostOpGasLimit: BigInt(workerEstimate.paymasterPostOpGasLimit),
        };

        console.log("[Gas Estimation] Using worker gas estimates:", gasParams);
        return gasParams;

    } catch (error) {
        console.error("[Gas Estimation] Worker gas estimation failed:", error);
        // Re-throw error - no fallback for now as we're testing worker estimation
        throw error;
    }
}
