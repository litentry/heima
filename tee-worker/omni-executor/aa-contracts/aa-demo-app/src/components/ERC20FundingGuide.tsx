"use client";

import { useState, useEffect } from "react";
import { useAccount, usePublicClient, useWalletClient } from "wagmi";
import { AlertCircle, CheckCircle, Coins, RefreshCw } from "lucide-react";
import { formatUnits, parseUnits } from "viem";
import { ERC20_TOKENS } from "@/lib/constants";

interface TokenBalance {
	symbol: string;
	balance: bigint;
	decimals: number;
}

interface ERC20FundingGuideProps {
	omniAccountAddress?: string;
	onTokensAdded?: () => void;
	hasERC20Tokens?: boolean;
}

interface WalletTokenBalance extends TokenBalance {
	address: `0x${string}`;
	abi: any;
}

export function ERC20FundingGuide({
	omniAccountAddress: omniAccountAddress,
	onTokensAdded,
	hasERC20Tokens,
}: ERC20FundingGuideProps) {
	const { address: evmAddress } = useAccount();
	const publicClient = usePublicClient();
	const { data: walletClient } = useWalletClient();
	const [walletTokenBalances, setWalletTokenBalances] = useState<
		WalletTokenBalance[]
	>([]);
	const [mintingToken, setMintingToken] = useState<string | null>(null);
	const [transferringToken, setTransferringToken] = useState<string | null>(
		null,
	);
	const [isRefreshingWallet, setIsRefreshingWallet] = useState(false);
	const [tokensTransferred, setTokensTransferred] = useState(false);

	// Available ERC20 tokens
	const availableTokens = [ERC20_TOKENS.USDC, ERC20_TOKENS.USDT];

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
		token: typeof ERC20_TOKENS.USDC | typeof ERC20_TOKENS.USDT,
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
				abi: token.abi || ERC20_TOKENS.USDC.abi,
				functionName: "transfer" as const,
				args: [omniAccountAddress, token.balance],
				chain: walletClient.chain,
				account: walletClient.account!,
			});

			// Wait for transaction
			await publicClient.waitForTransactionReceipt({ hash });

			// Refresh wallet balances
			await fetchWalletBalances();

			// Mark as transferred
			setTokensTransferred(true);

			// Notify parent
			if (onTokensAdded) {
				onTokensAdded();
			}

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

	// Initial wallet balance fetch
	useEffect(() => {
		fetchWalletBalances();
	}, [evmAddress, publicClient]);

	// Poll wallet balances every 10 seconds
	useEffect(() => {
		if (evmAddress) {
			const interval = setInterval(() => {
				fetchWalletBalances();
			}, 10000);

			return () => clearInterval(interval);
		}
	}, [evmAddress]);


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
				{tokensTransferred || hasERC20Tokens ? (
					<CheckCircle className="mx-auto h-12 w-12 text-green-500 mb-4" />
				) : (
					<Coins className="mx-auto h-12 w-12 text-purple-500 mb-4" />
				)}
				<h2 className="text-2xl font-bold">
					{tokensTransferred || hasERC20Tokens ? "Tokens Transferred!" : "Transfer ERC20 Tokens"}
				</h2>
				<p className="text-gray-600 mt-2">
					{tokensTransferred || hasERC20Tokens
						? "Your Omni Account has ERC20 tokens and is ready for swapping"
						: "Transfer test USDC or USDT from your wallet to your Omni Account"}
				</p>
			</div>


			{tokensTransferred || hasERC20Tokens ? (
				<div className="space-y-4">
					<div className="bg-green-50 border border-green-200 rounded-lg p-4">
						<div className="flex items-center">
							<CheckCircle className="h-5 w-5 text-green-500 mr-2" />
							<span className="text-green-700 font-medium">
								ERC20 Tokens Added
							</span>
						</div>
						<p className="text-green-600 text-sm mt-2">
							Your Omni Account has been funded with ERC20 tokens. You can add more tokens or proceed to swap.
						</p>
					</div>

					{/* Still show option to add more tokens */}
					{evmAddress && (
						<div>
							<h3 className="text-lg font-semibold mb-3">Add More Tokens</h3>
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
																? ERC20_TOKENS.USDC
																: ERC20_TOKENS.USDT;
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

					<button
						onClick={() => fetchWalletBalances()}
						className="w-full bg-gray-600 hover:bg-gray-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
					>
						Refresh Balances
					</button>
				</div>
			) : (
				<div className="space-y-6">
					{/* Instructions */}
					<div className="space-y-3">
						<h3 className="text-lg font-semibold">Transfer Test Tokens to Your Omni Account:</h3>
						<ol className="text-sm text-gray-700 space-y-2">
							<li className="flex">
								<span className="bg-purple-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									1
								</span>
								<span>
									Mint test tokens (USDC or USDT) using the buttons below
								</span>
							</li>
							<li className="flex">
								<span className="bg-purple-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									2
								</span>
								<span>
									Once minted, transfer them to your Omni Account
								</span>
							</li>
							<li className="flex">
								<span className="bg-purple-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									3
								</span>
								<span>
									Your tokens will be ready for swapping once transferred
								</span>
							</li>
						</ol>
					</div>

					{/* Wallet Balances Section */}
					{evmAddress && (
						<div>
							<div className="flex items-center justify-between mb-3">
								<h3 className="text-lg font-semibold">Available Test Tokens</h3>
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
																? ERC20_TOKENS.USDC
																: ERC20_TOKENS.USDT;
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

					<button
						onClick={() => fetchWalletBalances()}
						className="w-full bg-gray-600 hover:bg-gray-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
					>
						Check Balance
					</button>

					<div className="bg-purple-50 border border-purple-200 rounded-lg p-3">
						<div className="flex">
							<AlertCircle className="h-5 w-5 text-purple-500 mr-2 mt-0.5" />
							<div className="text-sm text-purple-700">
								<p className="font-medium mb-1">Important:</p>
								<p>
									These are test tokens on a local network. Make sure your Omni
									Account has ETH for gas fees before transferring tokens.
								</p>
							</div>
						</div>
					</div>
				</div>
			)}

		</div>
	);
}

