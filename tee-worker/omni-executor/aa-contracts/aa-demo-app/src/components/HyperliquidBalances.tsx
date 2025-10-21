import { useState, useEffect } from "react";
import { AlertCircle, RefreshCw, Wallet, TrendingUp, ShoppingCart, DollarSign } from "lucide-react";
import { HYPERLIQUID_CORE_CONFIG } from "@/lib/constants";

interface HyperliquidBalancesProps {
    omniAccountAddress: string;
}

interface AssetBalance {
    coin: string;
    hold: string;
    total: string;
}

interface SpotBalance {
    type: "spot";
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

export function HyperliquidBalances({ omniAccountAddress }: HyperliquidBalancesProps) {
    const [balances, setBalances] = useState<AssetBalance[]>([]);
    const [positions, setPositions] = useState<Position[]>([]);
    const [openOrders, setOpenOrders] = useState<OpenOrder[]>([]);
    const [accountSummary, setAccountSummary] = useState<any>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

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

            const spotData: SpotBalance = await spotResponse.json();

            if (spotData.type === "spot" && spotData.balances) {
                setBalances(spotData.balances);
            } else {
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
                                Account Summary
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
                    {balances.length > 0 && (
                        <div>
                            <h3 className="text-sm font-medium text-gray-700 mb-3 flex items-center">
                                <Wallet className="h-4 w-4 mr-1" />
                                Spot Balances
                            </h3>
                            <div className="space-y-2">
                                {balances.map((asset, index) => (
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
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

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
