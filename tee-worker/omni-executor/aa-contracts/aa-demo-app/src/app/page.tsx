"use client";

import { useState, useEffect, useCallback } from "react";
import { useAccount, usePublicClient } from "wagmi";
import { WalletConnect } from "@/components/WalletConnect";
import { AccountsDashboard } from "@/components/AccountsDashboard";
import { FundingGuide } from "@/components/FundingGuide";
import { CreateOmniAccount } from "@/components/CreateOmniAccount";
import { AuthorizedSigners } from "@/components/AuthorizedSigners";
import { AuthorizeTEEWorker } from "@/components/AuthorizeTEEWorker";
import { TEETokenTransfer } from "@/components/TEETokenTransfer";
import { ClientOnly } from "@/components/ClientOnly";
import { ChevronRight, Check } from "lucide-react";

function HomeContent() {
	const { address: evmAddress, chain, isConnected } = useAccount();
	const publicClient = usePublicClient();
	const [omniAccountAddress, setOmniAccountAddress] = useState<string>("");
	const [omniAccountHash, setOmniAccountHash] = useState<string>("");
	const [currentStep, setCurrentStep] = useState(1);
	const [isAuthorized, setIsAuthorized] = useState(false);
	const [hasContract, setHasContract] = useState(false);
	const [authorizedSigners, setAuthorizedSigners] = useState<string[]>([]);
	const [isLoadingSigners, setIsLoadingSigners] = useState(false);
	const [teeWorkerAddress, setTeeWorkerAddress] = useState<string | null>(null);
	const [isTeeWorkerAuthorized, setIsTeeWorkerAuthorized] = useState(false);


	// Debug logging
	useEffect(() => {
		console.log("HomeContent Debug:", {
			evmAddress,
			chainId: chain?.id,
			chainName: chain?.name,
			isConnected,
			omniAccountAddress,
			publicClientChainId: publicClient?.chain?.id,
		});
	}, [evmAddress, chain, isConnected, omniAccountAddress, publicClient]);

	// Check if Omni Account contract exists
	useEffect(() => {
		const checkContract = async () => {
			if (!omniAccountAddress || !publicClient) {
				setHasContract(false);
				return;
			}

			try {
				const code = await publicClient.getCode({
					address: omniAccountAddress as `0x${string}`,
				});
				setHasContract(!!code && code !== "0x");
			} catch (error) {
				console.error("Error checking contract:", error);
				setHasContract(false);
			}
		};

		checkContract();
	}, [omniAccountAddress, publicClient, isAuthorized]);

	// Fetch all root signers by monitoring events
	const fetchSigners = useCallback(async () => {
		if (!omniAccountAddress || !publicClient || !hasContract) return;

		console.log("Fetching signers for account:", omniAccountAddress);
		setIsLoadingSigners(true);
		try {
			// Get all RootSignerAdded and RootSignerRemoved events
			const addedLogs = await publicClient.getLogs({
				address: omniAccountAddress as `0x${string}`,
				event: {
					type: "event",
					name: "RootSignerAdded",
					inputs: [{ name: "root", type: "address", indexed: false }],
				},
				fromBlock: "earliest",
				toBlock: "latest",
			});

			const removedLogs = await publicClient.getLogs({
				address: omniAccountAddress as `0x${string}`,
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
				address: omniAccountAddress as `0x${string}`,
				event: {
					type: "event",
					name: "AccountInitialized",
					inputs: [
						{ name: "entryPoint", type: "address", indexed: true },
						{ name: "owner", type: "bytes32", indexed: true },
						{ name: "ownerType", type: "uint8", indexed: false },
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
	}, [omniAccountAddress, publicClient, hasContract]);

	useEffect(() => {
		if (hasContract) {
			fetchSigners();
		}
	}, [hasContract, fetchSigners]);

	// Monitor AA wallet ETH balance
	const [ethBalance, setEthBalance] = useState<bigint>(BigInt(0));

	const fetchEthBalance = useCallback(async () => {
		if (!omniAccountAddress || !publicClient) return;

		try {
			const balance = await publicClient.getBalance({
				address: omniAccountAddress as `0x${string}`,
			});
			setEthBalance(balance);
			console.log(
				"Fetched balance:",
				balance.toString(),
				"for address:",
				omniAccountAddress,
			);
		} catch (error) {
			console.error("Error fetching balance:", error);
		}
	}, [omniAccountAddress, publicClient]);

	// Fetch balance when address changes
	useEffect(() => {
		fetchEthBalance();
	}, [fetchEthBalance]);

	// Also poll for balance updates every 5 seconds when on step 2 (funding)
	useEffect(() => {
		if (currentStep === 2 && omniAccountAddress && publicClient) {
			// Immediate check when entering step 2
			fetchEthBalance();

			const interval = setInterval(() => {
				fetchEthBalance();
			}, 5000);

			return () => clearInterval(interval);
		}
	}, [currentStep, fetchEthBalance, omniAccountAddress, publicClient]);

	const isFunded = ethBalance > BigInt(0);

	// Debug logging
	useEffect(() => {
		console.log("Balance state:", {
			ethBalance: ethBalance.toString(),
			isFunded,
			currentStep,
			omniAccountAddress,
		});
	}, [ethBalance, isFunded, currentStep, omniAccountAddress]);

	// Update current step based on completion status
	useEffect(() => {
		if (authorizedSigners.length > 0) {
			setIsAuthorized(true);
		}

		if (!evmAddress) {
			setCurrentStep(1);
		} else if (!omniAccountAddress) {
			// Stay on step 1 until we have the omni account address
			setCurrentStep(1);
		} else if (!isFunded) {
			setCurrentStep(2);
		} else if (!isAuthorized) {
			setCurrentStep(3);
		} else if (!isTeeWorkerAuthorized) {
			setCurrentStep(4);
		} else {
			setCurrentStep(5);
		}
	}, [evmAddress, omniAccountAddress, isFunded, isAuthorized, authorizedSigners, isTeeWorkerAuthorized]);

	const steps = [
		{
			id: 1,
			title: "Connect Wallet",
			description: "Connect your EVM or Solana wallet",
			completed: !!evmAddress,
		},
		{
			id: 2,
			title: "Fund with ETH",
			description: "Send ETH to your Omni Account for gas",
			completed: !!isFunded,
		},
		{
			id: 3,
			title: "Create Omni Account",
			description: "Deploy your smart account contract",
			completed: isAuthorized,
		},
		{
			id: 4,
			title: "Authorize TEE Worker",
			description: "Add TEE worker as authorized signer",
			completed: isTeeWorkerAuthorized,
		},
		{
			id: 5,
			title: "Send Token Transfer",
			description: "Transfer tokens through the TEE worker",
			completed: false,
		},
	];

	// For the progress sidebar, we want to show all steps
	const allSteps = steps;

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
							Step {currentStep} of {allSteps.length}
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
								{allSteps.map((step) => (
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

							{/* Show Omni Account details when wallet is connected */}
							{isConnected && evmAddress && (
								<div className="mb-8">
									<AccountsDashboard
										onAddressCalculated={setOmniAccountAddress}
										onOmniAccountCalculated={setOmniAccountHash}
										ethBalance={ethBalance}
										isAccountCreated={isAuthorized}
									/>
								</div>
							)}

							{currentStep >= 2 && currentStep <= 2 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 2: Fund Your Omni Account with ETH
									</h2>
									<p className="text-gray-600 mb-6">
										Send ETH to your Omni Account address to pay for gas fees.
									</p>
									<FundingGuide
										omniAccountAddress={omniAccountAddress}
										ethBalance={ethBalance}
										fetchEthBalance={fetchEthBalance}
										onFundingComplete={() => {
											console.log("ETH funding complete callback triggered");
											fetchEthBalance();
										}}
									/>
								</div>
							)}

							{currentStep >= 3 && currentStep <= 3 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 3: Create Your Omni Account
									</h2>
									<p className="text-gray-600 mb-6">
										Deploy your smart account contract on the blockchain.
									</p>
									<CreateOmniAccount
										omniAccountAddress={omniAccountAddress}
										isFunded={!!isFunded}
										onAccountCreated={() => {
											setIsAuthorized(true);
											fetchSigners();
										}}
									/>
								</div>
							)}

							{/* Show authorized signers only after account is funded and deployed */}
							{isFunded && hasContract && omniAccountAddress && (
								<div className="mt-8">
									<AuthorizedSigners
										omniAccountAddress={omniAccountAddress}
										isDeployed={hasContract}
										signers={authorizedSigners}
										isLoading={isLoadingSigners}
										refreshSigners={fetchSigners}
										teeWorkerAddress={teeWorkerAddress}
									/>
								</div>
							)}

							{currentStep >= 4 && currentStep <= 4 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 4: Authorize TEE Worker
									</h2>
									<p className="text-gray-600 mb-6">
										Authorize the TEE worker to execute transactions on behalf of your Omni Account.
									</p>
									<AuthorizeTEEWorker
										omniAccountAddress={omniAccountAddress}
										omniAccountHash={omniAccountHash}
										isDeployed={hasContract}
										onWorkerAuthorized={(address) => {
											setTeeWorkerAddress(address);
											setIsTeeWorkerAuthorized(true);
										}}
										onComplete={() => {
											fetchSigners();
										}}
									/>
								</div>
							)}

							{currentStep >= 5 && currentStep <= 5 && (
								<div>
									<h2 className="text-xl font-semibold mb-4">
										Step 5: Send Token Transfer
									</h2>
									<p className="text-gray-600 mb-6">
										Transfer USDC or USDT through the TEE worker using UserOperations.
									</p>
									<TEETokenTransfer
										omniAccountAddress={omniAccountAddress}
										omniAccountHash={omniAccountHash}
										isDeployed={hasContract}
										teeWorkerAddress={teeWorkerAddress}
									/>
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

								{currentStep > 2 && isFunded && (
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
													Omni Account Created
												</span>
											</div>
											<span className="text-sm text-gray-500">
												Account deployed
											</span>
										</div>
									</div>
								)}

								{isTeeWorkerAuthorized && (
									<div className="bg-white rounded-lg shadow p-4">
										<div className="flex items-center justify-between">
											<div className="flex items-center space-x-3">
												<div className="w-6 h-6 bg-green-500 rounded-full flex items-center justify-center">
													<Check className="w-4 h-4 text-white" />
												</div>
												<span className="font-medium text-green-700">
													TEE Worker Authorized
												</span>
											</div>
											<span className="text-sm text-gray-500">
												{teeWorkerAddress && `${teeWorkerAddress.slice(0, 6)}...${teeWorkerAddress.slice(-4)}`}
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
