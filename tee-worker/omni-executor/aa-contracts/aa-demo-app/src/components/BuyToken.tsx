import { useState, useEffect } from "react";
import { useChainId, usePublicClient, useAccount } from "wagmi";
import { ShoppingCart, AlertCircle, CheckCircle, Loader2, Clock } from "lucide-react";
import { submitUserOpTest, getTEEWorkerAddress } from "@/lib/tee-worker-client";
import { CONTRACTS, DEFAULT_CLIENT_ID, OwnerType, HYPERLIQUID_CORE_CONFIG } from "@/lib/constants";
import { createUserOperation, packUserOperation, toSerializablePackedUserOperation, generateInitCode, stringToBytes, calculateOmniAccount } from "@/lib/aa-utils";
import { buildSpotBuyCallData, getSpotAssetId, generateCloid } from "@/lib/hypercore-utils";
import { useAuth } from "@/contexts/AuthContext";

interface BuyTokenProps {
    omniAccountAddress: string;
    omniAccountHash: string;
    accountExists: boolean;
}

export function BuyToken({ omniAccountAddress, omniAccountHash, accountExists }: BuyTokenProps) {
    const chainId = useChainId();
    const publicClient = usePublicClient();
    const { address: evmAddress } = useAccount();
    const { authType } = useAuth();

    const [tokenName, setTokenName] = useState<string>("PURR");
    const [size, setSize] = useState<string>("100");
    const [price, setPrice] = useState<string>("5.2");
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [success, setSuccess] = useState<{
        txHash: string | null;
    } | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);
        setSuccess(null);

        // Validation
        if (!tokenName) {
            setError("Please enter a token name");
            return;
        }

        if (!size || parseFloat(size) <= 0) {
            setError("Invalid size");
            return;
        }

        if (!price || parseFloat(price) <= 0) {
            setError("Invalid price");
            return;
        }

        setIsSubmitting(true);

        try {
            if (!publicClient) {
                throw new Error("Public client not available");
            }

            // Hardcoded asset ID for PURR/USDC spot trading pair
            const assetId = 10000;

            console.log(`Token ${tokenName} has asset ID: ${assetId}`);

            // Build the callData for the spot buy order
            const callData = buildSpotBuyCallData(
                assetId,
                parseFloat(size),
                parseFloat(price)
            );

            console.log("Generated callData:", callData);

            // Determine if we need to include initCode
            let initCode: `0x${string}` = "0x";

            if (!accountExists) {
                console.log("Account does not exist, generating initCode...");

                let calculatedOmniAccount: `0x${string}`;
                let rootSigner: `0x${string}`;
                let ownerType: number;

                if (authType === "email") {
                    calculatedOmniAccount = omniAccountHash as `0x${string}`;
                    rootSigner = await getTEEWorkerAddress(omniAccountHash) as `0x${string}`;
                    ownerType = OwnerType.Email;
                } else {
                    if (!evmAddress) {
                        throw new Error("Wallet not connected");
                    }
                    calculatedOmniAccount = calculateOmniAccount(
                        evmAddress,
                        DEFAULT_CLIENT_ID,
                        "evm",
                    );
                    rootSigner = evmAddress as `0x${string}`;
                    ownerType = OwnerType.Evm;
                }

                const clientIdBytes = stringToBytes(DEFAULT_CLIENT_ID);

                initCode = generateInitCode(
                    CONTRACTS.OmniAccountFactory.address,
                    calculatedOmniAccount,
                    ownerType,
                    clientIdBytes,
                    rootSigner,
                ) as `0x${string}`;
            }

            // Get current nonce
            const nonce = (await publicClient.readContract({
                address: CONTRACTS.EntryPoint.address,
                abi: CONTRACTS.EntryPoint.abi,
                functionName: "getNonce",
                args: [omniAccountAddress as `0x${string}`, BigInt(0)],
            })) as bigint;

            // Create UserOperation with the buy order callData
            const userOp = createUserOperation({
                sender: omniAccountAddress as `0x${string}`,
                nonce,
                callData,
                initCode,
                gasParams: {
                    callGasLimit: BigInt(100000),
                    verificationGasLimit: BigInt(1000000),
                    preVerificationGas: BigInt(100000),
                    maxFeePerGas: BigInt(120000000),
                    maxPriorityFeePerGas: BigInt(0),
                },
                paymaster: "0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912000000000000000000000000000f4240000000000000000000000000000186a0",
            });

            // Pack and serialize the UserOperation
            const packedOp = packUserOperation(userOp);
            const serializableOp = toSerializablePackedUserOperation(packedOp);

            console.log("Submitting UserOp:", serializableOp);

            // Submit the UserOperation
            const response = await submitUserOpTest(
                [serializableOp],
                chainId || 998,
                0, // wallet_index
                omniAccountHash,
                DEFAULT_CLIENT_ID
            );

            console.log("Response:", response);

            setSuccess({
                txHash: response.transaction_hash,
            });

        } catch (err) {
            console.error("Buy token error:", err);
            setError(err instanceof Error ? err.message : "Failed to buy token");
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <div className="bg-white rounded-lg shadow-lg p-6">
            <div className="flex items-center gap-2 mb-6">
                <ShoppingCart className="w-5 h-5 text-blue-600" />
                <h2 className="text-xl font-semibold">Buy Token on Hyperliquid</h2>
            </div>

            <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                        Token Name
                    </label>
                    <input
                        type="text"
                        value={tokenName}
                        onChange={(e) => setTokenName(e.target.value.toUpperCase())}
                        disabled={isSubmitting}
                        className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100"
                        placeholder="PURR"
                    />
                </div>

                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Size
                        </label>
                        <input
                            type="number"
                            step="0.00000001"
                            value={size}
                            onChange={(e) => setSize(e.target.value)}
                            disabled={isSubmitting}
                            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100"
                            placeholder="100"
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                            Limit Price (USDC)
                        </label>
                        <input
                            type="number"
                            step="0.00000001"
                            value={price}
                            onChange={(e) => setPrice(e.target.value)}
                            disabled={isSubmitting}
                            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100"
                            placeholder="5.2"
                        />
                    </div>
                </div>

                {/* Estimated Total */}
                {size && price && (
                    <div className="p-3 bg-blue-50 rounded-lg">
                        <div className="flex justify-between text-sm">
                            <span className="text-gray-600">Estimated Total:</span>
                            <span className="font-medium text-gray-900">
                                {(parseFloat(size) * parseFloat(price)).toFixed(2)} USDC
                            </span>
                        </div>
                    </div>
                )}

                {/* Error Message */}
                {error && (
                    <div className="p-4 bg-red-50 border border-red-200 rounded-lg flex items-start gap-3">
                        <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                        <div>
                            <p className="text-sm font-medium text-red-800">Error</p>
                            <p className="text-sm text-red-600 mt-1">{error}</p>
                        </div>
                    </div>
                )}

                {/* Success Message */}
                {success && (
                    <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                        <div className="flex items-start gap-3">
                            <CheckCircle className="w-5 h-5 text-green-600 flex-shrink-0 mt-0.5" />
                            <div>
                                <p className="text-sm font-medium text-green-800">Order Submitted Successfully!</p>
                                {success.txHash && (
                                    <p className="text-xs text-green-700 mt-1 font-mono break-all">
                                        Tx: {success.txHash}
                                    </p>
                                )}
                            </div>
                        </div>
                    </div>
                )}

                {/* Submit Button */}
                <button
                    type="submit"
                    disabled={isSubmitting || !omniAccountAddress}
                    className="w-full px-4 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 font-medium"
                >
                    {isSubmitting ? (
                        <>
                            <Loader2 className="w-5 h-5 animate-spin" />
                            Submitting Order...
                        </>
                    ) : (
                        <>
                            <ShoppingCart className="w-5 h-5" />
                            Buy {tokenName || "Token"}
                        </>
                    )}
                </button>
            </form>

            <div className="mt-4 p-3 bg-gray-50 rounded-lg">
                <p className="text-xs text-gray-600">
                    This will create a limit buy order on Hyperliquid Testnet using your OmniAccount.
                    The order will be executed through the CoreWriter contract.
                </p>
            </div>
        </div>
    );
}
