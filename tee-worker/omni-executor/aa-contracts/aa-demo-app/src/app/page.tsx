"use client";

import { useState, useEffect, useCallback } from "react";
import { useAccount, usePublicClient } from "wagmi";
import { WalletConnect } from "@/components/WalletConnect";
import { OmniAccountWalletInfo } from "@/components/OmniAccountWalletInfo";
import { FundingGuide } from "@/components/FundingGuide";
import { ERC20FundingGuide } from "@/components/ERC20FundingGuide";
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
	const [authorizedSigners, setAuthorizedSigners] = useState<string[]>([]);
	const [isLoadingSigners, setIsLoadingSigners] = useState(false);
	const [hasERC20Tokens, setHasERC20Tokens] = useState(false);

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

	// Fetch all root signers by monitoring events
	const fetchSigners = useCallback(async () => {
		if (!aaWalletAddress || !publicClient || !hasContract) return;

		console.log("Fetching signers for account:", aaWalletAddress);
		setIsLoadingSigners(true);
		try {
			// Get all RootSignerAdded and RootSignerRemoved events
			const addedLogs = await publicClient.getLogs({
				address: aaWalletAddress as `0x${string}`,
				event: {
					type: "event",
					name: "RootSignerAdded",
					inputs: [{ name: "root", type: "address", indexed: false }],
				},
				fromBlock: "earliest",
				toBlock: "latest",
			});

			const removedLogs = await publicClient.getLogs({
				address: aaWalletAddress as `0x${string}`,
				event: {
					type: "event",
					name: "RootSignerRemoved",
					inputs: [{ name: "root", type: "address", indexed: false }],
				},
				fromBlock: "earliest",
				toBlock: "latest",
			});

			// Also get the initial signer from AccountInitialized event
			const initLogs = await publicClient.getLogs({
				address: aaWalletAddress as `0x${string}`,
				event: {
					type: "event",
					name: "AccountInitialized",
					inputs: [
						{ name: "entryPoint", type: "address", indexed: true },
						{ name: "owner", type: "bytes32", indexed: true },
						{ name: "clientId", type: "bytes", indexed: false },
						{ name: "root", type: "address", indexed: true },
					],
				},
				fromBlock: "earliest",
				toBlock: "latest",
			});

			// Build current signer list
			const signerMap = new Map<string, boolean>();

			initLogs.forEach((log) => {
				const root = log.args?.root as string;
				if (root) {
					signerMap.set(root.toLowerCase(), true);
				}
			});

			addedLogs.forEach((log) => {
				const root = log.args?.root as string;
				if (root) {
					signerMap.set(root.toLowerCase(), true);
				}
			});

			removedLogs.forEach((log) => {
				const root = log.args?.root as string;
				if (root) {
					signerMap.delete(root.toLowerCase());
				}
			});

			const currentSigners = Array.from(signerMap.keys()).filter((s) =>
				signerMap.get(s),
			);
			setAuthorizedSigners(currentSigners);
		} catch (error) {
			console.error("Error fetching signers:", error);
		} finally {
			setIsLoadingSigners(false);
		}
	}, [aaWalletAddress, publicClient, hasContract]);

	useEffect(() => {
		if (hasContract) {
			fetchSigners();
		}
	}, [hasContract, fetchSigners]);

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
		if (authorizedSigners.length > 0) {
			setIsAuthorized(true);
		}

		if (!evmAddress) {
			setCurrentStep(1);
		} else if (!isFunded) {
			setCurrentStep(3);
		} else if (!isAuthorized) {
			setCurrentStep(4);
		} else if (!hasERC20Tokens) {
			setCurrentStep(5);
		} else {
			setCurrentStep(6);
		}
	}, [evmAddress, isFunded, isAuthorized, authorizedSigners, hasERC20Tokens]);

	const steps = [
		{
			id: 1,
			title: "Connect Wallet",
			description: "Connect your EVM or Solana wallet",
			completed: !!evmAddress,
		},
		{
			id: 2,
			title: "View Omni Account Wallet",
			description: "See your pre-calculated Omni Account address",
			completed: !!evmAddress,
		},
		{
			id: 3,
			title: "Fund with ETH",
			description: "Send ETH to your Omni Account for gas",
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
			title: "Add ERC20 Tokens",
			description: "Fund your account with USDC or USDT",
			completed: hasERC20Tokens,
		},
		{
			id: 6,
			title: "Ready to Swap",
			description: "Send swap requests to the worker",
			completed: false,
		},
	];

	const filteredSteps = steps.filter(
		(step) => !(step.id === 4 && isAuthorized && authorizedSigners.length > 0),
	);

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
							Step {currentStep} of {filteredSteps.length}
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
								{filteredSteps.map((step) => (
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
									<OmniAccountWalletInfo
										onAddressCalculated={setAAWalletAddress}
									/>
								</div>
							)}

							{currentStep >= 3 && currentStep <= 3 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 3: Fund Your Omni Account with ETH
									</h2>
									<p className="text-gray-600 mb-6">
										Send ETH to your Omni Account address to pay for gas fees.
									</p>
									<FundingGuide
										aaWalletAddress={aaWalletAddress}
										onFundingComplete={() => {
											console.log("ETH funding complete callback triggered");
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
										onAuthorizationComplete={() => {
											setIsAuthorized(true);
											fetchSigners();
										}}
									/>
								</div>
							)}

							{/* Show authorized signers only after account is funded and deployed */}
							{isFunded && hasContract && aaWalletAddress && (
								<div className="mt-8">
									<AuthorizedSigners
										aaWalletAddress={aaWalletAddress}
										isDeployed={hasContract}
										signers={authorizedSigners}
										isLoading={isLoadingSigners}
										refreshSigners={fetchSigners}
									/>
								</div>
							)}

							{currentStep >= 5 && currentStep <= 5 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 5: Add ERC20 Tokens
									</h2>
									<p className="text-gray-600 mb-6">
										Optionally add USDC or USDT to your Omni Account for token
										swaps.
									</p>
									<ERC20FundingGuide
										aaWalletAddress={aaWalletAddress}
										onTokensAdded={() => {
											console.log("ERC20 tokens added");
											setHasERC20Tokens(true);
										}}
									/>
								</div>
							)}

							{currentStep >= 6 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 6: Start Swapping
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
												Ready for tokens
											</span>
										</div>
									</div>
								)}

								{hasERC20Tokens && (
									<div className="bg-white rounded-lg shadow p-4">
										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-3">
												<div className="w-6 h-6 bg-green-500 rounded-full flex items-center justify-center">
													<Check className="w-4 h-4 text-white" />
												</div>
												<span className="font-medium text-green-700">
													ERC20 Tokens Added
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
