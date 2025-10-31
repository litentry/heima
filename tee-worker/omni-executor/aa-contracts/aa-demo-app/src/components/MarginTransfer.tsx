import { useState } from "react";
import { useChainId, usePublicClient, useAccount } from "wagmi";
import { AlertCircle, CheckCircle, Loader2, ArrowRightLeft } from "lucide-react";
import { submitUserOpTest, getTEEWorkerAddress } from "@/lib/tee-worker-client";
import { CONTRACTS, DEFAULT_CLIENT_ID, OwnerType } from "@/lib/constants";
import { createUserOperation, packUserOperation, toSerializablePackedUserOperation, generateInitCode, stringToBytes, calculateOmniAccount } from "@/lib/aa-utils";
import { buildSpotToPerpTransferCallData, buildPerpToSpotTransferCallData } from "@/lib/hypercore-utils";
import { useAuth } from "@/contexts/AuthContext";

interface MarginTransferProps {
    omniAccountAddress: string;
    omniAccountHash: string;
    accountExists: boolean;
    callData: `0x${string}` | null;
    actionDescription: string;
    onComplete?: () => void;
    onCancel?: () => void;
}

export function MarginTransfer({
    omniAccountAddress,
    omniAccountHash,
    accountExists,
    callData,
    actionDescription,
    onComplete,
    onCancel
}: MarginTransferProps) {
    const chainId = useChainId();
    const publicClient = usePublicClient();
    const { address: evmAddress } = useAccount();
    const { authType } = useAuth();

    const [isSubmitting, setIsSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<{
        txHash: string | null;
    } | null>(null);

    const handleSubmit = async () => {
        if (!callData) {
            setError("No transfer data provided");
            return;
        }

        setError(null);
        setSuccess(null);
        setIsSubmitting(true);

        try {
            if (!publicClient) {
                throw new Error("Public client not available");
            }

            // Determine if we need to include initCode
            let initCode: `0x${string}` = "0x";

            if (!accountExists) {
                console.log("Account does not exist, generating initCode...");

                // IMPORTANT: Always use the omniAccountHash prop, not recalculated value
                const calculatedOmniAccount: `0x${string}` = omniAccountHash as `0x${string}`;
                let ownerType: number;

                // CRITICAL FIX: The rootSigner must be the TEE worker address for BOTH
                // email and wallet auth because the TEE worker signs the UserOperation.
                const rootSigner = await getTEEWorkerAddress(omniAccountHash) as `0x${string}`;

                if (authType === "email") {
                    ownerType = OwnerType.Email;
                } else {
                    if (!evmAddress) {
                        throw new Error("Wallet not connected");
                    }
                    ownerType = OwnerType.Evm;
                }

                const clientIdBytes = stringToBytes(DEFAULT_CLIENT_ID);

                initCode = generateInitCode(
                    CONTRACTS.OmniAccountFactory.address,
                    calculatedOmniAccount,
                    ownerType,
                    clientIdBytes,
                    rootSigner
                );

                console.log("Generated initCode:", {
                    omniAccountHash,
                    calculatedOmniAccount,
                    rootSigner,
                    ownerType,
                    authType,
                    evmAddress,
                    initCode,
                    note: "rootSigner is TEE worker for both email and wallet auth",
                });
            }

            // Get the account nonce
            const nonce = await publicClient.readContract({
                address: accountExists ? omniAccountAddress as `0x${string}` : CONTRACTS.EntryPoint.address,
                abi: accountExists
                    ? CONTRACTS.OmniAccountImplementation.abi
                    : CONTRACTS.EntryPoint.abi,
                functionName: accountExists ? "getNonce" : "getNonce",
                args: accountExists
                    ? []
                    : [omniAccountAddress as `0x${string}`, BigInt(0)],
            });

            console.log("Account nonce:", nonce);

            // Create UserOperation with paymaster
            const userOp = await createUserOperation({
                sender: omniAccountAddress as `0x${string}`,
                nonce: BigInt(nonce.toString()),
                initCode,
                callData: callData,
                gasParams: {
                    callGasLimit: BigInt(100000),
                    verificationGasLimit: BigInt(1000000),
                    preVerificationGas: BigInt(100000),
                    maxFeePerGas: BigInt(120000000),
                    maxPriorityFeePerGas: BigInt(0),
                },
                // Use paymaster to pay for gas fees
                paymaster: "0x6255B9F4A4E80BC20eE389fD35DE9D2c029D5912000000000000000000000000000f4240000000000000000000000000000186a0",
            });

            console.log("Created UserOperation:", userOp);

            // Pack and serialize the UserOperation
            const packedUserOp = packUserOperation(userOp);
            const serializableUserOp = toSerializablePackedUserOperation(packedUserOp);

            console.log("Packed UserOperation:", packedUserOp);
            console.log("Serializable UserOperation:", serializableUserOp);

            // Submit to TEE worker
            const result = await submitUserOpTest(
                [serializableUserOp],
                chainId,
                0, // wallet_index
                omniAccountHash,
                DEFAULT_CLIENT_ID
            );

            console.log("TEE worker response:", result);

            setSuccess({
                txHash: result.transaction_hash,
            });

            // Call onComplete callback
            onComplete?.();

        } catch (err: any) {
            console.error("Error submitting margin transfer:", err);
            setError(err.message || "Failed to submit transfer");
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div className="bg-white rounded-lg shadow-xl p-6 max-w-md w-full">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
                    <ArrowRightLeft className="h-5 w-5 text-blue-500" />
                    Confirm Transfer
                </h2>

                <div className="mb-6">
                    <p className="text-gray-700">{actionDescription}</p>
                </div>

                {error && (
                    <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
                        <div className="flex items-center">
                            <AlertCircle className="h-5 w-5 text-red-500 mr-2 flex-shrink-0" />
                            <p className="text-red-700 text-sm">{error}</p>
                        </div>
                    </div>
                )}

                {success && (
                    <div className="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg">
                        <div className="flex items-center mb-2">
                            <CheckCircle className="h-5 w-5 text-green-500 mr-2 flex-shrink-0" />
                            <p className="text-green-700 font-semibold">Transfer Submitted!</p>
                        </div>
                        {success.txHash && (
                            <p className="text-xs text-green-600 break-all mt-2">
                                TX: {success.txHash}
                            </p>
                        )}
                        <p className="text-xs text-gray-600 mt-3">
                            ⏱️ The transfer action is being bridged to Hyperliquid Core.
                            Please wait a few moments and refresh the balances to see the updated amounts.
                        </p>
                    </div>
                )}

                <div className="flex gap-3">
                    <button
                        onClick={onCancel}
                        disabled={isSubmitting}
                        className="flex-1 px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition-colors disabled:opacity-50"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={handleSubmit}
                        disabled={isSubmitting || !!success}
                        className="flex-1 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                    >
                        {isSubmitting ? (
                            <>
                                <Loader2 className="h-4 w-4 animate-spin" />
                                Submitting...
                            </>
                        ) : success ? (
                            "Completed"
                        ) : (
                            "Confirm Transfer"
                        )}
                    </button>
                </div>
            </div>
        </div>
    );
}
