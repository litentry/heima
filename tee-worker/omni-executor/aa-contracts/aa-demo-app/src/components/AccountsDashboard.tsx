import React, { useState, useEffect } from "react";
import { useAccount, useReadContract } from "wagmi";
import { Copy, Wallet, RefreshCw, Mail } from "lucide-react";
import { calculateOmniAccount, stringToBytes } from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS, OwnerType } from "@/lib/constants";
import { useAuth } from "@/contexts/AuthContext";
import { getTEEWorkerAddress } from "@/lib/tee-worker-client";
import type { Address } from "viem";

interface AccountsDashboardProps {
    onAddressCalculated?: (address: string) => void;
    onOmniAccountCalculated?: (omniAccount: string) => void;
    isAccountCreated?: boolean;
}

export function AccountsDashboard({
    onAddressCalculated,
    onOmniAccountCalculated,
    isAccountCreated = false,
}: AccountsDashboardProps = {}) {
    const { address: evmAddress } = useAccount();
    const { authType, identifier, omniAccountHash: authOmniHash } = useAuth();

    const [omniAccount, setOmniAccount] = useState<`0x${string}`>("0x");
    const [clientId] = useState<string>(DEFAULT_CLIENT_ID);
    const [clientIdBytes] = useState<`0x${string}`>(
        stringToBytes(DEFAULT_CLIENT_ID),
    );
    const [rootSigner, setRootSigner] = useState<Address>(
        "0x0000000000000000000000000000000000000000" as Address
    );
    const [ownerType, setOwnerType] = useState<number>(OwnerType.Evm);
    const [isLoadingRootSigner, setIsLoadingRootSigner] = useState(false);

    // Set root signer and owner type based on auth type
    useEffect(() => {
        const fetchRootSigner = async () => {
            if (authType === "email" && authOmniHash) {
                setIsLoadingRootSigner(true);
                try {
                    // For email accounts, TEE worker is the root signer
                    const workerAddress = await getTEEWorkerAddress(authOmniHash);
                    setRootSigner(workerAddress as Address);
                    setOwnerType(OwnerType.Email);
                } catch (error) {
                    console.error("Failed to get TEE worker address:", error);
                    setRootSigner("0x0000000000000000000000000000000000000000" as Address);
                } finally {
                    setIsLoadingRootSigner(false);
                }
            } else if (authType === "wallet" && evmAddress) {
                setRootSigner(evmAddress);
                setOwnerType(OwnerType.Evm);
            }
        };

        fetchRootSigner();
    }, [authType, authOmniHash, evmAddress]);

    useEffect(() => {
        if (authType === "wallet" && evmAddress) {
            const account = calculateOmniAccount(
                evmAddress,
                DEFAULT_CLIENT_ID,
                "evm",
            );
            setOmniAccount(account);
        } else if (authType === "email" && authOmniHash) {
            setOmniAccount(authOmniHash as `0x${string}`);
        } else {
            setOmniAccount("0x" as `0x${string}`);
        }
    }, [authType, evmAddress, authOmniHash]);

    const { data: omniAccountAddress, error: contractError } = useReadContract({
        address: CONTRACTS.OmniAccountFactory.address,
        abi: CONTRACTS.OmniAccountFactory.abi,
        functionName: "getAddress",
        args: [omniAccount, ownerType, clientIdBytes, rootSigner],
        query: {
            enabled: omniAccount !== "0x" && rootSigner !== ("0x0000000000000000000000000000000000000000" as Address),
        },
    });

    // Debug logging
    useEffect(() => {
        console.log("AccountsDashboard Debug:", {
            evmAddress,
            omniAccount,
            clientIdBytes,
            rootSigner,
            factoryAddress: CONTRACTS.OmniAccountFactory.address,
            omniAccountAddress,
            contractError,
            enabled: !!evmAddress && omniAccount !== "0x",
        });
    }, [
        evmAddress,
        omniAccount,
        clientIdBytes,
        rootSigner,
        omniAccountAddress,
        contractError,
    ]);

    // Notify parent when address is calculated
    useEffect(() => {
        if (omniAccountAddress && onAddressCalculated) {
            onAddressCalculated(omniAccountAddress as string);
        }
    }, [omniAccountAddress, onAddressCalculated]);

    // Notify parent when omni account hash is calculated
    useEffect(() => {
        if (omniAccount && omniAccount !== "0x" && onOmniAccountCalculated) {
            onOmniAccountCalculated(omniAccount);
        }
    }, [omniAccount, onOmniAccountCalculated]);

    const copyToClipboard = async (text: string) => {
        try {
            await navigator.clipboard.writeText(text);
            // Could add toast notification here
        } catch (err) {
            console.error("Failed to copy text: ", err);
        }
    };

    const truncateAddress = (address: string) => {
        if (!address) return "";
        return `${address.slice(0, 6)}...${address.slice(-4)}`;
    };

    const isAuthenticated = authType && identifier;

    if (!isAuthenticated) {
        return (
            <div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
                <div className="text-center">
                    <Wallet className="mx-auto h-12 w-12 text-gray-400 mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">
                        No Account Connected
                    </h3>
                    <p className="text-gray-600">
                        Connect your wallet or email to view your accounts overview.
                    </p>
                </div>
            </div>
        );
    }

    return (
        <div className="w-full p-6 bg-white rounded-lg shadow-lg">
            <div className="space-y-6">
                {/* Connected Account */}
                <div className="flex items-center justify-between bg-gray-50 p-4 rounded-lg">
                    <div className="flex items-center gap-3">
                        {authType === "email" ? (
                            <Mail className="h-5 w-5 text-gray-500" />
                        ) : (
                            <Wallet className="h-5 w-5 text-gray-500" />
                        )}
                        <div>
                            <div className="text-sm text-gray-600">
                                {authType === "email" ? "Email" : "Connected Wallet"}
                            </div>
                            <div className="font-medium">
                                {authType === "email"
                                    ? identifier
                                    : truncateAddress(identifier || "")}
                            </div>
                        </div>
                    </div>
                    {authType === "wallet" && identifier && (
                        <button
                            onClick={() => copyToClipboard(identifier)}
                            className="p-2 hover:bg-gray-200 rounded transition-colors"
                        >
                            <Copy className="h-4 w-4 text-gray-500" />
                        </button>
                    )}
                </div>

                {/* Smart Account Address */}
                {(omniAccountAddress || isLoadingRootSigner) && (
                    <div className="space-y-2">
                        <div className="text-sm font-medium text-gray-700">Smart Account Address</div>
                        {isLoadingRootSigner ? (
                            <div className="flex items-center text-gray-600 bg-gray-100 p-3 rounded">
                                <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                                Loading...
                            </div>
                        ) : omniAccountAddress ? (
                            <div className="flex items-center justify-between bg-green-50 p-3 rounded border border-green-200">
                                <div className="text-sm font-mono text-green-800 break-all">
                                    {omniAccountAddress as string}
                                </div>
                                <button
                                    onClick={() => copyToClipboard(omniAccountAddress as string)}
                                    className="ml-2 p-2 hover:bg-green-100 rounded transition-colors flex-shrink-0"
                                >
                                    <Copy className="h-4 w-4 text-green-700" />
                                </button>
                            </div>
                        ) : (
                            <div className="text-sm text-gray-500 bg-gray-100 p-3 rounded">
                                Unable to calculate address
                            </div>
                        )}
                        {!isAccountCreated && omniAccountAddress && (
                            <div className="text-sm text-gray-500 italic">
                                Account will be created with your first transaction
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
