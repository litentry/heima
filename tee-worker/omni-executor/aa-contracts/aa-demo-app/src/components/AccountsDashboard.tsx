"use client";

import React, { useState, useEffect, useCallback, useRef } from "react";
import { useAccount, useReadContract, usePublicClient } from "wagmi";
import { useWallet } from "@solana/wallet-adapter-react";
import { Copy, ExternalLink, Wallet, RefreshCw } from "lucide-react";
import { formatUnits } from "viem";
import { calculateOmniAccount, stringToBytes } from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS, TEST_TOKENS } from "@/lib/constants";
import type { Address } from "viem";

interface TokenBalance {
	symbol: string;
	balance: bigint;
	decimals: number;
}

interface AccountsDashboardProps {
	onAddressCalculated?: (address: string) => void;
	ethBalance?: bigint;
	onBalancesUpdate?: (balances: TokenBalance[]) => void;
	isAccountCreated?: boolean;
}

export function AccountsDashboard({
	onAddressCalculated,
	ethBalance: propEthBalance,
	onBalancesUpdate,
	isAccountCreated = false,
}: AccountsDashboardProps = {}) {
	const { address: evmAddress } = useAccount();
	const { publicKey: solanaAddress } = useWallet();
	const publicClient = usePublicClient();
	const [omniAccount, setOmniAccount] = useState<`0x${string}`>("0x");
	const [clientId] = useState<string>(DEFAULT_CLIENT_ID);
	const [clientIdBytes] = useState<`0x${string}`>(
		stringToBytes(DEFAULT_CLIENT_ID),
	);
	const rootSigner =
		evmAddress || ("0x0000000000000000000000000000000000000000" as Address);
	const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([]);
	const [walletTokenBalances, setWalletTokenBalances] = useState<TokenBalance[]>([]);
	const [isRefreshing, setIsRefreshing] = useState(false);
	const [isRefreshingWallet, setIsRefreshingWallet] = useState(false);
	
	// Use ref to store the latest onBalancesUpdate callback
	const onBalancesUpdateRef = useRef(onBalancesUpdate);
	useEffect(() => {
		onBalancesUpdateRef.current = onBalancesUpdate;
	}, [onBalancesUpdate]);

	useEffect(() => {
		if (evmAddress) {
			const account = calculateOmniAccount(
				evmAddress,
				DEFAULT_CLIENT_ID,
				"evm",
			);
			setOmniAccount(account);
		} else {
			setOmniAccount("0x" as `0x${string}`);
		}
	}, [evmAddress]);

	const { data: omniAccountAddress, error: contractError } = useReadContract({
		address: CONTRACTS.OmniAccountFactory.address,
		abi: CONTRACTS.OmniAccountFactory.abi,
		functionName: "getAddress",
		args: [omniAccount, clientIdBytes, rootSigner],
		query: {
			enabled: !!evmAddress && omniAccount !== "0x",
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

	// Available ERC20 tokens
	const availableTokens = [TEST_TOKENS.USDC, TEST_TOKENS.USDT];

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
				console.error(`Error fetching wallet ${token.symbol} balance:`, error);
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
				console.error(`Error fetching ${token.symbol} balance:`, error);
				balances.push({
					symbol: token.symbol,
					balance: BigInt(0),
					decimals: token.decimals,
				});
			}
		}

		setTokenBalances(balances);
		setIsRefreshing(false);

		// Notify parent of balance updates using ref
		if (onBalancesUpdateRef.current) {
			onBalancesUpdateRef.current(balances);
		}
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

	if (!evmAddress && !solanaAddress) {
		return (
			<div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
				<div className="text-center">
					<Wallet className="mx-auto h-12 w-12 text-gray-400 mb-4" />
					<h3 className="text-lg font-medium text-gray-900 mb-2">
						No Wallet Connected
					</h3>
					<p className="text-gray-600">
						Connect your wallet to view your accounts overview.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full p-6 bg-white rounded-lg shadow-lg">
			<h2 className="text-2xl font-bold mb-6 text-center">Accounts Overview</h2>

			<div className="space-y-8">
				{/* Connected Wallet Section */}
				{evmAddress && (
					<div className="space-y-4">
						<div className="border-b pb-2">
							<h3 className="text-xl font-semibold text-gray-900">
								Your Wallet
							</h3>
							<p className="text-sm text-gray-600 mt-1">
								This is your connected wallet that controls the smart account
							</p>
						</div>
						<div className="bg-gray-50 p-4 rounded-lg">
							<div className="flex items-center justify-between mb-4">
								<span className="text-sm font-medium text-gray-700">Address:</span>
								<div className="flex items-center space-x-2">
									<span className="text-sm font-mono">
										{truncateAddress(evmAddress)}
									</span>
									<button
										onClick={() => copyToClipboard(evmAddress)}
										className="p-1 hover:bg-gray-200 rounded"
									>
										<Copy className="h-4 w-4 text-gray-500" />
									</button>
								</div>
							</div>
							
							{/* Wallet Balances */}
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
				{omniAccountAddress && (
					<div className="space-y-4">
						<div className="border-b pb-2">
							<h3 className="text-xl font-semibold text-gray-900">
								Smart Account (Omni Account)
							</h3>
							<p className="text-sm text-gray-600 mt-1">
								Your Omni Account address is pre-calculated using your wallet address and client ID
							</p>
						</div>
						<div className="bg-gray-50 p-4 rounded-lg">
							<div className="space-y-4">
								{/* Omni Account Address */}
								<div>
									<div className="flex items-center justify-between mb-2">
										<span className="text-sm font-medium text-gray-700">Address:</span>
										<button
											onClick={() => copyToClipboard(omniAccountAddress as string)}
											className="p-1 hover:bg-gray-200 rounded"
										>
											<Copy className="h-4 w-4 text-gray-500" />
										</button>
									</div>
									<div className="text-sm font-mono text-green-800 break-all bg-green-100 p-2 rounded border border-green-200">
										{(omniAccountAddress as string) || "Calculating..."}
									</div>
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
