"use client";

import { useState } from "react";
import { usePublicClient, useWalletClient, useAccount } from "wagmi";
import {
	Users,
	Plus,
	Trash2,
	Copy,
	AlertCircle,
	RefreshCw,
} from "lucide-react";
import { encodeFunctionData } from "viem";
import { CONTRACTS } from "@/lib/constants";
import {
	createUserOperation,
	signUserOperation,
	packUserOperation,
	type UserOperation,
} from "@/lib/aa-utils";

interface AuthorizedSignersProps {
	aaWalletAddress?: string;
	isDeployed?: boolean;
	signers: string[];
	isLoading: boolean;
	refreshSigners: () => void;
}

export function AuthorizedSigners({
	aaWalletAddress,
	isDeployed,
	signers,
	isLoading,
	refreshSigners,
}: AuthorizedSignersProps) {
	const { address: evmAddress } = useAccount();
	const publicClient = usePublicClient();
	const { data: walletClient } = useWalletClient();
	const [newSignerAddress, setNewSignerAddress] = useState("");
	const [isAddingSigner, setIsAddingSigner] = useState(false);
	const [showAddForm, setShowAddForm] = useState(false);
	const [isRefreshing, setIsRefreshing] = useState(false);

	const handleRefresh = () => {
		setIsRefreshing(true);
		refreshSigners();
		setIsRefreshing(false);
	};

	// Add a new root signer
	const addSigner = async () => {
		if (
			!aaWalletAddress ||
			!publicClient ||
			!walletClient ||
			!evmAddress ||
			!newSignerAddress
		)
			return;

		// Validate address
		if (!/^0x[a-fA-F0-9]{40}$/.test(newSignerAddress)) {
			alert("Invalid Ethereum address");
			return;
		}

		setIsAddingSigner(true);
		try {
			// IMPORTANT: We need to wrap the addRootSigner call in an execute call
			// This is because the EntryPoint calls the account, and the account needs
			// to call itself to satisfy the onlyOwner modifier (msg.sender == address(this))

			// First encode the addRootSigner call
			const addSignerData = encodeFunctionData({
				abi: CONTRACTS.OmniAccountImplementation.abi,
				functionName: "addRootSigner",
				args: [newSignerAddress as `0x${string}`],
			});

			// Then wrap it in an execute call (the account calls itself)
			const callData = encodeFunctionData({
				abi: [
					{
						name: "execute",
						type: "function",
						inputs: [
							{ name: "target", type: "address" },
							{ name: "value", type: "uint256" },
							{ name: "data", type: "bytes" },
						],
						outputs: [],
						stateMutability: "nonpayable",
					},
				],
				functionName: "execute",
				args: [aaWalletAddress as `0x${string}`, BigInt(0), addSignerData],
			});

			// Get current nonce for the account
			const nonce = (await publicClient.readContract({
				address: CONTRACTS.EntryPoint.address,
				abi: CONTRACTS.EntryPoint.abi,
				functionName: "getNonce",
				args: [aaWalletAddress as `0x${string}`, BigInt(0)],
			})) as bigint;

			// Create UserOperation
			const userOp = createUserOperation({
				sender: aaWalletAddress as `0x${string}`,
				nonce,
				callData,
				initCode: "0x", // Account already deployed
			});

			// Sign UserOperation
			const signature = await signUserOperation(
				walletClient,
				evmAddress,
				userOp as UserOperation,
				CONTRACTS.EntryPoint.address,
				BigInt(await publicClient.getChainId()),
			);

			userOp.signature = signature;

			// Convert to PackedUserOperation for EntryPoint v0.7
			const packedUserOp = packUserOperation(userOp as UserOperation);

			// Send UserOperation via EntryPoint
			const hash = await walletClient.writeContract({
				address: CONTRACTS.EntryPoint.address,
				abi: CONTRACTS.EntryPoint.abi,
				functionName: "handleOps",
				args: [[packedUserOp], evmAddress],
			});

			// Wait for transaction
			const receipt = await publicClient.waitForTransactionReceipt({ hash });
			console.log("Transaction receipt:", receipt);

			// Add a delay to ensure events are indexed
			console.log("Waiting for events to be indexed...");
			await new Promise((resolve) => setTimeout(resolve, 2000)); // 2 second delay

			// Refresh signers list
			await refreshSigners();

			// Clear form
			setNewSignerAddress("");
			setShowAddForm(false);

			// Verify the signer was actually added
			const isSignerAdded = await publicClient.readContract({
				address: aaWalletAddress as `0x${string}`,
				abi: CONTRACTS.OmniAccountImplementation.abi,
				functionName: "isRootSigner",
				args: [newSignerAddress as `0x${string}`],
			});

			if (isSignerAdded) {
				alert("Signer added successfully!");
			} else {
				alert(
					"Transaction succeeded but signer was not added. The UserOperation may have reverted.",
				);
				console.error("Signer not added despite successful transaction");
			}
		} catch (error) {
			console.error("Error adding signer:", error);
			alert("Failed to add signer. Please try again.");
		} finally {
			setIsAddingSigner(false);
		}
	};

	// Remove a root signer
	const removeSigner = async (signerToRemove: string) => {
		if (!aaWalletAddress || !publicClient || !walletClient || !evmAddress)
			return;

		if (!confirm(`Are you sure you want to remove signer ${signerToRemove}?`))
			return;

		try {
			// First encode the removeRootSigner call
			const removeSignerData = encodeFunctionData({
				abi: CONTRACTS.OmniAccountImplementation.abi,
				functionName: "removeRootSigner",
				args: [signerToRemove as `0x${string}`],
			});

			// Then wrap it in an execute call (the account calls itself)
			const callData = encodeFunctionData({
				abi: [
					{
						name: "execute",
						type: "function",
						inputs: [
							{ name: "target", type: "address" },
							{ name: "value", type: "uint256" },
							{ name: "data", type: "bytes" },
						],
						outputs: [],
						stateMutability: "nonpayable",
					},
				],
				functionName: "execute",
				args: [aaWalletAddress as `0x${string}`, BigInt(0), removeSignerData],
			});

			// Get current nonce for the account
			const nonce = (await publicClient.readContract({
				address: CONTRACTS.EntryPoint.address,
				abi: CONTRACTS.EntryPoint.abi,
				functionName: "getNonce",
				args: [aaWalletAddress as `0x${string}`, BigInt(0)],
			})) as bigint;

			// Create UserOperation
			const userOp = createUserOperation({
				sender: aaWalletAddress as `0x${string}`,
				nonce,
				callData,
				initCode: "0x", // Account already deployed
			});

			// Sign UserOperation
			const signature = await signUserOperation(
				walletClient,
				evmAddress,
				userOp as UserOperation,
				CONTRACTS.EntryPoint.address,
				BigInt(await publicClient.getChainId()),
			);

			userOp.signature = signature;

			// Convert to PackedUserOperation for EntryPoint v0.7
			const packedUserOp = packUserOperation(userOp as UserOperation);

			// Send UserOperation via EntryPoint
			const hash = await walletClient.writeContract({
				address: CONTRACTS.EntryPoint.address,
				abi: CONTRACTS.EntryPoint.abi,
				functionName: "handleOps",
				args: [[packedUserOp], evmAddress],
			});

			// Wait for transaction
			const receipt = await publicClient.waitForTransactionReceipt({ hash });
			console.log("Transaction receipt:", receipt);

			// Add a delay to ensure events are indexed
			console.log("Waiting for events to be indexed...");
			await new Promise((resolve) => setTimeout(resolve, 2000)); // 2 second delay

			// Refresh signers list
			await refreshSigners();

			alert("Signer removed successfully!");
		} catch (error) {
			console.error("Error removing signer:", error);
			alert("Failed to remove signer. Please try again.");
		}
	};

	const copyToClipboard = async (text: string) => {
		try {
			await navigator.clipboard.writeText(text);
		} catch (err) {
			console.error("Failed to copy text: ", err);
		}
	};

	if (!aaWalletAddress) {
		return null;
	}

	if (!isDeployed) {
		return (
			<div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
				<div className="text-center">
					<AlertCircle className="mx-auto h-12 w-12 text-gray-400 mb-4" />
					<h3 className="text-lg font-medium text-gray-900 mb-2">
						Account Not Deployed
					</h3>
					<p className="text-gray-600">
						The Omni Account needs to be deployed before managing signers.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full p-6 bg-white rounded-lg shadow-lg">
			<div className="flex items-center justify-between mb-6">
				<div className="flex items-center">
					<Users className="h-6 w-6 text-blue-500 mr-2" />
					<h2 className="text-2xl font-bold">Authorized Signers</h2>
				</div>
				<div className="flex items-center gap-2">
					<button
						onClick={handleRefresh}
						disabled={isRefreshing || isLoading}
						className="flex items-center px-3 py-2 bg-gray-600 hover:bg-gray-700 disabled:bg-gray-400 text-white rounded-lg transition-colors"
						title="Refresh signer list"
					>
						<RefreshCw
							className={`h-4 w-4 ${isRefreshing ? "animate-spin" : ""}`}
						/>
					</button>
					<button
						onClick={() => setShowAddForm(!showAddForm)}
						className="flex items-center px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
					>
						<Plus className="h-4 w-4 mr-1" />
						Add Signer
					</button>
				</div>
			</div>

			{showAddForm && (
				<div className="mb-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
					<h3 className="font-medium mb-3">Add New Signer</h3>
					<div className="flex gap-2">
						<input
							type="text"
							value={newSignerAddress}
							onChange={(e) => setNewSignerAddress(e.target.value)}
							placeholder="0x..."
							className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
						/>
						<button
							onClick={addSigner}
							disabled={isAddingSigner || !newSignerAddress}
							className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
						>
							{isAddingSigner ? "Adding..." : "Add"}
						</button>
						<button
							onClick={() => {
								setShowAddForm(false);
								setNewSignerAddress("");
							}}
							className="px-4 py-2 bg-gray-300 hover:bg-gray-400 text-gray-700 rounded-lg transition-colors"
						>
							Cancel
						</button>
					</div>
				</div>
			)}

			{isLoading ? (
				<div className="text-center py-8">
					<div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500 mx-auto"></div>
					<p className="text-gray-600 mt-2">Loading signers...</p>
				</div>
			) : signers.length === 0 ? (
				<div className="text-center py-8">
					<p className="text-gray-600">No authorized signers found.</p>
				</div>
			) : (
				<div className="space-y-3">
					{signers.map((signer) => (
						<div
							key={signer}
							className="flex items-center justify-between p-4 bg-gray-50 rounded-lg"
						>
							<div className="flex items-center flex-1">
								<span className="font-mono text-sm break-all mr-2">
									{signer}
								</span>
								<button
									onClick={() => copyToClipboard(signer)}
									className="p-1 hover:bg-gray-200 rounded"
								>
									<Copy className="h-4 w-4 text-gray-600" />
								</button>
							</div>
							<div className="flex items-center gap-2">
								{evmAddress?.toLowerCase() === signer.toLowerCase() && (
									<span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">
										Current Wallet
									</span>
								)}
								<button
									onClick={() => removeSigner(signer)}
									className="p-2 hover:bg-red-100 rounded-lg text-red-600 transition-colors"
									title="Remove signer"
								>
									<Trash2 className="h-4 w-4" />
								</button>
							</div>
						</div>
					))}
				</div>
			)}

			<div className="mt-6 bg-yellow-50 border border-yellow-200 rounded-lg p-4">
				<div className="flex">
					<AlertCircle className="h-5 w-5 text-yellow-500 mr-2 mt-0.5" />
					<div className="text-sm text-yellow-700">
						<p className="font-medium mb-1">Important:</p>
						<p>
							Root signers can authorize transactions and create session
							accounts. Only add addresses you trust.
						</p>
					</div>
				</div>
			</div>
		</div>
	);
}
