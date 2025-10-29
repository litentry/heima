import { useState, useEffect } from "react";
import { AlertCircle, RefreshCw, Wallet, TrendingUp, ShoppingCart, DollarSign, ArrowRightLeft } from "lucide-react";
import { HYPERLIQUID_CORE_CONFIG } from "@/lib/constants";
import { buildSpotToPerpTransferCallData, buildPerpToSpotTransferCallData } from "@/lib/hypercore-utils";

interface HyperliquidBalancesProps {
    omniAccountAddress: string;
    onTransferAction?: (callData: `0x${string}`, actionDescription: string) => void;
}

interface AssetBalance {
    coin: string;
    hold: string;
    total: string;
    token: number;
    entryNtl: string;
}

interface SpotBalanceResponse {
    balances: AssetBalance[];
}

interface Position {
    coin: string;
    szi: string; // Position size
    entryPx: string; // Entry price
    positionValue: string;
    unrealizedPnl: string;
    liquidationPx: string | null; // Liquidation price (can be null)
    leverage: {
        type: string;
        value: number;
    };
    marginUsed: string;
}

interface OpenOrder {
    coin: string;
    side: string; // "B" for buy, "A" for sell
    limitPx: string; // Limit price
    sz: string; // Size
    oid: number; // Order ID
    timestamp: number;
    origSz: string; // Original size
}

interface ClearinghouseState {
    assetPositions: Array<{
        position: Position;
        type: string;
    }>;
    crossMarginSummary: {
        accountValue: string;
        totalMarginUsed: string;
        totalNtlPos: string;
        totalRawUsd: string;
    };
    marginSummary: {
        accountValue: string;
        totalMarginUsed: string;
        totalNtlPos: string;
        totalRawUsd: string;
    };
    withdrawable: string;
}

export function HyperliquidBalances({ omniAccountAddress, onTransferAction }: HyperliquidBalancesProps) {
    const [balances, setBalances] = useState<AssetBalance[]>([]);
    const [positions, setPositions] = useState<Position[]>([]);
    const [openOrders, setOpenOrders] = useState<OpenOrder[]>([]);
    const [accountSummary, setAccountSummary] = useState<any>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [spotTransferAmount, setSpotTransferAmount] = useState<string>("");
    const [perpTransferAmount, setPerpTransferAmount] = useState<string>("");

    const fetchBalances = async () => {
        if (!omniAccountAddress) return;

        setIsLoading(true);
        setError(null);

        try {
            // Fetch spot balances
            const spotResponse = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "spotClearinghouseState",
                    user: omniAccountAddress,
                }),
            });

            if (!spotResponse.ok) {
                throw new Error(`HTTP error! status: ${spotResponse.status}`);
            }

            const spotData: any = await spotResponse.json();

            console.log("[HyperliquidBalances] Spot data received:", spotData);

            if (spotData.balances && Array.isArray(spotData.balances)) {
                console.log("[HyperliquidBalances] Setting balances:", spotData.balances);
                // Filter out balances with zero total
                const nonZeroBalances = spotData.balances.filter((b: AssetBalance) => parseFloat(b.total) > 0);
                console.log("[HyperliquidBalances] Non-zero balances:", nonZeroBalances);
                setBalances(nonZeroBalances);
            } else {
                console.log("[HyperliquidBalances] No balances found in spot data");
                setBalances([]);
            }

            // Fetch perpetual positions and account info
            const perpResponse = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "clearinghouseState",
                    user: omniAccountAddress,
                }),
            });

            if (perpResponse.ok) {
                const perpData: ClearinghouseState = await perpResponse.json();

                // Extract positions
                const activePositions = perpData.assetPositions
                    .map(ap => ap.position)
                    .filter(p => parseFloat(p.szi) !== 0); // Only show non-zero positions

                setPositions(activePositions);
                setAccountSummary(perpData.crossMarginSummary || perpData.marginSummary);
            }

            // Fetch open orders
            const ordersResponse = await fetch(`${HYPERLIQUID_CORE_CONFIG.apiUrl}/info`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "openOrders",
                    user: omniAccountAddress,
                }),
            });

            if (ordersResponse.ok) {
                const ordersData: OpenOrder[] = await ordersResponse.json();
                setOpenOrders(ordersData || []);
            }

        } catch (err: any) {
            console.error("Error fetching Hyperliquid data:", err);
            setError(err.message || "Failed to fetch data");
            setBalances([]);
            setPositions([]);
            setOpenOrders([]);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        fetchBalances();
        // Poll every 10 seconds
        const interval = setInterval(fetchBalances, 10000);
        return () => clearInterval(interval);
    }, [omniAccountAddress]);

    const formatBalance = (balance: string): string => {
        const num = parseFloat(balance);
        if (isNaN(num)) return "0";
        // Format with up to 6 decimal places, remove trailing zeros
        return num.toFixed(6).replace(/\.?0+$/, "");
    };

    const formatPrice = (price: string): string => {
        const num = parseFloat(price);
        if (isNaN(num)) return "0";
        return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 4 });
    };

    const formatPnl = (pnl: string): { value: string; isPositive: boolean } => {
        const num = parseFloat(pnl);
        if (isNaN(num)) return { value: "0", isPositive: true };
        return {
            value: num.toFixed(2),
            isPositive: num >= 0,
        };
    };

    const handleSpotToPerp = (amount: string) => {
        if (!amount || parseFloat(amount) <= 0) {
            alert("Please enter a valid amount");
            return;
        }

        // Check if there's sufficient spot USDC balance
        const usdcBalance = balances.find(b => b.coin === "USDC");
        const availableSpot = usdcBalance ? parseFloat(usdcBalance.total) : 0;

        console.log("[handleSpotToPerp] Transfer request:", {
            amount,
            availableSpot,
            omniAccountAddress,
            usdcBalance
        });

        if (availableSpot <= 0) {
            alert("No USDC available in spot balance");
            return;
        }

        if (parseFloat(amount) > availableSpot) {
            alert(`Insufficient spot balance. You have ${availableSpot.toFixed(2)} USDC in spot, but tried to transfer ${amount} USDC.`);
            return;
        }

        const callData = buildSpotToPerpTransferCallData(amount);
        console.log("[handleSpotToPerp] Generated callData:", callData);
        onTransferAction?.(callData, `Transfer ${amount} USDC from Spot to Perp for address ${omniAccountAddress}`);
        setSpotTransferAmount("");
    };

    const handlePerpToSpot = (amount: string) => {
        if (!amount || parseFloat(amount) <= 0) {
            alert("Please enter a valid amount");
            return;
        }

        // Check if there's sufficient withdrawable balance
        const withdrawable = accountSummary ? parseFloat(accountSummary.totalRawUsd) : 0;
        if (withdrawable <= 0) {
            alert(`Cannot transfer from Perp: No withdrawable balance (${withdrawable.toFixed(2)} USDC available)`);
            return;
        }

        if (parseFloat(amount) > withdrawable) {
            alert(`Insufficient withdrawable balance. You have ${withdrawable.toFixed(2)} USDC withdrawable, but tried to transfer ${amount} USDC.`);
            return;
        }

        const callData = buildPerpToSpotTransferCallData(amount);
        onTransferAction?.(callData, `Transfer ${amount} USDC from Perp to Spot`);
        setPerpTransferAmount("");
    };

    return (
        <div className="w-full p-6 bg-white rounded-lg shadow-lg">
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center">
                    <Wallet className="h-6 w-6 text-blue-500 mr-2" />
                    <h2 className="text-2xl font-bold">Hyperliquid Core Balances</h2>
                </div>
                <button
                    onClick={fetchBalances}
                    disabled={isLoading}
                    className="p-2 hover:bg-gray-100 rounded-lg transition-colors disabled:opacity-50"
                    title="Refresh balances"
                >
                    <RefreshCw className={`h-5 w-5 ${isLoading ? "animate-spin" : ""}`} />
                </button>
            </div>

            <div className="mb-4 p-3 bg-blue-50 rounded-lg border border-blue-200">
                <p className="text-sm text-blue-700">
                    <strong>Account:</strong>{" "}
                    <span className="font-mono text-xs break-all">{omniAccountAddress}</span>
                </p>
            </div>

            {error && (
                <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
                    <div className="flex items-center">
                        <AlertCircle className="h-5 w-5 text-red-500 mr-2" />
                        <p className="text-red-700 text-sm">{error}</p>
                    </div>
                </div>
            )}

            {isLoading && balances.length === 0 && positions.length === 0 ? (
                <div className="text-center py-8">
                    <RefreshCw className="h-8 w-8 animate-spin mx-auto text-gray-400 mb-2" />
                    <p className="text-gray-500">Loading data...</p>
                </div>
            ) : (
                <div className="space-y-6">
                    {/* Account Summary */}
                    {accountSummary && (
                        <div className="p-4 rounded-lg bg-gradient-to-r from-green-50 to-blue-50 border border-gray-200">
                            <h3 className="text-sm font-medium text-gray-700 mb-3 flex items-center">
                                <DollarSign className="h-4 w-4 mr-1" />
                                Perpetual Account Summary
                            </h3>
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <p className="text-xs text-gray-600">Account Value</p>
                                    <p className="text-lg font-bold text-gray-900">
                                        ${formatBalance(accountSummary.accountValue)}
                                    </p>
                                </div>
                                <div>
                                    <p className="text-xs text-gray-600">Withdrawable</p>
                                    <p className="text-lg font-bold text-gray-900">
                                        ${formatBalance(accountSummary.totalRawUsd)}
                                    </p>
                                </div>
                                <div>
                                    <p className="text-xs text-gray-600">Margin Used</p>
                                    <p className="text-lg font-bold text-gray-900">
                                        ${formatBalance(accountSummary.totalMarginUsed)}
                                    </p>
                                </div>
                                <div>
                                    <p className="text-xs text-gray-600">Position Value</p>
                                    <p className="text-lg font-bold text-gray-900">
                                        ${formatBalance(accountSummary.totalNtlPos)}
                                    </p>
                                </div>
                            </div>

                            {/* Transfer to Spot button - always shown if onTransferAction exists */}
                            {onTransferAction && (
                                <div className="mt-4 pt-4 border-t border-gray-300">
                                    <p className="text-xs text-gray-600 mb-2 font-medium">
                                        Transfer to Spot (Remove Margin)
                                        {parseFloat(accountSummary.totalRawUsd) > 0 ? (
                                            <span className="ml-2 text-gray-500">Max: {formatBalance(accountSummary.totalRawUsd)}</span>
                                        ) : (
                                            <span className="text-red-500 ml-2">(No withdrawable balance)</span>
                                        )}
                                    </p>
                                    <div className="flex gap-2">
                                        <input
                                            type="number"
                                            placeholder="Amount"
                                            value={perpTransferAmount}
                                            onChange={(e) => setPerpTransferAmount(e.target.value)}
                                            className="flex-1 px-3 py-2 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            min="0"
                                            max={parseFloat(accountSummary.totalRawUsd) > 0 ? accountSummary.totalRawUsd : undefined}
                                            step="0.01"
                                            disabled={parseFloat(accountSummary.totalRawUsd) <= 0}
                                        />
                                        {parseFloat(accountSummary.totalRawUsd) > 0 && (
                                            <button
                                                onClick={() => setPerpTransferAmount(formatBalance(accountSummary.totalRawUsd))}
                                                className="px-3 py-2 bg-gray-200 text-gray-700 text-xs font-medium rounded-lg hover:bg-gray-300 transition-colors"
                                            >
                                                Max
                                            </button>
                                        )}
                                        <button
                                            onClick={() => handlePerpToSpot(perpTransferAmount)}
                                            disabled={!perpTransferAmount || parseFloat(perpTransferAmount) <= 0 || parseFloat(accountSummary.totalRawUsd) <= 0}
                                            className="px-4 py-2 bg-orange-500 text-white text-sm font-medium rounded-lg hover:bg-orange-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                                        >
                                            <ArrowRightLeft className="h-4 w-4" />
                                            To Spot
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>
                    )}

                    {/* Open Positions */}
                    {positions.length > 0 && (
                        <div>
                            <h3 className="text-sm font-medium text-gray-700 mb-3 flex items-center">
                                <TrendingUp className="h-4 w-4 mr-1" />
                                Open Positions ({positions.length})
                            </h3>
                            <div className="space-y-3">
                                {positions.map((position, index) => {
                                    const pnl = formatPnl(position.unrealizedPnl);
                                    const isLong = parseFloat(position.szi) > 0;
                                    return (
                                        <div
                                            key={`${position.coin}-${index}`}
                                            className={`p-4 rounded-lg border-2 ${
                                                isLong
                                                    ? "bg-green-50 border-green-300"
                                                    : "bg-red-50 border-red-300"
                                            }`}
                                        >
                                            <div className="flex justify-between items-start mb-2">
                                                <div>
                                                    <h4 className="font-bold text-lg text-gray-900">
                                                        {position.coin}
                                                        <span
                                                            className={`ml-2 text-sm ${
                                                                isLong ? "text-green-600" : "text-red-600"
                                                            }`}
                                                        >
                                                            {isLong ? "LONG" : "SHORT"}
                                                        </span>
                                                    </h4>
                                                    <p className="text-xs text-gray-500">
                                                        Size: {formatBalance(position.szi)} | Leverage: {position.leverage.value}x
                                                    </p>
                                                </div>
                                                <div className="text-right">
                                                    <p
                                                        className={`text-lg font-bold ${
                                                            pnl.isPositive ? "text-green-600" : "text-red-600"
                                                        }`}
                                                    >
                                                        {pnl.isPositive ? "+" : ""}${pnl.value}
                                                    </p>
                                                    <p className="text-xs text-gray-500">PnL</p>
                                                </div>
                                            </div>
                                            <div className="grid grid-cols-3 gap-2 mt-3 pt-3 border-t border-gray-300">
                                                <div>
                                                    <p className="text-xs text-gray-600">Entry</p>
                                                    <p className="font-mono text-sm font-semibold">
                                                        ${formatPrice(position.entryPx)}
                                                    </p>
                                                </div>
                                                <div>
                                                    <p className="text-xs text-gray-600">Liquidation</p>
                                                    <p className="font-mono text-sm font-semibold">
                                                        {position.liquidationPx
                                                            ? `$${formatPrice(position.liquidationPx)}`
                                                            : "N/A"}
                                                    </p>
                                                </div>
                                                <div>
                                                    <p className="text-xs text-gray-600">Margin</p>
                                                    <p className="font-mono text-sm font-semibold">
                                                        ${formatBalance(position.marginUsed)}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}

                    {/* Open Orders */}
                    {openOrders.length > 0 && (
                        <div>
                            <h3 className="text-sm font-medium text-gray-700 mb-3 flex items-center">
                                <ShoppingCart className="h-4 w-4 mr-1" />
                                Open Orders ({openOrders.length})
                            </h3>
                            <div className="space-y-2">
                                {openOrders.map((order) => {
                                    const isBuy = order.side === "B";
                                    return (
                                        <div
                                            key={order.oid}
                                            className={`p-3 rounded-lg border ${
                                                isBuy
                                                    ? "bg-green-50 border-green-200"
                                                    : "bg-red-50 border-red-200"
                                            }`}
                                        >
                                            <div className="flex justify-between items-center">
                                                <div>
                                                    <h4 className="font-semibold text-gray-900">
                                                        {order.coin}
                                                        <span
                                                            className={`ml-2 text-xs px-2 py-1 rounded ${
                                                                isBuy
                                                                    ? "bg-green-200 text-green-800"
                                                                    : "bg-red-200 text-red-800"
                                                            }`}
                                                        >
                                                            {isBuy ? "BUY" : "SELL"}
                                                        </span>
                                                    </h4>
                                                    <p className="text-xs text-gray-500 mt-1">
                                                        Order ID: {order.oid}
                                                    </p>
                                                </div>
                                                <div className="text-right">
                                                    <p className="font-mono text-sm font-semibold">
                                                        ${formatPrice(order.limitPx)}
                                                    </p>
                                                    <p className="text-xs text-gray-500">
                                                        Size: {formatBalance(order.sz)} / {formatBalance(order.origSz)}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}

                    {/* Spot Balances */}
                    <div>
                        <h3 className="text-sm font-medium text-gray-700 mb-3 flex items-center">
                            <Wallet className="h-4 w-4 mr-1" />
                            Spot Balances
                        </h3>
                        {balances.length > 0 ? (
                            <div className="space-y-2">
                                {balances.map((asset, index) => {
                                    const isUSDC = asset.coin === "USDC";
                                    return (
                                        <div
                                            key={`${asset.coin}-${index}`}
                                            className="p-4 rounded-lg bg-gradient-to-r from-blue-50 to-purple-50 border border-gray-200"
                                        >
                                            <div className="flex justify-between items-center">
                                                <div>
                                                    <h4 className="font-semibold text-lg text-gray-900">
                                                        {asset.coin}
                                                    </h4>
                                                    <p className="text-xs text-gray-500 mt-1">
                                                        Hold: {formatBalance(asset.hold)}
                                                    </p>
                                                </div>
                                                <div className="text-right">
                                                    <p className="text-2xl font-bold text-gray-900">
                                                        {formatBalance(asset.total)}
                                                    </p>
                                                    <p className="text-xs text-gray-500 mt-1">Total</p>
                                                </div>
                                            </div>

                                            {/* Transfer buttons for USDC */}
                                            {isUSDC && onTransferAction && (
                                                <div className="mt-4 pt-4 border-t border-gray-300">
                                                    <p className="text-xs text-gray-600 mb-2 font-medium">
                                                        Transfer to Perp (Add Margin)
                                                        <span className="ml-2 text-gray-500">Max: {formatBalance(asset.total)}</span>
                                                    </p>
                                                    <div className="flex gap-2">
                                                        <input
                                                            type="number"
                                                            placeholder="Amount"
                                                            value={spotTransferAmount}
                                                            onChange={(e) => setSpotTransferAmount(e.target.value)}
                                                            className="flex-1 px-3 py-2 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                            min="0"
                                                            max={asset.total}
                                                            step="0.01"
                                                        />
                                                        <button
                                                            onClick={() => setSpotTransferAmount(formatBalance(asset.total))}
                                                            className="px-3 py-2 bg-gray-200 text-gray-700 text-xs font-medium rounded-lg hover:bg-gray-300 transition-colors"
                                                        >
                                                            Max
                                                        </button>
                                                        <button
                                                            onClick={() => handleSpotToPerp(spotTransferAmount)}
                                                            disabled={!spotTransferAmount || parseFloat(spotTransferAmount) <= 0}
                                                            className="px-4 py-2 bg-green-500 text-white text-sm font-medium rounded-lg hover:bg-green-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                                                        >
                                                            <ArrowRightLeft className="h-4 w-4" />
                                                            Add Margin
                                                        </button>
                                                    </div>
                                                </div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        ) : (
                            <div className="p-4 rounded-lg bg-gradient-to-r from-blue-50 to-purple-50 border border-gray-200">
                                <p className="text-sm text-gray-600 mb-4">No spot balances found. Transfer from perpetual to spot to see balances here.</p>

                                {/* Always show Add Margin option even if no spot USDC */}
                                {onTransferAction && accountSummary && (
                                    <div className="mt-4 pt-4 border-t border-gray-300">
                                        <p className="text-xs text-gray-600 mb-2 font-medium">Transfer from Perp to Spot</p>
                                        <div className="flex gap-2">
                                            <input
                                                type="number"
                                                placeholder="Amount"
                                                value={spotTransferAmount}
                                                onChange={(e) => setSpotTransferAmount(e.target.value)}
                                                className="flex-1 px-3 py-2 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                min="0"
                                                step="0.01"
                                            />
                                            <button
                                                onClick={() => handlePerpToSpot(spotTransferAmount)}
                                                disabled={!spotTransferAmount || parseFloat(spotTransferAmount) <= 0}
                                                className="px-4 py-2 bg-orange-500 text-white text-sm font-medium rounded-lg hover:bg-orange-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                                            >
                                                <ArrowRightLeft className="h-4 w-4" />
                                                To Spot
                                            </button>
                                        </div>
                                    </div>
                                )}
                            </div>
                        )}
                    </div>

                    {/* No data message */}
                    {balances.length === 0 && positions.length === 0 && openOrders.length === 0 && !isLoading && (
                        <div className="text-center py-8">
                            <Wallet className="h-12 w-12 mx-auto text-gray-300 mb-4" />
                            <p className="text-gray-500">No data found</p>
                            <p className="text-sm text-gray-400 mt-2">
                                This account has no assets, positions, or orders on Hyperliquid
                            </p>
                        </div>
                    )}
                </div>
            )}

            <div className="mt-6 p-4 bg-purple-50 border border-purple-200 rounded-lg">
                <div className="flex">
                    <AlertCircle className="h-5 w-5 text-purple-500 mr-2 mt-0.5" />
                    <div className="text-sm text-purple-700">
                        <p className="font-medium mb-1">About Hyperliquid Core</p>
                        <p>
                            These balances represent assets held on the Hyperliquid native L1
                            blockchain, separate from the EVM-compatible layer.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
}
