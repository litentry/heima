import { useState, useEffect } from "react";
import { usePublicClient, useWalletClient, useAccount } from "wagmi";
import { Shield, CheckCircle, AlertCircle, Loader2 } from "lucide-react";
import { encodeFunctionData } from "viem";
import { CONTRACTS, PAYMASTER_CONFIG } from "@/lib/constants";
import { getTEEWorkerAddress } from "@/lib/tee-worker-client";
import {
    createUserOperation,
    signUserOperation,
    packUserOperation,
    UserOpSigner,
    type UserOperation,
    checkPaymasterStatus,
} from "@/lib/aa-utils";

interface AuthorizeTEEWorkerProps {
    omniAccountAddress?: string;
    omniAccountHash?: string;
    isDeployed?: boolean;
    onWorkerAuthorized?: (workerAddress: string) => void;
    onComplete?: () => void;
}

export function AuthorizeTEEWorker({
    omniAccountAddress,
    omniAccountHash,
    isDeployed,
    onWorkerAuthorized,
    onComplete,
}: AuthorizeTEEWorkerProps) {
    const { address: evmAddress } = useAccount();
    const publicClient = usePublicClient();
    const { data: walletClient } = useWalletClient();
    const [isAuthorizing, setIsAuthorizing] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [workerAddress, setWorkerAddress] = useState<string | null>(null);
    const [isAddingSigner, setIsAddingSigner] = useState(false);
    const [isCompleted, setIsCompleted] = useState(false);
    const [usePaymaster, setUsePaymaster] = useState<boolean>(PAYMASTER_CONFIG.enabledByDefault);
    const [paymasterStatus, setPaymasterStatus] = useState<{
        isAvailable: boolean;
        balance: bigint;
        error?: string;
    } | null>(null);

    // Check paymaster status
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

        if (usePaymaster && isDeployed) {
            checkPaymaster();
        }
    }, [publicClient, usePaymaster, isDeployed]);

    const handleAuthorize = async () => {
        if (
            !omniAccountAddress ||
            !omniAccountHash ||
            !publicClient ||
            !walletClient ||
            !evmAddress ||
            !isDeployed
        )
            return;

        setIsAuthorizing(true);
        setError(null);

        try {
            // Step 1: Get TEE worker address
            const teeWorkerAddress = await getTEEWorkerAddress(omniAccountHash);

            setWorkerAddress(teeWorkerAddress);

            // Step 2: Add the TEE worker as an authorized signer
            setIsAddingSigner(true);

            // First encode the addRootSigner call
            const addSignerData = encodeFunctionData({
                abi: CONTRACTS.OmniAccountImplementation.abi,
                functionName: "addRootSigner",
                args: [teeWorkerAddress as `0x${string}`],
            });

            // Then wrap it in an execute call (the account calls itself)
            const callData = encodeFunctionData({
                abi: [
                    {
                        name: "execute",
                        type: "function",
                        inputs: [
                            { name: "target", type: "address" },
                            { name: "value", type: "uint256" },
                            { name: "data", type: "bytes" },
                        ],
                        outputs: [],
                        stateMutability: "nonpayable",
                    },
                ],
                functionName: "execute",
                args: [omniAccountAddress as `0x${string}`, BigInt(0), addSignerData],
            });

            // Get current nonce for the account
            const nonce = (await publicClient.readContract({
                address: CONTRACTS.EntryPoint.address,
                abi: CONTRACTS.EntryPoint.abi,
                functionName: "getNonce",
                args: [omniAccountAddress as `0x${string}`, BigInt(0)],
            })) as bigint;

            // Create UserOperation
            const userOp = createUserOperation({
                sender: omniAccountAddress as `0x${string}`,
                nonce,
                callData,
                initCode: "0x", // Account already deployed
                paymaster: usePaymaster && paymasterStatus?.isAvailable
                    ? {
                        address: CONTRACTS.SimplePaymaster.address,
                        validationGasLimit: PAYMASTER_CONFIG.defaultValidationGasLimit,
                        postOpGasLimit: PAYMASTER_CONFIG.defaultPostOpGasLimit,
                    }
                    : undefined,
            });

            // Sign UserOperation with RootKey signer type
            const signature = await signUserOperation(
                walletClient,
                evmAddress,
                userOp as UserOperation,
                CONTRACTS.EntryPoint.address,
                BigInt(await publicClient.getChainId()),
                UserOpSigner.RootKey,
            );

            userOp.signature = signature;

            // Convert to PackedUserOperation for EntryPoint v0.7
            const packedUserOp = packUserOperation(userOp as UserOperation);

            // Send UserOperation via EntryPoint
            const hash = await walletClient.writeContract({
                address: CONTRACTS.EntryPoint.address,
                abi: CONTRACTS.EntryPoint.abi,
                functionName: "handleOps",
                args: [[packedUserOp], evmAddress] as const,
                chain: walletClient.chain,
                account: walletClient.account!,
            });

            // Wait for transaction
            const receipt = await publicClient.waitForTransactionReceipt({ hash });
            console.log("Transaction receipt:", receipt);

            // Check if transaction was successful
            if (receipt.status !== "success") {
                throw new Error("Transaction failed");
            }

            // Verify the signer was actually added
            const isSignerAdded = await publicClient.readContract({
                address: omniAccountAddress as `0x${string}`,
                abi: CONTRACTS.OmniAccountImplementation.abi,
                functionName: "isRootSigner",
                args: [teeWorkerAddress as `0x${string}`],
            });

            if (isSignerAdded) {
                setIsCompleted(true);
                if (onWorkerAuthorized) {
                    onWorkerAuthorized(teeWorkerAddress);
                }
                if (onComplete) {
                    onComplete();
                }
            } else {
                throw new Error(
                    "Transaction succeeded but signer was not added. The UserOperation may have reverted."
                );
            }
        } catch (err) {
            console.error("Error authorizing TEE worker:", err);
            setError(err instanceof Error ? err.message : "Failed to authorize TEE worker");
        } finally {
            setIsAuthorizing(false);
            setIsAddingSigner(false);
        }
    };

    if (!omniAccountAddress) {
        return null;
    }

    if (!isDeployed) {
        return (
            <div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
                <div className="text-center">
                    <AlertCircle className="mx-auto h-12 w-12 text-gray-400 mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">
                        Account Not Deployed
                    </h3>
                    <p className="text-gray-600">
                        The Omni Account needs to be deployed before authorizing the TEE worker.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className="w-full p-6 bg-white rounded-lg shadow-lg">
            <div className="flex items-center mb-6">
                <Shield className="h-6 w-6 text-purple-500 mr-2" />
                <h2 className="text-2xl font-bold">Add TEE Worker Signer</h2>
            </div>

            {!isCompleted ? (
                <>
                    <p className="text-gray-600 mb-6">
                        Add the TEE worker as an authorized signer to your Omni Account.
                        This allows the worker to execute transactions securely on your behalf.
                    </p>

                    {error && (
                        <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
                            <div className="flex items-center">
                                <AlertCircle className="h-5 w-5 text-red-500 mr-2" />
                                <p className="text-red-700">{error}</p>
                            </div>
                        </div>
                    )}

                    <button
                        onClick={handleAuthorize}
                        disabled={isAuthorizing || isAddingSigner}
                        className="w-full px-6 py-3 bg-purple-600 hover:bg-purple-700 text-white font-medium rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
                    >
                        {isAuthorizing ? (
                            <>
                                <Loader2 className="animate-spin h-5 w-5 mr-2" />
                                {isAddingSigner ? "Adding TEE Worker as Signer..." : "Getting TEE Worker Address..."}
                            </>
                        ) : (
                            <>
                                <Shield className="h-5 w-5 mr-2" />
                                Add TEE Worker as Signer
                            </>
                        )}
                    </button>

                    {workerAddress && !isAddingSigner && (
                        <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-lg">
                            <p className="text-sm text-blue-700">
                                TEE Worker Address: <span className="font-mono">{workerAddress}</span>
                            </p>
                            <p className="text-sm text-blue-700 mt-1">
                                Adding as authorized signer...
                            </p>
                        </div>
                    )}
                </>
            ) : (
                <div className="text-center">
                    <CheckCircle className="mx-auto h-16 w-16 text-green-500 mb-4" />
                    <h3 className="text-xl font-semibold text-green-700 mb-2">
                        TEE Worker Authorized!
                    </h3>
                    <p className="text-gray-600 mb-4">
                        The TEE worker has been successfully added as an authorized signer.
                    </p>
                    {workerAddress && (
                        <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                            <p className="text-sm text-green-700">
                                Worker Address: <span className="font-mono">{workerAddress}</span>
                            </p>
                        </div>
                    )}
                </div>
            )}

            <div className="mt-6 p-4 bg-purple-50 border border-purple-200 rounded-lg">
                <div className="flex">
                    <AlertCircle className="h-5 w-5 text-purple-500 mr-2 mt-0.5" />
                    <div className="text-sm text-purple-700">
                        <p className="font-medium mb-1">What happens next?</p>
                        <p>
                            The TEE worker can now execute transactions on your behalf. It will
                            use its secure enclave to protect your transaction data while
                            performing swaps and other operations.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
}
