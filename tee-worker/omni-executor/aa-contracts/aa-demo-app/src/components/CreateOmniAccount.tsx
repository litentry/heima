import { useState, useEffect } from "react";
import { useAccount, useWalletClient, usePublicClient } from "wagmi";
import {
    Wallet,
    AlertTriangle,
    CheckCircle,
    Loader2,
    Info,
    Plus,
} from "lucide-react";
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
} from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS, PAYMASTER_CONFIG, OwnerType } from "@/lib/constants";

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

    const [isProcessing, setIsProcessing] = useState(false);
    const [accountCreated, setAccountCreated] = useState(false);
    const [error, setError] = useState("");
    const [txHash, setTxHash] = useState("");
    const [usePaymaster, setUsePaymaster] = useState<boolean>(PAYMASTER_CONFIG.enabledByDefault);
    const [paymasterStatus, setPaymasterStatus] = useState<{
        isAvailable: boolean;
        balance: bigint;
        error?: string;
    } | null>(null);

    const truncateAddress = (address: string) => {
        if (!address) return "";
        return `${address.slice(0, 6)}...${address.slice(-4)}`;
    };

    // Check paymaster status when component mounts or when usePaymaster changes
    useEffect(() => {
        const checkPaymaster = async () => {
            if (!publicClient || CONTRACTS.SimplePaymaster.address === "0x0000000000000000000000000000000000000000") {
                return;
            }

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
        };

        if (usePaymaster) {
            checkPaymaster();
        }
    }, [publicClient, usePaymaster]);

    const handleCreateAccount = async () => {
        if (
            !evmAddress ||
            !omniAccountAddress ||
            !walletClient ||
            !publicClient ||
            !chain
        ) {
            setError("Missing wallet connection or smart account address");
            return;
        }

        setIsProcessing(true);
        setError("");

        try {
            // Calculate omni account and client ID bytes32
            const omniAccount = calculateOmniAccount(
                evmAddress,
                DEFAULT_CLIENT_ID,
                "evm",
            );
            const clientIdBytes = stringToBytes(DEFAULT_CLIENT_ID);

            console.log("Creating Omni Account with:", {
                address: evmAddress,
                clientId: DEFAULT_CLIENT_ID,
                omniAccountAddress,
                omniAccount,
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

            // Use the connected wallet as the root signer
            const rootSigner = evmAddress as `0x${string}`;

            // Generate initCode for Omni Account deployment
            const initCode = generateInitCode(
                CONTRACTS.OmniAccountFactory.address,
                omniAccount,
                OwnerType.Evm,
                clientIdBytes,
                rootSigner,
            ) as `0x${string}`;

            console.log("Deployment parameters:", {
                rootSigner,
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
                callData: "0x", // No additional operations needed
                gasParams,
                paymaster: usePaymaster && paymasterStatus?.isAvailable
                    ? {
                        address: CONTRACTS.SimplePaymaster.address,
                        validationGasLimit: PAYMASTER_CONFIG.defaultValidationGasLimit,
                        postOpGasLimit: PAYMASTER_CONFIG.defaultPostOpGasLimit,
                    }
                    : undefined,
            });

            // Sign the UserOperation with Owner signer type
            const signature = await signUserOperation(
                walletClient,
                evmAddress,
                userOp as UserOperation,
                CONTRACTS.EntryPoint.address,
                BigInt(chain.id),
                UserOpSigner.Owner,
            );

            // Update the UserOperation with signature
            const signedUserOp = {
                ...userOp,
                signature,
            } as UserOperation;

            // Convert to PackedUserOperation for EntryPoint v0.7
            const packedUserOp = packUserOperation(signedUserOp);

            console.log("Submitting transaction to create Omni Account...");

            // Simulate first to get better error messages
            try {
                await publicClient.simulateContract({
                    address: CONTRACTS.EntryPoint.address,
                    abi: CONTRACTS.EntryPoint.abi,
                    functionName: "handleOps",
                    args: [[packedUserOp], evmAddress],
                    account: evmAddress,
                });
            } catch (simError: any) {
                console.error("Simulation failed:", simError);
                if (simError.cause?.reason) {
                    throw new Error(`Simulation failed: ${simError.cause.reason}`);
                }
                throw simError;
            }

            // Execute the transaction
            const tx = await walletClient.writeContract({
                address: CONTRACTS.EntryPoint.address,
                abi: CONTRACTS.EntryPoint.abi,
                functionName: "handleOps",
                args: [[packedUserOp], evmAddress] as const,
                chain: walletClient.chain,
                account: walletClient.account!,
            });

            console.log("Transaction hash:", tx);

            // Wait for confirmation
            const receipt = await publicClient.waitForTransactionReceipt({
                hash: tx,
            });

            console.log("Transaction receipt:", receipt);

            if (receipt.status === "success") {
                setTxHash(tx);
                setAccountCreated(true);
                if (onAccountCreated) {
                    onAccountCreated();
                }
            } else {
                throw new Error("Transaction reverted");
            }
        } catch (err) {
            console.error("Account creation failed:", err);
            setError(err instanceof Error ? err.message : "Account creation failed");
        } finally {
            setIsProcessing(false);
        }
    };

    if (!evmAddress) {
        return (
            <div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
                <div className="text-center">
                    <Wallet className="mx-auto h-12 w-12 text-gray-400 mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">
                        Connect Wallet
                    </h3>
                    <p className="text-gray-600">
                        Please connect your wallet to create an Omni Account.
                    </p>
                </div>
            </div>
        );
    }

    if (!isFunded) {
        return (
            <div className="w-full p-6 bg-yellow-50 rounded-lg border border-yellow-200">
                <div className="text-center">
                    <AlertTriangle className="mx-auto h-12 w-12 text-yellow-500 mb-4" />
                    <h3 className="text-lg font-medium text-yellow-800 mb-2">
                        Fund Account First
                    </h3>
                    <p className="text-yellow-700">
                        Please fund your Omni Account with ETH before creating the smart
                        contract.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className="w-full p-6 bg-white rounded-lg shadow-lg">
            <div className="text-center mb-6">
                {accountCreated ? (
                    <CheckCircle className="mx-auto h-12 w-12 text-green-500 mb-4" />
                ) : (
                    <Plus className="mx-auto h-12 w-12 text-blue-500 mb-4" />
                )}
                <h2 className="text-2xl font-bold">
                    {accountCreated
                        ? "Omni Account Created!"
                        : "Create Your Omni Account"}
                </h2>
            </div>

            {accountCreated ? (
                <div className="space-y-4">
                    <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                        <div className="flex items-center mb-2">
                            <CheckCircle className="h-5 w-5 text-green-500 mr-2" />
                            <span className="text-green-700 font-medium">
                                Smart Account Deployed Successfully
                            </span>
                        </div>
                        <p className="text-green-600 text-sm">
                            Your Omni Account has been created with your wallet as the initial
                            root signer:
                            <span className="font-mono text-xs block mt-1 break-all">
                                {evmAddress}
                            </span>
                        </p>
                    </div>

                    {txHash && (
                        <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                            <p className="text-sm text-blue-700">
                                <span className="font-medium">Transaction Hash:</span>
                                <span className="font-mono text-xs block mt-1 break-all">
                                    {txHash}
                                </span>
                            </p>
                        </div>
                    )}

                    <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                        <h3 className="font-medium text-gray-900 mb-2">What's Next?</h3>
                        <ul className="text-sm text-gray-700 space-y-1">
                            <li>• Your Omni Account is now ready to use</li>
                            <li>• You can add additional signers if needed</li>
                            <li>• Create sessions for delegated access</li>
                            <li>• Execute batch transactions via Account Abstraction</li>
                        </ul>
                    </div>
                </div>
            ) : (
                <div className="space-y-6">
                    {/* Information Box */}
                    <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <div className="flex items-start">
                            <Info className="h-5 w-5 text-blue-500 mr-3 mt-0.5" />
                            <div>
                                <h3 className="font-medium text-blue-900 mb-2">
                                    What happens when you create your Omni Account?
                                </h3>
                                <ul className="text-sm text-blue-700 space-y-1">
                                    <li>
                                        • A smart contract wallet is deployed to the blockchain
                                    </li>
                                    <li>
                                        • Your connected wallet ({truncateAddress(evmAddress)})
                                        becomes the initial root signer
                                    </li>
                                    <li>
                                        • Account Abstraction features are enabled for your account
                                    </li>
                                    <li>• You can manage multiple signers and create sessions</li>
                                </ul>
                            </div>
                        </div>
                    </div>

                    {/* Error Message */}
                    {error && (
                        <div className="bg-red-50 border border-red-200 rounded-lg p-3">
                            <div className="flex">
                                <AlertTriangle className="h-5 w-5 text-red-500 mr-2" />
                                <span className="text-sm text-red-700">{error}</span>
                            </div>
                        </div>
                    )}

                    {/* Paymaster Option */}
                    {CONTRACTS.SimplePaymaster.address !== "0x0000000000000000000000000000000000000000" && (
                        <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                            <div className="flex items-center justify-between mb-2">
                                <label className="flex items-center cursor-pointer">
                                    <input
                                        type="checkbox"
                                        checked={usePaymaster}
                                        onChange={(e) => setUsePaymaster(e.target.checked)}
                                        className="mr-3 h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                                    />
                                    <span className="font-medium text-gray-900">
                                        Use Paymaster (Gas Sponsorship)
                                    </span>
                                </label>
                            </div>
                            {usePaymaster && paymasterStatus && (
                                <div className="text-sm text-gray-600 ml-7">
                                    {paymasterStatus.isAvailable ? (
                                        <span className="text-green-600">
                                            ✓ Paymaster available (Balance: {(paymasterStatus.balance / BigInt("1000000000000000000")).toString()} ETH)
                                        </span>
                                    ) : (
                                        <span className="text-red-600">
                                            ✗ {paymasterStatus.error || "Paymaster not available"}
                                        </span>
                                    )}
                                </div>
                            )}
                            <p className="text-xs text-gray-500 mt-2 ml-7">
                                When enabled, the paymaster will cover gas fees for this transaction
                            </p>
                        </div>
                    )}

                    {/* Create Button */}
                    <button
                        onClick={handleCreateAccount}
                        disabled={isProcessing}
                        className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-medium py-3 px-4 rounded-lg transition-colors flex items-center justify-center"
                    >
                        {isProcessing ? (
                            <>
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                                Creating Account...
                            </>
                        ) : (
                            <>
                                <Wallet className="h-4 w-4 mr-2" />
                                Create Omni Account
                            </>
                        )}
                    </button>
                </div>
            )}
        </div>
    );
}

