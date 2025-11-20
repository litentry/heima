import { type PublicClient, type WalletClient, parseEventLogs } from "viem";
import { CONTRACTS } from "./constants";

export async function simulateUserOperation(
  publicClient: PublicClient,
  packedUserOp: any,
  beneficiary: `0x${string}`
) {
  try {
    console.log("Simulating UserOperation...");
    const result = await publicClient.simulateContract({
      address: CONTRACTS.EntryPoint.address,
      abi: CONTRACTS.EntryPoint.abi,
      functionName: "handleOps",
      args: [[packedUserOp], beneficiary],
      account: beneficiary,
    });
    console.log("Simulation successful:", result);
    return { success: true, result };
  } catch (error: any) {
    console.error("Simulation failed:", error);
    
    // Extract error details
    const errorMessage = error.message || error.toString();
    const revertReason = error.cause?.reason || error.shortMessage || errorMessage;
    
    // Common error patterns
    if (errorMessage.includes("AA24") || revertReason.includes("AA24")) {
      return { 
        success: false, 
        error: "Invalid signature: The signer is not authorized for this account",
        details: revertReason
      };
    }
    
    if (errorMessage.includes("AA25") || revertReason.includes("AA25")) {
      return { 
        success: false, 
        error: "Invalid nonce",
        details: revertReason
      };
    }
    
    if (errorMessage.includes("only owner")) {
      return { 
        success: false, 
        error: "Only the account owner can perform this operation",
        details: revertReason
      };
    }
    
    return { 
      success: false, 
      error: revertReason,
      fullError: error
    };
  }
}

export async function debugTransaction(
  publicClient: PublicClient,
  txHash: `0x${string}`
) {
  const receipt = await publicClient.getTransactionReceipt({ hash: txHash });
  
  // Parse EntryPoint events
  const entryPointEvents = parseEventLogs({
    abi: CONTRACTS.EntryPoint.abi,
    logs: receipt.logs,
  });
  
  const userOpEvent = entryPointEvents.find(e => (e as any).eventName === "UserOperationEvent");
  const revertEvent = entryPointEvents.find(e => (e as any).eventName === "UserOperationRevertReason");
  
  return {
    receipt,
    events: entryPointEvents,
    userOpSuccess: (userOpEvent as any)?.args?.success,
    revertReason: (revertEvent as any)?.args?.revertReason,
    userOpEvent,
    revertEvent,
  };
}