import EntryPointArtifact from "@/contracts/abis/EntryPoint.json";
import OmniAccountArtifact from "@/contracts/abis/OmniAccount.json";
import OmniAccountFactoryArtifact from "@/contracts/abis/OmniAccountFactory.json";
import StandardERC20Artifact from "@/contracts/abis/StandardERC20.json";
import SimplePaymasterArtifact from "@/contracts/abis/SimplePaymaster.json";

// Extract ABIs from artifacts
const EntryPointABI = (EntryPointArtifact as any).abi || EntryPointArtifact;
const OmniAccountABI = (OmniAccountArtifact as any).abi || OmniAccountArtifact;
const OmniAccountFactoryABI = (OmniAccountFactoryArtifact as any).abi || OmniAccountFactoryArtifact;
const StandardERC20ABI = (StandardERC20Artifact as any).abi || StandardERC20Artifact;
const SimplePaymasterABI = (SimplePaymasterArtifact as any).abi || SimplePaymasterArtifact;

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
	OmniAccountFactory: {
		// Default: 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 (chainId 31337)
		// deploy-local.sh will output the actual address for chainId 1337
		address: (process.env.NEXT_PUBLIC_FACTORY_ADDRESS ||
			"0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512") as `0x${string}`,
		abi: OmniAccountFactoryABI,
	},
	OmniAccountImplementation: {
		// This is deployed by the factory and address may vary
		address: (process.env.NEXT_PUBLIC_OMNI_ACCOUNT_IMPL_ADDRESS ||
			"0xCafac3dD18aC6c6e92c921884f9E4176737C052c") as `0x${string}`,
		abi: OmniAccountABI,
	},
	SimplePaymaster: {
		// Optional - deployed by deploy-local.sh but not used in basic flow
		address: (process.env.NEXT_PUBLIC_PAYMASTER_ADDRESS ||
			"0x0000000000000000000000000000000000000000") as `0x${string}`,
		abi: SimplePaymasterABI,
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
export const DEFAULT_CLIENT_ID = "wildmeta";

// TEE Worker configuration
export const TEE_WORKER_CONFIG = {
	rpcUrl: process.env.NEXT_PUBLIC_TEE_WORKER_RPC_URL || "http://localhost:3000",
	clientId: "wildmeta",
	chainType: "Evm" as const,
	signerIndex: 0,
	// Use proxy to avoid CORS issues when running in browser
	useProxy: true,
};

// Solana configuration
export const SOLANA_CONFIG = {
	network: "devnet" as "devnet" | "testnet" | "mainnet-beta",
	rpcUrl: "https://api.devnet.solana.com",
};

// ERC20 Token configurations
export const ERC20_TOKENS = {
	USDC: {
		address: (process.env.NEXT_PUBLIC_TEST_USDC_ADDRESS ||
			"0x0000000000000000000000000000000000000000") as `0x${string}`,
		symbol: "USDC",
		name: "USD Coin",
		decimals: 6,
		abi: StandardERC20ABI,
	},
	USDT: {
		address: (process.env.NEXT_PUBLIC_TEST_USDT_ADDRESS ||
			"0x0000000000000000000000000000000000000000") as `0x${string}`,
		symbol: "USDT",
		name: "Tether USD",
		decimals: 6,
		abi: StandardERC20ABI,
	},
} as const;

// Token list for UI
export const SUPPORTED_TOKENS = [
	{
		symbol: "ETH",
		name: "Ethereum",
		decimals: 18,
		address: "0x0000000000000000000000000000000000000000" as `0x${string}`,
		isNative: true,
	},
	ERC20_TOKENS.USDC,
	ERC20_TOKENS.USDT,
] as const;

// Paymaster configuration
export const PAYMASTER_CONFIG = {
	// Default gas limits for paymaster operations
	defaultValidationGasLimit: BigInt(100000),
	defaultPostOpGasLimit: BigInt(50000),
	// Whether paymaster is enabled by default
	enabledByDefault: false,
} as const;
