"use client";

import React, { useState, useEffect } from "react";
import { useAccount, useReadContract } from "wagmi";
import { useWallet } from "@solana/wallet-adapter-react";
import { Copy, ExternalLink, Wallet } from "lucide-react";
import { calculateOmniAccount, stringToBytes } from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS } from "@/lib/constants";
import type { Address } from "viem";

interface AAWalletInfoProps {
	onAddressCalculated?: (address: string) => void;
}

export function OmniAccountWalletInfo({
	onAddressCalculated,
}: AAWalletInfoProps = {}) {
	const { address: evmAddress } = useAccount();
	const { publicKey: solanaAddress } = useWallet();
	const [omniAccount, setOmniAccount] = useState<`0x${string}`>("0x");
	const [clientId] = useState<string>(DEFAULT_CLIENT_ID);
	const [clientIdBytes] = useState<`0x${string}`>(
		stringToBytes(DEFAULT_CLIENT_ID),
	);
	const rootSigner =
		evmAddress || ("0x0000000000000000000000000000000000000000" as Address);

	useEffect(() => {
		if (evmAddress) {
			const account = calculateOmniAccount(
				evmAddress,
				DEFAULT_CLIENT_ID,
				"evm",
			);
			setOmniAccount(account);
		} else {
			setOmniAccount("0x" as `0x${string}`);
		}
	}, [evmAddress]);

	const { data: omniAccountAddress, error: contractError } = useReadContract({
		address: CONTRACTS.OmniAccountFactory.address,
		abi: CONTRACTS.OmniAccountFactory.abi,
		functionName: "getAddress",
		args: [omniAccount, clientIdBytes, rootSigner],
		query: {
			enabled: !!evmAddress && omniAccount !== "0x",
		},
	});

	// Debug logging
	useEffect(() => {
		console.log("AAWalletInfo Debug:", {
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

	if (!evmAddress && !solanaAddress) {
		return (
			<div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
				<div className="text-center">
					<Wallet className="mx-auto h-12 w-12 text-gray-400 mb-4" />
					<h3 className="text-lg font-medium text-gray-900 mb-2">
						No Wallet Connected
					</h3>
					<p className="text-gray-600">
						Connect your wallet to view your Account Abstraction wallet
						information.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full p-6 bg-white rounded-lg shadow-lg">
			<h2 className="text-2xl font-bold mb-6 text-center">Wallet Info</h2>

			<div className="space-y-6">
				{evmAddress && (
					<div className="space-y-3">
						<h3 className="text-lg font-semibold text-gray-900">
							Connected EVM Wallet
						</h3>
						<div className="bg-gray-50 p-3 rounded-lg">
							<div className="flex items-center justify-between">
								<span className="text-sm text-gray-600">Address:</span>
								<div className="flex items-center space-x-2">
									<span className="text-sm font-mono">
										{truncateAddress(evmAddress)}
									</span>
									<button
										onClick={() => copyToClipboard(evmAddress)}
										className="p-1 hover:bg-gray-200 rounded"
									>
										<Copy className="h-4 w-4 text-gray-500" />
									</button>
								</div>
							</div>
						</div>
					</div>
				)}

				{solanaAddress && (
					<div className="space-y-3">
						<h3 className="text-lg font-semibold text-gray-900">
							Connected Solana Wallet
						</h3>
						<div className="bg-gray-50 p-3 rounded-lg">
							<div className="flex items-center justify-between">
								<span className="text-sm text-gray-600">Address:</span>
								<div className="flex items-center space-x-2">
									<span className="text-sm font-mono">
										{truncateAddress(solanaAddress.toString())}
									</span>
									<button
										onClick={() => copyToClipboard(solanaAddress.toString())}
										className="p-1 hover:bg-gray-200 rounded"
									>
										<Copy className="h-4 w-4 text-gray-500" />
									</button>
								</div>
							</div>
						</div>
					</div>
				)}

				<div className="space-y-3">
					<h3 className="text-lg font-semibold text-gray-900">Client ID</h3>
					<div className="bg-gray-50 p-3 rounded-lg">
						<div className="flex items-center justify-between">
							<span className="text-sm text-gray-600">ID:</span>
							<div className="flex items-center space-x-2">
								<span className="text-sm font-mono">{clientId}</span>
								<button
									onClick={() => copyToClipboard(clientId)}
									className="p-1 hover:bg-gray-200 rounded"
								>
									<Copy className="h-4 w-4 text-gray-500" />
								</button>
							</div>
						</div>
					</div>
				</div>

				{omniAccountAddress && (
					<div className="space-y-3">
						<h3 className="text-lg font-semibold text-green-700">
							Omni Account
						</h3>
						<div className="bg-green-50 p-4 rounded-lg border border-green-200">
							<div className="flex items-center justify-center mb-2">
								<div className="text-sm font-mono text-green-800 break-all">
									{(omniAccountAddress as string) || "Calculating..."}
								</div>
							</div>
						</div>
					</div>
				)}
			</div>
		</div>
	);
}
