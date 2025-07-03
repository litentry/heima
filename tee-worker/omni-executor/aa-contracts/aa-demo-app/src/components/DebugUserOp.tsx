"use client";

import { usePublicClient } from "wagmi";
import { useState } from "react";
import { parseEventLogs } from "viem";
import { CONTRACTS } from "@/lib/constants";

export function DebugUserOp({ txHash }: { txHash?: string }) {
	const publicClient = usePublicClient();
	const [debugInfo, setDebugInfo] = useState<any>(null);
	const [inputHash, setInputHash] = useState("");

	const debugTransaction = async (hash: string) => {
		if (!publicClient) return;
		
		try {
			const receipt = await publicClient.getTransactionReceipt({ hash: hash as `0x${string}` });
			console.log("Full receipt:", receipt);
			
			// Parse EntryPoint events
			const entryPointEvents = parseEventLogs({
				abi: CONTRACTS.EntryPoint.abi,
				logs: receipt.logs,
			});
			
			console.log("EntryPoint events:", entryPointEvents);
			
			// Find UserOperationEvent
			const userOpEvent = entryPointEvents.find(e => e.eventName === "UserOperationEvent");
			const revertEvent = entryPointEvents.find(e => e.eventName === "UserOperationRevertReason");
			
			setDebugInfo({
				receipt,
				events: entryPointEvents,
				userOpEvent,
				revertEvent,
				success: userOpEvent?.args?.success,
				revertReason: revertEvent?.args?.revertReason,
			});
			
		} catch (error) {
			console.error("Debug error:", error);
			setDebugInfo({ error: error.message });
		}
	};

	return (
		<div className="p-4 bg-gray-100 rounded-lg">
			<h3 className="font-bold mb-2">Debug UserOperation</h3>
			<div className="flex gap-2 mb-4">
				<input
					type="text"
					value={inputHash}
					onChange={(e) => setInputHash(e.target.value)}
					placeholder="Transaction hash"
					className="flex-1 px-3 py-2 border rounded"
				/>
				<button
					onClick={() => debugTransaction(inputHash || txHash || "")}
					className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
				>
					Debug
				</button>
			</div>
			
			{debugInfo && (
				<div className="space-y-2">
					<div className="p-2 bg-white rounded">
						<strong>Success:</strong> {debugInfo.success?.toString() || "N/A"}
					</div>
					{debugInfo.revertReason && (
						<div className="p-2 bg-red-100 rounded">
							<strong>Revert Reason:</strong> {debugInfo.revertReason}
						</div>
					)}
					<details>
						<summary className="cursor-pointer">Full Debug Info</summary>
						<pre className="text-xs overflow-auto p-2 bg-white rounded">
							{JSON.stringify(debugInfo, null, 2)}
						</pre>
					</details>
				</div>
			)}
		</div>
	);
}