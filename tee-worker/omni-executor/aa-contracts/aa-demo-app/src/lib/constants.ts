import EntryPointABI from "@/contracts/abis/EntryPoint.json";
import SmartAccountABI from "@/contracts/abis/SmartAccount.json";
import SmartAccountFactoryABI from "@/contracts/abis/SmartAccountFactory.json";

// Contract addresses - update these after running deploy-local.sh
// Default addresses are for chainId 31337 (Hardhat)
// For chainId 1337 (Anvil via deploy-local.sh), update accordingly
export const CONTRACTS = {
	EntryPoint: {
		// Default: 0x5FbDB2315678afecb367f032d93F642f64180aa3 (chainId 31337)
		// deploy-local.sh will output the actual address for chainId 1337
		address: (process.env.NEXT_PUBLIC_ENTRYPOINT_ADDRESS ||
			"0x5FbDB2315678afecb367f032d93F642f64180aa3") as `0x${string}`,
		abi: EntryPointABI,
	},
	SmartAccountFactory: {
		// Default: 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 (chainId 31337)
		// deploy-local.sh will output the actual address for chainId 1337
		address: (process.env.NEXT_PUBLIC_FACTORY_ADDRESS ||
			"0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512") as `0x${string}`,
		abi: SmartAccountFactoryABI,
	},
	SmartAccountImplementation: {
		// This is deployed by the factory and address may vary
		address: (process.env.NEXT_PUBLIC_SMART_ACCOUNT_IMPL_ADDRESS ||
			"0xCafac3dD18aC6c6e92c921884f9E4176737C052c") as `0x${string}`,
		abi: SmartAccountABI,
	},
	SimplePaymaster: {
		// Optional - deployed by deploy-local.sh but not used in basic flow
		address: (process.env.NEXT_PUBLIC_PAYMASTER_ADDRESS ||
			"0x0000000000000000000000000000000000000000") as `0x${string}`,
		abi: [] as any, // Add ABI when needed
	},
} as const;

// Chain configurations
export const SUPPORTED_CHAINS = {
	// Local development - Anvil (deploy-local.sh)
	anvil: {
		id: 1337,
		name: "Anvil Local",
		rpcUrl: "http://localhost:8545",
	},
	// Testnets
	sepolia: {
		id: 11155111,
		name: "Sepolia",
		rpcUrl:
			process.env.NEXT_PUBLIC_SEPOLIA_RPC_URL ||
			"https://sepolia.infura.io/v3/YOUR_INFURA_KEY",
	},
	// Add more chains as needed
} as const;

// Get the active chain based on environment
export const getActiveChain = () => {
	const chainId = process.env.NEXT_PUBLIC_CHAIN_ID;
	if (chainId === "1337") return SUPPORTED_CHAINS.anvil;
	if (chainId === "11155111") return SUPPORTED_CHAINS.sepolia;
	return SUPPORTED_CHAINS.anvil;
};

// Default client ID for demo
export const DEFAULT_CLIENT_ID = "heima";

// Solana configuration
export const SOLANA_CONFIG = {
	network: "devnet" as "devnet" | "testnet" | "mainnet-beta",
	rpcUrl: "https://api.devnet.solana.com",
};
