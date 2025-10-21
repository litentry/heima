import React, { useState, useEffect, useCallback } from "react";
import { useAccount, useReadContract, usePublicClient } from "wagmi";
import { useWallet } from "@solana/wallet-adapter-react";
import { Copy, Wallet, RefreshCw, Mail } from "lucide-react";
import { formatUnits } from "viem";
import { calculateOmniAccount, stringToBytes } from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS, ERC20_TOKENS, OwnerType } from "@/lib/constants";
import { useAuth } from "@/contexts/AuthContext";
import { getTEEWorkerAddress } from "@/lib/tee-worker-client";
import type { Address } from "viem";

interface TokenBalance {
    symbol: string;
    balance: bigint;
    decimals: number;
}

interface AccountsDashboardProps {
    onAddressCalculated?: (address: string) => void;
    onOmniAccountCalculated?: (omniAccount: string) => void;
    ethBalance?: bigint;
    isAccountCreated?: boolean;
}

export function AccountsDashboard({
    onAddressCalculated,
    onOmniAccountCalculated,
    ethBalance: propEthBalance,
    isAccountCreated = false,
}: AccountsDashboardProps = {}) {
    const { address: evmAddress } = useAccount();
    const { publicKey: solanaAddress } = useWallet();
    const publicClient = usePublicClient();
    const { authType, identifier, omniAccountHash: authOmniHash } = useAuth();

    const [omniAccount, setOmniAccount] = useState<`0x${string}`>("0x");
    const [clientId] = useState<string>(DEFAULT_CLIENT_ID);
    const [clientIdBytes] = useState<`0x${string}`>(
        stringToBytes(DEFAULT_CLIENT_ID),
    );
    const [rootSigner, setRootSigner] = useState<Address>(
        "0x0000000000000000000000000000000000000000" as Address
    );
    const [ownerType, setOwnerType] = useState<number>(OwnerType.Evm);
    const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([]);
    const [walletTokenBalances, setWalletTokenBalances] = useState<TokenBalance[]>([]);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [isRefreshingWallet, setIsRefreshingWallet] = useState(false);
    const [isLoadingRootSigner, setIsLoadingRootSigner] = useState(false);

    // Set root signer and owner type based on auth type
    useEffect(() => {
        const fetchRootSigner = async () => {
            if (authType === "email" && authOmniHash) {
                setIsLoadingRootSigner(true);
                try {
                    // For email accounts, TEE worker is the root signer
                    const workerAddress = await getTEEWorkerAddress(authOmniHash);
                    setRootSigner(workerAddress as Address);
                    setOwnerType(OwnerType.Email);
                } catch (error) {
                    console.error("Failed to get TEE worker address:", error);
                    setRootSigner("0x0000000000000000000000000000000000000000" as Address);
                } finally {
                    setIsLoadingRootSigner(false);
                }
            } else if (authType === "wallet" && evmAddress) {
                setRootSigner(evmAddress);
                setOwnerType(OwnerType.Evm);
            }
        };

        fetchRootSigner();
    }, [authType, authOmniHash, evmAddress]);

    useEffect(() => {
        if (authType === "wallet" && evmAddress) {
            const account = calculateOmniAccount(
                evmAddress,
                DEFAULT_CLIENT_ID,
                "evm",
            );
            setOmniAccount(account);
        } else if (authType === "email" && authOmniHash) {
            setOmniAccount(authOmniHash as `0x${string}`);
        } else {
            setOmniAccount("0x" as `0x${string}`);
        }
    }, [authType, evmAddress, authOmniHash]);

    const { data: omniAccountAddress, error: contractError } = useReadContract({
        address: CONTRACTS.OmniAccountFactory.address,
        abi: CONTRACTS.OmniAccountFactory.abi,
        functionName: "getAddress",
        args: [omniAccount, ownerType, clientIdBytes, rootSigner],
        query: {
            enabled: omniAccount !== "0x" && rootSigner !== ("0x0000000000000000000000000000000000000000" as Address),
        },
    });

    // Debug logging
    useEffect(() => {
        console.log("AccountsDashboard Debug:", {
            evmAddress,
            omniAccount,
            clientIdBytes,
            rootSigner,
            factoryAddress: CONTRACTS.OmniAccountFactory.address,
            omniAccountAddress,
            contractError,
            enabled: !!evmAddress && omniAccount !== "0x",
        });
    }, [
        evmAddress,
        omniAccount,
        clientIdBytes,
        rootSigner,
        omniAccountAddress,
        contractError,
    ]);

    // Notify parent when address is calculated
    useEffect(() => {
        if (omniAccountAddress && onAddressCalculated) {
            onAddressCalculated(omniAccountAddress as string);
        }
    }, [omniAccountAddress, onAddressCalculated]);

    // Notify parent when omni account hash is calculated
    useEffect(() => {
        if (omniAccount && omniAccount !== "0x" && onOmniAccountCalculated) {
            onOmniAccountCalculated(omniAccount);
        }
    }, [omniAccount, onOmniAccountCalculated]);

    // Available ERC20 tokens
    const availableTokens = [ERC20_TOKENS.USDC, ERC20_TOKENS.USDT];

    // Fetch connected wallet balances
    const fetchWalletBalances = useCallback(async () => {
        if (!evmAddress || !publicClient) return;

        setIsRefreshingWallet(true);
        const balances: TokenBalance[] = [];

        // Fetch ETH balance for wallet
        try {
            const ethBalance = await publicClient.getBalance({
                address: evmAddress,
            });
            balances.push({
                symbol: "ETH",
                balance: ethBalance,
                decimals: 18,
            });
        } catch (error) {
            console.error("Error fetching wallet ETH balance:", error);
            balances.push({
                symbol: "ETH",
                balance: BigInt(0),
                decimals: 18,
            });
        }

        // Fetch ERC20 balances for wallet
        for (const token of availableTokens) {
            try {
                const balance = (await publicClient.readContract({
                    address: token.address,
                    abi: token.abi,
                    functionName: "balanceOf",
                    args: [evmAddress],
                })) as bigint;

                balances.push({
                    symbol: token.symbol,
                    balance,
                    decimals: token.decimals,
                });
            } catch (error) {
                // Silently handle token balance fetch errors (token might not exist on this network)
                balances.push({
                    symbol: token.symbol,
                    balance: BigInt(0),
                    decimals: token.decimals,
                });
            }
        }

        setWalletTokenBalances(balances);
        setIsRefreshingWallet(false);
    }, [evmAddress, publicClient]);

    // Fetch all token balances including ETH
    const fetchTokenBalances = useCallback(async () => {
        if (!omniAccountAddress || !publicClient || !isAccountCreated) return;

        setIsRefreshing(true);
        const balances: TokenBalance[] = [];

        // Fetch ETH balance
        try {
            const ethBalance =
                propEthBalance !== undefined
                    ? propEthBalance
                    : await publicClient.getBalance({
                        address: omniAccountAddress as `0x${string}`,
                    });
            balances.push({
                symbol: "ETH",
                balance: ethBalance,
                decimals: 18,
            });
        } catch (error) {
            console.error("Error fetching ETH balance:", error);
            balances.push({
                symbol: "ETH",
                balance: BigInt(0),
                decimals: 18,
            });
        }

        // Fetch ERC20 balances
        for (const token of availableTokens) {
            try {
                const balance = (await publicClient.readContract({
                    address: token.address,
                    abi: token.abi,
                    functionName: "balanceOf",
                    args: [omniAccountAddress],
                })) as bigint;

                balances.push({
                    symbol: token.symbol,
                    balance,
                    decimals: token.decimals,
                });
            } catch (error) {
                // Silently handle token balance fetch errors (token might not exist on this network)
                balances.push({
                    symbol: token.symbol,
                    balance: BigInt(0),
                    decimals: token.decimals,
                });
            }
        }

        setTokenBalances(balances);
        setIsRefreshing(false);
    }, [omniAccountAddress, publicClient, propEthBalance, isAccountCreated]);

    // Initial balance fetch
    useEffect(() => {
        fetchTokenBalances();
        fetchWalletBalances();
    }, [fetchTokenBalances, fetchWalletBalances]);

    // Poll balances every 10 seconds
    useEffect(() => {
        const interval = setInterval(() => {
            if (omniAccountAddress && isAccountCreated) {
                fetchTokenBalances();
            }
            if (evmAddress) {
                fetchWalletBalances();
            }
        }, 10000);

        return () => clearInterval(interval);
    }, [omniAccountAddress, evmAddress, fetchTokenBalances, fetchWalletBalances, isAccountCreated]);

    const copyToClipboard = async (text: string) => {
        try {
            await navigator.clipboard.writeText(text);
            // Could add toast notification here
        } catch (err) {
            console.error("Failed to copy text: ", err);
        }
    };

    const truncateAddress = (address: string) => {
        if (!address) return "";
        return `${address.slice(0, 6)}...${address.slice(-4)}`;
    };

    const isAuthenticated = authType && identifier;

    if (!isAuthenticated) {
        return (
            <div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
                <div className="text-center">
                    <Wallet className="mx-auto h-12 w-12 text-gray-400 mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">
                        No Account Connected
                    </h3>
                    <p className="text-gray-600">
                        Connect your wallet or email to view your accounts overview.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className="w-full p-6 bg-white rounded-lg shadow-lg">
            <h2 className="text-2xl font-bold mb-6 text-center">Accounts Overview</h2>

            <div className="space-y-8">
                {/* Connected Account Section */}
                {isAuthenticated && (
                    <div className="space-y-4">
                        <div className="border-b pb-2">
                            <h3 className="text-xl font-semibold text-gray-900">
                                {authType === "email" ? "Your Email Account" : "Your Wallet"}
                            </h3>
                            <p className="text-sm text-gray-600 mt-1">
                                {authType === "email"
                                    ? "This email address controls your smart account via TEE worker"
                                    : "This is your connected wallet that controls the smart account"}
                            </p>
                        </div>
                        <div className="bg-gray-50 p-4 rounded-lg">
                            <div className="flex items-center justify-between mb-4">
                                <span className="text-sm font-medium text-gray-700">
                                    {authType === "email" ? "Email:" : "Address:"}
                                </span>
                                <div className="flex items-center space-x-2">
                                    {authType === "email" ? (
                                        <>
                                            <Mail className="h-4 w-4 text-gray-500" />
                                            <span className="text-sm font-medium">
                                                {identifier}
                                            </span>
                                        </>
                                    ) : (
                                        <>
                                            <span className="text-sm font-mono">
                                                {truncateAddress(identifier || "")}
                                            </span>
                                            <button
                                                onClick={() => copyToClipboard(identifier || "")}
                                                className="p-1 hover:bg-gray-200 rounded"
                                            >
                                                <Copy className="h-4 w-4 text-gray-500" />
                                            </button>
                                        </>
                                    )}
                                </div>
                            </div>

                            {/* Wallet Balances - Only for wallet auth */}
                            {authType === "wallet" && (
                                <div className="space-y-2">
                                    <div className="flex items-center justify-between mb-2">
                                        <span className="text-sm font-medium text-gray-700">Balances:</span>
                                        <button
                                            onClick={() => fetchWalletBalances()}
                                            disabled={isRefreshingWallet}
                                            className="p-1 hover:bg-gray-100 rounded transition-colors disabled:opacity-50"
                                        >
                                            <RefreshCw
                                                className={`h-3 w-3 ${isRefreshingWallet ? "animate-spin" : ""}`}
                                            />
                                        </button>
                                    </div>
                                    {walletTokenBalances.map((tb) => (
                                    <div
                                        key={tb.symbol}
                                        className="pl-4 flex justify-between items-center py-1"
                                    >
                                        <span className="text-sm text-gray-600">{tb.symbol}</span>
                                        <span className="font-mono text-sm">
                                            {formatUnits(tb.balance, tb.decimals)}
                                        </span>
                                    </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {solanaAddress && (
                    <div className="space-y-3">
                        <h3 className="text-lg font-semibold text-gray-900">
                            Connected Solana Wallet
                        </h3>
                        <div className="bg-gray-50 p-3 rounded-lg">
                            <div className="flex items-center justify-between">
                                <span className="text-sm text-gray-600">Address:</span>
                                <div className="flex items-center space-x-2">
                                    <span className="text-sm font-mono">
                                        {truncateAddress(solanaAddress.toString())}
                                    </span>
                                    <button
                                        onClick={() => copyToClipboard(solanaAddress.toString())}
                                        className="p-1 hover:bg-gray-200 rounded"
                                    >
                                        <Copy className="h-4 w-4 text-gray-500" />
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                )}

                {/* Client ID Section */}
                <div className="space-y-2">
                    <div className="border-b pb-2">
                        <h3 className="text-lg font-semibold text-gray-900">Client ID</h3>
                    </div>
                    <div className="bg-gray-50 p-3 rounded-lg">
                        <div className="flex items-center justify-between">
                            <span className="text-sm text-gray-600">ID:</span>
                            <div className="flex items-center space-x-2">
                                <span className="text-sm font-mono">{clientId}</span>
                                <button
                                    onClick={() => copyToClipboard(clientId)}
                                    className="p-1 hover:bg-gray-200 rounded"
                                >
                                    <Copy className="h-4 w-4 text-gray-500" />
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Smart Account Section */}
                {(omniAccountAddress || isLoadingRootSigner) && (
                    <div className="space-y-4">
                        <div className="border-b pb-2">
                            <h3 className="text-xl font-semibold text-gray-900">
                                Smart Account (Omni Account)
                            </h3>
                            <p className="text-sm text-gray-600 mt-1">
                                {authType === "email"
                                    ? "Your Omni Account address is calculated using your email and TEE worker"
                                    : "Your Omni Account address is pre-calculated using your wallet address and client ID"}
                            </p>
                        </div>
                        <div className="bg-gray-50 p-4 rounded-lg">
                            <div className="space-y-4">
                                {/* Omni Account Address */}
                                <div>
                                    <div className="flex items-center justify-between mb-2">
                                        <span className="text-sm font-medium text-gray-700">Address:</span>
                                        {omniAccountAddress && (
                                            <button
                                                onClick={() => copyToClipboard(omniAccountAddress as string)}
                                                className="p-1 hover:bg-gray-200 rounded"
                                            >
                                                <Copy className="h-4 w-4 text-gray-500" />
                                            </button>
                                        )}
                                    </div>
                                    {isLoadingRootSigner ? (
                                        <div className="flex items-center text-gray-600 bg-gray-100 p-2 rounded border border-gray-200">
                                            <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                                            Fetching TEE worker address...
                                        </div>
                                    ) : omniAccountAddress ? (
                                        <div className="text-sm font-mono text-green-800 break-all bg-green-100 p-2 rounded border border-green-200">
                                            {omniAccountAddress as string}
                                        </div>
                                    ) : (
                                        <div className="text-sm text-gray-500 bg-gray-100 p-2 rounded border border-gray-200">
                                            Unable to calculate address
                                        </div>
                                    )}
                                </div>

                                {/* Omni Account Balances */}
                                <div className="space-y-2">
                                    <div className="flex items-center justify-between mb-2">
                                        <span className="text-sm font-medium text-gray-700">Balances:</span>
                                        {isAccountCreated && (
                                            <button
                                                onClick={() => fetchTokenBalances()}
                                                disabled={isRefreshing}
                                                className="p-1 hover:bg-gray-100 rounded transition-colors disabled:opacity-50"
                                            >
                                                <RefreshCw
                                                    className={`h-3 w-3 ${isRefreshing ? "animate-spin" : ""}`}
                                                />
                                            </button>
                                        )}
                                    </div>
                                    {!isAccountCreated ? (
                                        <div className="text-sm text-gray-500 italic pl-4">
                                            Account not created on-chain yet
                                        </div>
                                    ) : (
                                        tokenBalances.map((tb) => (
                                            <div
                                                key={tb.symbol}
                                                className="pl-4 flex justify-between items-center py-1"
                                            >
                                                <span className="text-sm text-gray-600">{tb.symbol}</span>
                                                <span className="font-mono text-sm">
                                                    {formatUnits(tb.balance, tb.decimals)}
                                                </span>
                                            </div>
                                        ))
                                    )}
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
