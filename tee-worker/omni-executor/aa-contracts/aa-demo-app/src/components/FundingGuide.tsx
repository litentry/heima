"use client";

import { useState, useEffect } from "react";
import { useAccount, usePublicClient, useWalletClient } from "wagmi";
import {
	QrCode,
	Copy,
	AlertCircle,
	CheckCircle,
	Wallet,
	ChevronDown,
	Coins,
} from "lucide-react";
import { formatUnits, parseUnits } from "viem";
import { SUPPORTED_TOKENS, TEST_TOKENS } from "@/lib/constants";

interface FundingGuideProps {
	aaWalletAddress?: string;
	onFundingComplete?: () => void;
}

interface TokenBalance {
	symbol: string;
	balance: bigint;
	decimals: number;
}

export function FundingGuide({
	aaWalletAddress,
	onFundingComplete,
}: FundingGuideProps) {
	const { address: evmAddress } = useAccount();
	const publicClient = usePublicClient();
	const { data: walletClient } = useWalletClient();
	const [qrCodeUrl, setQrCodeUrl] = useState<string>("");
	const [fundingComplete, setFundingComplete] = useState(false);
	const [selectedToken, setSelectedToken] = useState(SUPPORTED_TOKENS[0]);
	const [showTokenDropdown, setShowTokenDropdown] = useState(false);
	const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([]);
	const [isMinting, setIsMinting] = useState(false);

	// Fetch all token balances
	const fetchTokenBalances = async () => {
		if (!aaWalletAddress || !publicClient) return;

		const balances: TokenBalance[] = [];

		for (const token of SUPPORTED_TOKENS) {
			try {
				let balance: bigint;

				if (token.isNative) {
					// Native ETH balance
					balance = await publicClient.getBalance({
						address: aaWalletAddress as `0x${string}`,
					});
				} else {
					// ERC20 token balance
					balance = (await publicClient.readContract({
						address: token.address,
						abi: token.abi!,
						functionName: "balanceOf",
						args: [aaWalletAddress],
					})) as bigint;
				}

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
	};

	// Mint test tokens
	const mintTestTokens = async (tokenSymbol: string) => {
		if (!walletClient || !evmAddress) return;

		setIsMinting(true);
		try {
			const token =
				tokenSymbol === "USDC" ? TEST_TOKENS.USDC : TEST_TOKENS.USDT;
			const amount = parseUnits("1000", token.decimals); // Mint 1000 tokens

			const hash = await walletClient.writeContract({
				address: token.address,
				abi: token.abi,
				functionName: "mint",
				args: [evmAddress, amount],
			});

			// Wait for transaction
			await publicClient?.waitForTransactionReceipt({ hash });

			// Refresh balances
			fetchTokenBalances();

			// Show success message (you could use a toast here)
			alert(`Successfully minted 1000 ${tokenSymbol} to your wallet!`);
		} catch (error) {
			console.error("Error minting tokens:", error);
			alert(`Failed to mint ${tokenSymbol}. Please try again.`);
		} finally {
			setIsMinting(false);
		}
	};

	// Generate QR code URL for the AA wallet address
	useEffect(() => {
		if (aaWalletAddress) {
			// Using a simple QR code service
			const qrUrl = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${aaWalletAddress}`;
			setQrCodeUrl(qrUrl);
		}
	}, [aaWalletAddress]);

	// Initial balance fetch
	useEffect(() => {
		fetchTokenBalances();
	}, [aaWalletAddress, publicClient]);

	// Check if wallet is funded (has any token balance)
	useEffect(() => {
		const hasFunds = tokenBalances.some((tb) => tb.balance > BigInt(0));
		setFundingComplete(hasFunds);

		if (hasFunds && onFundingComplete) {
			onFundingComplete();
		}
	}, [tokenBalances, onFundingComplete]);

	// Poll balances every 10 seconds
	useEffect(() => {
		if (aaWalletAddress) {
			const interval = setInterval(() => {
				fetchTokenBalances();
			}, 10000);

			return () => clearInterval(interval);
		}
	}, [aaWalletAddress]);

	const copyToClipboard = async (text: string) => {
		try {
			await navigator.clipboard.writeText(text);
			// Could add toast notification here
		} catch (err) {
			console.error("Failed to copy text: ", err);
		}
	};

	if (!evmAddress) {
		return (
			<div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
				<div className="text-center">
					<Wallet className="mx-auto h-12 w-12 text-gray-400 mb-4" />
					<h3 className="text-lg font-medium text-gray-900 mb-2">
						Connect Wallet First
					</h3>
					<p className="text-gray-600">
						Please connect your EVM wallet to see funding instructions.
					</p>
				</div>
			</div>
		);
	}

	if (!aaWalletAddress) {
		return (
			<div className="w-full p-6 bg-yellow-50 rounded-lg border border-yellow-200">
				<div className="text-center">
					<AlertCircle className="mx-auto h-12 w-12 text-yellow-500 mb-4" />
					<h3 className="text-lg font-medium text-yellow-800 mb-2">
						AA Wallet Not Ready
					</h3>
					<p className="text-yellow-700">
						Your Omni Account address is not available yet. Please make sure
						your wallet is connected.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full p-6 bg-white rounded-lg shadow-lg">
			<div className="text-center mb-6">
				{fundingComplete ? (
					<CheckCircle className="mx-auto h-12 w-12 text-green-500 mb-4" />
				) : (
					<Coins className="mx-auto h-12 w-12 text-blue-500 mb-4" />
				)}
				<h2 className="text-2xl font-bold">
					{fundingComplete ? "Wallet Funded!" : "Fund Your Omni Account"}
				</h2>
				<p className="text-gray-600 mt-2">
					{fundingComplete
						? "Your Omni Account has funds and is ready to use"
						: "Send ETH or tokens to your Omni Account to get started"}
				</p>
			</div>

			{/* Token Balances Section */}
			<div className="mb-6">
				<h3 className="text-lg font-semibold mb-3">Account Balances</h3>
				<div className="space-y-2">
					{tokenBalances.map((tb) => (
						<div
							key={tb.symbol}
							className="bg-gray-50 p-3 rounded-lg flex justify-between items-center"
						>
							<span className="font-medium">{tb.symbol}</span>
							<span className="font-mono text-sm">
								{formatUnits(tb.balance, tb.decimals)} {tb.symbol}
							</span>
						</div>
					))}
				</div>
			</div>

			{fundingComplete ? (
				<div className="space-y-4">
					<div className="bg-green-50 border border-green-200 rounded-lg p-4">
						<div className="flex items-center">
							<CheckCircle className="h-5 w-5 text-green-500 mr-2" />
							<span className="text-green-700 font-medium">
								Funding Complete
							</span>
						</div>
						<p className="text-green-600 text-sm mt-2">
							Your Omni Account has been funded. You can now proceed to
							authorize your root key.
						</p>
					</div>

					<button
						onClick={() => fetchTokenBalances()}
						className="w-full bg-green-600 hover:bg-green-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
					>
						Refresh Balances
					</button>

					{onFundingComplete && (
						<button
							onClick={() => onFundingComplete()}
							className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
						>
							Continue to Step 4 →
						</button>
					)}
				</div>
			) : (
				<div className="space-y-6">
					{/* Token Selection Dropdown */}
					<div className="relative">
						<label className="block text-sm font-medium text-gray-700 mb-2">
							Select Token to Send
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
								{SUPPORTED_TOKENS.map((token) => (
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
											<CheckCircle className="h-4 w-4 text-blue-500" />
										)}
									</button>
								))}
							</div>
						)}
					</div>

					{qrCodeUrl && (
						<div className="text-center">
							<img
								src={qrCodeUrl}
								alt="AA Wallet QR Code"
								className="mx-auto border rounded-lg"
								width={200}
								height={200}
							/>
							<p className="text-sm text-gray-600 mt-2">
								Scan with your mobile wallet
							</p>
						</div>
					)}

					<div className="space-y-3">
						<h3 className="text-lg font-semibold">
							Send {selectedToken.symbol} to this address:
						</h3>
						<div className="bg-blue-50 p-4 rounded-lg border border-blue-200">
							<div className="flex items-center justify-between">
								<span className="text-sm font-mono text-blue-800 break-all flex-1 mr-2">
									{aaWalletAddress}
								</span>
								<button
									onClick={() => copyToClipboard(aaWalletAddress)}
									className="p-2 hover:bg-blue-100 rounded flex-shrink-0"
								>
									<Copy className="h-4 w-4 text-blue-600" />
								</button>
							</div>
						</div>
					</div>

					{/* Test Token Minting Section */}
					{!selectedToken.isNative && (
						<div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
							<h4 className="font-medium text-blue-800 mb-2">
								Get Test Tokens
							</h4>
							<p className="text-sm text-blue-700 mb-3">
								For testing purposes, you can mint test tokens directly to your
								wallet.
							</p>
							<button
								onClick={() => mintTestTokens(selectedToken.symbol)}
								disabled={isMinting}
								className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
							>
								{isMinting
									? "Minting..."
									: `Mint 1000 Test ${selectedToken.symbol}`}
							</button>
						</div>
					)}

					<div className="space-y-3">
						<h3 className="text-lg font-semibold">Instructions:</h3>
						<ol className="text-sm text-gray-700 space-y-2">
							<li className="flex">
								<span className="bg-blue-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									1
								</span>
								<span>
									Select the token you want to send from the dropdown above
								</span>
							</li>
							<li className="flex">
								<span className="bg-blue-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									2
								</span>
								<span>
									{selectedToken.isNative
										? "Send some ETH (minimum 0.01 ETH recommended) to the address"
										: `Send some ${selectedToken.symbol} tokens to the address`}
								</span>
							</li>
							<li className="flex">
								<span className="bg-blue-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									3
								</span>
								<span>
									Wait for the transaction to confirm (we'll detect it
									automatically)
								</span>
							</li>
							<li className="flex">
								<span className="bg-blue-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									4
								</span>
								<span>
									Once funded, you can proceed to set up your root key
								</span>
							</li>
						</ol>
					</div>

					<button
						onClick={() => fetchTokenBalances()}
						className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
					>
						Check Balances
					</button>

					<div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
						<div className="flex">
							<AlertCircle className="h-5 w-5 text-yellow-500 mr-2 mt-0.5" />
							<div className="text-sm text-yellow-700">
								<p className="font-medium mb-1">Important:</p>
								<p>
									Make sure you're sending tokens on the correct network. This
									Omni Account will only work on the network where the contracts
									are deployed.
								</p>
							</div>
						</div>
					</div>
				</div>
			)}
		</div>
	);
}

