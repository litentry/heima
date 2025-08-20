import {
    encodeFunctionData,
    encodePacked,
    encodeAbiParameters,
    keccak256,
    pad,
    toHex,
    type Address,
    type Hash,
} from 'viem';
import { TEST_CONFIG, type ContractAddress } from '../config';

// OwnerType enum from the contract
export const OWNER_TYPE = {
    PUMPX: 0,
    EMAIL: 1,
    TWITTER: 2,
    DISCORD: 3,
    GITHUB: 4,
    SUBSTRATE: 5,
    EVM: 6,
    BITCOIN: 7,
    SOLANA: 8,
    GOOGLE: 9,
    PASSKEY: 10,
} as const;

// Contract ABIs - simplified for testing
export const CONTRACT_ABIS = {
    ENTRY_POINT: [
        {
            name: 'handleOps',
            type: 'function',
            inputs: [
                {
                    name: 'ops',
                    type: 'tuple[]',
                    components: [
                        { name: 'sender', type: 'address' },
                        { name: 'nonce', type: 'uint256' },
                        { name: 'initCode', type: 'bytes' },
                        { name: 'callData', type: 'bytes' },
                        { name: 'accountGasLimits', type: 'bytes32' },
                        { name: 'preVerificationGas', type: 'uint256' },
                        { name: 'gasFees', type: 'bytes32' },
                        { name: 'paymasterAndData', type: 'bytes' },
                        { name: 'signature', type: 'bytes' },
                    ],
                },
                { name: 'beneficiary', type: 'address' },
            ],
            outputs: [],
            stateMutability: 'nonpayable',
        },
        {
            name: 'getNonce',
            type: 'function',
            inputs: [
                { name: 'sender', type: 'address' },
                { name: 'key', type: 'uint192' },
            ],
            outputs: [{ name: 'nonce', type: 'uint256' }],
            stateMutability: 'view',
        },
    ],

    OMNI_ACCOUNT_FACTORY: [
        {
            name: 'createAccount',
            type: 'function',
            inputs: [
                { name: 'oa', type: 'bytes32' },
                { name: 'oaType', type: 'uint8' }, // OwnerType enum as uint8
                { name: 'clientId', type: 'bytes' },
                { name: 'root', type: 'address' },
            ],
            outputs: [{ name: 'ret', type: 'address' }],
            stateMutability: 'nonpayable',
        },
        {
            name: 'getAddress',
            type: 'function',
            inputs: [
                { name: 'oa', type: 'bytes32' },
                { name: 'oaType', type: 'uint8' }, // OwnerType enum as uint8
                { name: 'clientId', type: 'bytes' },
                { name: 'root', type: 'address' },
            ],
            outputs: [{ name: '', type: 'address' }],
            stateMutability: 'view',
        },
    ],

    OMNI_ACCOUNT: [
        {
            name: 'addRootSigner',
            type: 'function',
            inputs: [{ name: 'root', type: 'address' }],
            outputs: [],
            stateMutability: 'nonpayable',
        },
        {
            name: 'isRootSigner',
            type: 'function',
            inputs: [{ name: 'root', type: 'address' }],
            outputs: [{ name: 'result', type: 'bool' }],
            stateMutability: 'view',
        },
        {
            name: 'execute',
            type: 'function',
            inputs: [
                { name: 'target', type: 'address' },
                { name: 'value', type: 'uint256' },
                { name: 'data', type: 'bytes' },
            ],
            outputs: [],
            stateMutability: 'nonpayable',
        },
    ],

    TEST_TOKEN: [
        {
            name: 'mint',
            type: 'function',
            inputs: [
                { name: 'to', type: 'address' },
                { name: 'amount', type: 'uint256' },
            ],
            outputs: [],
            stateMutability: 'nonpayable',
        },
        {
            name: 'transfer',
            type: 'function',
            inputs: [
                { name: 'to', type: 'address' },
                { name: 'amount', type: 'uint256' },
            ],
            outputs: [{ name: 'result', type: 'bool' }],
            stateMutability: 'nonpayable',
        },
        {
            name: 'balanceOf',
            type: 'function',
            inputs: [{ name: 'account', type: 'address' }],
            outputs: [{ name: 'balance', type: 'uint256' }],
            stateMutability: 'view',
        },
    ],
} as const;

// UserOperation interfaces
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

export interface PackedUserOperation {
    sender: Address;
    nonce: bigint;
    initCode: `0x${string}`;
    callData: `0x${string}`;
    accountGasLimits: `0x${string}`;
    preVerificationGas: bigint;
    gasFees: `0x${string}`;
    paymasterAndData: `0x${string}`;
    signature: `0x${string}`;
}

// Utility functions
export function stringToBytes(str: string): `0x${string}` {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(str);
    return `0x${Array.from(bytes)
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('')}`;
}

export function packAccountGasLimits(callGasLimit: bigint, verificationGasLimit: bigint): `0x${string}` {
    const packed = (verificationGasLimit << BigInt(128)) | callGasLimit;
    return pad(toHex(packed), { size: 32 });
}

export function packGasFees(maxFeePerGas: bigint, maxPriorityFeePerGas: bigint): `0x${string}` {
    const packed = (maxPriorityFeePerGas << BigInt(128)) | maxFeePerGas;
    return pad(toHex(packed), { size: 32 });
}

export function packUserOperation(userOp: UserOperation): PackedUserOperation {
    return {
        sender: userOp.sender,
        nonce: userOp.nonce,
        initCode: userOp.initCode,
        callData: userOp.callData,
        accountGasLimits: packAccountGasLimits(userOp.callGasLimit, userOp.verificationGasLimit),
        preVerificationGas: userOp.preVerificationGas,
        gasFees: packGasFees(userOp.maxFeePerGas, userOp.maxPriorityFeePerGas),
        paymasterAndData: userOp.paymasterAndData,
        signature: userOp.signature,
    };
}

export function createUserOperation(params: {
    sender: Address;
    nonce?: bigint;
    callData?: `0x${string}`;
    initCode?: `0x${string}`;
    gasConfig?: typeof TEST_CONFIG.GAS_CONFIG;
}): UserOperation {
    const gasConfig = params.gasConfig || TEST_CONFIG.GAS_CONFIG;

    return {
        sender: params.sender,
        nonce: params.nonce || BigInt(0),
        initCode: params.initCode || '0x',
        callData: params.callData || '0x',
        callGasLimit: gasConfig.CALL_GAS_LIMIT,
        verificationGasLimit: gasConfig.VERIFICATION_GAS_LIMIT,
        preVerificationGas: gasConfig.PRE_VERIFICATION_GAS,
        maxFeePerGas: gasConfig.MAX_FEE_PER_GAS,
        maxPriorityFeePerGas: gasConfig.MAX_PRIORITY_FEE_PER_GAS,
        paymasterAndData: '0x',
        signature: '0x',
    };
}

export function addSignaturePrefix(signature: `0x${string}`, signerType: number): `0x${string}` {
    // Remove 0x prefix from signature
    const sigWithoutPrefix = signature.slice(2);
    // Add signer type byte
    const prefixedSig = `0x${signerType.toString(16).padStart(2, '0')}${sigWithoutPrefix}`;
    return prefixedSig as `0x${string}`;
}

export async function signUserOperation(
    walletClient: any,
    userOp: UserOperation,
    entryPointAddress: Address,
    chainId: number,
    signerType: number = TEST_CONFIG.SIGNER_TYPES.OWNER
): Promise<`0x${string}`> {
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
        console.log('Signing PackedUserOperation with EIP-712...');
        const signature = await walletClient.signTypedData({
            account: walletClient.account,
            domain,
            types,
            primaryType: 'PackedUserOperation',
            message,
        });
        console.log('Successfully signed with EIP-712');
        // Add signer type prefix
        return addSignaturePrefix(signature, signerType);
    } catch (e) {
        console.error('EIP-712 signing failed:', e);
        
        // Fallback: Use personal_sign (adds message prefix)
        const userOpHash = getUserOpHash(userOp, entryPointAddress, chainId);
        const signature = await walletClient.signMessage({
            account: walletClient.account,
            message: { raw: userOpHash },
        });
        
        console.warn('WARNING: Using personal_sign which adds message prefix');
        // Add signer type prefix
        return addSignaturePrefix(signature, signerType);
    }
}

export function generateInitCode(
    factoryAddress: Address,
    omniAccount: `0x${string}`,
    ownerType: number,
    clientId: `0x${string}`,
    rootSigner: Address
): `0x${string}` {
    // Encode the factory call data
    const initCalldata = encodeFunctionData({
        abi: CONTRACT_ABIS.OMNI_ACCOUNT_FACTORY,
        functionName: 'createAccount',
        args: [omniAccount, ownerType, clientId, rootSigner],
    });

    // Combine factory address and calldata
    return `${factoryAddress}${initCalldata.slice(2)}` as `0x${string}`;
}

export function createTokenTransferCalldata(tokenAddress: ContractAddress, to: Address, amount: bigint): `0x${string}` {
    return encodeFunctionData({
        abi: CONTRACT_ABIS.TEST_TOKEN,
        functionName: 'transfer',
        args: [to, amount],
    });
}

export function createExecuteCalldata(target: Address, value: bigint, data: `0x${string}`): `0x${string}` {
    return encodeFunctionData({
        abi: CONTRACT_ABIS.OMNI_ACCOUNT,
        functionName: 'execute',
        args: [target, value, data],
    });
}

export function createAddSignerCalldata(signer: Address): `0x${string}` {
    return encodeFunctionData({
        abi: CONTRACT_ABIS.OMNI_ACCOUNT,
        functionName: 'addRootSigner',
        args: [signer],
    });
}

// Add getUserOpHash function for signature compatibility
export function getUserOpHash(
    userOp: UserOperation,
    entryPointAddress: Address,
    chainId: number
): Hash {
    const packedOp = packUserOperation(userOp);
    
    const PACKED_USEROP_TYPEHASH = keccak256(
        toHex('PackedUserOperation(address sender,uint256 nonce,bytes initCode,bytes callData,bytes32 accountGasLimits,uint256 preVerificationGas,bytes32 gasFees,bytes paymasterAndData)')
    );
    
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
        ]
    );
    
    const userOpHashInner = keccak256(encoded);
    
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
    
    return keccak256(
        encodePacked(
            ['bytes1', 'bytes1', 'bytes32', 'bytes32'],
            ['0x19', '0x01', domainSeparator, userOpHashInner]
        )
    );
}

// SerializablePackedUserOperation interface matching Rust struct
export interface SerializablePackedUserOperation {
    sender: string;
    nonce: number;
    init_code: string;
    call_data: string;
    account_gas_limits: string;
    pre_verification_gas: number;
    gas_fees: string;
    paymaster_and_data: string;
    signature?: string;
}

// Convert PackedUserOperation to SerializablePackedUserOperation
export function toSerializablePackedUserOperation(
    packedOp: PackedUserOperation
): SerializablePackedUserOperation {
    return {
        sender: packedOp.sender,
        nonce: Number(packedOp.nonce),
        init_code: packedOp.initCode,
        call_data: packedOp.callData,
        account_gas_limits: packedOp.accountGasLimits,
        pre_verification_gas: Number(packedOp.preVerificationGas),
        gas_fees: packedOp.gasFees,
        paymaster_and_data: packedOp.paymasterAndData,
        signature: packedOp.signature === '0x' ? undefined : packedOp.signature,
    };
}
