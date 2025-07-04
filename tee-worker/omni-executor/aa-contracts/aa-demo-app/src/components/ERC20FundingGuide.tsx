"use client";

import { useState, useEffect } from "react";
import { useAccount, usePublicClient, useWalletClient } from "wagmi";
import { Copy, AlertCircle, CheckCircle, Coins, RefreshCw } from "lucide-react";
import { formatUnits, parseUnits } from "viem";
import { TEST_TOKENS } from "@/lib/constants";

interface ERC20FundingGuideProps {
	omniAccountAddress?: string;
	onTokensAdded?: () => void;
}

interface TokenBalance {
	symbol: string;
	balance: bigint;
	decimals: number;
}

interface WalletTokenBalance extends TokenBalance {
	address: `0x${string}`;
	abi: any;
}

export function ERC20FundingGuide({
	omniAccountAddress: omniAccountAddress,
	onTokensAdded,
}: ERC20FundingGuideProps) {
	const { address: evmAddress } = useAccount();
	const publicClient = usePublicClient();
	const { data: walletClient } = useWalletClient();
	const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([]);
	const [walletTokenBalances, setWalletTokenBalances] = useState<
		WalletTokenBalance[]
	>([]);
	const [mintingToken, setMintingToken] = useState<string | null>(null);
	const [transferringToken, setTransferringToken] = useState<string | null>(
		null,
	);
	const [isRefreshing, setIsRefreshing] = useState(false);
	const [isRefreshingWallet, setIsRefreshingWallet] = useState(false);

	// Available ERC20 tokens
	const availableTokens = [TEST_TOKENS.USDC, TEST_TOKENS.USDT];

	// Fetch Omni Account token balances
	const fetchTokenBalances = async () => {
		if (!omniAccountAddress || !publicClient) return;

		setIsRefreshing(true);
		const balances: TokenBalance[] = [];

		// Fetch ETH balance
		try {
			const ethBalance = await publicClient.getBalance({
				address: omniAccountAddress as `0x${string}`,
			});
			balances.push({
				symbol: "ETH",
				balance: ethBalance,
				decimals: 18,
			});
		} catch (error) {
			console.error("Error fetching ETH balance:", error);
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
	};

	// Fetch wallet token balances
	const fetchWalletBalances = async () => {
		if (!evmAddress || !publicClient) return;

		setIsRefreshingWallet(true);
		const balances: WalletTokenBalance[] = [];

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
					address: token.address,
					abi: token.abi,
				});
			} catch (error) {
				console.error(`Error fetching wallet ${token.symbol} balance:`, error);
				balances.push({
					symbol: token.symbol,
					balance: BigInt(0),
					decimals: token.decimals,
					address: token.address,
					abi: token.abi,
				});
			}
		}

		setWalletTokenBalances(balances);
		setIsRefreshingWallet(false);
	};

	// Mint test tokens
	const mintTestTokens = async (
		token: typeof TEST_TOKENS.USDC | typeof TEST_TOKENS.USDT,
	) => {
		if (!walletClient || !evmAddress || !publicClient) return;

		setMintingToken(token.symbol);
		try {
			const amount = parseUnits("1000", token.decimals); // Mint 1000 tokens

			const hash = await walletClient.writeContract({
				address: token.address,
				abi: token.abi,
				functionName: "mint" as const,
				args: [evmAddress, amount],
				chain: walletClient.chain,
				account: walletClient.account!,
			});

			// Wait for transaction
			await publicClient.waitForTransactionReceipt({ hash });

			// Refresh wallet balances
			await fetchWalletBalances();

			// Show success message with clear destination
			alert(
				`Successfully minted 1000 ${token.symbol}! Transfer them to your Omni Account using the button below.`,
			);
		} catch (error) {
			console.error("Error minting tokens:", error);
			alert(`Failed to mint ${token.symbol}. Please try again.`);
		} finally {
			setMintingToken(null);
		}
	};

	// Transfer tokens from wallet to Omni Account
	const transferToOmniAccount = async (token: WalletTokenBalance) => {
		if (!walletClient || !evmAddress || !publicClient || !omniAccountAddress)
			return;

		setTransferringToken(token.symbol);
		try {
			const hash = await walletClient.writeContract({
				address: token.address,
				abi: token.abi || TEST_TOKENS.USDC.abi,
				functionName: "transfer" as const,
				args: [omniAccountAddress, token.balance],
				chain: walletClient.chain,
				account: walletClient.account!,
			});

			// Wait for transaction
			await publicClient.waitForTransactionReceipt({ hash });

			// Refresh both balances
			await Promise.all([fetchTokenBalances(), fetchWalletBalances()]);

			// Show success message
			alert(
				`Successfully transferred ${formatUnits(token.balance, token.decimals)} ${token.symbol} to your Omni Account!`,
			);
		} catch (error) {
			console.error("Error transferring tokens:", error);
			alert(`Failed to transfer ${token.symbol}. Please try again.`);
		} finally {
			setTransferringToken(null);
		}
	};

	// Initial balance fetch
	useEffect(() => {
		fetchTokenBalances();
		fetchWalletBalances();
	}, [omniAccountAddress, evmAddress, publicClient]);

	// Poll balances every 10 seconds
	useEffect(() => {
		if (omniAccountAddress || evmAddress) {
			const interval = setInterval(() => {
				fetchTokenBalances();
				fetchWalletBalances();
			}, 10000);

			return () => clearInterval(interval);
		}
	}, [omniAccountAddress, evmAddress]);

	// Check if any ERC20 tokens are present
	const hasERC20Tokens = tokenBalances.some(
		(tb) => tb.symbol !== "ETH" && tb.balance > BigInt(0),
	);

	// Check if wallet has tokens that need to be transferred
	const hasWalletTokens = walletTokenBalances.some(
		(tb) => tb.balance > BigInt(0),
	);

	// Notify when tokens are added
	useEffect(() => {
		if (hasERC20Tokens && onTokensAdded) {
			onTokensAdded();
		}
	}, [hasERC20Tokens, onTokensAdded]);

	const copyToClipboard = async (text: string) => {
		try {
			await navigator.clipboard.writeText(text);
			// Could add toast notification here
		} catch (err) {
			console.error("Failed to copy text: ", err);
		}
	};

	if (!omniAccountAddress) {
		return (
			<div className="w-full p-6 bg-yellow-50 rounded-lg border border-yellow-200">
				<div className="text-center">
					<AlertCircle className="mx-auto h-12 w-12 text-yellow-500 mb-4" />
					<h3 className="text-lg font-medium text-yellow-800 mb-2">
						AA Wallet Not Ready
					</h3>
					<p className="text-yellow-700">
						Your Omni Account address is not available yet.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full p-6 bg-white rounded-lg shadow-lg">
			<div className="text-center mb-6">
				<Coins className="mx-auto h-12 w-12 text-purple-500 mb-4" />
				<h2 className="text-2xl font-bold">Transfer ERC20 Tokens</h2>
				<p className="text-gray-600 mt-2">
					Transfer test USDC or USDT from your wallet to your Omni Account
				</p>
			</div>

			{/* Alert if wallet has tokens but Omni Account doesn't */}
			{hasWalletTokens && !hasERC20Tokens && (
				<div className="mb-6">
					<div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
						<div className="flex items-center">
							<AlertCircle className="h-5 w-5 text-yellow-500 mr-2" />
							<span className="text-yellow-700 font-medium">
								Tokens in your wallet need to be transferred
							</span>
						</div>
						<p className="text-yellow-600 text-sm mt-2">
							You have tokens in your wallet. Use the "Transfer to Omni" button
							below to move them to your Omni Account.
						</p>
					</div>
				</div>
			)}

			{/* Wallet Balances Section */}
			{evmAddress && (
				<div className="mb-6">
					<div className="flex items-center justify-between mb-3">
						<h3 className="text-lg font-semibold">Your Wallet Balances</h3>
						<button
							onClick={() => fetchWalletBalances()}
							disabled={isRefreshingWallet}
							className="p-2 hover:bg-gray-100 rounded-lg transition-colors disabled:opacity-50"
						>
							<RefreshCw
								className={`h-4 w-4 ${isRefreshingWallet ? "animate-spin" : ""}`}
							/>
						</button>
					</div>
					<div className="space-y-2">
						{walletTokenBalances.map((tb) => (
							<div
								key={tb.symbol}
								className={`p-3 rounded-lg flex justify-between items-center ${
									tb.symbol === "USDC" ? "bg-blue-50" : "bg-green-50"
								}`}
							>
								<span className="font-medium">{tb.symbol}</span>
								<div className="flex items-center gap-2">
									<span className="font-mono text-sm">
										{formatUnits(tb.balance, tb.decimals)} {tb.symbol}
									</span>
									{tb.balance > BigInt(0) ? (
										<button
											onClick={() => transferToOmniAccount(tb)}
											disabled={transferringToken !== null}
											className={`px-3 py-1 text-white text-sm rounded-md transition-colors disabled:opacity-50 ${
												tb.symbol === "USDC"
													? "bg-blue-600 hover:bg-blue-700"
													: "bg-green-600 hover:bg-green-700"
											}`}
										>
											{transferringToken === tb.symbol
												? "Transferring..."
												: "Transfer to Omni"}
										</button>
									) : (
										<button
											onClick={() => {
												const token =
													tb.symbol === "USDC"
														? TEST_TOKENS.USDC
														: TEST_TOKENS.USDT;
												mintTestTokens(token);
											}}
											disabled={mintingToken !== null}
											className={`px-3 py-1 text-white text-sm rounded-md transition-colors disabled:opacity-50 ${
												tb.symbol === "USDC"
													? "bg-blue-500 hover:bg-blue-600"
													: "bg-green-500 hover:bg-green-600"
											}`}
										>
											{mintingToken === tb.symbol
												? "Minting..."
												: `Mint 1000 ${tb.symbol}`}
										</button>
									)}
								</div>
							</div>
						))}
					</div>
				</div>
			)}

			{/* Omni Account Balances Section */}
			<div className="mb-6">
				<div className="flex items-center justify-between mb-3">
					<h3 className="text-lg font-semibold">Omni Account Balances</h3>
					<button
						onClick={() => fetchTokenBalances()}
						disabled={isRefreshing}
						className="p-2 hover:bg-gray-100 rounded-lg transition-colors disabled:opacity-50"
					>
						<RefreshCw
							className={`h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`}
						/>
					</button>
				</div>
				<div className="space-y-2">
					{tokenBalances.map((tb) => (
						<div
							key={tb.symbol}
							className={`p-3 rounded-lg flex justify-between items-center ${
								tb.symbol === "ETH"
									? "bg-gray-50"
									: tb.symbol === "USDC"
										? "bg-blue-50"
										: "bg-green-50"
							}`}
						>
							<span className="font-medium">{tb.symbol}</span>
							<span className="font-mono text-sm">
								{formatUnits(tb.balance, tb.decimals)} {tb.symbol}
							</span>
						</div>
					))}
				</div>
			</div>

			{hasERC20Tokens && (
				<div className="mb-6">
					<div className="bg-green-50 border border-green-200 rounded-lg p-4">
						<div className="flex items-center">
							<CheckCircle className="h-5 w-5 text-green-500 mr-2" />
							<span className="text-green-700 font-medium">
								ERC20 Tokens Added
							</span>
						</div>
						<p className="text-green-600 text-sm mt-2">
							Your Omni Account has ERC20 tokens and is ready for swapping. You
							can add more tokens or proceed to the next step.
						</p>
					</div>
				</div>
			)}
		</div>
	);
}

