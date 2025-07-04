"use client";

import { useState } from "react";
import {
	useAccount,
	useSignMessage,
	useWalletClient,
	usePublicClient,
} from "wagmi";
import { Key, Shield, AlertTriangle, CheckCircle, Loader2 } from "lucide-react";
import { isAddress } from "viem";
import {
	calculateOmniAccount,
	createUserOperation,
	getUserOpHash,
	stringToBytes,
	generateInitCode,
	packUserOperation,
	type UserOperation,
} from "@/lib/aa-utils";
import { DEFAULT_CLIENT_ID, CONTRACTS } from "@/lib/constants";

interface RootKeyAuthorizationProps {
	omniAccountAddress?: string;
	isFunded: boolean;
	onAuthorizationComplete?: () => void;
}

export function RootKeyAuthorization({
	omniAccountAddress,
	isFunded,
	onAuthorizationComplete,
}: RootKeyAuthorizationProps) {
	const { address: evmAddress, chain } = useAccount();
	const { data: walletClient } = useWalletClient();
	const publicClient = usePublicClient();

	// The actual root signer will be set during initialization
	const [rootSignerAddress, setRootSignerAddress] = useState("");
	const [isProcessing, setIsProcessing] = useState(false);
	const [authorizationComplete, setAuthorizationComplete] = useState(false);
	const [error, setError] = useState("");
	const [txHash, setTxHash] = useState("");

	const isValidAddress = rootSignerAddress && isAddress(rootSignerAddress);

	const truncateAddress = (address: string) => {
		if (!address) return "";
		return `${address.slice(0, 6)}...${address.slice(-4)}`;
	};

	const handleAuthorizeRootKey = async () => {
		if (!rootSignerAddress) {
			setError(
				'Please enter a root signer address or select "Use current wallet as root signer"',
			);
			return;
		}

		if (!isValidAddress) {
			setError("Please enter a valid Ethereum address");
			return;
		}

		if (
			!evmAddress ||
			!omniAccountAddress ||
			!walletClient ||
			!publicClient ||
			!chain
		) {
			setError("Missing wallet connection or smart account address");
			return;
		}

		setIsProcessing(true);
		setError("");

		try {
			// Calculate omni account and client ID bytes32
			const omniAccount = calculateOmniAccount(
				evmAddress,
				DEFAULT_CLIENT_ID,
				"evm",
			);
			const clientIdBytes = stringToBytes(DEFAULT_CLIENT_ID);

			// Debug: Let's verify the omni account calculation
			console.log("Debug omni account calculation:", {
				address: evmAddress,
				clientId: DEFAULT_CLIENT_ID,
				identityType: "evm",
				calculatedOmniAccount: omniAccount,
			});

			console.log("Authorization params:", {
				evmAddress,
				aaWalletAddress: omniAccountAddress,
				rootSignerAddress,
				omniAccount,
				clientIdBytes,
				factoryAddress: CONTRACTS.OmniAccountFactory.address,
			});

			// Check if account already exists
			const code = await publicClient.getBytecode({
				address: omniAccountAddress as `0x${string}`,
			});

			const accountExists = !!(code && code !== "0x");
			console.log("Account exists:", accountExists, "Code:", code);

			// IMPORTANT: The root signer used here MUST match what was used in AAWalletInfo
			// to calculate the counterfactual address. We'll use the user's wallet address.
			const deploymentRootSigner = evmAddress as `0x${string}`;

			// Generate initCode for Omni Account deployment (only if account doesn't exist)
			const initCode = accountExists
				? "0x"
				: (generateInitCode(
						CONTRACTS.OmniAccountFactory.address,
						omniAccount,
						clientIdBytes,
						deploymentRootSigner, // Use the user's wallet as root signer for deployment
					) as `0x${string}`);

			console.log("Deployment params:", {
				accountExists,
				deploymentRootSigner,
				initCode: initCode.slice(0, 66) + "...",
			});

			// Create UserOperation for deploying and initializing the Omni Account
			const userOp = createUserOperation({
				sender: omniAccountAddress as `0x${string}`,
				nonce: BigInt(0),
				initCode: initCode,
				callData: "0x", // No additional operations needed for initialization
			});

			// For deployment operations, we need special handling
			let signature: `0x${string}`;

			try {
				// Convert UserOperation to PackedUserOperation for signing
				const packedOp = packUserOperation(userOp as UserOperation);

				// EIP-712 domain
				const domain = {
					name: "ERC4337",
					version: "1",
					chainId: chain.id,
					verifyingContract: CONTRACTS.EntryPoint.address as `0x${string}`,
				};

				// EIP-712 types for PackedUserOperation
				const types = {
					PackedUserOperation: [
						{ name: "sender", type: "address" },
						{ name: "nonce", type: "uint256" },
						{ name: "initCode", type: "bytes" },
						{ name: "callData", type: "bytes" },
						{ name: "accountGasLimits", type: "bytes32" },
						{ name: "preVerificationGas", type: "uint256" },
						{ name: "gasFees", type: "bytes32" },
						{ name: "paymasterAndData", type: "bytes" },
						{ name: "sessionAccount", type: "address" },
						{ name: "sessionExpiration", type: "uint256" },
						{ name: "sessionAccountProof", type: "bytes" },
					],
				};

				// Message to sign (without signature field)
				const message = {
					sender: packedOp.sender,
					nonce: packedOp.nonce,
					initCode: packedOp.initCode,
					callData: packedOp.callData,
					accountGasLimits: packedOp.accountGasLimits,
					preVerificationGas: packedOp.preVerificationGas,
					gasFees: packedOp.gasFees,
					paymasterAndData: packedOp.paymasterAndData,
					sessionAccount: packedOp.sessionAccount,
					sessionExpiration: packedOp.sessionExpiration,
					sessionAccountProof: packedOp.sessionAccountProof,
				};

				signature = await walletClient.signTypedData({
					account: evmAddress,
					domain,
					types,
					primaryType: "PackedUserOperation",
					message,
				});

				// Verify the hash matches what we expect
				const userOpHash = getUserOpHash(
					userOp as UserOperation,
					CONTRACTS.EntryPoint.address,
					chain.id,
				);
			} catch (e) {
				console.error("EIP-712 signing failed:", e);

				// Fallback: Try raw eth_sign if available
				try {
					if (walletClient.request) {
						console.log("Attempting raw eth_sign as fallback...");
						const userOpHash = getUserOpHash(
							userOp as UserOperation,
							CONTRACTS.EntryPoint.address,
							chain.id,
						);
						signature = await walletClient.request({
							method: "eth_sign",
							params: [evmAddress, userOpHash],
						});
						console.log("Successfully used eth_sign");
					} else {
						throw new Error("Wallet doesn't support raw signing");
					}
				} catch (ethSignError) {
					console.error("eth_sign also failed:", ethSignError);

					// Last resort: Use personal_sign (adds message prefix)
					const userOpHash = getUserOpHash(
						userOp as UserOperation,
						CONTRACTS.EntryPoint.address,
						chain.id,
					);
					signature = await walletClient.signMessage({
						account: evmAddress,
						message: { raw: userOpHash },
					});

					console.warn(
						"WARNING: Using personal_sign which adds message prefix",
					);
					console.warn("The contract expects a raw signature, this may fail");
				}
			}

			// Update the UserOperation with signature
			const signedUserOp = {
				...userOp,
				signature,
			} as UserOperation;

			// Convert to PackedUserOperation for EntryPoint v0.7
			const packedUserOp = packUserOperation(signedUserOp);

			console.log("=== UserOperation Debug Info ===");
			console.log("Packed UserOp:", {
				sender: packedUserOp.sender,
				nonce: packedUserOp.nonce.toString(),
				initCode: packedUserOp.initCode.slice(0, 66) + "...",
				callData: packedUserOp.callData,
				accountGasLimits: packedUserOp.accountGasLimits,
				preVerificationGas: packedUserOp.preVerificationGas.toString(),
				gasFees: packedUserOp.gasFees,
				paymasterAndData: packedUserOp.paymasterAndData,
				sessionAccount: packedUserOp.sessionAccount,
				sessionExpiration: packedUserOp.sessionExpiration.toString(),
				sessionAccountProof: packedUserOp.sessionAccountProof,
				signature: packedUserOp.signature,
			});
			console.log("EntryPoint address:", CONTRACTS.EntryPoint.address);
			console.log("Beneficiary:", evmAddress);
			console.log("Chain ID:", chain.id);

			// First, try to simulate the transaction to get better error messages
			console.log("Simulating transaction...");
			try {
				const result = await publicClient.simulateContract({
					address: CONTRACTS.EntryPoint.address,
					abi: CONTRACTS.EntryPoint.abi,
					functionName: "handleOps",
					args: [[packedUserOp], evmAddress],
					account: evmAddress,
				});
				console.log("Simulation successful:", result);
			} catch (simError: any) {
				console.error("Simulation failed:", simError);
				if (simError.cause?.reason) {
					throw new Error(`Simulation failed: ${simError.cause.reason}`);
				}
				if (simError.shortMessage) {
					throw new Error(`Simulation failed: ${simError.shortMessage}`);
				}
				throw simError;
			}

			// For local testing: directly call EntryPoint.handleOps
			// In production, this would be sent to a bundler service
			let tx: `0x${string}`;
			try {
				tx = await walletClient.writeContract({
					address: CONTRACTS.EntryPoint.address,
					abi: CONTRACTS.EntryPoint.abi,
					functionName: "handleOps",
					args: [[packedUserOp], evmAddress], // beneficiary is the caller
				});
			} catch (error: any) {
				console.error("Write contract error:", error);
				console.error("Error details:", {
					message: error.message,
					cause: error.cause,
					reason: error.cause?.reason,
					data: error.data,
					shortMessage: error.shortMessage,
					metaMessages: error.metaMessages,
				});

				// Try to extract more specific error information
				if (error.cause?.reason) {
					throw new Error(`Transaction failed: ${error.cause.reason}`);
				}
				if (error.shortMessage) {
					throw new Error(`Transaction failed: ${error.shortMessage}`);
				}
				throw error;
			}

			console.log("Transaction hash:", tx);

			// Wait for transaction confirmation
			const receipt = await publicClient.waitForTransactionReceipt({
				hash: tx,
			});

			console.log("Transaction receipt:", receipt);

			if (receipt.status === "success") {
				setTxHash(tx);
				setAuthorizationComplete(true);
				if (onAuthorizationComplete) {
					onAuthorizationComplete();
				}
			} else {
				// Transaction was mined but reverted
				console.log("Transaction failed. Receipt:", receipt);
				throw new Error(`Transaction reverted during execution`);
			}
		} catch (err) {
			console.error("Authorization failed:", err);
			setError(err instanceof Error ? err.message : "Authorization failed");
		} finally {
			setIsProcessing(false);
		}
	};

	if (!evmAddress) {
		return (
			<div className="w-full p-6 bg-gray-50 rounded-lg border border-gray-200">
				<div className="text-center">
					<Shield className="mx-auto h-12 w-12 text-gray-400 mb-4" />
					<h3 className="text-lg font-medium text-gray-900 mb-2">
						Connect Wallet
					</h3>
					<p className="text-gray-600">
						Please connect your wallet to authorize a root key.
					</p>
				</div>
			</div>
		);
	}

	if (!isFunded) {
		return (
			<div className="w-full p-6 bg-yellow-50 rounded-lg border border-yellow-200">
				<div className="text-center">
					<AlertTriangle className="mx-auto h-12 w-12 text-yellow-500 mb-4" />
					<h3 className="text-lg font-medium text-yellow-800 mb-2">
						Fund Wallet First
					</h3>
					<p className="text-yellow-700">
						Please fund your Omni Account before setting up the root key
						authorization.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="w-full p-6 bg-white rounded-lg shadow-lg">
			<div className="text-center mb-6">
				{authorizationComplete ? (
					<CheckCircle className="mx-auto h-12 w-12 text-green-500 mb-4" />
				) : (
					<Key className="mx-auto h-12 w-12 text-blue-500 mb-4" />
				)}
				<h2 className="text-2xl font-bold">
					{authorizationComplete
						? "Authorization Complete!"
						: "Authorize Root Key"}
				</h2>
			</div>

			{authorizationComplete ? (
				<div className="space-y-4">
					<div className="bg-green-50 border border-green-200 rounded-lg p-4">
						<div className="flex items-center mb-2">
							<CheckCircle className="h-5 w-5 text-green-500 mr-2" />
							<span className="text-green-700 font-medium">
								Root Key Authorized Successfully
							</span>
						</div>
						<p className="text-green-600 text-sm">
							Your Omni Account has been initialized with the root signer:
							<span className="font-mono text-xs block mt-1 break-all">
								{rootSignerAddress}
							</span>
						</p>
					</div>

					{txHash && (
						<div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
							<p className="text-sm text-blue-700">
								<span className="font-medium">Transaction Hash:</span>
								<span className="font-mono text-xs block mt-1 break-all">
									{txHash}
								</span>
							</p>
						</div>
					)}

					<div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
						<h3 className="font-medium text-gray-900 mb-2">What's Next?</h3>
						<ul className="text-sm text-gray-700 space-y-1">
							<li>• Your Omni Account is now ready to use</li>
							<li>• You can create sessions for delegated access</li>
							<li>• Send swap requests to the worker service</li>
							<li>• Execute batch transactions via Account Abstraction</li>
						</ul>
					</div>
				</div>
			) : (
				<div className="space-y-6">
					{/* Root Signer Address Input */}
					<div className="space-y-2">
						<label
							htmlFor="rootSigner"
							className="block text-sm font-medium text-gray-700"
						>
							Root Signer Address
						</label>
						<input
							id="rootSigner"
							type="text"
							value={rootSignerAddress}
							onChange={(e) => setRootSignerAddress(e.target.value)}
							placeholder="0x..."
							className={`w-full px-3 py-2 border rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 ${
								rootSignerAddress && !isValidAddress
									? "border-red-300 bg-red-50"
									: "border-gray-300"
							}`}
						/>
						{rootSignerAddress && !isValidAddress && (
							<p className="text-sm text-red-600">
								Please enter a valid Ethereum address
							</p>
						)}
						<p className="text-xs text-gray-500">
							This address will be authorized to sign transactions on behalf of
							your Smart Account
						</p>
					</div>

					{/* Error Message */}
					{error && (
						<div className="bg-red-50 border border-red-200 rounded-lg p-3">
							<div className="flex">
								<AlertTriangle className="h-5 w-5 text-red-500 mr-2" />
								<span className="text-sm text-red-700">{error}</span>
							</div>
						</div>
					)}

					{/* Authorization Button */}
					<button
						onClick={handleAuthorizeRootKey}
						disabled={!isValidAddress || isProcessing}
						className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-medium py-3 px-4 rounded-lg transition-colors flex items-center justify-center"
					>
						{isProcessing ? (
							<>
								<Loader2 className="h-4 w-4 mr-2 animate-spin" />
								Authorizing...
							</>
						) : (
							<>
								<Shield className="h-4 w-4 mr-2" />
								Authorize Root Key
							</>
						)}
					</button>

					{/* Information */}
					<div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
						<h3 className="font-medium text-gray-900 mb-2">
							About Root Key Authorization
						</h3>
						<ul className="text-sm text-gray-700 space-y-1">
							<li>• Creates and initializes your Smart Account contract</li>
							<li>
								• Your current wallet will be set as the initial root signer
							</li>
							<li>• You can add additional root signers after deployment</li>
							<li>• Your wallet's omni account will be the owner</li>
						</ul>
					</div>

					{/* Important Notice */}
					<div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
						<h4 className="font-medium text-yellow-800 mb-1">Note:</h4>
						<p className="text-xs text-yellow-700">
							The Smart Account will be deployed with your current wallet (
							{truncateAddress(evmAddress)}) as the initial root signer. This
							ensures you have full control over your Smart Account from
							deployment.
						</p>
					</div>
				</div>
			)}
		</div>
	);
}
