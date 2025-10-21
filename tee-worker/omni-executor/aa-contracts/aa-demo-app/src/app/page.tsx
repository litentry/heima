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
import { HyperliquidBalances } from "@/components/HyperliquidBalances";
import { RequestLoan } from "@/components/RequestLoan";
import { ClientOnly } from "@/components/ClientOnly";
import { Check, Mail } from "lucide-react";
import { getTEEWorkerAddress } from "@/lib/tee-worker-client";
import { useAuth } from "@/contexts/AuthContext";
import { calculateOmniAccount } from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID } from "@/lib/constants";

function HomeContent() {
    const { address: evmAddress, chain } = useAccount();
    const publicClient = usePublicClient();
    const { authType, identifier, omniAccountHash: authOmniHash } = useAuth();

    const [omniAccountAddress, setOmniAccountAddress] = useState<string>("");
    const [omniAccountHash, setOmniAccountHash] = useState<string>("");
    const [currentStep, setCurrentStep] = useState(1);
    const [isAuthorized, setIsAuthorized] = useState(false);
    const [hasContract, setHasContract] = useState(false);
    const [authorizedSigners, setAuthorizedSigners] = useState<string[]>([]);
    const [isLoadingSigners, setIsLoadingSigners] = useState(false);
    const [teeWorkerAddress, setTeeWorkerAddress] = useState<string | null>(null);
    const [isTeeWorkerAuthorized, setIsTeeWorkerAuthorized] = useState(false);

    // Check if authenticated
    const isAuthenticated = (authType === "wallet" && evmAddress) || (authType === "email" && identifier);

    // Update OmniAccount hash and address based on auth type
    useEffect(() => {
        if (authType === "wallet" && evmAddress) {
            const hash = calculateOmniAccount(evmAddress, DEFAULT_CLIENT_ID, "evm");
            setOmniAccountHash(hash);
        } else if (authType === "email" && authOmniHash) {
            setOmniAccountHash(authOmniHash);
        }
    }, [authType, evmAddress, authOmniHash]);

    // Debug logging
    useEffect(() => {
        console.log("HomeContent Debug:", {
            authType,
            identifier,
            evmAddress,
            chainId: chain?.id,
            chainName: chain?.name,
            isAuthenticated,
            omniAccountAddress,
            omniAccountHash,
            publicClientChainId: publicClient?.chain?.id,
        });
    }, [authType, identifier, evmAddress, chain, isAuthenticated, omniAccountAddress, omniAccountHash, publicClient]);

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

    // Check if TEE worker is already authorized by reading from the contract
    useEffect(() => {
        const checkTeeWorkerAuthorization = async () => {
            if (!omniAccountHash || !hasContract || !omniAccountAddress || !publicClient) {
                return;
            }

            try {
                // Fetch the TEE worker address
                const workerAddress = await getTEEWorkerAddress(omniAccountHash);
                setTeeWorkerAddress(workerAddress);

                // Read from the contract directly to check if worker is authorized
                const isAuthorized = await publicClient.readContract({
                    address: omniAccountAddress as `0x${string}`,
                    abi: [
                        {
                            inputs: [{ name: "signer", type: "address" }],
                            name: "isRootSigner",
                            outputs: [{ name: "", type: "bool" }],
                            stateMutability: "view",
                            type: "function",
                        },
                    ],
                    functionName: "isRootSigner",
                    args: [workerAddress as `0x${string}`],
                });

                console.log("TEE worker authorization status:", {
                    workerAddress,
                    isAuthorized,
                });

                setIsTeeWorkerAuthorized(!!isAuthorized);
            } catch (error) {
                console.error("Error checking TEE worker authorization:", error);
                setIsTeeWorkerAuthorized(false);
            }
        };

        checkTeeWorkerAuthorization();
    }, [omniAccountHash, hasContract, omniAccountAddress, publicClient]);

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
        // Set isAuthorized based on contract deployment, not just signers
        if (hasContract && authorizedSigners.length > 0) {
            setIsAuthorized(true);
        } else {
            setIsAuthorized(false);
        }

        if (!isAuthenticated) {
            setCurrentStep(1);
        } else if (!omniAccountAddress) {
            // Stay on step 1 until we have the omni account address
            setCurrentStep(1);
        } else if (!isFunded) {
            setCurrentStep(2);
        } else if (!hasContract) {
            // Stay on step 3 until contract is deployed
            setCurrentStep(3);
        } else if (!isTeeWorkerAuthorized && authType === "wallet") {
            // Email accounts have TEE worker auto-authorized
            setCurrentStep(4);
        } else {
            setCurrentStep(5);
        }
    }, [authType, isAuthenticated, omniAccountAddress, isFunded, hasContract, authorizedSigners, isTeeWorkerAuthorized]);

    const steps = [
        {
            id: 1,
            title: authType === "email" ? "Email Account" : "Connect Wallet",
            description: authType === "email" ? "Using email for authentication" : "Connect your EVM or Solana wallet",
            completed: !!isAuthenticated,
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
            completed: hasContract,
        },
        {
            id: 4,
            title: authType === "email" ? "TEE Worker Ready" : "Authorize TEE Worker",
            description: authType === "email" ? "TEE worker is auto-authorized" : "Add TEE worker as authorized signer",
            completed: isTeeWorkerAuthorized || (authType === "email" && hasContract),
        },
        {
            id: 5,
            title: "Hyperliquid Dashboard",
            description: "View balances and request loans",
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
                            {authType === "email" && (
                                <div className="flex items-center gap-2">
                                    <Mail className="w-4 h-4" />
                                    <span className="font-medium">{identifier}</span>
                                </div>
                            )}
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
                                        className={`flex items-start space-x-3 p-3 rounded-lg transition-colors ${step.id === currentStep
                                            ? "bg-blue-50 border border-blue-200"
                                            : step.completed
                                                ? "bg-green-50"
                                                : "bg-gray-50"
                                            }`}
                                    >
                                        <div
                                            className={`flex-shrink-0 w-6 h-6 rounded-full flex items-center justify-center text-sm font-medium ${step.completed
                                                ? "bg-green-500 text-white"
                                                : step.id === currentStep
                                                    ? "bg-blue-500 text-white"
                                                    : "bg-gray-300 text-gray-600"
                                                }`}
                                        >
                                            {step.completed ? (
                                                <Check className="w-4 h-4" />
                                            ) : (
                                                step.id
                                            )}
                                        </div>
                                        <div className="flex-1">
                                            <h3 className="font-medium text-gray-900">
                                                {step.title}
                                            </h3>
                                            <p className="text-sm text-gray-600 mt-1">
                                                {step.description}
                                            </p>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>

                    {/* Main Content Area */}
                    <div className="lg:col-span-2">
                        <div className="bg-white rounded-lg shadow-lg p-8">
                            {/* Main Content */}
                            {currentStep === 1 && !isAuthenticated && (
                                <div>
                                    <h2 className="text-xl font-semibold mb-4">
                                        Step 1: Connect Your Account
                                    </h2>
                                    <p className="text-gray-600 mb-6">
                                        Connect your EVM wallet, Solana wallet, or use your email address to get started.
                                    </p>
                                    <WalletConnect />
                                </div>
                            )}

                            {/* Show Omni Account details when authenticated */}
                            {isAuthenticated && (
                                <div className="mb-8">
                                    <AccountsDashboard
                                        onAddressCalculated={setOmniAccountAddress}
                                        onOmniAccountCalculated={setOmniAccountHash}
                                        ethBalance={ethBalance}
                                        isAccountCreated={hasContract}
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
                                        isFunded={isFunded}
                                        onAccountCreated={() => {
                                            console.log("Account created callback triggered");
                                            setHasContract(true);
                                            fetchSigners();
                                        }}
                                    />
                                </div>
                            )}

                            {currentStep >= 4 && currentStep <= 4 && authType === "wallet" && (
                                <div>
                                    <h2 className="text-xl font-semibold mb-4">
                                        Step 4: Authorize TEE Worker
                                    </h2>
                                    <p className="text-gray-600 mb-6">
                                        Add the TEE Worker as an authorized signer to enable secure transaction execution.
                                    </p>
                                    <AuthorizeTEEWorker
                                        omniAccountAddress={omniAccountAddress}
                                        omniAccountHash={omniAccountHash}
                                        isDeployed={hasContract}
                                        onWorkerAuthorized={(workerAddress) => {
                                            console.log("TEE Worker authorized:", workerAddress);
                                            setTeeWorkerAddress(workerAddress);
                                            setIsTeeWorkerAuthorized(true);
                                            fetchSigners(); // Refresh signers list
                                        }}
                                    />
                                </div>
                            )}

                            {currentStep >= 4 && currentStep <= 4 && authType === "email" && (
                                <div className="bg-green-50 border border-green-200 rounded-lg p-6">
                                    <div className="flex items-start space-x-3">
                                        <Check className="w-6 h-6 text-green-600 flex-shrink-0" />
                                        <div>
                                            <h3 className="text-lg font-semibold text-green-900">
                                                TEE Worker Auto-Authorized
                                            </h3>
                                            <p className="text-green-700 mt-2">
                                                For email accounts, the TEE worker is automatically authorized as the primary signer.
                                                You can proceed to send transactions.
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            )}

                            {currentStep >= 5 && (
                                <div className="space-y-8">
                                    <div>
                                        <h2 className="text-xl font-semibold mb-4">
                                            Step 5: Hyperliquid Core Dashboard
                                        </h2>
                                        <p className="text-gray-600 mb-6">
                                            View your balances, positions, and request loans using your Hyperliquid assets.
                                        </p>
                                    </div>

                                    {/* Loan Request Panel */}
                                    <RequestLoan
                                        omniAccountAddress={omniAccountAddress}
                                        omniAccountHash={omniAccountHash}
                                    />

                                    {/* Balances and Positions */}
                                    <HyperliquidBalances
                                        omniAccountAddress={omniAccountAddress}
                                    />
                                </div>
                            )}

                            {/* Authorized Signers - Always visible after account creation */}
                            {isAuthorized && authorizedSigners.length > 0 && currentStep >= 4 && (
                                <div className="mt-8 pt-8 border-t">
                                    <AuthorizedSigners
                                        omniAccountAddress={omniAccountAddress}
                                        isDeployed={hasContract}
                                        signers={authorizedSigners}
                                        isLoading={isLoadingSigners}
                                        teeWorkerAddress={teeWorkerAddress}
                                        refreshSigners={() => {
                                            console.log("Signers updated, refreshing list");
                                            fetchSigners();
                                        }}
                                    />
                                </div>
                            )}
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
