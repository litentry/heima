// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/core/Create2Factory.sol";

/**
 * @title DeployCreate2Factory
 * @notice Deployment script for the Create2Factory contract
 * @dev This script should be run with a FRESH EOA that has not deployed contracts before
 *      The factory address will be deterministic based on the deployer's address and nonce
 *
 * Usage:
 *   forge script script/DeployCreate2Factory.s.sol:DeployCreate2Factory --rpc-url <RPC_URL> --broadcast --verify
 *
 * Environment Variables:
 *   PRIVATE_KEY - Private key of the deployer (should be a fresh EOA)
 *   RPC_URL - RPC endpoint for the target network
 *   ETHERSCAN_API_KEY - API key for contract verification (optional)
 */
contract DeployCreate2Factory is Script {
    // Network configuration
    struct NetworkConfig {
        string name;
        uint256 chainId;
    }

    function run() external {
        // Get deployment parameters
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        // Get network info
        NetworkConfig memory networkConfig = getNetworkConfig();

        // Log deployment info
        console.log("=== Create2Factory Deployment ===");
        console.log("Network:", networkConfig.name);
        console.log("Chain ID:", networkConfig.chainId);
        console.log("Deployer address:", deployer);
        console.log("Deployer balance:", deployer.balance / 1e18, "ETH");
        console.log("Deployer nonce:", vm.getNonce(deployer));
        console.log("");

        // Warning about fresh EOA requirement
        if (vm.getNonce(deployer) > 0) {
            console.log("WARNING: Deployer has non-zero nonce!");
            console.log(
                "For consistent factory addresses across chains, it's recommended to use a fresh EOA (nonce 0)"
            );
            console.log("Current nonce:", vm.getNonce(deployer));
            console.log("");
        }

        // Predict factory address
        address predictedFactory = vm.computeCreateAddress(deployer, vm.getNonce(deployer));
        console.log("Predicted factory address:", predictedFactory);
        console.log("");

        // Deploy factory
        console.log("Deploying Create2Factory...");
        vm.startBroadcast(deployerPrivateKey);

        Create2Factory factory = new Create2Factory();

        vm.stopBroadcast();

        // Verify deployment
        require(address(factory) == predictedFactory, "Factory address mismatch!");
        console.log("Create2Factory deployed at:", address(factory));
        console.log("");

        // Print summary
        console.log("=== Deployment Summary ===");
        console.log("Network:", networkConfig.name);
        console.log("Chain ID:", networkConfig.chainId);
        console.log("Factory Address:", address(factory));
        console.log("Deployer:", deployer);
        console.log("Final Nonce:", vm.getNonce(deployer));
        console.log("");
        console.log("IMPORTANT: Save this factory address for future deployments!");
        console.log("Add it to: deployments/create2-factories.json");
        console.log("");

        // Save deployment info if requested
        saveDeploymentInfo(networkConfig, address(factory), deployer);
    }

    function getNetworkConfig() internal view returns (NetworkConfig memory) {
        uint256 chainId = block.chainid;
        string memory name = getNetworkName(chainId);
        return NetworkConfig({name: name, chainId: chainId});
    }

    function getNetworkName(uint256 chainId) internal pure returns (string memory) {
        if (chainId == 1) return "mainnet";
        if (chainId == 11155111) return "sepolia";
        if (chainId == 42161) return "arbitrum";
        if (chainId == 421614) return "arbitrum-sepolia";
        if (chainId == 10) return "optimism";
        if (chainId == 11155420) return "optimism-sepolia";
        if (chainId == 8453) return "base";
        if (chainId == 84532) return "base-sepolia";
        if (chainId == 137) return "polygon";
        if (chainId == 80002) return "polygon-amoy";
        if (chainId == 56) return "bsc";
        if (chainId == 97) return "bsc-testnet";
        if (chainId == 43114) return "avalanche";
        if (chainId == 43113) return "avalanche-fuji";
        if (chainId == 250) return "fantom";
        if (chainId == 100) return "gnosis";
        if (chainId == 1284) return "moonbeam";
        if (chainId == 1285) return "moonriver";
        if (chainId == 42220) return "celo";
        if (chainId == 1313161554) return "aurora";
        if (chainId == 25) return "cronos";
        if (chainId == 2001) return "hyperspace";
        if (chainId == 1337) return "localhost";
        if (chainId == 31337) return "anvil";
        return string(abi.encodePacked("unknown-", vm.toString(chainId)));
    }

    function saveDeploymentInfo(NetworkConfig memory config, address factory, address deployer) internal {
        bool shouldSave = vm.envOr("SAVE_DEPLOYMENT_FILE", true);
        if (!shouldSave) {
            console.log("Skipping deployment file save (SAVE_DEPLOYMENT_FILE=false)");
            return;
        }

        string memory deploymentEnv = vm.envOr("DEPLOYMENT_ENV", string(""));
        string memory basePath = "deployments";

        if (bytes(deploymentEnv).length > 0) {
            basePath = string(abi.encodePacked(basePath, "/", deploymentEnv));
        }

        string memory filePath = string(abi.encodePacked(basePath, "/", config.name, "-create2-factory.json"));

        // Create JSON object
        string memory json = "deployment";
        vm.serializeString(json, "network", config.name);
        vm.serializeUint(json, "chainId", config.chainId);
        vm.serializeAddress(json, "factoryAddress", factory);
        vm.serializeAddress(json, "deployer", deployer);
        vm.serializeUint(json, "blockNumber", block.number);
        string memory finalJson = vm.serializeUint(json, "timestamp", block.timestamp);

        // Write to file
        vm.writeJson(finalJson, filePath);
        console.log("Deployment info saved to:", filePath);
    }
}
