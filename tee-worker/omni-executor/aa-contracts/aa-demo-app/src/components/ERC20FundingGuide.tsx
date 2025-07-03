"use client";

import { useState, useEffect } from "react";
import { useAccount, usePublicClient, useWalletClient } from "wagmi";
import {
	Copy,
	AlertCircle,
	CheckCircle,
	ChevronDown,
	Coins,
	RefreshCw,
} from "lucide-react";
import { formatUnits, parseUnits } from "viem";
import { SUPPORTED_TOKENS, TEST_TOKENS } from "@/lib/constants";

interface ERC20FundingGuideProps {
	aaWalletAddress?: string;
	onTokensAdded?: () => void;
}

interface TokenBalance {
	symbol: string;
	balance: bigint;
	decimals: number;
}

export function ERC20FundingGuide({
	aaWalletAddress,
	onTokensAdded,
}: ERC20FundingGuideProps) {
	const { address: evmAddress } = useAccount();
	const publicClient = usePublicClient();
	const { data: walletClient } = useWalletClient();
	const [selectedToken, setSelectedToken] = useState<typeof TEST_TOKENS.USDC | typeof TEST_TOKENS.USDT>(TEST_TOKENS.USDC);
	const [showTokenDropdown, setShowTokenDropdown] = useState(false);
	const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([]);
	const [isMinting, setIsMinting] = useState(false);
	const [isRefreshing, setIsRefreshing] = useState(false);

	// Available ERC20 tokens
	const availableTokens = [TEST_TOKENS.USDC, TEST_TOKENS.USDT];

	// Fetch all token balances
	const fetchTokenBalances = async () => {
		if (!aaWalletAddress || !publicClient) return;

		setIsRefreshing(true);
		const balances: TokenBalance[] = [];

		// Fetch ETH balance
		try {
			const ethBalance = await publicClient.getBalance({
				address: aaWalletAddress as `0x${string}`,
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
					args: [aaWalletAddress],
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

	// Mint test tokens
	const mintTestTokens = async (token: typeof TEST_TOKENS.USDC) => {
		if (!walletClient || !evmAddress || !publicClient) return;

		setIsMinting(true);
		try {
			const amount = parseUnits("1000", token.decimals); // Mint 1000 tokens

			const hash = await walletClient.writeContract({
				address: token.address,
				abi: token.abi,
				functionName: "mint",
				args: [evmAddress, amount],
				chain: undefined,
			});

			// Wait for transaction
			await publicClient.waitForTransactionReceipt({ hash });

			// Refresh balances
			await fetchTokenBalances();

			// Show success message (you could use a toast here)
			alert(`Successfully minted 1000 ${token.symbol} to your wallet!`);
		} catch (error) {
			console.error("Error minting tokens:", error);
			alert(`Failed to mint ${token.symbol}. Please try again.`);
		} finally {
			setIsMinting(false);
		}
	};

	// Initial balance fetch
	useEffect(() => {
		fetchTokenBalances();
	}, [aaWalletAddress, publicClient]);

	// Poll balances every 10 seconds
	useEffect(() => {
		if (aaWalletAddress) {
			const interval = setInterval(() => {
				fetchTokenBalances();
			}, 10000);

			return () => clearInterval(interval);
		}
	}, [aaWalletAddress]);

	// Check if any ERC20 tokens are present
	const hasERC20Tokens = tokenBalances.some(
		(tb) => tb.symbol !== "ETH" && tb.balance > BigInt(0)
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

	if (!aaWalletAddress) {
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
				<h2 className="text-2xl font-bold">Add ERC20 Tokens</h2>
				<p className="text-gray-600 mt-2">
					Add USDC or USDT to your Omni Account for swapping
				</p>
			</div>

			{/* Token Balances Section */}
			<div className="mb-6">
				<div className="flex items-center justify-between mb-3">
					<h3 className="text-lg font-semibold">Account Balances</h3>
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
								tb.symbol === "ETH" ? "bg-gray-50" : "bg-purple-50"
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
							Your Omni Account has ERC20 tokens and is ready for swapping.
							You can add more tokens or proceed to the next step.
						</p>
					</div>
				</div>
			)}

			<div className="space-y-6">
				{/* Token Selection Dropdown */}
				<div className="relative">
					<label className="block text-sm font-medium text-gray-700 mb-2">
						Select Token to Add
					</label>
					<button
						onClick={() => setShowTokenDropdown(!showTokenDropdown)}
						className="w-full bg-white border border-gray-300 rounded-lg px-4 py-3 text-left flex items-center justify-between hover:bg-gray-50"
					>
						<span className="font-medium">
							{selectedToken.symbol} - {selectedToken.name}
						</span>
						<ChevronDown className="h-5 w-5 text-gray-400" />
					</button>

					{showTokenDropdown && (
						<div className="absolute top-full mt-1 w-full bg-white border border-gray-300 rounded-lg shadow-lg z-10">
							{availableTokens.map((token) => (
								<button
									key={token.symbol}
									onClick={() => {
										setSelectedToken(token);
										setShowTokenDropdown(false);
									}}
									className="w-full px-4 py-3 text-left hover:bg-gray-50 flex items-center justify-between"
								>
									<span>
										{token.symbol} - {token.name}
									</span>
									{token.symbol === selectedToken.symbol && (
										<CheckCircle className="h-4 w-4 text-purple-500" />
									)}
								</button>
							))}
						</div>
					)}
				</div>

				{/* Address Display */}
				<div className="space-y-3">
					<h3 className="text-lg font-semibold">
						Send {selectedToken.symbol} to:
					</h3>
					<div className="bg-purple-50 p-4 rounded-lg border border-purple-200">
						<div className="flex items-center justify-between">
							<span className="text-sm font-mono text-purple-800 break-all flex-1 mr-2">
								{aaWalletAddress}
							</span>
							<button
								onClick={() => copyToClipboard(aaWalletAddress)}
								className="p-2 hover:bg-purple-100 rounded flex-shrink-0"
							>
								<Copy className="h-4 w-4 text-purple-600" />
							</button>
						</div>
					</div>
				</div>

				{/* Test Token Minting */}
				<div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
					<h4 className="font-medium text-blue-800 mb-2">
						Get Test Tokens
					</h4>
					<p className="text-sm text-blue-700 mb-3">
						For testing purposes, you can mint test tokens directly to your
						wallet, then transfer them to your Omni Account.
					</p>
					<button
						onClick={() => mintTestTokens(selectedToken)}
						disabled={isMinting}
						className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{isMinting
							? "Minting..."
							: `Mint 1000 Test ${selectedToken.symbol} to My Wallet`}
					</button>
				</div>

				{/* Instructions */}
				<div className="space-y-3">
					<h3 className="text-lg font-semibold">Instructions:</h3>
					<ol className="text-sm text-gray-700 space-y-2">
						<li className="flex">
							<span className="bg-purple-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
								1
							</span>
							<span>
								Select the token you want to add (USDC or USDT)
							</span>
						</li>
						<li className="flex">
							<span className="bg-purple-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
								2
							</span>
							<span>
								Either mint test tokens to your wallet or use existing tokens
							</span>
						</li>
						<li className="flex">
							<span className="bg-purple-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
								3
							</span>
							<span>
								Send the tokens to your Omni Account address
							</span>
						</li>
						<li className="flex">
							<span className="bg-purple-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
								4
							</span>
							<span>
								Your balances will update automatically
							</span>
						</li>
					</ol>
				</div>

				<div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
					<div className="flex">
						<AlertCircle className="h-5 w-5 text-yellow-500 mr-2 mt-0.5" />
						<div className="text-sm text-yellow-700">
							<p className="font-medium mb-1">Note:</p>
							<p>
								Adding ERC20 tokens is optional but required for token swaps.
								You can skip this step if you only need ETH functionality.
							</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}