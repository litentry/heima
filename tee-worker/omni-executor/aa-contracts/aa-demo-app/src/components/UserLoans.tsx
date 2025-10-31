import React, { useState, useEffect } from "react";
import { AlertCircle, RefreshCw, FileText, ArrowLeftRight, CheckCircle, Loader2, X, ChevronDown, ChevronUp, ExternalLink } from "lucide-react";
import { useChainId, usePublicClient, useAccount } from "wagmi";
import { queryLoanTest, paybackLoanTest, getTEEWorkerAddress, type LoanRecord, LoanState } from "@/lib/tee-worker-client";
import { CONTRACTS, DEFAULT_CLIENT_ID, OwnerType, HYPERLIQUID_CORE_CONFIG } from "@/lib/constants";
import { createUserOperation, packUserOperation, toSerializablePackedUserOperation, generateInitCode, stringToBytes, calculateOmniAccount } from "@/lib/aa-utils";
import { useAuth } from "@/contexts/AuthContext";

interface UserLoansProps {
    omniAccountHash: string;
    omniAccountAddress?: string;
}

interface LoanRecordWithNonce extends LoanRecord {
    nonce: string;
}

export function UserLoans({ omniAccountHash, omniAccountAddress }: UserLoansProps) {
    const chainId = useChainId();
    const publicClient = usePublicClient();
    const { address: evmAddress } = useAccount();
    const { authType } = useAuth();

    const [loans, setLoans] = useState<LoanRecordWithNonce[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [liquidationPrices, setLiquidationPrices] = useState<Map<string, string>>(new Map());
    const [accountExists, setAccountExists] = useState<boolean>(false);
    const [expandedLoans, setExpandedLoans] = useState<Set<string>>(new Set());

    // Payback modal state
    const [showPaybackModal, setShowPaybackModal] = useState(false);
    const [selectedLoan, setSelectedLoan] = useState<LoanRecordWithNonce | null>(null);
    const [isPayingBack, setIsPayingBack] = useState(false);
    const [paybackError, setPaybackError] = useState<string | null>(null);
    const [paybackSuccess, setPaybackSuccess] = useState<{
        collateralTicker: string;
        collateralSize: string;
        hedgeCloseTxHash: string | null;
        spotBuyTxHash: string | null;
    } | null>(null);
    const [estimatedCollateralReturn, setEstimatedCollateralReturn] = useState<string | null>(null);
    const [isCalculatingReturn, setIsCalculatingReturn] = useState(false);

    const fetchLoans = async () => {
        if (!omniAccountHash) return;

        setIsLoading(true);
        setError(null);

        try {
            // Query all loan records for this omni account
            const response = await queryLoanTest(omniAccountHash);

            // Convert the records object to an array with nonces
            const loanArray: LoanRecordWithNonce[] = Object.entries(response.records).map(
                ([nonce, record]) => ({
                    ...record,
                    nonce,
                })
            );

            // Sort by nonce (descending, newest first)
            loanArray.sort((a, b) => Number(b.nonce) - Number(a.nonce));

            setLoans(loanArray);

            // Fetch liquidation prices from Hyperliquid
            if (omniAccountAddress) {
                fetchLiquidationPrices();
            }
        } catch (err) {
            console.error("Failed to fetch loans:", err);
            setError(err instanceof Error ? err.message : "Failed to fetch loans");
            setLoans([]);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        fetchLoans();
    }, [omniAccountHash]);

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
            } catch (error) {
                console.error("Error checking account existence:", error);
                setAccountExists(false);
            }
        };

        checkAccountExists();
    }, [omniAccountAddress, publicClient]);

    // Calculate estimated collateral return based on current market conditions
    const calculateCollateralReturn = async (loan: LoanRecordWithNonce) => {
        if (!omniAccountAddress) return;

        setIsCalculatingReturn(true);
        try {
            // 1. Fetch current spot price for the collateral asset
            const priceResponse = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    type: "spotMeta",
                }),
            });

            let spotPrice = 0;
            let tokenIndex = -1;

            // First, get token metadata to find the index
            if (priceResponse.ok) {
                const priceData = await priceResponse.json();

                const ticker = loan.collateral_ticker.toUpperCase();

                // Find token in the tokens array
                if (priceData.tokens && Array.isArray(priceData.tokens)) {
                    const tokenInfo = priceData.tokens.find((t: any) =>
                        t.name?.toUpperCase() === ticker
                    );

                    if (tokenInfo) {
                        console.log("Found token info:", tokenInfo);
                        tokenIndex = tokenInfo.index;
                    }
                }
            }

            // Now fetch actual market prices using allMids
            console.log("Token index:", tokenIndex);
            if (tokenIndex >= 0) {
                const midsResponse = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        type: "allMids",
                    }),
                });

                console.log("Mids response status:", midsResponse.status);

                if (midsResponse.ok) {
                    const midsData = await midsResponse.json();
                    console.log("All mids data sample:", Object.keys(midsData).slice(0, 10));
                    console.log("Total pairs:", Object.keys(midsData).length);

                    // midsData is an object like { "HYPE": "25.67", "BTC": "98000", ... }
                    // Keys are token symbols, values are prices in USDC
                    const ticker = loan.collateral_ticker;

                    console.log("Looking for ticker:", ticker);
                    console.log("Exact match exists?", ticker in midsData);
                    console.log("Value:", midsData[ticker]);

                    if (midsData[ticker]) {
                        spotPrice = parseFloat(midsData[ticker]);
                        console.log(`Found price for ${ticker}:`, spotPrice);
                    } else {
                        // Try case variations
                        console.log("Trying case variations...");
                        const tickerUpper = ticker.toUpperCase();
                        const tickerLower = ticker.toLowerCase();

                        if (midsData[tickerUpper]) {
                            spotPrice = parseFloat(midsData[tickerUpper]);
                            console.log(`Found price for ${tickerUpper}:`, spotPrice);
                        } else if (midsData[tickerLower]) {
                            spotPrice = parseFloat(midsData[tickerLower]);
                            console.log(`Found price for ${tickerLower}:`, spotPrice);
                        } else {
                            const allKeys = Object.keys(midsData);
                            const matchingKeys = allKeys.filter(k => k.toUpperCase() === tickerUpper);
                            console.log("Matching keys:", matchingKeys);

                            if (matchingKeys.length > 0) {
                                spotPrice = parseFloat(midsData[matchingKeys[0]]);
                                console.log(`Found price for ${matchingKeys[0]}:`, spotPrice);
                            } else {
                                console.warn(`Could not find price for ${ticker}`);
                            }
                        }
                    }
                }
            } else {
                console.warn("Token index not found, cannot fetch price");
            }

            // 2. Use loan record data - it already contains all the info we need
            const usdcForPerp = parseFloat(loan.usdc_for_perp);
            const positionSize = parseFloat(loan.position_size);
            const hedgeCloid = loan.cloids.find(([name]) => name === "hedge_open")?.[1];

            // 3. Fetch current position P&L using cloid if hedge is still open
            let unrealizedPnl = 0;
            if (positionSize !== 0 && hedgeCloid) {
                // Position is still open, fetch unrealized P&L
                const positionResponse = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        type: "clearinghouseState",
                        user: omniAccountAddress,
                    }),
                });

                if (positionResponse.ok) {
                    const positionData = await positionResponse.json();
                    console.log("Looking for hedge position with cloid:", hedgeCloid);

                    // Find position by ticker (since API doesn't filter by cloid)
                    // We verify it's the right position by checking the size matches
                    const position = positionData.assetPositions?.find((p: any) => {
                        const coin = p.position?.coin;
                        const size = Math.abs(parseFloat(p.position?.szi || "0"));
                        const expectedSize = Math.abs(positionSize);

                        // Match by ticker and position size
                        return coin === loan.collateral_ticker &&
                               Math.abs(size - expectedSize) < 0.0001; // Small tolerance for floating point
                    });

                    if (position?.position?.unrealizedPnl) {
                        unrealizedPnl = parseFloat(position.position.unrealizedPnl);
                        console.log("Found matching position, unrealized P&L:", unrealizedPnl);
                    } else {
                        console.warn("Could not find matching position for hedge cloid:", hedgeCloid);
                    }
                }
            }

            // 4. Calculate available USDC for this loan
            // Total USDC = USDC in spot (usdc_loaned) + USDC in perp + P&L from hedge
            const usdcLoaned = parseFloat(loan.usdc_loaned);

            // If hedge is closed (positionSize = 0): usdcForPerp already includes realized P&L
            // If hedge is open: usdcForPerp + unrealizedPnl
            const availableUsdc = usdcLoaned + usdcForPerp + unrealizedPnl;

            // 5. Calculate collateral that can be bought back
            let estimatedCollateral = 0;
            if (spotPrice > 0 && availableUsdc > 0) {
                estimatedCollateral = availableUsdc / spotPrice;
            }

            console.log("Collateral return calculation:", {
                collateralTicker: loan.collateral_ticker,
                spotPrice,
                usdcLoaned,
                usdcForPerp,
                positionSize,
                unrealizedPnl,
                availableUsdc,
                estimatedCollateral,
                breakdown: {
                    usdcInSpot: usdcLoaned,
                    usdcInPerp: usdcForPerp,
                    hedgePnL: unrealizedPnl,
                    total: availableUsdc,
                },
                loanData: {
                    usdc_for_perp: loan.usdc_for_perp,
                    position_size: loan.position_size,
                    usdc_loaned: loan.usdc_loaned,
                    usdc_sold: loan.usdc_sold,
                    collateral_size: loan.collateral_size,
                },
            });

            setEstimatedCollateralReturn(estimatedCollateral > 0 ? estimatedCollateral.toFixed(6) : "0");
        } catch (error) {
            console.error("Error calculating collateral return:", error);
            setEstimatedCollateralReturn(null);
        } finally {
            setIsCalculatingReturn(false);
        }
    };

    const handlePaybackClick = (loan: LoanRecordWithNonce) => {
        setSelectedLoan(loan);
        setPaybackError(null);
        setPaybackSuccess(null);
        setEstimatedCollateralReturn(null);
        setShowPaybackModal(true);

        // Calculate estimated return
        calculateCollateralReturn(loan);
    };

    const handlePaybackSubmit = async () => {
        if (!selectedLoan || !omniAccountAddress) return;

        setIsPayingBack(true);
        setPaybackError(null);
        setPaybackSuccess(null);

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
                    rootSigner,
                ) as `0x${string}`;

                console.log("Generated initCode for payback:", {
                    omniAccountHash,
                    calculatedOmniAccount,
                    rootSigner,
                    ownerType,
                    authType,
                    evmAddress,
                    note: "rootSigner is TEE worker for both email and wallet auth",
                });
            }

            // Get current nonce
            const nonce = (await publicClient.readContract({
                address: CONTRACTS.EntryPoint.address,
                abi: CONTRACTS.EntryPoint.abi,
                functionName: "getNonce",
                args: [omniAccountAddress as `0x${string}`, BigInt(0)],
            })) as bigint;

            // Create UserOperation
            const dummyUserOp = createUserOperation({
                sender: omniAccountAddress as `0x${string}`,
                nonce,
                callData: "0x",
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

            const packedOp = packUserOperation(dummyUserOp);
            const serializableOp = toSerializablePackedUserOperation(packedOp);

            const response = await paybackLoanTest(
                serializableOp,
                chainId || 998,
                0, // wallet_index
                omniAccountHash,
                DEFAULT_CLIENT_ID,
                parseInt(selectedLoan.nonce),
                "1" // Fixed minimum account value
            );

            setPaybackSuccess({
                collateralTicker: response.collateral_ticker,
                collateralSize: response.collateral_size,
                hedgeCloseTxHash: response.hedge_close_tx_hash,
                spotBuyTxHash: response.spot_buy_tx_hash,
            });

            // Refresh loans after successful payback
            setTimeout(() => {
                fetchLoans();
            }, 2000);

        } catch (err) {
            console.error("Payback error:", err);
            setPaybackError(err instanceof Error ? err.message : "Failed to payback loan");
        } finally {
            setIsPayingBack(false);
        }
    };

    const formatNumber = (value: string) => {
        try {
            const num = parseFloat(value);
            if (isNaN(num)) return value;
            return num.toLocaleString("en-US", {
                minimumFractionDigits: 2,
                maximumFractionDigits: 8,
            });
        } catch {
            return value;
        }
    };

    // Helper to toggle loan expansion
    const toggleLoanExpansion = (nonce: string) => {
        setExpandedLoans(prev => {
            const newSet = new Set(prev);
            if (newSet.has(nonce)) {
                newSet.delete(nonce);
            } else {
                newSet.add(nonce);
            }
            return newSet;
        });
    };

    // Helper to get cloid by name from the cloids array
    const getCloid = (loan: LoanRecordWithNonce, name: string): string => {
        const cloid = loan.cloids.find(([n, _]) => n === name);
        return cloid ? cloid[1] : "N/A";
    };

    // Helper to get transaction by name from the txs array
    const getTx = (loan: LoanRecordWithNonce, name: string): string | null => {
        const tx = loan.txs.find(([n, _]) => n === name);
        return tx ? tx[1] : null;
    };

    // Helper to format transaction hash for display (shortened)
    const formatTxHash = (hash: string): string => {
        if (hash.length <= 16) return hash;
        return `${hash.slice(0, 8)}...${hash.slice(-8)}`;
    };

    // Helper to get explorer URL for transaction
    const getExplorerUrl = (txHash: string): string => {
        return `${HYPERLIQUID_CORE_CONFIG.explorerUrl}/tx/${txHash}`;
    };

    // Helper to map transaction codes to human-readable names
    const getTransactionDisplayName = (txName: string): string => {
        const nameMap: Record<string, string> = {
            'spot_sell': 'Spot Sell',
            'spot_buy': 'Spot Buy',
            'hedge_open': 'Hedge Open',
            'hedge_close': 'Hedge Close',
            'hedge_cancel': 'Hedge Cancel',
            'usd_transfer': 'USD Transfer to Perp',
            'to_perp_move': 'Move to Perp',
            'to_spot_move': 'Move to Spot',
        };
        return nameMap[txName] || txName.split('_').map(word =>
            word.charAt(0).toUpperCase() + word.slice(1)
        ).join(' ');
    };

    // Helper to check if loan can be paid back (not in final states)
    const canPayback = (state: LoanState): boolean => {
        const finalStates = [
            LoanState.HedgeClosed,
            LoanState.ToSpotMoved,
            LoanState.SpotBought,
        ];
        return !finalStates.includes(state);
    };

    // Helper to format loan state
    const formatState = (state: LoanState): string => {
        return state.replace(/([A-Z])/g, ' $1').trim();
    };

    // Helper to get state badge color
    const getStateBadgeColor = (state: LoanState): string => {
        switch (state) {
            case LoanState.SpotSold:
                return "bg-yellow-100 text-yellow-800";
            case LoanState.ToPerpMoved:
                return "bg-blue-100 text-blue-800";
            case LoanState.HedgeOpened:
                return "bg-green-100 text-green-800";
            case LoanState.HedgeClosed:
                return "bg-orange-100 text-orange-800";
            case LoanState.ToSpotMoved:
                return "bg-purple-100 text-purple-800";
            case LoanState.SpotBought:
                return "bg-gray-100 text-gray-800";
            default:
                return "bg-gray-100 text-gray-800";
        }
    };

    // Fetch liquidation prices from Hyperliquid API
    const fetchLiquidationPrices = async () => {
        if (!omniAccountAddress) return;

        try {
            const response = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "clearinghouseState",
                    user: omniAccountAddress,
                }),
            });

            if (response.ok) {
                const data = await response.json();
                console.log("Clearinghouse state:", data);

                // Extract liquidation prices from positions
                const liqPrices = new Map<string, string>();

                if (data.assetPositions && Array.isArray(data.assetPositions)) {
                    data.assetPositions.forEach((position: any) => {
                        if (position.position && position.position.liquidationPx) {
                            const coin = position.position.coin;
                            const liqPx = position.position.liquidationPx;
                            liqPrices.set(coin, liqPx);
                        }
                    });
                }

                setLiquidationPrices(liqPrices);
            }
        } catch (error) {
            console.error("Error fetching liquidation prices:", error);
        }
    };

    // Get liquidation price for a loan
    const getLiquidationPrice = (loan: LoanRecordWithNonce): string => {
        const ticker = loan.collateral_ticker;
        return liquidationPrices.get(ticker) || "N/A";
    };

    return (
        <div className="bg-white rounded-lg shadow-lg p-6">
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-2">
                    <FileText className="w-5 h-5 text-blue-600" />
                    <h2 className="text-xl font-semibold">Your Loans</h2>
                </div>
                <button
                    onClick={fetchLoans}
                    disabled={isLoading}
                    className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded-lg transition-colors disabled:opacity-50"
                >
                    <RefreshCw className={`w-4 h-4 ${isLoading ? "animate-spin" : ""}`} />
                    Refresh
                </button>
            </div>

            {error && (
                <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg flex items-start gap-3">
                    <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                    <div>
                        <p className="text-sm font-medium text-red-800">Error loading loans</p>
                        <p className="text-sm text-red-600 mt-1">{error}</p>
                    </div>
                </div>
            )}

            {isLoading && loans.length === 0 ? (
                <div className="text-center py-12">
                    <RefreshCw className="w-8 h-8 text-gray-400 mx-auto mb-3 animate-spin" />
                    <p className="text-gray-500">Loading loans...</p>
                </div>
            ) : loans.length === 0 ? (
                <div className="text-center py-12">
                    <FileText className="w-12 h-12 text-gray-300 mx-auto mb-3" />
                    <p className="text-gray-500">No loans found</p>
                    <p className="text-sm text-gray-400 mt-1">
                        Request a loan to see it appear here
                    </p>
                </div>
            ) : (
                <div className="overflow-x-auto">
                    <table className="w-full">
                        <thead>
                            <tr className="border-b border-gray-200">
                                <th className="text-left py-3 px-4 text-sm font-semibold text-gray-700">
                                    Nonce
                                </th>
                                <th className="text-left py-3 px-4 text-sm font-semibold text-gray-700">
                                    State
                                </th>
                                <th className="text-left py-3 px-4 text-sm font-semibold text-gray-700">
                                    Collateral
                                </th>
                                <th className="text-right py-3 px-4 text-sm font-semibold text-gray-700">
                                    Collateral Size
                                </th>
                                <th className="text-right py-3 px-4 text-sm font-semibold text-gray-700">
                                    Position Size
                                </th>
                                <th className="text-right py-3 px-4 text-sm font-semibold text-gray-700">
                                    USDC Sold
                                </th>
                                <th className="text-right py-3 px-4 text-sm font-semibold text-gray-700">
                                    USDC Loaned
                                </th>
                                <th className="text-left py-3 px-4 text-sm font-semibold text-gray-700">
                                    Spot Sell CLOID
                                </th>
                                <th className="text-left py-3 px-4 text-sm font-semibold text-gray-700">
                                    Hedge Open CLOID
                                </th>
                                <th className="text-right py-3 px-4 text-sm font-semibold text-gray-700">
                                    Liquidation Price
                                </th>
                                <th className="text-center py-3 px-4 text-sm font-semibold text-gray-700">
                                    Action
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {loans.map((loan) => {
                                const isExpanded = expandedLoans.has(loan.nonce);
                                return (
                                    <React.Fragment key={loan.nonce}>
                                        <tr className="border-b border-gray-100 hover:bg-gray-50 transition-colors">
                                            <td className="py-4 px-4 text-sm text-gray-900 font-mono">
                                                <div className="flex items-center gap-2">
                                                    <button
                                                        onClick={() => toggleLoanExpansion(loan.nonce)}
                                                        className="p-1 hover:bg-gray-200 rounded transition-colors"
                                                        title={isExpanded ? "Hide transactions" : "Show transactions"}
                                                    >
                                                        {isExpanded ? (
                                                            <ChevronUp className="w-4 h-4 text-gray-600" />
                                                        ) : (
                                                            <ChevronDown className="w-4 h-4 text-gray-600" />
                                                        )}
                                                    </button>
                                                    {loan.nonce}
                                                </div>
                                            </td>
                                            <td className="py-4 px-4">
                                                <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStateBadgeColor(loan.state)}`}>
                                                    {formatState(loan.state)}
                                                </span>
                                            </td>
                                            <td className="py-4 px-4">
                                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                                                    {loan.collateral_ticker}
                                                </span>
                                            </td>
                                            <td className="py-4 px-4 text-sm text-right text-gray-900 font-mono">
                                                {formatNumber(loan.collateral_size)}
                                            </td>
                                            <td className="py-4 px-4 text-sm text-right text-purple-600 font-medium font-mono">
                                                {formatNumber(loan.position_size)}
                                            </td>
                                            <td className="py-4 px-4 text-sm text-right text-gray-900 font-mono">
                                                ${formatNumber(loan.usdc_sold)}
                                            </td>
                                            <td className="py-4 px-4 text-sm text-right font-medium text-green-600 font-mono">
                                                ${formatNumber(loan.usdc_loaned)}
                                            </td>
                                            <td className="py-4 px-4 text-sm text-gray-600 font-mono truncate max-w-[150px]">
                                                {getCloid(loan, "spot_sell")}
                                            </td>
                                            <td className="py-4 px-4 text-sm text-gray-600 font-mono truncate max-w-[150px]">
                                                {getCloid(loan, "hedge_open")}
                                            </td>
                                            <td className="py-4 px-4 text-sm text-right font-medium text-red-600 font-mono">
                                                {(() => {
                                                    const liqPrice = getLiquidationPrice(loan);
                                                    return liqPrice !== "N/A" ? `$${formatNumber(liqPrice)}` : liqPrice;
                                                })()}
                                            </td>
                                            <td className="py-4 px-4 text-center">
                                                {canPayback(loan.state) ? (
                                                    <button
                                                        onClick={() => handlePaybackClick(loan)}
                                                        disabled={!omniAccountAddress}
                                                        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-white bg-orange-600 hover:bg-orange-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                                    >
                                                        <ArrowLeftRight className="w-4 h-4" />
                                                        Payback
                                                    </button>
                                                ) : (
                                                    <span className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-gray-500">
                                                        <CheckCircle className="w-4 h-4 text-green-500" />
                                                        Completed
                                                    </span>
                                                )}
                                            </td>
                                        </tr>
                                        {isExpanded && (
                                            <tr className="bg-gray-50">
                                                <td colSpan={11} className="py-4 px-8">
                                                    <div className="space-y-3">
                                                        <h4 className="text-sm font-semibold text-gray-700 flex items-center gap-2">
                                                            <FileText className="w-4 h-4" />
                                                            Transaction History
                                                        </h4>
                                                        {loan.txs.length === 0 ? (
                                                            <p className="text-sm text-gray-500 italic">No transactions recorded yet</p>
                                                        ) : (
                                                            <div className="grid grid-cols-1 gap-2">
                                                                {loan.txs.map(([name, hash], idx) => (
                                                                    <div
                                                                        key={`${loan.nonce}-${idx}`}
                                                                        className="flex items-center justify-between p-3 bg-white rounded-lg border border-gray-200"
                                                                    >
                                                                        <div className="flex items-center gap-3">
                                                                            <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-indigo-100 text-indigo-800 min-w-[140px]">
                                                                                {getTransactionDisplayName(name)}
                                                                            </span>
                                                                            <span className="text-sm font-mono text-gray-700">
                                                                                {formatTxHash(hash)}
                                                                            </span>
                                                                        </div>
                                                                        <a
                                                                            href={getExplorerUrl(hash)}
                                                                            target="_blank"
                                                                            rel="noopener noreferrer"
                                                                            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded transition-colors"
                                                                        >
                                                                            View in Explorer
                                                                            <ExternalLink className="w-3 h-3" />
                                                                        </a>
                                                                    </div>
                                                                ))}
                                                            </div>
                                                        )}
                                                    </div>
                                                </td>
                                            </tr>
                                        )}
                                    </React.Fragment>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            )}

            {loans.length > 0 && (
                <div className="mt-4 pt-4 border-t border-gray-200">
                    <div className="flex items-center justify-between text-sm text-gray-600">
                        <span>Total loans: {loans.length}</span>
                        <span className="text-xs text-gray-500">
                            Latest loan: Nonce {loans[0]?.nonce}
                        </span>
                    </div>
                </div>
            )}

            {/* Payback Modal */}
            {showPaybackModal && selectedLoan && (
                <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
                    <div className="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
                        <div className="flex items-center justify-between mb-4">
                            <h3 className="text-xl font-semibold text-gray-900">Payback Loan</h3>
                            <button
                                onClick={() => setShowPaybackModal(false)}
                                disabled={isPayingBack}
                                className="text-gray-400 hover:text-gray-600 transition-colors"
                            >
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        {/* Loan Details */}
                        <div className="mb-6 p-4 bg-gray-50 rounded-lg space-y-2">
                            <div className="flex justify-between text-sm">
                                <span className="text-gray-600">Nonce:</span>
                                <span className="font-mono text-gray-900">{selectedLoan.nonce}</span>
                            </div>
                            <div className="flex justify-between text-sm">
                                <span className="text-gray-600">Original Collateral:</span>
                                <span className="font-medium text-gray-500">
                                    {formatNumber(selectedLoan.collateral_size)} {selectedLoan.collateral_ticker}
                                </span>
                            </div>
                            <div className="flex justify-between text-sm items-center">
                                <span className="text-gray-600">Estimated Collateral Return:</span>
                                <span className="font-bold text-blue-600">
                                    {isCalculatingReturn ? (
                                        <span className="flex items-center gap-2">
                                            <Loader2 className="w-4 h-4 animate-spin" />
                                            Calculating...
                                        </span>
                                    ) : estimatedCollateralReturn ? (
                                        `${formatNumber(estimatedCollateralReturn)} ${selectedLoan.collateral_ticker}`
                                    ) : (
                                        "Unable to calculate"
                                    )}
                                </span>
                            </div>
                            <div className="flex justify-between text-sm">
                                <span className="text-gray-600">Position Size:</span>
                                <span className="font-medium text-purple-600">
                                    {formatNumber(selectedLoan.position_size)}
                                </span>
                            </div>
                            <div className="flex justify-between text-sm">
                                <span className="text-gray-600">USDC Loaned:</span>
                                <span className="font-medium text-green-600">
                                    ${formatNumber(selectedLoan.usdc_loaned)}
                                </span>
                            </div>
                        </div>

                        {/* Error Message */}
                        {paybackError && (
                            <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg flex items-start gap-3">
                                <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                                <div>
                                    <p className="text-sm font-medium text-red-800">Payback Failed</p>
                                    <p className="text-sm text-red-600 mt-1">{paybackError}</p>
                                </div>
                            </div>
                        )}

                        {/* Success Message */}
                        {paybackSuccess && (
                            <div className="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg">
                                <div className="flex items-start gap-3 mb-3">
                                    <CheckCircle className="w-5 h-5 text-green-600 flex-shrink-0 mt-0.5" />
                                    <div>
                                        <p className="text-sm font-medium text-green-800">Loan Paid Back Successfully!</p>
                                        <p className="text-sm text-green-600 mt-1">
                                            Collateral: {formatNumber(paybackSuccess.collateralSize)} {paybackSuccess.collateralTicker}
                                        </p>
                                    </div>
                                </div>
                                {paybackSuccess.hedgeCloseTxHash && (
                                    <div className="text-xs text-green-700 space-y-1">
                                        <p className="font-medium">Transaction Hashes:</p>
                                        <p className="font-mono break-all">Hedge Close: {paybackSuccess.hedgeCloseTxHash}</p>
                                        {paybackSuccess.spotBuyTxHash && (
                                            <p className="font-mono break-all">Spot Buy: {paybackSuccess.spotBuyTxHash}</p>
                                        )}
                                    </div>
                                )}
                            </div>
                        )}

                        {/* Action Buttons */}
                        <div className="flex gap-3">
                            <button
                                onClick={() => setShowPaybackModal(false)}
                                disabled={isPayingBack}
                                className="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handlePaybackSubmit}
                                disabled={isPayingBack || !!paybackSuccess}
                                className="flex-1 px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
                            >
                                {isPayingBack ? (
                                    <>
                                        <Loader2 className="w-4 h-4 animate-spin" />
                                        Processing...
                                    </>
                                ) : (
                                    <>
                                        <ArrowLeftRight className="w-4 h-4" />
                                        Confirm Payback
                                    </>
                                )}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
