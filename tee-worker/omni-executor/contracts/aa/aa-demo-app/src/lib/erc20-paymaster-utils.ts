import { Address, PublicClient } from "viem";
import { CONTRACTS, ERC20_TOKENS } from "./constants";

/**
 * Check if ERC20 paymaster is ready for use
 */
export async function checkERC20PaymasterReadiness(
    publicClient: PublicClient,
    omniAccountAddress: Address,
    gasToken: "USDC" | "USDT"
): Promise<{
    isReady: boolean;
    issues: string[];
    details: {
        paymasterDeployed: boolean;
        paymasterDeposit: bigint;
        tokenBalance: bigint;
        tokenAllowance: bigint;
        sufficientBalance: boolean;
        sufficientAllowance: boolean;
    };
}> {
    const issues: string[] = [];
    const tokenInfo = gasToken === "USDC" ? ERC20_TOKENS.USDC : ERC20_TOKENS.USDT;
    
    // Check if paymaster is deployed
    const paymasterCode = await publicClient.getBytecode({
        address: CONTRACTS.ERC20PaymasterV1.address
    });
    const paymasterDeployed = !!(paymasterCode && paymasterCode !== "0x");
    
    if (!paymasterDeployed) {
        issues.push("ERC20 Paymaster is not deployed");
    }
    
    // Check paymaster deposit at EntryPoint
    let paymasterDeposit = BigInt(0);
    try {
        paymasterDeposit = await publicClient.readContract({
            address: CONTRACTS.EntryPoint.address,
            abi: CONTRACTS.EntryPoint.abi,
            functionName: "balanceOf",
            args: [CONTRACTS.ERC20PaymasterV1.address],
        }) as bigint;
        
        if (paymasterDeposit < BigInt("10000000000000000")) { // 0.01 ETH
            issues.push(`Paymaster has insufficient deposit: ${paymasterDeposit} wei`);
        }
    } catch (e) {
        issues.push("Could not check paymaster deposit");
    }
    
    // Check token balance
    let tokenBalance = BigInt(0);
    try {
        tokenBalance = await publicClient.readContract({
            address: tokenInfo.address,
            abi: tokenInfo.abi,
            functionName: "balanceOf",
            args: [omniAccountAddress],
        }) as bigint;
        
        // Need at least 5 tokens for gas (conservative estimate)
        const minRequired = BigInt(5) * BigInt(10 ** tokenInfo.decimals);
        if (tokenBalance < minRequired) {
            issues.push(`Insufficient ${gasToken} balance: ${tokenBalance / BigInt(10 ** tokenInfo.decimals)} ${gasToken} (need at least 5)`);
        }
    } catch (e) {
        issues.push(`Could not check ${gasToken} balance`);
    }
    
    // Check token allowance
    let tokenAllowance = BigInt(0);
    try {
        tokenAllowance = await publicClient.readContract({
            address: tokenInfo.address,
            abi: tokenInfo.abi,
            functionName: "allowance",
            args: [omniAccountAddress, CONTRACTS.ERC20PaymasterV1.address],
        }) as bigint;
    } catch (e) {
        issues.push(`Could not check ${gasToken} allowance`);
    }
    
    const sufficientBalance = tokenBalance >= BigInt(5) * BigInt(10 ** tokenInfo.decimals);
    const sufficientAllowance = tokenAllowance >= BigInt(10) * BigInt(10 ** tokenInfo.decimals);
    
    return {
        isReady: paymasterDeployed && issues.length === 0 && sufficientBalance,
        issues,
        details: {
            paymasterDeployed,
            paymasterDeposit,
            tokenBalance,
            tokenAllowance,
            sufficientBalance,
            sufficientAllowance,
        }
    };
}

/**
 * Estimate required tokens for gas payment
 */
export function estimateRequiredTokens(
    gasUnits: bigint,
    maxFeePerGas: bigint,
    exchangeRate: bigint, // tokens per ETH * 10^decimals
    tokenDecimals: number
): bigint {
    // Calculate ETH cost
    const ethCost = gasUnits * maxFeePerGas;
    
    // Calculate token cost
    // tokenCost = (ethCost * exchangeRate) / 10^18
    const tokenCost = (ethCost * exchangeRate) / BigInt(10 ** 18);
    
    return tokenCost;
}