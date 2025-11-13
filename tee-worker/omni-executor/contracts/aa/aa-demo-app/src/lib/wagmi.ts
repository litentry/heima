"use client";

import { getDefaultConfig } from "@rainbow-me/rainbowkit";
import { http, defineChain, Chain } from "viem";
import { 
  mainnet, 
  sepolia, 
  arbitrum, 
  arbitrumSepolia,
  optimism,
  optimismSepolia,
  polygon,
  polygonMumbai,
  base,
  baseSepolia 
} from "viem/chains";

// Get chain configuration from environment
const chainId = parseInt(process.env.NEXT_PUBLIC_CHAIN_ID || "1337");
const rpcUrl = process.env.NEXT_PUBLIC_RPC_URL || "http://127.0.0.1:8545";

// Map of known chains
const knownChains: Record<number, Chain> = {
  1: mainnet,
  11155111: sepolia,
  42161: arbitrum,
  421614: arbitrumSepolia,
  10: optimism,
  11155420: optimismSepolia,
  137: polygon,
  80001: polygonMumbai,
  8453: base,
  84532: baseSepolia,
};

// Define custom chain for local development or unknown chains
const customChain = defineChain({
  id: chainId,
  name: process.env.NEXT_PUBLIC_CHAIN_NAME || `Chain ${chainId}`,
  nativeCurrency: {
    decimals: 18,
    name: process.env.NEXT_PUBLIC_NATIVE_CURRENCY_NAME || "Ether",
    symbol: process.env.NEXT_PUBLIC_NATIVE_CURRENCY_SYMBOL || "ETH",
  },
  rpcUrls: {
    default: { http: [rpcUrl] },
    public: { http: [rpcUrl] },
  },
  blockExplorers: process.env.NEXT_PUBLIC_BLOCK_EXPLORER_URL ? {
    default: { 
      name: process.env.NEXT_PUBLIC_BLOCK_EXPLORER_NAME || "Explorer",
      url: process.env.NEXT_PUBLIC_BLOCK_EXPLORER_URL 
    }
  } : undefined,
});

// Get the active chain - use known chain if available, otherwise use custom
const activeChain = knownChains[chainId] || customChain;

// Create config with only the active chain
export const config = getDefaultConfig({
  appName: "AA Demo App",
  projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || "PROJECT_ID",
  chains: [activeChain],
  transports: {
    [activeChain.id]: http(rpcUrl),
  },
  ssr: true,
});