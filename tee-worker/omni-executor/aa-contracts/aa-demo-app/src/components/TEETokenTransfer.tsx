import { useState, useEffect } from "react";
import { usePublicClient, useChainId } from "wagmi";
import { Send, AlertCircle, CheckCircle, Loader2, CreditCard } from "lucide-react";
import { formatUnits, parseUnits, isAddress, Address } from "viem";
import { ERC20_TOKENS, CONTRACTS, DEFAULT_CLIENT_ID } from "@/lib/constants";
import { submitUserOpTest } from "@/lib/tee-worker-client";
import {
    buildTokenTransferUserOp,
    buildNativeTransferUserOp,
    buildApprovalUserOp,
    packUserOperation,
    toSerializablePackedUserOperation,
    estimateUserOpGasFromWorker,
    buildErc20PaymasterData,
} from "@/lib/aa-utils";

interface TEETokenTransferProps {
    omniAccountAddress: string;
    omniAccountHash: string;
    isDeployed: boolean;
    teeWorkerAddress: string | null;
}

interface TokenBalance {
    symbol: string;
    balance: bigint;
    decimals: number;
    address: `0x${string}`;
}

type PaymasterType = "none" | "simple" | "erc20";
type GasToken = "USDC" | "USDT";

export function TEETokenTransfer({
    omniAccountAddress,
    omniAccountHash,
    isDeployed,
    teeWorkerAddress,
}: TEETokenTransferProps) {
    const publicClient = usePublicClient();
    const chainId = useChainId();
    const [selectedToken, setSelectedToken] = useState<"ETH" | "USDC" | "USDT">("ETH");
    const [recipient, setRecipient] = useState("");
    const [amount, setAmount] = useState("");
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [txHash, setTxHash] = useState<string | null>(null);
    const [tokenBalances, setTokenBalances] = useState<TokenBalance[]>([]);
    const [nonce, setNonce] = useState<bigint>(BigInt(0));
    const [paymasterType, setPaymasterType] = useState<PaymasterType>("none");
    const [gasToken, setGasToken] = useState<GasToken>("USDC");
    const [tokenAllowances, setTokenAllowances] = useState<Record<string, bigint>>({});

    // Available tokens including ETH
    const availableTokens = [
        { symbol: "ETH", decimals: 18, address: "0x0000000000000000000000000000000000000000" as `0x${string}`, isNative: true },
        ERC20_TOKENS.USDC,
        ERC20_TOKENS.USDT
    ];

    // Fetch token balances
    const fetchBalances = async () => {
        if (!omniAccountAddress || !publicClient) return;

        const balances: TokenBalance[] = [];

        // Fetch ETH balance first
        try {
            const ethBalance = await publicClient.getBalance({
                address: omniAccountAddress as `0x${string}`,
            });
            balances.push({
                symbol: "ETH",
                balance: ethBalance,
                decimals: 18,
                address: "0x0000000000000000000000000000000000000000" as `0x${string}`,
            });
        } catch (error) {
            console.error("Error fetching ETH balance:", error);
            balances.push({
                symbol: "ETH",
                balance: BigInt(0),
                decimals: 18,
                address: "0x0000000000000000000000000000000000000000" as `0x${string}`,
            });
        }

        // Fetch ERC20 balances
        for (const token of [ERC20_TOKENS.USDC, ERC20_TOKENS.USDT]) {
            try {
                const balance = (await publicClient.readContract({
                    address: token.address,
                    abi: token.abi,
                    functionName: "balanceOf",
                    args: [omniAccountAddress as `0x${string}`],
                })) as bigint;

                balances.push({
                    symbol: token.symbol,
                    balance,
                    decimals: token.decimals,
                    address: token.address,
                });
            } catch (error) {
                console.error(`Error fetching ${token.symbol} balance:`, error);
                balances.push({
                    symbol: token.symbol,
                    balance: BigInt(0),
                    decimals: token.decimals,
                    address: token.address,
                });
            }
        }
        setTokenBalances(balances);
    };

    // Fetch current nonce
    const fetchNonce = async () => {
        if (!omniAccountAddress || !publicClient) return;

        try {
            const currentNonce = (await publicClient.readContract({
                address: CONTRACTS.EntryPoint.address,
                abi: CONTRACTS.EntryPoint.abi,
                functionName: "getNonce",
                args: [omniAccountAddress as `0x${string}`, BigInt(0)],
            })) as bigint;
            setNonce(currentNonce);
        } catch (error) {
            console.error("Error fetching nonce:", error);
        }
    };

    // Fetch token allowances for ERC20 paymaster
    const fetchTokenAllowances = async () => {
        if (!omniAccountAddress || !publicClient || CONTRACTS.ERC20PaymasterV1.address === "0x0000000000000000000000000000000000000000") return;

        const allowances: Record<string, bigint> = {};
        
        for (const token of [ERC20_TOKENS.USDC, ERC20_TOKENS.USDT]) {
            try {
                const allowance = (await publicClient.readContract({
                    address: token.address,
                    abi: token.abi,
                    functionName: "allowance",
                    args: [omniAccountAddress as `0x${string}`, CONTRACTS.ERC20PaymasterV1.address],
                })) as bigint;
                allowances[token.symbol] = allowance;
            } catch (error) {
                console.error(`Error fetching ${token.symbol} allowance:`, error);
                allowances[token.symbol] = BigInt(0);
            }
        }
        
        setTokenAllowances(allowances);
    };

    useEffect(() => {
        fetchBalances();
        fetchNonce();
        fetchTokenAllowances();
        const interval = setInterval(() => {
            fetchBalances();
            fetchNonce();
            fetchTokenAllowances();
        }, 10000);
        return () => clearInterval(interval);
    }, [omniAccountAddress, publicClient]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);
        setTxHash(null);

        // Validation
        if (!isAddress(recipient)) {
            setError("Invalid recipient address");
            return;
        }

        if (!amount || parseFloat(amount) <= 0) {
            setError("Invalid amount");
            return;
        }

        const tokenBalance = tokenBalances.find(tb => tb.symbol === selectedToken);

        if (!tokenBalance) {
            setError("Token balance not loaded");
            return;
        }

        const decimals = selectedToken === "ETH" ? 18 :
            selectedToken === "USDC" ? ERC20_TOKENS.USDC.decimals :
                ERC20_TOKENS.USDT.decimals;
        const amountBigInt = parseUnits(amount, decimals);

        if (amountBigInt > tokenBalance.balance) {
            setError("Insufficient balance");
            return;
        }

        // Check if using ERC20 paymaster and if approval is needed
        if (paymasterType === "erc20") {
            const gasTokenInfo = gasToken === "USDC" ? ERC20_TOKENS.USDC : ERC20_TOKENS.USDT;
            const currentAllowance = tokenAllowances[gasToken] || BigInt(0);
            
            // Estimate required amount for gas (conservative estimate)
            const estimatedGasAmount = parseUnits("10", gasTokenInfo.decimals); // 10 tokens for gas
            
            if (currentAllowance < estimatedGasAmount) {
                setIsSubmitting(true);
                try {
                    console.log(`Approving ${gasToken} for ERC20 paymaster...`);
                    // Build ERC20 paymaster data segment (rate=0 for worker to fill)
                    const nowSec = Math.floor(Date.now() / 1000);
                    const erc20PaymasterData = buildErc20PaymasterData(
                        gasTokenInfo.address,
                        BigInt(0), // exchangeRate=0 -> worker fills
                        BigInt(nowSec + 3600), // valid for 1 hour
                        BigInt(0), // valid immediately
                    );

                    // Build approval UserOp with paymaster for gas estimation
                    const approvalAmount = parseUnits("1000", gasTokenInfo.decimals); // Approve 1000 tokens
                    const approvalUserOp = buildApprovalUserOp({
                        omniAccountAddress: omniAccountAddress as `0x${string}`,
                        tokenAddress: gasTokenInfo.address,
                        spender: CONTRACTS.ERC20PaymasterV1.address,
                        amount: approvalAmount,
                        nonce,
                        paymaster: {
                            address: CONTRACTS.ERC20PaymasterV1.address,
                            validationGasLimit: BigInt(100000),
                            postOpGasLimit: BigInt(50000),
                            data: erc20PaymasterData,
                        },
                        forGasEstimation: true,
                    });
                    
                    // Estimate gas for approval
                    const approvalGasParams = await estimateUserOpGasFromWorker(
                        approvalUserOp,
                        chainId,
                        0,
                        omniAccountHash,
                        DEFAULT_CLIENT_ID,
                        publicClient
                    );
                    
                    // Rebuild paymaster with worker-estimated paymaster gas limits
                    const finalApprovalPaymaster = {
                        address: CONTRACTS.ERC20PaymasterV1.address as Address,
                        validationGasLimit: approvalGasParams.paymasterVerificationGasLimit || BigInt(100000),
                        postOpGasLimit: approvalGasParams.paymasterPostOpGasLimit || BigInt(50000),
                        data: erc20PaymasterData,
                    };

                    // Rebuild approval with gas estimates and updated paymaster envelope
                    const finalApprovalOp = buildApprovalUserOp({
                        omniAccountAddress: omniAccountAddress as `0x${string}`,
                        tokenAddress: gasTokenInfo.address,
                        spender: CONTRACTS.ERC20PaymasterV1.address,
                        amount: approvalAmount,
                        nonce,
                        gasParams: approvalGasParams,
                        paymaster: finalApprovalPaymaster,
                    });
                    
                    // Submit approval
                    const approvalPackedOp = packUserOperation(finalApprovalOp);
                    const approvalSerializable = toSerializablePackedUserOperation(approvalPackedOp);
                    
                    const approvalResponse = await submitUserOpTest(
                        [approvalSerializable],
                        chainId,
                        0,
                        omniAccountHash,
                        DEFAULT_CLIENT_ID
                    );
                    
                    if (!approvalResponse.transaction_hash) {
                        throw new Error("Token approval failed");
                    }
                    
                    console.log("Approval successful:", approvalResponse.transaction_hash);

                    // Wait for approval inclusion on-chain before proceeding
                    try {
                        await publicClient?.waitForTransactionReceipt({
                            hash: approvalResponse.transaction_hash as `0x${string}`,
                            confirmations: 1,
                        });
                    } catch (waitErr) {
                        console.warn("Waiting for approval receipt failed or timed out:", waitErr);
                    }

                    // Refresh nonce and allowances after approval mined
                    await fetchNonce();
                    await fetchTokenAllowances();
                } catch (err: any) {
                    console.error("Error approving token:", err);
                    setError(`Failed to approve ${gasToken}: ${err.message}`);
                    setIsSubmitting(false);
                    return;
                }
            }
        }

        setIsSubmitting(true);

        try {
            // Prepare paymaster parameter if using paymaster
            let paymaster: { address: Address; validationGasLimit?: bigint; postOpGasLimit?: bigint; data?: `0x${string}` } | undefined;
            if (paymasterType === "simple") {
                // Simple paymaster
                paymaster = {
                    address: CONTRACTS.SimplePaymaster.address,
                    validationGasLimit: BigInt(100000),
                    postOpGasLimit: BigInt(50000),
                };
            } else if (paymasterType === "erc20") {
                // ERC20 paymaster needs token and exchange rate data
                const gasTokenInfo = gasToken === "USDC" ? ERC20_TOKENS.USDC : ERC20_TOKENS.USDT;
                // Build ERC20 paymaster data (rate=0, worker fills)
                const nowSec = Math.floor(Date.now() / 1000);
                const erc20PaymasterData = buildErc20PaymasterData(
                    gasTokenInfo.address,
                    BigInt(0),
                    BigInt(nowSec + 3600),
                    BigInt(0),
                );

                paymaster = {
                    address: CONTRACTS.ERC20PaymasterV1.address,
                    validationGasLimit: BigInt(100000),
                    postOpGasLimit: BigInt(50000),
                    data: erc20PaymasterData,
                };
            }

            // Build the initial UserOperation for transfer with minimal gas for estimation
            const userOpForEstimation = selectedToken === "ETH" ?
                buildNativeTransferUserOp({
                    omniAccountAddress: omniAccountAddress as `0x${string}`,
                    recipient: recipient as `0x${string}`,
                    amount: amountBigInt,
                    nonce,
                    forGasEstimation: true,  // Use dummy signature for gas estimation
                    paymaster,
                    gasParams: {
                        // Use minimal gas values for estimation to avoid prefund issues
                        callGasLimit: BigInt(100000),        // Minimal for simulation
                        verificationGasLimit: BigInt(150000), // Enough for signature validation
                        preVerificationGas: BigInt(21000),    // Base transaction cost
                        maxFeePerGas: BigInt(1000000000),     // 1 gwei - minimal for simulation
                        maxPriorityFeePerGas: BigInt(1000000000), // 1 gwei - minimal
                    }
                }) :
                buildTokenTransferUserOp({
                    omniAccountAddress: omniAccountAddress as `0x${string}`,
                    tokenAddress: selectedToken === "USDC" ? ERC20_TOKENS.USDC.address : ERC20_TOKENS.USDT.address,
                    recipient: recipient as `0x${string}`,
                    amount: amountBigInt,
                    nonce,
                    forGasEstimation: true,  // Use dummy signature for gas estimation
                    paymaster,
                    gasParams: {
                        // Use minimal gas values for estimation to avoid prefund issues
                        callGasLimit: BigInt(100000),        // Minimal for simulation
                        verificationGasLimit: BigInt(150000), // Enough for signature validation
                        preVerificationGas: BigInt(21000),    // Base transaction cost
                        maxFeePerGas: BigInt(1000000000),     // 1 gwei - minimal for simulation
                        maxPriorityFeePerGas: BigInt(1000000000), // 1 gwei - minimal
                    }
                });

            // Estimate gas using the TEE worker
            console.log("Attempting to estimate gas using TEE worker...");
            const gasParams = await estimateUserOpGasFromWorker(
                userOpForEstimation,
                chainId,
                0, // wallet_index
                omniAccountHash,
                DEFAULT_CLIENT_ID,
                publicClient
            );
            console.log("Successfully estimated gas using TEE worker:", gasParams);

            // Build the final UserOperation with gas estimates
            // If using ERC20 paymaster, rebuild paymaster with worker-estimated paymaster gas limits
            const finalPaymaster = paymaster && {
                address: paymaster.address,
                validationGasLimit: gasParams.paymasterVerificationGasLimit || paymaster.validationGasLimit || BigInt(100000),
                postOpGasLimit: gasParams.paymasterPostOpGasLimit || paymaster.postOpGasLimit || BigInt(50000),
                data: paymaster.data,
            };

            const userOp = selectedToken === "ETH" ?
                buildNativeTransferUserOp({
                    omniAccountAddress: omniAccountAddress as `0x${string}`,
                    recipient: recipient as `0x${string}`,
                    amount: amountBigInt,
                    nonce,
                    paymaster: finalPaymaster,
                    gasParams,
                }) :
                buildTokenTransferUserOp({
                    omniAccountAddress: omniAccountAddress as `0x${string}`,
                    tokenAddress: selectedToken === "USDC" ? ERC20_TOKENS.USDC.address : ERC20_TOKENS.USDT.address,
                    recipient: recipient as `0x${string}`,
                    amount: amountBigInt,
                    nonce,
                    paymaster: finalPaymaster,
                    gasParams,
                });

            // Pack the UserOperation
            const packedOp = packUserOperation(userOp);

            // Convert to serializable format
            const serializableOp = toSerializablePackedUserOperation(packedOp);

            console.log("Submitting UserOperation through TEE worker:", serializableOp);

            // Submit through TEE worker
            const response = await submitUserOpTest(
                [serializableOp],
                chainId,
                0, // wallet_index
                omniAccountHash,
                DEFAULT_CLIENT_ID
            );

            console.log("TEE Worker response:", response);

            if (response.transaction_hash) {
                setTxHash(response.transaction_hash);
                // Clear form
                setRecipient("");
                setAmount("");
                // Refresh balances and nonce
                fetchBalances();
                fetchNonce();
            } else {
                setError("Transaction failed - no hash returned");
            }
        } catch (err: any) {
            console.error("Error submitting transaction:", err);
            setError(err.message || "Failed to submit transaction");
        } finally {
            setIsSubmitting(false);
        }
    };

    if (!isDeployed) {
        return (
            <div className="w-full p-6 bg-yellow-50 rounded-lg border border-yellow-200">
                <div className="text-center">
                    <AlertCircle className="mx-auto h-12 w-12 text-yellow-500 mb-4" />
                    <h3 className="text-lg font-medium text-yellow-800 mb-2">
                        Omni Account Not Deployed
                    </h3>
                    <p className="text-yellow-700">
                        Deploy your Omni Account before sending token transfers.
                    </p>
                </div>
            </div>
        );
    }

    if (!teeWorkerAddress) {
        return (
            <div className="w-full p-6 bg-yellow-50 rounded-lg border border-yellow-200">
                <div className="text-center">
                    <AlertCircle className="mx-auto h-12 w-12 text-yellow-500 mb-4" />
                    <h3 className="text-lg font-medium text-yellow-800 mb-2">
                        TEE Worker Not Authorized
                    </h3>
                    <p className="text-yellow-700">
                        Authorize the TEE worker before sending token transfers.
                    </p>
                </div>
            </div>
        );
    }

    const selectedTokenBalance = tokenBalances.find(tb => tb.symbol === selectedToken);

    return (
        <div className="w-full p-6 bg-white rounded-lg shadow-lg">
            <div className="text-center mb-6">
                <Send className="mx-auto h-12 w-12 text-blue-500 mb-4" />
                <h2 className="text-2xl font-bold">Send Token Transfer</h2>
                <p className="text-gray-600 mt-2">
                    Transfer ETH, USDC, or USDT through the TEE worker
                </p>
            </div>

            {/* Token Balances */}
            <div className="mb-6">
                <h3 className="text-sm font-medium text-gray-700 mb-2">Token Balances</h3>
                <div className="space-y-2">
                    {tokenBalances.map((tb) => (
                        <div
                            key={tb.symbol}
                            className={`p-3 rounded-lg flex justify-between items-center cursor-pointer transition-colors ${selectedToken === tb.symbol
                                ? tb.symbol === "ETH"
                                    ? "bg-purple-100 border-2 border-purple-500"
                                    : tb.symbol === "USDC"
                                        ? "bg-blue-100 border-2 border-blue-500"
                                        : "bg-green-100 border-2 border-green-500"
                                : tb.symbol === "ETH"
                                    ? "bg-purple-50 hover:bg-purple-100"
                                    : tb.symbol === "USDC"
                                        ? "bg-blue-50 hover:bg-blue-100"
                                        : "bg-green-50 hover:bg-green-100"
                                }`}
                            onClick={() => setSelectedToken(tb.symbol as "ETH" | "USDC" | "USDT")}
                        >
                            <span className="font-medium">{tb.symbol}</span>
                            <span className="font-mono text-sm">
                                {formatUnits(tb.balance, tb.decimals)} {tb.symbol}
                            </span>
                        </div>
                    ))}
                </div>
            </div>

            {/* Paymaster Selection */}
            {(CONTRACTS.ERC20PaymasterV1.address !== "0x0000000000000000000000000000000000000000") && (
                <div className="mb-6">
                    <h3 className="text-sm font-medium text-gray-700 mb-2">
                        <CreditCard className="inline-block h-4 w-4 mr-1" />
                        Gas Payment Method
                    </h3>
                    <div className="space-y-2">
                        <label className="flex items-center p-3 rounded-lg border cursor-pointer hover:bg-gray-50">
                            <input
                                type="radio"
                                name="paymaster"
                                value="none"
                                checked={paymasterType === "none"}
                                onChange={() => setPaymasterType("none")}
                                className="mr-3"
                            />
                            <div>
                                <div className="font-medium">Pay with ETH</div>
                                <div className="text-sm text-gray-500">Traditional gas payment from account</div>
                            </div>
                        </label>
                        
                        <label className="flex items-center p-3 rounded-lg border cursor-pointer hover:bg-gray-50">
                            <input
                                type="radio"
                                name="paymaster"
                                value="simple"
                                checked={paymasterType === "simple"}
                                onChange={() => setPaymasterType("simple")}
                                className="mr-3"
                            />
                            <div>
                                <div className="font-medium">Sponsored Gas</div>
                                <div className="text-sm text-gray-500">Paymaster covers gas fees</div>
                            </div>
                        </label>
                        
                        <label className="flex items-center p-3 rounded-lg border cursor-pointer hover:bg-gray-50">
                            <input
                                type="radio"
                                name="paymaster"
                                value="erc20"
                                checked={paymasterType === "erc20"}
                                onChange={() => setPaymasterType("erc20")}
                                className="mr-3"
                            />
                            <div>
                                <div className="font-medium">Pay with ERC20 Token</div>
                                <div className="text-sm text-gray-500">Use USDC or USDT for gas fees</div>
                            </div>
                        </label>
                    </div>
                    
                    {/* Gas Token Selection for ERC20 Paymaster */}
                    {paymasterType === "erc20" && (
                        <div className="mt-4 p-3 bg-yellow-50 rounded-lg border border-yellow-200">
                            <h4 className="text-sm font-medium text-gray-700 mb-2">Select Gas Token</h4>
                            <div className="flex gap-2">
                                <button
                                    type="button"
                                    onClick={() => setGasToken("USDC")}
                                    className={`flex-1 py-2 px-3 rounded-lg border transition-colors ${
                                        gasToken === "USDC"
                                            ? "bg-blue-100 border-blue-500 text-blue-700"
                                            : "bg-white border-gray-300 hover:bg-gray-50"
                                    }`}
                                >
                                    USDC
                                </button>
                                <button
                                    type="button"
                                    onClick={() => setGasToken("USDT")}
                                    className={`flex-1 py-2 px-3 rounded-lg border transition-colors ${
                                        gasToken === "USDT"
                                            ? "bg-green-100 border-green-500 text-green-700"
                                            : "bg-white border-gray-300 hover:bg-gray-50"
                                    }`}
                                >
                                    USDT
                                </button>
                            </div>
                            
                            {/* Show current allowance */}
                            <div className="mt-3 text-sm">
                                <div className="flex justify-between">
                                    <span className="text-gray-600">Current Allowance:</span>
                                    <span className="font-mono">
                                        {formatUnits(
                                            tokenAllowances[gasToken] || BigInt(0),
                                            gasToken === "USDC" ? 6 : 6
                                        )} {gasToken}
                                    </span>
                                </div>
                                {(tokenAllowances[gasToken] || BigInt(0)) < parseUnits("10", 6) && (
                                    <div className="mt-2 text-yellow-700 text-xs">
                                        ⚠️ Approval will be requested before transfer
                                    </div>
                                )}
                            </div>
                        </div>
                    )}
                </div>
            )}

            {/* Transfer Form */}
            <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                        Recipient Address
                    </label>
                    <input
                        type="text"
                        value={recipient}
                        onChange={(e) => setRecipient(e.target.value)}
                        placeholder="0x..."
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    />
                </div>

                <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                        Amount ({selectedToken})
                    </label>
                    <input
                        type="number"
                        step="0.000001"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                        placeholder="0.0"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    />
                    {selectedTokenBalance && amount && (
                        <p className="text-sm text-gray-500 mt-1">
                            Balance: {formatUnits(selectedTokenBalance.balance, selectedTokenBalance.decimals)} {selectedToken}
                        </p>
                    )}
                </div>

                {error && (
                    <div className="bg-red-50 border border-red-200 rounded-lg p-3">
                        <p className="text-red-700 text-sm">{error}</p>
                    </div>
                )}

                {txHash && (
                    <div className="bg-green-50 border border-green-200 rounded-lg p-3">
                        <div className="flex items-center">
                            <CheckCircle className="h-5 w-5 text-green-500 mr-2" />
                            <div>
                                <p className="text-green-700 font-medium">Transaction Submitted!</p>
                                <p className="text-green-600 text-sm mt-1 font-mono break-all">
                                    {txHash}
                                </p>
                            </div>
                        </div>
                    </div>
                )}

                <button
                    type="submit"
                    disabled={isSubmitting || !selectedTokenBalance || selectedTokenBalance.balance === BigInt(0)}
                    className="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-4 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
                >
                    {isSubmitting ? (
                        <>
                            <Loader2 className="animate-spin h-5 w-5 mr-2" />
                            Submitting...
                        </>
                    ) : (
                        <>
                            <Send className="h-5 w-5 mr-2" />
                            Send {selectedToken}
                        </>
                    )}
                </button>
            </form>

            <div className="mt-6 bg-blue-50 border border-blue-200 rounded-lg p-3">
                <div className="flex">
                    <AlertCircle className="h-5 w-5 text-blue-500 mr-2 mt-0.5" />
                    <div className="text-sm text-blue-700">
                        <p className="font-medium mb-1">How it works:</p>
                        <ul className="list-disc list-inside space-y-1">
                            <li>This creates a UserOperation for {selectedToken === "ETH" ? "a native ETH" : "an ERC20"} transfer</li>
                            <li>The TEE worker signs and submits the operation</li>
                            {paymasterType === "simple" && (
                                <li>The paymaster sponsors your gas fees</li>
                            )}
                            {paymasterType === "erc20" && (
                                <>
                                    <li>Gas fees are paid with {gasToken} tokens</li>
                                    <li>TEE worker automatically fetches exchange rates</li>
                                </>
                            )}
                            <li>Your Omni Account executes the transfer</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    );
}
