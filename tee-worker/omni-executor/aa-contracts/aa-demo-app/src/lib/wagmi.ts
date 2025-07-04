"use client";

import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { http, defineChain } from "viem";
import { sepolia } from "viem/chains";

// Define Anvil chain for local development
const anvil = defineChain({
	id: 1337,
	name: "Anvil Local",
	nativeCurrency: {
		decimals: 18,
		name: "Ether",
		symbol: "ETH",
	},
	rpcUrls: {
		default: { http: ["http://127.0.0.1:8545"] },
		public: { http: ["http://127.0.0.1:8545"] },
	},
});

export const config = getDefaultConfig({
	appName: "AA Demo App",
	projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || "PROJECT_ID",
	chains: [anvil, sepolia],
	transports: {
		[anvil.id]: http("http://127.0.0.1:8545"),
		[sepolia.id]: http(
			process.env.NEXT_PUBLIC_SEPOLIA_RPC_URL || "add-provider-url-here", // TODO;
		),
	},
	ssr: true,
});
