"use client";

import { useState, useEffect, useCallback } from "react";
import { useAccount, usePublicClient } from "wagmi";
import { WalletConnect } from "@/components/WalletConnect";
import { AccountsDashboard } from "@/components/AccountsDashboard";
import { HyperliquidBalances } from "@/components/HyperliquidBalances";
import { RequestLoan } from "@/components/RequestLoan";
import { ClientOnly } from "@/components/ClientOnly";
import { Mail } from "lucide-react";
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
    const [hasContract, setHasContract] = useState(false);
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
    }, [omniAccountAddress, publicClient]);

    // Check if TEE worker is already authorized by reading from the contract
    useEffect(() => {
        const checkTeeWorkerAuthorization = async () => {
            if (!omniAccountHash || !hasContract || !omniAccountAddress || !publicClient) {
                return;
            }

            try {
                // Fetch the TEE worker address
                const workerAddress = await getTEEWorkerAddress(omniAccountHash);

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

    return (
        <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
            {/* Header */}
            <header className="bg-white shadow-sm border-b">
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
                    <div className="flex items-center justify-between">
                        <div>
                            <h1 className="text-2xl font-bold text-gray-900">
                                Hyperliquid Loan Demo
                            </h1>
                            <p className="text-gray-600 mt-1">
                                Request loans using your Hyperliquid assets as collateral
                            </p>
                        </div>
                        {authType === "email" && identifier && (
                            <div className="flex items-center gap-2 text-sm text-gray-500">
                                <Mail className="w-4 h-4" />
                                <span className="font-medium">{identifier}</span>
                            </div>
                        )}
                    </div>
                </div>
            </header>

            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                {/* Wallet Connection */}
                {!isAuthenticated && (
                    <div className="bg-white rounded-lg shadow-lg p-8 mb-8">
                        <h2 className="text-xl font-semibold mb-4">Connect Your Wallet</h2>
                        <p className="text-gray-600 mb-6">
                            Connect your wallet or use email to get started
                        </p>
                        <WalletConnect />
                    </div>
                )}

                {/* Account Info */}
                {isAuthenticated && (
                    <div className="mb-8">
                        <AccountsDashboard
                            onAddressCalculated={setOmniAccountAddress}
                            onOmniAccountCalculated={setOmniAccountHash}
                            isAccountCreated={hasContract}
                        />
                    </div>
                )}

                {/* Dashboard - shown after wallet connection */}
                {isAuthenticated && omniAccountAddress && (
                    <div className="space-y-8">
                        {/* Request Loan Panel */}
                        <RequestLoan
                            omniAccountAddress={omniAccountAddress}
                            omniAccountHash={omniAccountHash}
                            onAccountCreated={() => {
                                console.log("Account created via first transaction");
                                setHasContract(true);
                                setIsTeeWorkerAuthorized(true);
                            }}
                        />

                        {/* Hyperliquid Balances, Positions, and Orders */}
                        <HyperliquidBalances
                            omniAccountAddress={omniAccountAddress}
                        />
                    </div>
                )}
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
