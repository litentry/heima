"use client";

import { useState, useEffect } from "react";
import { useAccount, useBalance, usePublicClient } from "wagmi";
import { QrCode, Copy, AlertCircle, CheckCircle, Wallet } from "lucide-react";

interface FundingGuideProps {
	aaWalletAddress?: string;
	onFundingComplete?: () => void;
}

export function FundingGuide({
	aaWalletAddress,
	onFundingComplete,
}: FundingGuideProps) {
	const { address: evmAddress } = useAccount();
	const publicClient = usePublicClient();
	const [qrCodeUrl, setQrCodeUrl] = useState<string>("");
	const [fundingComplete, setFundingComplete] = useState(false);
	const [hasContract, setHasContract] = useState(false);

	// Check if contract exists before monitoring balance
	useEffect(() => {
		const checkContract = async () => {
			if (!aaWalletAddress || !publicClient) {
				setHasContract(false);
				return;
			}

			try {
				const code = await publicClient.getBytecode({
					address: aaWalletAddress as `0x${string}`,
				});
				setHasContract(!!code && code !== "0x");
			} catch (error) {
				console.error("Error checking contract:", error);
				setHasContract(false);
			}
		};

		checkContract();
	}, [aaWalletAddress, publicClient]);

	// Monitor AA wallet ETH balance using eth_getBalance instead of balance()
	const [ethBalance, setEthBalance] = useState<bigint>(BigInt(0));

	const fetchEthBalance = async () => {
		if (!aaWalletAddress || !publicClient) return;

		try {
			const balance = await publicClient.getBalance({
				address: aaWalletAddress as `0x${string}`,
			});
			setEthBalance(balance);
		} catch (error) {
			console.error("Error fetching balance:", error);
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
		fetchEthBalance();
	}, [aaWalletAddress, publicClient]);

	// Check if wallet is funded
	useEffect(() => {
		if (ethBalance > BigInt(0)) {
			setFundingComplete(true);
			// Notify parent component
			if (onFundingComplete) {
				onFundingComplete();
			}
		} else {
			setFundingComplete(false);
		}
	}, [ethBalance, onFundingComplete]);

	// Poll balance every 10 seconds
	useEffect(() => {
		if (aaWalletAddress) {
			const interval = setInterval(() => {
				fetchEthBalance();
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
						Your Smart Account address is not available yet. Please make sure
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
					<QrCode className="mx-auto h-12 w-12 text-blue-500 mb-4" />
				)}
				<h2 className="text-2xl font-bold">
					{fundingComplete ? "Wallet Funded!" : "Fund Your Smart Account"}
				</h2>
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
							Your Smart Account has been funded with{" "}
							{(Number(ethBalance) / 1e18).toFixed(6)} ETH. You can now proceed
							to authorize your root key.
						</p>
					</div>

					<button
						onClick={() => fetchEthBalance()}
						className="w-full bg-green-600 hover:bg-green-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
					>
						Refresh Balance
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
						<h3 className="text-lg font-semibold">Send ETH to this address:</h3>
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

					<div className="space-y-3">
						<h3 className="text-lg font-semibold">Instructions:</h3>
						<ol className="text-sm text-gray-700 space-y-2">
							<li className="flex">
								<span className="bg-blue-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									1
								</span>
								<span>
									Copy the Smart Account address above or scan the QR code
								</span>
							</li>
							<li className="flex">
								<span className="bg-blue-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs mr-2 mt-0.5">
									2
								</span>
								<span>
									Send some ETH (minimum 0.01 ETH recommended) to this address
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

					<div className="bg-gray-50 p-4 rounded-lg">
						<div className="flex justify-between items-center">
							<span className="text-gray-700">Current Balance:</span>
							<span className="font-mono">
								{ethBalance > BigInt(0)
									? `${(Number(ethBalance) / 1e18).toFixed(6)} ETH`
									: "0 ETH"}
							</span>
						</div>
					</div>

					<button
						onClick={() => fetchEthBalance()}
						className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-4 rounded-lg transition-colors"
					>
						Check Balance
					</button>

					<div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
						<div className="flex">
							<AlertCircle className="h-5 w-5 text-yellow-500 mr-2 mt-0.5" />
							<div className="text-sm text-yellow-700">
								<p className="font-medium mb-1">Important:</p>
								<p>
									Make sure you're sending ETH on the correct network. This
									Smart Account will only work on the network where the
									contracts are deployed.
								</p>
							</div>
						</div>
					</div>
				</div>
			)}
		</div>
	);
}

