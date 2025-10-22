import { useState, useEffect, useCallback } from "react";
import { useChainId, usePublicClient, useAccount } from "wagmi";
import { DollarSign, AlertCircle, CheckCircle, Loader2, TrendingUp } from "lucide-react";
import { requestLoanTest, getTEEWorkerAddress } from "@/lib/tee-worker-client";
import { HYPERLIQUID_CORE_CONFIG, DEFAULT_CLIENT_ID, CONTRACTS, PAYMASTER_CONFIG, OwnerType } from "@/lib/constants";
import { createUserOperation, packUserOperation, toSerializablePackedUserOperation, estimateUserOpGasFromWorker, generateInitCode, stringToBytes, calculateOmniAccount } from "@/lib/aa-utils";
import { useAuth } from "@/contexts/AuthContext";

interface RequestLoanProps {
    omniAccountAddress: string;
    omniAccountHash: string;
    onAccountCreated?: () => void;
}

interface AssetInfo {
    coin: string;
    balance: string;
}

interface AssetPrice {
    coin: string;
    price: number;
}

export function RequestLoan({ omniAccountAddress, omniAccountHash, onAccountCreated }: RequestLoanProps) {
    console.log("RequestLoan component rendering with address:", omniAccountAddress);

    const chainId = useChainId();
    const publicClient = usePublicClient();
    const { address: evmAddress } = useAccount();
    const { authType, identifier } = useAuth();
    const [collateralTicker, setCollateralTicker] = useState<string>("");
    const [collateralSize, setCollateralSize] = useState<string>("");
    const [lendingRatio, setLendingRatio] = useState<number>(80);
    const [availableAssets, setAvailableAssets] = useState<AssetInfo[]>([]);
    const [assetPrices, setAssetPrices] = useState<Map<string, number>>(new Map());
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [accountExists, setAccountExists] = useState<boolean>(false);
    const [success, setSuccess] = useState<{
        spotSellCloid: string;
        hedgeOpenCloid: string;
        usdcReceived: string;
        spotSellTxHash: string | null;
        hedgeOpenTxHash: string | null;
    } | null>(null);

    // Fetch available assets from Hyperliquid
    const fetchAvailableAssets = useCallback(async () => {
        if (!omniAccountAddress) {
            console.log("No address to fetch assets for");
            return;
        }

        console.log("Starting fetch for address:", omniAccountAddress);

        try {
            const response = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "spotClearinghouseState",
                    user: omniAccountAddress,
                }),
            });

            console.log("Fetch response status:", response.status);

            if (response.ok) {
                const data = await response.json();
                console.log("Spot clearinghouse data:", data);
                console.log("data.type:", data.type);
                console.log("data.balances:", data.balances);

                // Check if balances array exists (note: no "type" field in the response)
                if (data.balances && Array.isArray(data.balances)) {
                    console.log("Processing balances array with length:", data.balances.length);
                    const assets = data.balances
                        .filter((asset: any) => {
                            const total = parseFloat(asset.total);
                            console.log(`Asset ${asset.coin}: total=${asset.total}, parsed=${total}, passes filter: ${total > 0}`);
                            return total > 0;
                        })
                        .map((asset: any) => ({
                            coin: asset.coin,
                            balance: asset.total,
                        }));
                    console.log("Filtered assets for collateral:", assets);
                    setAvailableAssets(assets);
                    if (assets.length > 0 && !collateralTicker) {
                        console.log("Setting initial collateral ticker to:", assets[0].coin);
                        setCollateralTicker(assets[0].coin);
                    }
                } else {
                    console.log("Unexpected data structure - no balances array:", data);
                }
            } else {
                console.error("Failed to fetch assets, status:", response.status);
            }
        } catch (err) {
            console.error("Error fetching assets:", err);
        }
    }, [omniAccountAddress, collateralTicker]);

    // Fetch current prices for all assets
    const fetchPrices = useCallback(async () => {
        try {
            const response = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "allMids",
                }),
            });

            if (response.ok) {
                const data = await response.json();
                console.log("Price data:", data);

                const priceMap = new Map<string, number>();

                // data is an object with coin names as keys and prices as values
                Object.entries(data).forEach(([coin, price]) => {
                    priceMap.set(coin, parseFloat(price as string));
                });

                setAssetPrices(priceMap);
            }
        } catch (err) {
            console.error("Error fetching prices:", err);
        }
    }, []);

    // Check if account exists on-chain
    useEffect(() => {
        const checkAccountExists = async () => {
            if (!omniAccountAddress || !publicClient) {
                setAccountExists(false);
                return;
            }

            try {
                const code = await publicClient.getCode({
                    address: omniAccountAddress as `0x${string}`,
                });
                const exists = !!code && code !== "0x";
                setAccountExists(exists);
                console.log("Account exists:", exists);
            } catch (error) {
                console.error("Error checking account existence:", error);
                setAccountExists(false);
            }
        };

        checkAccountExists();
    }, [omniAccountAddress, publicClient]);

    useEffect(() => {
        console.log("RequestLoan useEffect triggered:", { omniAccountAddress });
        if (omniAccountAddress) {
            console.log("Fetching assets for address:", omniAccountAddress);
            fetchAvailableAssets();
            fetchPrices();
        } else {
            console.log("No omniAccountAddress provided");
        }
    }, [omniAccountAddress, fetchAvailableAssets, fetchPrices]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);
        setSuccess(null);

        // Validation
        if (!collateralTicker) {
            setError("Please select a collateral asset");
            return;
        }

        if (!collateralSize || parseFloat(collateralSize) <= 0) {
            setError("Invalid collateral size");
            return;
        }

        if (lendingRatio < 1 || lendingRatio > 100) {
            setError("Lending ratio must be between 1 and 100");
            return;
        }

        const selectedAsset = availableAssets.find(a => a.coin === collateralTicker);
        if (selectedAsset && parseFloat(collateralSize) > parseFloat(selectedAsset.balance)) {
            setError("Insufficient collateral balance");
            return;
        }

        setIsSubmitting(true);

        try {
            if (!publicClient) {
                throw new Error("Public client not available");
            }

            // Determine if we need to include initCode (account doesn't exist yet)
            let initCode: `0x${string}` = "0x";

            if (!accountExists) {
                console.log("Account does not exist, generating initCode...");

                // Calculate parameters for account creation
                let calculatedOmniAccount: `0x${string}`;
                let rootSigner: `0x${string}`;
                let ownerType: number;

                if (authType === "email") {
                    // For email accounts, use the provided omniAccountHash
                    calculatedOmniAccount = omniAccountHash as `0x${string}`;
                    // Get TEE worker as root signer
                    rootSigner = await getTEEWorkerAddress(omniAccountHash) as `0x${string}`;
                    ownerType = OwnerType.Email;
                } else {
                    // For wallet accounts
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

                // Generate initCode for account deployment
                initCode = generateInitCode(
                    CONTRACTS.OmniAccountFactory.address,
                    calculatedOmniAccount,
                    ownerType,
                    clientIdBytes,
                    rootSigner,
                ) as `0x${string}`;

                console.log("Generated initCode for new account:", {
                    calculatedOmniAccount,
                    rootSigner,
                    ownerType,
                    initCodeLength: initCode.length,
                });
            }

            // Get current nonce
            const nonce = (await publicClient.readContract({
                address: CONTRACTS.EntryPoint.address,
                abi: CONTRACTS.EntryPoint.abi,
                functionName: "getNonce",
                args: [omniAccountAddress as `0x${string}`, BigInt(0)],
            })) as bigint;

            // Create UserOperation with exact settings from working example (0xaff...)
            const dummyUserOp = createUserOperation({
                sender: omniAccountAddress as `0x${string}`,
                nonce,
                callData: "0x",
                initCode, // Include initCode if account doesn't exist
                gasParams: {
                    callGasLimit: BigInt(100000),      // 0x186a0 from example
                    verificationGasLimit: BigInt(1000000), // 0xf4240 from example
                    preVerificationGas: BigInt(100000),   // 0x186a0 from example
                    maxFeePerGas: BigInt(120000000),      // 0x7270e00 from example
                    maxPriorityFeePerGas: BigInt(0),      // 0x0 from example
                },
                // Use exact paymasterAndData hex string from working example
                paymaster: "0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912000000000000000000000000000f4240000000000000000000000000000186a0",
            });

            // Pack and serialize the UserOperation
            const packedOp = packUserOperation(dummyUserOp);
            const serializableOp = toSerializablePackedUserOperation(packedOp);

            const response = await requestLoanTest(
                serializableOp,
                chainId || 998, // Default to Hyperliquid Testnet
                0, // wallet_index
                omniAccountHash,
                DEFAULT_CLIENT_ID,
                collateralTicker,
                collateralSize,
                lendingRatio
            );

            setSuccess({
                spotSellCloid: response.spot_sell_cloid,
                hedgeOpenCloid: response.hedge_open_cloid,
                usdcReceived: response.usdc_received,
                spotSellTxHash: response.spot_sell_tx_hash,
                hedgeOpenTxHash: response.hedge_open_tx_hash,
            });

            // If account was just created, notify parent
            if (!accountExists && onAccountCreated) {
                onAccountCreated();
                setAccountExists(true);
            }

            // Clear form
            setCollateralSize("");
            // Refresh assets
            fetchAvailableAssets();
        } catch (err: any) {
            console.error("Error requesting loan:", err);
            setError(err.message || "Failed to request loan");
        } finally {
            setIsSubmitting(false);
        }
    };

    const formatBalance = (balance: string): string => {
        const num = parseFloat(balance);
        if (isNaN(num)) return "0";
        return num.toFixed(6).replace(/\.?0+$/, "");
    };

    const estimatedUSDC = (): string => {
        if (!collateralSize || parseFloat(collateralSize) <= 0) return "0";

        const size = parseFloat(collateralSize);
        const ratio = lendingRatio / 100;

        // For stablecoins (USDC, USDT), the value is ~1:1
        if (collateralTicker === "USDC" || collateralTicker === "USDT") {
            return (size * ratio).toFixed(2);
        }

        // Get real-time price from Hyperliquid
        const price = assetPrices.get(collateralTicker);
        if (price) {
            const estimatedValue = size * price * ratio;
            return estimatedValue.toFixed(2);
        }

        return "Loading price...";
    };

    return (
        <div className="w-full p-6 bg-white rounded-lg shadow-lg">
            <div className="flex items-center mb-6">
                <TrendingUp className="h-6 w-6 text-green-500 mr-2" />
                <h2 className="text-2xl font-bold">Request Loan</h2>
            </div>

            <p className="text-gray-600 mb-4">
                Use your Hyperliquid assets as collateral to borrow USDC. The system will sell your collateral on spot and open a hedging position.
            </p>

            {/* Account creation notice */}
            {!accountExists && (
                <div className="mb-4 p-4 bg-blue-50 border border-blue-200 rounded-lg">
                    <div className="flex items-start">
                        <AlertCircle className="h-5 w-5 text-blue-600 mr-2 mt-0.5 flex-shrink-0" />
                        <div className="text-sm text-blue-700">
                            <p className="font-medium mb-1">First Transaction</p>
                            <p>
                                Your Omni Account will be automatically created with this transaction.
                                {authType === "wallet" ? " Your wallet will be set as the root signer." : " The TEE worker will be set as the root signer."}
                            </p>
                        </div>
                    </div>
                </div>
            )}

            {/* Debug info */}
            <div className="mb-4 p-2 bg-gray-50 rounded text-xs font-mono">
                <p>Address: {omniAccountAddress || "Not set"}</p>
                <p>Account exists: {accountExists ? "Yes" : "No"}</p>
                <p>Available assets: {availableAssets.length}</p>
                <p>Chain ID: {chainId}</p>
            </div>

            {error && (
                <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
                    <div className="flex items-center">
                        <AlertCircle className="h-5 w-5 text-red-500 mr-2" />
                        <p className="text-red-700 text-sm">{error}</p>
                    </div>
                </div>
            )}

            {success && (
                <div className="mb-6 p-4 bg-green-50 border border-green-200 rounded-lg">
                    <div className="flex items-center mb-3">
                        <CheckCircle className="h-5 w-5 text-green-500 mr-2" />
                        <p className="text-green-700 font-semibold">Loan Request Successful!</p>
                    </div>
                    <div className="space-y-2 text-sm">
                        <div className="flex justify-between">
                            <span className="text-green-600">USDC Received:</span>
                            <span className="font-mono font-semibold text-green-700">
                                ${formatBalance(success.usdcReceived)}
                            </span>
                        </div>
                        <div className="flex justify-between">
                            <span className="text-green-600">Spot Sell Order:</span>
                            <span className="font-mono text-xs text-green-700">
                                {success.spotSellCloid}
                            </span>
                        </div>
                        <div className="flex justify-between">
                            <span className="text-green-600">Hedge Position:</span>
                            <span className="font-mono text-xs text-green-700">
                                {success.hedgeOpenCloid}
                            </span>
                        </div>
                        {success.spotSellTxHash && (
                            <div className="flex justify-between">
                                <span className="text-green-600">Spot Tx:</span>
                                <span className="font-mono text-xs text-green-700">
                                    {success.spotSellTxHash}
                                </span>
                            </div>
                        )}
                        {success.hedgeOpenTxHash && (
                            <div className="flex justify-between">
                                <span className="text-green-600">Hedge Tx:</span>
                                <span className="font-mono text-xs text-green-700">
                                    {success.hedgeOpenTxHash}
                                </span>
                            </div>
                        )}
                    </div>
                </div>
            )}

            <form onSubmit={handleSubmit} className="space-y-4">
                {/* Collateral Asset Selection */}
                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                        Collateral Asset
                    </label>
                    {availableAssets.length === 0 ? (
                        <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg text-sm text-yellow-700">
                            No assets available. Fund your Hyperliquid account first.
                        </div>
                    ) : (
                        <select
                            value={collateralTicker}
                            onChange={(e) => setCollateralTicker(e.target.value)}
                            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500"
                            required
                        >
                            {availableAssets.map((asset) => (
                                <option key={asset.coin} value={asset.coin}>
                                    {asset.coin} (Balance: {formatBalance(asset.balance)})
                                </option>
                            ))}
                        </select>
                    )}
                </div>

                {/* Collateral Size */}
                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                        Collateral Size
                    </label>
                    <input
                        type="number"
                        step="0.000001"
                        value={collateralSize}
                        onChange={(e) => setCollateralSize(e.target.value)}
                        placeholder="0.0"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-green-500"
                        required
                    />
                    {collateralTicker && (
                        <p className="text-sm text-gray-500 mt-1">
                            Available:{" "}
                            {formatBalance(
                                availableAssets.find((a) => a.coin === collateralTicker)?.balance || "0"
                            )}{" "}
                            {collateralTicker}
                        </p>
                    )}
                </div>

                {/* Lending Ratio */}
                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                        Lending Ratio: {lendingRatio}%
                    </label>
                    <input
                        type="range"
                        min="1"
                        max="100"
                        value={lendingRatio}
                        onChange={(e) => setLendingRatio(parseInt(e.target.value))}
                        className="w-full"
                    />
                    <div className="flex justify-between text-xs text-gray-500 mt-1">
                        <span>Conservative (1%)</span>
                        <span>Aggressive (100%)</span>
                    </div>
                </div>

                {/* Estimated Loan Amount */}
                <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
                    <div className="flex items-center justify-between">
                        <span className="text-sm text-blue-700">Estimated USDC Loan:</span>
                        <span className="text-xl font-bold text-blue-900">
                            ~${estimatedUSDC()}
                        </span>
                    </div>
                    <p className="text-xs text-blue-600 mt-2">
                        * Actual amount depends on market prices and execution
                    </p>
                </div>

                <button
                    type="submit"
                    disabled={isSubmitting || availableAssets.length === 0}
                    className="w-full bg-green-600 hover:bg-green-700 text-white font-medium py-3 px-4 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
                >
                    {isSubmitting ? (
                        <>
                            <Loader2 className="animate-spin h-5 w-5 mr-2" />
                            Processing Loan Request...
                        </>
                    ) : (
                        <>
                            <DollarSign className="h-5 w-5 mr-2" />
                            Request Loan
                        </>
                    )}
                </button>
            </form>

            <div className="mt-6 p-4 bg-purple-50 border border-purple-200 rounded-lg">
                <div className="flex">
                    <AlertCircle className="h-5 w-5 text-purple-500 mr-2 mt-0.5" />
                    <div className="text-sm text-purple-700">
                        <p className="font-medium mb-1">How it works:</p>
                        <ul className="list-disc list-inside space-y-1">
                            <li>Your collateral is sold on the Hyperliquid spot market</li>
                            <li>A hedging position is opened to protect against price movements</li>
                            <li>USDC is transferred to your account</li>
                            <li>You can repay the loan by closing the hedge and buying back collateral</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    );
}
