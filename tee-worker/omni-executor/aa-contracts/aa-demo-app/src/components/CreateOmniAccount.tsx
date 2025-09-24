import { useState, useEffect } from "react";
import { useAccount, useWalletClient, usePublicClient } from "wagmi";
import { encodeAbiParameters, parseAbiParameters, concat, formatUnits } from "viem";
import {
    Wallet,
    AlertTriangle,
    CheckCircle,
    Loader2,
    Info,
    Plus,
    Mail,
} from "lucide-react";
import { useAuth } from "@/contexts/AuthContext";
import { getTEEWorkerAddress, submitUserOpTest } from "@/lib/tee-worker-client";
import {
    calculateOmniAccount,
    createUserOperation,
    stringToBytes,
    generateInitCode,
    packUserOperation,
    signUserOperation,
    UserOpSigner,
    type UserOperation,
    checkPaymasterStatus,
    estimateUserOperationGas,
    toSerializablePackedUserOperation,
} from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS, PAYMASTER_CONFIG, OwnerType, PaymasterType, ERC20_TOKENS } from "@/lib/constants";

interface CreateOmniAccountProps {
    omniAccountAddress?: string;
    isFunded: boolean;
    onAccountCreated?: () => void;
}

export function CreateOmniAccount({
    omniAccountAddress,
    isFunded,
    onAccountCreated,
}: CreateOmniAccountProps) {
    const { address: evmAddress, chain } = useAccount();
    const { data: walletClient } = useWalletClient();
    const publicClient = usePublicClient();
    const { authType, identifier, omniAccountHash } = useAuth();

    // Get chain ID - use wallet chain if available, otherwise use environment default
    const chainId = chain?.id || parseInt(process.env.NEXT_PUBLIC_CHAIN_ID || "1337");

    const [isProcessing, setIsProcessing] = useState(false);
    const [accountCreated, setAccountCreated] = useState(false);
    const [error, setError] = useState("");
    const [txHash, setTxHash] = useState("");
    const [paymasterType, setPaymasterType] = useState<PaymasterType>(PaymasterType.None);
    const [paymasterStatus, setPaymasterStatus] = useState<{
        isAvailable: boolean;
        balance: bigint;
        error?: string;
    } | null>(null);
    const [usdcBalance, setUsdcBalance] = useState<bigint>(BigInt(0));
    const [isLoadingUsdcBalance, setIsLoadingUsdcBalance] = useState(false);

    const truncateAddress = (address: string) => {
        if (!address) return "";
        return `${address.slice(0, 6)}...${address.slice(-4)}`;
    };

    // Check paymaster status and USDC balance when paymaster type changes
    useEffect(() => {
        const checkPaymaster = async () => {
            if (!publicClient) return;

            if (paymasterType === PaymasterType.Simple &&
                CONTRACTS.SimplePaymaster.address !== "0x0000000000000000000000000000000000000000") {
                const status = await checkPaymasterStatus(
                    publicClient,
                    CONTRACTS.SimplePaymaster.address,
                    CONTRACTS.EntryPoint.address,
                );
                setPaymasterStatus({
                    isAvailable: status.isAvailable,
                    balance: status.balance,
                    error: status.error,
                });
            } else if (paymasterType === PaymasterType.ERC20 &&
                       CONTRACTS.ERC20Paymaster.address !== "0x0000000000000000000000000000000000000000") {
                // Check ERC20 paymaster availability
                const status = await checkPaymasterStatus(
                    publicClient,
                    CONTRACTS.ERC20Paymaster.address,
                    CONTRACTS.EntryPoint.address,
                );
                setPaymasterStatus({
                    isAvailable: status.isAvailable,
                    balance: status.balance,
                    error: status.error,
                });
            }
        };

        checkPaymaster();
    }, [publicClient, paymasterType]);

    // Fetch USDC balance when using ERC20 paymaster
    useEffect(() => {
        const fetchUsdcBalance = async () => {
            if (!omniAccountAddress || !publicClient || paymasterType !== PaymasterType.ERC20) return;

            setIsLoadingUsdcBalance(true);
            try {
                const balance = await publicClient.readContract({
                    address: ERC20_TOKENS.USDC.address,
                    abi: ERC20_TOKENS.USDC.abi,
                    functionName: "balanceOf",
                    args: [omniAccountAddress as `0x${string}`],
                }) as bigint;
                setUsdcBalance(balance);
            } catch (error) {
                console.error("Failed to fetch USDC balance:", error);
                setUsdcBalance(BigInt(0));
            } finally {
                setIsLoadingUsdcBalance(false);
            }
        };

        fetchUsdcBalance();
    }, [omniAccountAddress, publicClient, paymasterType]);

    const handleCreateAccount = async () => {
        if (!omniAccountAddress) {
            setError("Smart account address is still being calculated. Please wait a moment.");
            return;
        }

        if (!publicClient) {
            setError("Missing blockchain connection");
            return;
        }

        // For wallet users, ensure chain is available
        if (authType === "wallet" && !chain) {
            setError("Please connect your wallet to a supported network");
            return;
        }

        // Additional checks for wallet auth
        if (authType === "wallet" && (!evmAddress || !walletClient)) {
            setError("Missing wallet connection");
            return;
        }

        // Additional checks for email auth
        if (authType === "email" && (!identifier || !omniAccountHash)) {
            setError("Missing email authentication");
            return;
        }

        setIsProcessing(true);
        setError("");

        try {
            // Calculate omni account and client ID bytes
            let calculatedOmniAccount: `0x${string}`;
            let rootSigner: `0x${string}`;
            let ownerType: number;

            if (authType === "email") {
                // For email accounts
                calculatedOmniAccount = omniAccountHash as `0x${string}`;
                // Get TEE worker as root signer
                rootSigner = await getTEEWorkerAddress(omniAccountHash) as `0x${string}`;
                ownerType = OwnerType.Email;
            } else {
                // For wallet accounts
                calculatedOmniAccount = calculateOmniAccount(
                    evmAddress!,
                    DEFAULT_CLIENT_ID,
                    "evm",
                );
                rootSigner = evmAddress as `0x${string}`;
                ownerType = OwnerType.Evm;
            }

            const clientIdBytes = stringToBytes(DEFAULT_CLIENT_ID);

            console.log("Creating Omni Account with:", {
                authType,
                identifier,
                clientId: DEFAULT_CLIENT_ID,
                omniAccountAddress,
                calculatedOmniAccount,
                rootSigner,
                ownerType,
            });

            // Check if account already exists
            const code = await publicClient.getBytecode({
                address: omniAccountAddress as `0x${string}`,
            });

            const accountExists = !!(code && code !== "0x");

            if (accountExists) {
                setError("Omni Account already exists at this address");
                return;
            }

            // Generate initCode for Omni Account deployment
            const initCode = generateInitCode(
                CONTRACTS.OmniAccountFactory.address,
                calculatedOmniAccount,
                ownerType,
                clientIdBytes,
                rootSigner,
            ) as `0x${string}`;

            console.log("Deployment parameters:", {
                rootSigner,
                ownerType,
                initCode: initCode.slice(0, 66) + "...",
            });

            // Estimate gas parameters for the deployment
            console.log("Estimating gas parameters...");
            const gasParams = await estimateUserOperationGas(publicClient, true);
            console.log("Estimated gas parameters:", gasParams);

            // Create UserOperation for deploying the Omni Account
            const userOp = createUserOperation({
                sender: omniAccountAddress as `0x${string}`,
                nonce: BigInt(0),
                initCode: initCode,
                callData: "0x" as `0x${string}`, // Empty callData for deployment
                gasParams,
            });

            // Initialize signature (will be set later based on auth type)
            userOp.signature = "0x" as `0x${string}`;

            console.log("Created UserOperation:", userOp);

            // Handle paymaster based on type
            if (paymasterType === PaymasterType.Simple && paymasterStatus?.isAvailable) {
                console.log("Using Simple Paymaster for deployment...");
                const paymasterData = encodeAbiParameters(
                    parseAbiParameters("uint48, uint48"),
                    [
                        Number(PAYMASTER_CONFIG.defaultValidationGasLimit),
                        Number(PAYMASTER_CONFIG.defaultPostOpGasLimit),
                    ],
                );

                userOp.paymasterAndData = concat([
                    CONTRACTS.SimplePaymaster.address as `0x${string}`,
                    paymasterData,
                ]) as `0x${string}`;
            } else if (paymasterType === PaymasterType.ERC20 && paymasterStatus?.isAvailable) {
                console.log("Using ERC20 Paymaster for deployment with USDC...");
                // For ERC20PaymasterV1, the data format is: token address + postOpGasLimit
                const paymasterData = encodeAbiParameters(
                    parseAbiParameters("address, uint256"),
                    [
                        ERC20_TOKENS.USDC.address,  // Payment token
                        BigInt(50000),              // postOpGasLimit
                    ],
                );

                userOp.paymasterAndData = concat([
                    CONTRACTS.ERC20Paymaster.address as `0x${string}`,
                    paymasterData,
                ]) as `0x${string}`;
            }

            // Pack the UserOperation
            const packedUserOp = packUserOperation(userOp);
            console.log("Packed UserOperation:", packedUserOp);

            let transactionHash: string | undefined;

            if (authType === "email") {
                // For email accounts, submit via TEE worker
                console.log("Submitting UserOp through TEE worker...");

                const serializedOp = toSerializablePackedUserOperation(packedUserOp);

                const response = await submitUserOpTest(
                    [serializedOp],
                    chainId,
                    0, // wallet_index
                    omniAccountHash,
                    DEFAULT_CLIENT_ID,
                );

                transactionHash = response.transaction_hash || undefined;
                console.log("TEE Worker response:", response);
            } else {
                // For wallet accounts, sign and submit normally
                const entryPointAddress = CONTRACTS.EntryPoint.address;

                // Sign the UserOperation with the wallet
                const signature = await signUserOperation(
                    walletClient!,
                    evmAddress!,
                    userOp,
                    entryPointAddress,
                    BigInt(chainId),
                    UserOpSigner.Owner,
                );

                // Add signature to UserOp
                userOp.signature = signature;

                // Pack the signed UserOperation
                const signedPackedUserOp = packUserOperation(userOp);

                // Submit the signed UserOperation to the EntryPoint
                const userOpHash = await walletClient!.writeContract({
                    address: entryPointAddress,
                    abi: CONTRACTS.EntryPoint.abi,
                    functionName: "handleOps",
                    args: [[signedPackedUserOp], evmAddress] as const,
                    account: walletClient!.account!,
                    chain: walletClient!.chain,
                });

                transactionHash = userOpHash;
                console.log("Transaction hash:", userOpHash);
            }

            if (transactionHash) {
                setTxHash(transactionHash);

                // Wait for transaction confirmation
                console.log("Waiting for confirmation...");
                const receipt = await publicClient.waitForTransactionReceipt({
                    hash: transactionHash as `0x${string}`,
                });

                console.log("Transaction receipt:", receipt);

                if (receipt.status === "success") {
                    setAccountCreated(true);
                    if (onAccountCreated) {
                        onAccountCreated();
                    }
                    console.log("Omni Account created successfully!");
                } else {
                    setError("Transaction failed. Please check the transaction on the explorer.");
                }
            } else {
                setError("Failed to submit transaction");
            }
        } catch (err: any) {
            console.error("Error creating account:", err);
            setError(err.message || "Failed to create account");
        } finally {
            setIsProcessing(false);
        }
    };

    if (!isFunded) {
        return (
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
                <div className="flex">
                    <AlertTriangle className="h-5 w-5 text-yellow-600 mt-0.5" />
                    <div className="ml-3">
                        <h3 className="text-sm font-medium text-yellow-800">
                            Account Not Funded
                        </h3>
                        <div className="mt-2 text-sm text-yellow-700">
                            <p>Please fund your Omni Account with ETH before creating it.</p>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    if (accountCreated) {
        return (
            <div className="bg-green-50 border border-green-200 rounded-lg p-6">
                <div className="flex">
                    <CheckCircle className="h-6 w-6 text-green-600 mt-0.5" />
                    <div className="ml-3 flex-1">
                        <h3 className="text-lg font-medium text-green-800">
                            Account Created Successfully!
                        </h3>
                        <p className="mt-2 text-sm text-green-700">
                            Your Omni Account has been deployed at:
                        </p>
                        <p className="mt-1 text-sm font-mono text-green-900">
                            {truncateAddress(omniAccountAddress || "")}
                        </p>
                        {txHash && (
                            <div className="mt-3">
                                <a
                                    href={`https://etherscan.io/tx/${txHash}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-sm text-green-600 hover:text-green-700 underline"
                                >
                                    View transaction on explorer →
                                </a>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        );
    }

    // For email accounts, show the email-specific UI
    if (authType === "email") {
        return (
            <div className="space-y-6">
                {/* Show loading state if address is not ready */}
                {!omniAccountAddress && (
                    <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                        <div className="flex items-center space-x-3">
                            <Loader2 className="h-5 w-5 text-yellow-600 animate-spin" />
                            <div className="flex-1">
                                <p className="text-sm text-yellow-800 font-medium">
                                    Calculating your Omni Account address...
                                </p>
                                <p className="text-xs text-yellow-700 mt-1">
                                    Please wait while we fetch the TEE worker address.
                                </p>
                            </div>
                        </div>
                    </div>
                )}

                {/* Email Account Info */}
                <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <div className="flex items-start space-x-3">
                        <Mail className="h-5 w-5 text-blue-600 mt-0.5" />
                        <div className="flex-1">
                            <h4 className="text-sm font-medium text-blue-900">
                                Email Account Setup
                            </h4>
                            <p className="text-sm text-blue-700 mt-1">
                                Your account will be controlled by: <strong>{identifier}</strong>
                            </p>
                            <p className="text-xs text-blue-600 mt-2">
                                The TEE worker will be set as the authorized signer for your account.
                            </p>
                        </div>
                    </div>
                </div>

                {/* Paymaster Selection */}
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                    <div className="space-y-3">
                        <div>
                            <label className="text-sm font-medium text-gray-700 block mb-2">
                                Gas Payment Method
                            </label>
                            <select
                                value={paymasterType}
                                onChange={(e) => setPaymasterType(e.target.value as PaymasterType)}
                                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                disabled={isProcessing}
                            >
                                <option value={PaymasterType.None}>Pay with ETH (Standard)</option>
                                {CONTRACTS.SimplePaymaster.address !== "0x0000000000000000000000000000000000000000" && (
                                    <option value={PaymasterType.Simple}>Use Simple Paymaster (Sponsored)</option>
                                )}
                                {CONTRACTS.ERC20Paymaster.address !== "0x0000000000000000000000000000000000000000" && (
                                    <option value={PaymasterType.ERC20}>Pay with USDC (ERC20 Paymaster)</option>
                                )}
                            </select>
                        </div>

                        {/* Show USDC balance when ERC20 paymaster is selected */}
                        {paymasterType === PaymasterType.ERC20 && (
                            <div className="bg-white border border-gray-200 rounded p-3">
                                <div className="flex justify-between items-center">
                                    <span className="text-sm text-gray-600">USDC Balance:</span>
                                    <span className="text-sm font-medium">
                                        {isLoadingUsdcBalance ? (
                                            <span className="text-gray-400">Loading...</span>
                                        ) : (
                                            `${(Number(usdcBalance) / 1e6).toFixed(2)} USDC`
                                        )}
                                    </span>
                                </div>
                                {usdcBalance === BigInt(0) && !isLoadingUsdcBalance && (
                                    <p className="text-xs text-red-600 mt-2">
                                        Please fund your account with USDC to use this payment method
                                    </p>
                                )}
                            </div>
                        )}

                        {/* Paymaster status */}
                        {paymasterType !== PaymasterType.None && paymasterStatus && !paymasterStatus.isAvailable && (
                            <div className="p-2 bg-yellow-50 border border-yellow-200 rounded">
                                <p className="text-xs text-yellow-800">
                                    {paymasterStatus.error || "Paymaster not available"}
                                </p>
                            </div>
                        )}
                    </div>
                </div>

                {error && (
                    <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                        <div className="flex">
                            <AlertTriangle className="h-5 w-5 text-red-600" />
                            <p className="ml-3 text-sm text-red-700">{error}</p>
                        </div>
                    </div>
                )}

                <button
                    onClick={handleCreateAccount}
                    disabled={isProcessing || !omniAccountAddress}
                    className={`w-full py-3 px-4 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2 ${
                        isProcessing || !omniAccountAddress
                            ? "bg-gray-300 cursor-not-allowed"
                            : "bg-blue-600 hover:bg-blue-700 text-white"
                    }`}
                >
                    {isProcessing ? (
                        <>
                            <Loader2 className="w-5 h-5 animate-spin" />
                            <span>Creating Account...</span>
                        </>
                    ) : (
                        <>
                            <Plus className="w-5 h-5" />
                            <span>Create Email-Controlled Account</span>
                        </>
                    )}
                </button>
            </div>
        );
    }

    // Default wallet UI (existing code)
    return (
        <div className="space-y-6">
            {/* Account Deployment Info */}
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <div className="flex items-start space-x-3">
                    <Info className="h-5 w-5 text-blue-600 mt-0.5" />
                    <div className="flex-1">
                        <h4 className="text-sm font-medium text-blue-900">
                            Ready to Deploy
                        </h4>
                        <p className="text-sm text-blue-700 mt-1">
                            Your smart account will be deployed at:
                        </p>
                        <p className="text-xs font-mono text-blue-800 mt-2 break-all">
                            {omniAccountAddress}
                        </p>
                    </div>
                </div>
            </div>

            {/* Connected Wallet Info */}
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                <div className="flex items-center space-x-3">
                    <Wallet className="h-5 w-5 text-gray-600" />
                    <div className="flex-1">
                        <p className="text-sm text-gray-700">
                            Deploying with wallet:
                        </p>
                        <p className="text-xs font-mono text-gray-800 mt-1">
                            {evmAddress}
                        </p>
                    </div>
                </div>
            </div>

            {/* Paymaster Selection */}
            <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                <div className="space-y-3">
                    <div>
                        <label className="text-sm font-medium text-gray-700 block mb-2">
                            Gas Payment Method
                        </label>
                        <select
                            value={paymasterType}
                            onChange={(e) => setPaymasterType(e.target.value as PaymasterType)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            disabled={isProcessing}
                        >
                            <option value={PaymasterType.None}>Pay with ETH (Standard)</option>
                            {CONTRACTS.SimplePaymaster.address !== "0x0000000000000000000000000000000000000000" && (
                                <option value={PaymasterType.Simple}>Use Simple Paymaster (Sponsored)</option>
                            )}
                            {CONTRACTS.ERC20Paymaster.address !== "0x0000000000000000000000000000000000000000" && (
                                <option value={PaymasterType.ERC20}>Pay with USDC (ERC20 Paymaster)</option>
                            )}
                        </select>
                    </div>

                    {/* Show USDC balance when ERC20 paymaster is selected */}
                    {paymasterType === PaymasterType.ERC20 && (
                        <div className="bg-white border border-gray-200 rounded p-3">
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-gray-600">USDC Balance:</span>
                                <span className="text-sm font-medium">
                                    {isLoadingUsdcBalance ? (
                                        <span className="text-gray-400">Loading...</span>
                                    ) : (
                                        `${(Number(usdcBalance) / 1e6).toFixed(2)} USDC`
                                    )}
                                </span>
                            </div>
                            {usdcBalance === BigInt(0) && !isLoadingUsdcBalance && (
                                <p className="text-xs text-red-600 mt-2">
                                    Please fund your account with USDC to use this payment method
                                </p>
                            )}
                        </div>
                    )}

                    {/* Paymaster status */}
                    {paymasterType !== PaymasterType.None && paymasterStatus && !paymasterStatus.isAvailable && (
                        <div className="p-2 bg-yellow-50 border border-yellow-200 rounded">
                            <p className="text-xs text-yellow-800">
                                {paymasterStatus.error || "Paymaster not available"}
                            </p>
                        </div>
                    )}
                </div>
            </div>

            {error && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                    <div className="flex">
                        <AlertTriangle className="h-5 w-5 text-red-600" />
                        <p className="ml-3 text-sm text-red-700">{error}</p>
                    </div>
                </div>
            )}

            <button
                onClick={handleCreateAccount}
                disabled={isProcessing}
                className={`w-full py-3 px-4 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2 ${
                    isProcessing
                        ? "bg-gray-300 cursor-not-allowed"
                        : "bg-blue-600 hover:bg-blue-700 text-white"
                }`}
            >
                {isProcessing ? (
                    <>
                        <Loader2 className="w-5 h-5 animate-spin" />
                        <span>Creating Account...</span>
                    </>
                ) : (
                    <>
                        <Plus className="w-5 h-5" />
                        <span>Create Omni Account</span>
                    </>
                )}
            </button>
        </div>
    );
}
