"use client";

import React, { createContext, useContext, useState, useCallback, ReactNode, useEffect } from "react";
import { useAccount } from "wagmi";
import { getOmniAccount } from "@/lib/tee-worker-client";
import { calculateOmniAccount, calculateOmniAccountFromEmail } from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID } from "@/lib/constants";

export type AuthType = "wallet" | "email" | null;

interface AuthContextType {
    authType: AuthType;
    identifier: string | null; // wallet address or email
    omniAccountHash: `0x${string}` | null; // 32-byte hash
    omniAccountAddress: `0x${string}` | null; // deterministic contract address
    isLoading: boolean;
    error: string | null;

    // Actions
    setEmailAuth: (email: string) => Promise<void>;
    clearAuth: () => void;
    refreshOmniAccount: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
    const { address: walletAddress, isConnected } = useAccount();

    const [authType, setAuthType] = useState<AuthType>(null);
    const [identifier, setIdentifier] = useState<string | null>(null);
    const [omniAccountHash, setOmniAccountHash] = useState<`0x${string}` | null>(null);
    const [omniAccountAddress, setOmniAccountAddress] = useState<`0x${string}` | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Calculate deterministic address from omni account hash
    const calculateContractAddress = useCallback((hash: `0x${string}`): `0x${string}` => {
        // The factory uses CREATE2 to deploy contracts deterministically
        // For now, we'll import and use the factory's getAddress method
        // This would need to be called on-chain to get the actual address
        // For demo purposes, we'll store this after calling the factory
        return hash; // Placeholder - actual implementation would call factory.getAddress()
    }, []);

    // Handle wallet connection changes
    useEffect(() => {
        if (isConnected && walletAddress) {
            setAuthType("wallet");
            setIdentifier(walletAddress);

            // Calculate omni account for wallet
            const hash = calculateOmniAccount(walletAddress, DEFAULT_CLIENT_ID, "evm");
            setOmniAccountHash(hash);

            // Note: The actual contract address needs to be fetched from the factory
            // For now, we'll handle this in the components that need it
            setOmniAccountAddress(null);
            setError(null);
        } else if (authType === "wallet") {
            // Wallet was disconnected
            clearAuth();
        }
    }, [isConnected, walletAddress, authType]);

    const setEmailAuth = useCallback(async (email: string) => {
        setIsLoading(true);
        setError(null);

        try {
            // Get omni account hash from TEE worker
            const hash = await getOmniAccount(DEFAULT_CLIENT_ID, email);

            setAuthType("email");
            setIdentifier(email);
            setOmniAccountHash(hash as `0x${string}`);

            // Note: The actual contract address needs to be fetched from the factory
            setOmniAccountAddress(null);
        } catch (err) {
            console.error("Failed to get omni account for email:", err);
            setError(err instanceof Error ? err.message : "Failed to get omni account");
            throw err;
        } finally {
            setIsLoading(false);
        }
    }, []);

    const clearAuth = useCallback(() => {
        setAuthType(null);
        setIdentifier(null);
        setOmniAccountHash(null);
        setOmniAccountAddress(null);
        setError(null);
    }, []);

    const refreshOmniAccount = useCallback(async () => {
        if (!authType || !identifier) return;

        setIsLoading(true);
        setError(null);

        try {
            if (authType === "email") {
                const hash = await getOmniAccount(DEFAULT_CLIENT_ID, identifier);
                setOmniAccountHash(hash as `0x${string}`);
            } else if (authType === "wallet") {
                const hash = calculateOmniAccount(identifier, DEFAULT_CLIENT_ID, "evm");
                setOmniAccountHash(hash);
            }
        } catch (err) {
            console.error("Failed to refresh omni account:", err);
            setError(err instanceof Error ? err.message : "Failed to refresh account");
        } finally {
            setIsLoading(false);
        }
    }, [authType, identifier]);

    // Set the contract address (called by components after fetching from factory)
    useEffect(() => {
        if (omniAccountHash && !omniAccountAddress) {
            // This will be set by the components that fetch the actual address from the factory
            console.log("OmniAccount hash set:", omniAccountHash);
        }
    }, [omniAccountHash, omniAccountAddress]);

    const value: AuthContextType = {
        authType,
        identifier,
        omniAccountHash,
        omniAccountAddress,
        isLoading,
        error,
        setEmailAuth,
        clearAuth,
        refreshOmniAccount,
    };

    return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error("useAuth must be used within an AuthProvider");
    }
    return context;
}

// Helper hook to determine if the current auth is email-based
export function useIsEmailAuth() {
    const { authType } = useAuth();
    return authType === "email";
}

// Helper hook to determine if the current auth is wallet-based
export function useIsWalletAuth() {
    const { authType } = useAuth();
    return authType === "wallet";
}