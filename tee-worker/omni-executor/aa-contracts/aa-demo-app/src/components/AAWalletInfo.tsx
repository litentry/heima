"use client";

import React, { useState, useEffect } from "react";
import { useAccount, useReadContract } from "wagmi";
import { useWallet } from "@solana/wallet-adapter-react";
import { Copy, ExternalLink, Wallet } from "lucide-react";
import { calculateOmniAccount, stringToBytes32 } from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS } from "@/lib/constants";
import type { Address } from "viem";

interface AAWalletInfoProps {
	onAddressCalculated?: (address: string) => void;
}

export function AAWalletInfo({ onAddressCalculated }: AAWalletInfoProps = {}) {
	const { address: evmAddress } = useAccount();
	const { publicKey: solanaAddress } = useWallet();
	const [omniAccount, setOmniAccount] = useState<`0x${string}`>("0x");
	const [clientId] = useState<string>(DEFAULT_CLIENT_ID);
	const [clientIdBytes32] = useState<`0x${string}`>(
		stringToBytes32(DEFAULT_CLIENT_ID),
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

	const { data: aaWalletAddress } = useReadContract({
		address: CONTRACTS.SmartAccountFactory.address,
		abi: CONTRACTS.SmartAccountFactory.abi,
		functionName: "getAddress",
		args: [omniAccount, clientIdBytes32, rootSigner],
		query: {
			enabled: !!evmAddress && omniAccount !== "0x",
		},
	});

	// Notify parent when address is calculated
	useEffect(() => {
		if (aaWalletAddress && onAddressCalculated) {
			onAddressCalculated(aaWalletAddress as string);
		}
	}, [aaWalletAddress, onAddressCalculated]);

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
			<h2 className="text-2xl font-bold mb-6 text-center">AA Wallet Info</h2>

			<div className="space-y-6">
				{/* Connected Wallet Info */}
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

				{/* Omni Account */}
				{Boolean(evmAddress && omniAccount && omniAccount.length > 2) && (
					<div className="space-y-3">
						<h3 className="text-lg font-semibold text-gray-900">
							Omni Account
						</h3>
						<div className="bg-blue-50 p-3 rounded-lg border border-blue-200">
							<div className="flex items-center justify-between mb-2">
								<span className="text-sm text-blue-700 font-medium">
									SHA256 Hash:
								</span>
								<button
									onClick={() => copyToClipboard(omniAccount)}
									className="p-1 hover:bg-blue-100 rounded"
								>
									<Copy className="h-4 w-4 text-blue-600" />
								</button>
							</div>
							<div className="text-xs font-mono text-blue-800 break-all">
								{omniAccount}
							</div>
							<div className="text-xs text-blue-600 mt-2">
								Formula: sha256(clientId + 'evm' + address)
							</div>
						</div>
					</div>
				)}

				{/* Client ID */}
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

				{/* AA Wallet Address */}
				{aaWalletAddress && (
					<div className="space-y-3">
						<h3 className="text-lg font-semibold text-green-700">
							Smart Account Address
						</h3>
						<div className="bg-green-50 p-4 rounded-lg border border-green-200">
							<div className="flex items-center justify-between mb-2">
								<span className="text-sm text-green-700 font-medium">
									Counterfactual:
								</span>
								<div className="flex items-center space-x-2">
									<button
										onClick={() => copyToClipboard(aaWalletAddress as string)}
										className="p-1 hover:bg-green-100 rounded"
									>
										<Copy className="h-4 w-4 text-green-600" />
									</button>
									<button
										onClick={() =>
											window.open(
												`https://etherscan.io/address/${aaWalletAddress}`,
												"_blank",
											)
										}
										className="p-1 hover:bg-green-100 rounded"
									>
										<ExternalLink className="h-4 w-4 text-green-600" />
									</button>
								</div>
							</div>
							<div className="text-sm font-mono text-green-800 break-all">
								{(aaWalletAddress as string) || "Calculating..."}
							</div>
							<div className="text-xs text-green-600 mt-2">
								This is your Smart Account address - fund this address to start
								using AA features.
							</div>
						</div>
					</div>
				)}
			</div>
		</div>
	);
}

