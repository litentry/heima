"use client";

import { useState, useEffect, useCallback } from "react";
import { useAccount, usePublicClient } from "wagmi";
import { WalletConnect } from "@/components/WalletConnect";
import { AAWalletInfo } from "@/components/AAWalletInfo";
import { FundingGuide } from "@/components/FundingGuide";
import { RootKeyAuthorization } from "@/components/RootKeyAuthorization";
import { AuthorizedSigners } from "@/components/AuthorizedSigners";
import { ClientOnly } from "@/components/ClientOnly";
import { ChevronRight, Check } from "lucide-react";

function HomeContent() {
	const { address: evmAddress, chain, isConnected } = useAccount();
	const publicClient = usePublicClient();
	const [aaWalletAddress, setAAWalletAddress] = useState<string>("");
	const [currentStep, setCurrentStep] = useState(1);
	const [isAuthorized, setIsAuthorized] = useState(false);
	const [hasContract, setHasContract] = useState(false);

	// Debug logging
	useEffect(() => {
		console.log("HomeContent Debug:", {
			evmAddress,
			chainId: chain?.id,
			chainName: chain?.name,
			isConnected,
			aaWalletAddress,
			publicClientChainId: publicClient?.chain?.id,
		});
	}, [evmAddress, chain, isConnected, aaWalletAddress, publicClient]);

	// Check if Omni Account contract exists
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
	}, [aaWalletAddress, publicClient, isAuthorized]);

	// Monitor AA wallet ETH balance
	const [ethBalance, setEthBalance] = useState<bigint>(BigInt(0));

	const fetchEthBalance = useCallback(async () => {
		if (!aaWalletAddress || !publicClient) return;

		try {
			const balance = await publicClient.getBalance({
				address: aaWalletAddress as `0x${string}`,
			});
			setEthBalance(balance);
			console.log(
				"Fetched balance:",
				balance.toString(),
				"for address:",
				aaWalletAddress,
			);
		} catch (error) {
			console.error("Error fetching balance:", error);
		}
	}, [aaWalletAddress, publicClient]);

	// Fetch balance when address changes or contract is deployed
	useEffect(() => {
		fetchEthBalance();
	}, [fetchEthBalance, hasContract]);

	// Also poll for balance updates every 5 seconds when on step 3
	useEffect(() => {
		if (currentStep === 3 && aaWalletAddress && publicClient) {
			// Immediate check when entering step 3
			fetchEthBalance();

			const interval = setInterval(() => {
				fetchEthBalance();
			}, 5000);

			return () => clearInterval(interval);
		}
	}, [currentStep, fetchEthBalance, aaWalletAddress, publicClient]);

	const isFunded = ethBalance > BigInt(0);

	// Debug logging
	useEffect(() => {
		console.log("Balance state:", {
			ethBalance: ethBalance.toString(),
			isFunded,
			currentStep,
			aaWalletAddress,
		});
	}, [ethBalance, isFunded, currentStep, aaWalletAddress]);

	// Update current step based on completion status
	useEffect(() => {
		if (!evmAddress) {
			setCurrentStep(1);
		} else if (!isFunded) {
			setCurrentStep(3);
		} else if (!isAuthorized) {
			setCurrentStep(4);
		} else {
			setCurrentStep(5);
		}
	}, [evmAddress, isFunded, isAuthorized]);

	const steps = [
		{
			id: 1,
			title: "Connect Wallet",
			description: "Connect your EVM or Solana wallet",
			completed: !!evmAddress,
		},
		{
			id: 2,
			title: "View AA Wallet",
			description: "See your pre-calculated Omni Account address",
			completed: !!evmAddress,
		},
		{
			id: 3,
			title: "Fund Wallet",
			description: "Send ETH to your Omni Account",
			completed: !!isFunded,
		},
		{
			id: 4,
			title: "Authorize Root Key",
			description: "Set up delegated access permissions",
			completed: isAuthorized,
		},
		{
			id: 5,
			title: "Ready to Swap",
			description: "Send swap requests to the worker",
			completed: false,
		},
	];

	return (
		<div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
			{/* Header */}
			<header className="bg-white shadow-sm border-b">
				<div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
					<div className="flex items-center justify-between">
						<div>
							<h1 className="text-2xl font-bold text-gray-900">
								Account Abstraction Demo
							</h1>
							<p className="text-gray-600 mt-1">
								Experience seamless multi-chain wallet interactions with AA
							</p>
						</div>
						<div className="text-sm text-gray-500">
							Step {currentStep} of {steps.length}
						</div>
					</div>
				</div>
			</header>

			<div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
				<div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
					{/* Progress Sidebar */}
					<div className="lg:col-span-1">
						<div className="bg-white rounded-lg shadow-lg p-6 sticky top-8">
							<h2 className="text-lg font-semibold mb-6">Setup Progress</h2>
							<div className="space-y-4">
								{steps.map((step, index) => (
									<div
										key={step.id}
										className={`flex items-start space-x-3 p-3 rounded-lg transition-colors ${
											step.id === currentStep
												? "bg-blue-50 border border-blue-200"
												: step.completed
													? "bg-green-50"
													: "bg-gray-50"
										}`}
									>
										<div
											className={`flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center text-sm font-medium ${
												step.completed
													? "bg-green-500 text-white"
													: step.id === currentStep
														? "bg-blue-500 text-white"
														: "bg-gray-300 text-gray-600"
											}`}
										>
											{step.completed ? <Check className="w-4 h-4" /> : step.id}
										</div>
										<div>
											<h3
												className={`font-medium ${
													step.completed
														? "text-green-700"
														: step.id === currentStep
															? "text-blue-700"
															: "text-gray-700"
												}`}
											>
												{step.title}
											</h3>
											<p className="text-sm text-gray-600 mt-1">
												{step.description}
											</p>
										</div>
										{step.id === currentStep && (
											<ChevronRight className="w-5 h-5 text-blue-500 ml-auto" />
										)}
									</div>
								))}
							</div>

							{/* Demo Info */}
							<div className="mt-8 p-4 bg-purple-50 border border-purple-200 rounded-lg">
								<h3 className="font-medium text-purple-800 mb-2">
									Demo Features
								</h3>
								<ul className="text-sm text-purple-700 space-y-1">
									<li>• Multi-chain wallet support</li>
									<li>• ERC20 token support (USDC, USDT)</li>
									<li>• Multiple signer management</li>
									<li>• View authorized signers list</li>
									<li>• Test token minting</li>
									<li>• Token balance monitoring</li>
									<li>• Root key delegation</li>
									<li>• Worker integration ready</li>
								</ul>
							</div>
						</div>
					</div>

					{/* Main Content */}
					<div className="lg:col-span-2">
						<div className="space-y-8">
							{/* Step Content */}
							{currentStep === 1 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 1: Connect Your Wallet
									</h2>
									<p className="text-gray-600 mb-6">
										Connect your EVM wallet (MetaMask, etc.) or Solana wallet
										(Phantom, etc.) to get started.
									</p>
									<WalletConnect />
								</div>
							)}

							{currentStep >= 2 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 2: View Your Omni Account
									</h2>
									<p className="text-gray-600 mb-6">
										Your Omni Account address is pre-calculated using your
										wallet address and client ID.
									</p>
									<AAWalletInfo onAddressCalculated={setAAWalletAddress} />
								</div>
							)}

							{currentStep >= 3 && currentStep <= 3 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 3: Fund Your Omni Account
									</h2>
									<p className="text-gray-600 mb-6">
										Send some ETH to your Omni Account address to enable Account
										Abstraction features.
									</p>
									<FundingGuide
										aaWalletAddress={aaWalletAddress}
										onFundingComplete={() => {
											console.log("Funding complete callback triggered");
											fetchEthBalance();
										}}
									/>
								</div>
							)}

							{currentStep >= 4 && currentStep <= 4 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 4: Authorize Root Key
									</h2>
									<p className="text-gray-600 mb-6">
										Set up a root signer that can create sessions and delegate
										access to your Omni Account.
									</p>
									<RootKeyAuthorization
										aaWalletAddress={aaWalletAddress}
										isFunded={!!isFunded}
										onAuthorizationComplete={() => setIsAuthorized(true)}
									/>
								</div>
							)}

							{/* Show authorized signers only after account is funded and deployed */}
							{isFunded && hasContract && aaWalletAddress && (
								<div className="mt-8">
									<AuthorizedSigners
										aaWalletAddress={aaWalletAddress}
										isDeployed={hasContract}
									/>
								</div>
							)}

							{currentStep >= 5 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 5: Start Swapping
									</h2>
									<p className="text-gray-600 mb-6">
										Your Omni Account is ready! Send swap requests to the TEE
										worker service.
									</p>
									{/* TODO */}
								</div>
							)}

							{/* Always show completed steps in collapsed form */}
							<div className="space-y-4">
								{currentStep > 1 && (
									<div className="bg-white rounded-lg shadow p-4">
										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-3">
												<div className="w-6 h-6 bg-green-500 rounded-full flex items-center justify-center">
													<Check className="w-4 h-4 text-white" />
												</div>
												<span className="font-medium text-green-700">
													Wallet Connected
												</span>
											</div>
											<span className="text-sm text-gray-500">
												{evmAddress &&
													`${evmAddress.slice(0, 6)}...${evmAddress.slice(-4)}`}
											</span>
										</div>
									</div>
								)}

								{currentStep > 3 && isFunded && (
									<div className="bg-white rounded-lg shadow p-4">
										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-3">
												<div className="w-6 h-6 bg-green-500 rounded-full flex items-center justify-center">
													<Check className="w-4 h-4 text-white" />
												</div>
												<span className="font-medium text-green-700">
													Omni Account Funded
												</span>
											</div>
											<span className="text-sm text-gray-500">
												{(Number(ethBalance) / 1e18).toFixed(6)} ETH
											</span>
										</div>
									</div>
								)}

								{isAuthorized && (
									<div className="bg-white rounded-lg shadow p-4">
										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-3">
												<div className="w-6 h-6 bg-green-500 rounded-full flex items-center justify-center">
													<Check className="w-4 h-4 text-white" />
												</div>
												<span className="font-medium text-green-700">
													Root Key Authorized
												</span>
											</div>
											<span className="text-sm text-gray-500">
												Ready for swaps
											</span>
										</div>
									</div>
								)}
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}

export default function Home() {
	return (
		<ClientOnly>
			<HomeContent />
		</ClientOnly>
	);
}
