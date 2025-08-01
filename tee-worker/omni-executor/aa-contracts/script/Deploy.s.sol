// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/core/EntryPointV1.sol";
import "../src/accounts/OmniAccountFactoryV1.sol";
import "../src/core/SimplePaymaster.sol";
import "./DeploymentHelper.sol";

/**
 * @title Deploy
 * @notice Universal deployment script for Account Abstraction contracts
 * @dev This script deploys EntryPointV1, OmniAccountFactoryV1, and SimplePaymaster contracts on any EVM network
 */
contract Deploy is Script {
    // Configuration - can be overridden via environment variables
    uint256 public paymasterInitialDeposit;
    bool public shouldInitializePaymaster;
    address public initialBundler;
    bool public saveDeploymentFile;

    // Contract addresses will be stored here after deployment
    address public entryPointAddress;
    address public factoryAddress;
    address public paymasterAddress;

    // Network configuration
    struct NetworkConfig {
        string name;
        uint256 chainId;
        uint256 minDeploymentBalance;
    }

    function run() external {
        // Get deployment parameters
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        // Load configuration
        loadConfiguration();

        // Get network info
        NetworkConfig memory networkConfig = getNetworkConfig();

        // Log deployment info
        console.log("=== AA Contracts Deployment ===");
        console.log("Network:", networkConfig.name);
        console.log("Chain ID:", networkConfig.chainId);
        console.log("Deployer address:", deployer);
        console.log("Deployer balance:", deployer.balance / 1e18, "ETH");
        console.log("Block number:", block.number);
        console.log("");

        // Check minimum deployment balance
        require(
            deployer.balance >= networkConfig.minDeploymentBalance,
            string(
                abi.encodePacked(
                    "Insufficient balance for deployment (need at least ",
                    vm.toString(networkConfig.minDeploymentBalance / 1e18),
                    " ETH)"
                )
            )
        );

        // Start broadcasting transactions
        vm.startBroadcast(deployerPrivateKey);

        // Deploy contracts in dependency order
        deployEntryPoint();
        deployFactory();
        deployPaymaster();

        // Initialize paymaster if configured
        if (shouldInitializePaymaster && paymasterInitialDeposit > 0) {
            initializePaymaster();
        }

        vm.stopBroadcast();

        // Log final deployment results
        logDeploymentResults(networkConfig);

        // Save deployment addresses to file (if enabled)
        if (saveDeploymentFile) {
            saveDeploymentAddresses(networkConfig);
        } else {
            console.log("Deployment file saving disabled (set SAVE_DEPLOYMENT_FILE=true to enable)");
        }
    }

    function loadConfiguration() internal {
        // Load paymaster deposit amount (default: 1 ETH, can be 0 to skip initialization)
        paymasterInitialDeposit = vm.envOr("PAYMASTER_INITIAL_DEPOSIT", uint256(1 ether));
        shouldInitializePaymaster = vm.envOr("INITIALIZE_PAYMASTER", true);

        // Load initial bundler (default: deployer address)
        address deployer = msg.sender;
        initialBundler = vm.envOr("INITIAL_BUNDLER", deployer);

        // Load save deployment file flag (default: false for testing, true for production)
        saveDeploymentFile = vm.envOr("SAVE_DEPLOYMENT_FILE", false);

        console.log("Configuration:");
        console.log("- Paymaster initial deposit:", paymasterInitialDeposit / 1e18, "ETH");
        console.log("- Initialize paymaster:", shouldInitializePaymaster ? "Yes" : "No");
        console.log("- Initial bundler:", initialBundler);
        console.log("- Save deployment file:", saveDeploymentFile ? "Yes" : "No");
        console.log("");
    }

    function getNetworkConfig() internal view returns (NetworkConfig memory) {
        uint256 chainId = block.chainid;

        if (chainId == 1) {
            return NetworkConfig("Ethereum Mainnet", 1, 0.5 ether);
        } else if (chainId == 11155111) {
            return NetworkConfig("Ethereum Sepolia", 11155111, 0.1 ether);
        } else if (chainId == 56) {
            return NetworkConfig("BSC Mainnet", 56, 0.1 ether);
        } else if (chainId == 97) {
            return NetworkConfig("BSC Testnet", 97, 0.05 ether);
        } else if (chainId == 137) {
            return NetworkConfig("Polygon Mainnet", 137, 1 ether);
        } else if (chainId == 80001) {
            return NetworkConfig("Polygon Mumbai", 80001, 0.1 ether);
        } else if (chainId == 42161) {
            return NetworkConfig("Arbitrum Mainnet", 42161, 0.01 ether);
        } else if (chainId == 421614) {
            return NetworkConfig("Arbitrum Sepolia", 421614, 0.01 ether);
        } else if (chainId == 999) {
            return NetworkConfig("HyperEVM Mainnet", 999, 0.01 ether);
        } else if (chainId == 998) {
            return NetworkConfig("HyperEVM Testnet", 998, 0.01 ether);
        } else if (chainId == 1337) {
            return NetworkConfig("Local Anvil", 1337, 0.01 ether);
        } else if (chainId == 31337) {
            return NetworkConfig("Local Anvil", 31337, 0.01 ether);
        } else {
            return NetworkConfig(
                string(abi.encodePacked("Unknown Network (", vm.toString(chainId), ")")), chainId, 0.01 ether
            );
        }
    }

    function deployEntryPoint() internal {
        console.log("Deploying EntryPointV1...");

        EntryPointV1 entryPoint = new EntryPointV1();
        entryPointAddress = address(entryPoint);

        console.log("EntryPointV1 deployed at:", entryPointAddress);
        console.log("");
    }

    function deployFactory() internal {
        console.log("Deploying OmniAccountFactoryV1...");

        OmniAccountFactoryV1 factory = new OmniAccountFactoryV1(IEntryPoint(entryPointAddress));
        factoryAddress = address(factory);

        console.log("OmniAccountFactoryV1 deployed at:", factoryAddress);
        console.log("EntryPoint reference:", entryPointAddress);
        console.log("");
    }

    function deployPaymaster() internal {
        console.log("Deploying SimplePaymaster...");

        SimplePaymaster paymaster = new SimplePaymaster(IEntryPoint(entryPointAddress), initialBundler);
        paymasterAddress = address(paymaster);

        console.log("SimplePaymaster deployed at:", paymasterAddress);
        console.log("EntryPoint reference:", entryPointAddress);
        console.log("Initial bundler:", initialBundler);
        console.log("");
    }

    function initializePaymaster() internal {
        console.log("Initializing Paymaster with deposit...");

        SimplePaymaster paymaster = SimplePaymaster(payable(paymasterAddress));

        // Add stake and deposit for the paymaster - for now all goes in deposit
        uint256 stakeAmount = 0;
        uint256 depositAmount = paymasterInitialDeposit;

        if (stakeAmount > 0) {
            paymaster.addStake{value: stakeAmount}(1 days);
            console.log("Added stake:", stakeAmount / 1e18, "ETH");
        }

        if (depositAmount > 0) {
            paymaster.deposit{value: depositAmount}();
            console.log("Added deposit:", depositAmount / 1e18, "ETH");
        }

        console.log("Paymaster initialized");
        console.log("");
    }

    function logDeploymentResults(NetworkConfig memory networkConfig) internal view {
        console.log("=== DEPLOYMENT COMPLETE ===");
        console.log("");
        console.log("Contract Addresses:");
        console.log("EntryPointV1:       ", entryPointAddress);
        console.log("OmniAccountFactoryV1: ", factoryAddress);
        console.log("SimplePaymaster:    ", paymasterAddress);
        console.log("");
        console.log("Network:", networkConfig.name);
        console.log("Chain ID:", networkConfig.chainId);
        console.log("Deployment Date:", block.timestamp);
        console.log("");
        console.log("IMPORTANT: Save these addresses securely!");
        console.log("Verify contracts on block explorer using the --verify flag");
        console.log("");
    }

    function saveDeploymentAddresses(NetworkConfig memory networkConfig) internal {
        // Get deployment environment (default to empty string for backward compatibility)
        string memory environment = "";
        try vm.envString("DEPLOYMENT_ENV") returns (string memory env) {
            environment = env;
        } catch {
            // No environment specified, use flat structure
        }

        // Create array of deployments with enhanced artifact data
        DeploymentHelper.ContractDeployment[] memory deployments = new DeploymentHelper.ContractDeployment[](3);

        // Add EntryPoint deployment with ABI and bytecode
        deployments[0] = DeploymentHelper.createContractDeployment(
            vm,
            "EntryPointV1",
            entryPointAddress,
            "" // No additional metadata for now
        );

        // Add OmniAccountFactory deployment with ABI and bytecode
        deployments[1] = DeploymentHelper.createContractDeployment(
            vm,
            "OmniAccountFactoryV1",
            factoryAddress,
            "" // No additional metadata for now
        );

        // Add SimplePaymaster deployment with ABI and bytecode
        deployments[2] = DeploymentHelper.createContractDeployment(
            vm,
            "SimplePaymaster",
            paymasterAddress,
            string(abi.encodePacked('{"initialBundler": "', vm.toString(initialBundler), '"}'))
        );

        // Save enhanced deployment artifacts with environment support
        DeploymentHelper.saveDeploymentArtifacts(
            vm, "deployments", environment, networkConfig.name, networkConfig.chainId, deployments
        );
    }
}
