// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/core/EntryPoint.sol";
import "../src/accounts/OmniAccountFactory.sol";
import "../src/core/SimplePaymaster.sol";

/**
 * @title Deploy
 * @notice Universal deployment script for Account Abstraction contracts
 * @dev This script deploys EntryPoint, OmniAccountFactory, and SimplePaymaster contracts on any EVM network
 */
contract Deploy is Script {
    // Configuration - can be overridden via environment variables
    uint256 public paymasterInitialDeposit;
    bool public shouldInitializePaymaster;
    address public initialBundler;
    bool public saveDeploymentFile;

    // Deploy setup - we use a common salt for now
    address public deployer;
    bytes32 public salt;

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
        deployer = vm.addr(deployerPrivateKey);

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
        // set salt, manually change it if you want a different address
        salt = keccak256(abi.encodePacked("wildmeta v0"));

        // Load paymaster deposit amount (default: 1 ETH, can be 0 to skip initialization)
        paymasterInitialDeposit = vm.envOr("PAYMASTER_INITIAL_DEPOSIT", uint256(1 ether));
        shouldInitializePaymaster = vm.envOr("INITIALIZE_PAYMASTER", true);

        // Load initial bundler (default: deployer address)
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
        bytes memory initCode = abi.encodePacked(type(EntryPoint).creationCode);

        console.log("Checking if Entrypoint is deployed...");
        if (checkDeployed(initCode)) {
            return;
        }

        console.log("Deploying EntryPoint...");

        EntryPoint entryPoint = new EntryPoint{salt: salt}();
        entryPointAddress = address(entryPoint);

        console.log("EntryPoint deployed at:", entryPointAddress);
        console.log("");
    }

    function deployFactory() internal {
        bytes memory initCode = abi.encodePacked(type(EntryPoint).creationCode, IEntryPoint(entryPointAddress));

        console.log("Checking if OmniAccountFactory is deployed...");
        if (checkDeployed(initCode)) {
            return;
        }

        console.log("Deploying OmniAccountFactory...");

        OmniAccountFactory factory = new OmniAccountFactory{salt: salt}(IEntryPoint(entryPointAddress));
        factoryAddress = address(factory);

        console.log("OmniAccountFactory deployed at:", factoryAddress);
        console.log("EntryPoint reference:", entryPointAddress);
        console.log("");
    }

    function deployPaymaster() internal {
        bytes memory initCode =
            abi.encodePacked(type(EntryPoint).creationCode, IEntryPoint(entryPointAddress), initialBundler);

        console.log("Checking if SimplePaymaster is deployed...");
        if (checkDeployed(initCode)) {
            return;
        }

        console.log("Deploying SimplePaymaster...");

        SimplePaymaster paymaster = new SimplePaymaster{salt: salt}(IEntryPoint(entryPointAddress), initialBundler);
        paymasterAddress = address(paymaster);

        console.log("SimplePaymaster deployed at:", paymasterAddress);
        console.log("EntryPoint reference:", entryPointAddress);
        console.log("Initial bundler:", initialBundler);
        console.log("");
    }

    function checkDeployed(bytes memory initCode) internal view returns (bool) {
        // calculate expected address
        bytes32 hash = keccak256(abi.encodePacked(bytes1(0xff), deployer, salt, keccak256(initCode)));
        address expected = address(uint160(uint256(hash)));

        console.log("Expected address:", expected);

        // check if already deployed
        uint256 codeSize;
        assembly {
            codeSize := extcodesize(expected)
        }

        if (codeSize > 0) {
            console.log("Contract already deployed at:", expected);
            return true;
        } else {
            console.log("Contract not deployed at:", expected);
            return false;
        }
    }

    function initializePaymaster() internal {
        console.log("Initializing Paymaster with deposit...");

        SimplePaymaster paymaster = SimplePaymaster(payable(paymasterAddress));

        // Add stake and deposit for the paymaster - all initial funds go to stake for now
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
        console.log("EntryPoint:         ", entryPointAddress);
        console.log("OmniAccountFactory: ", factoryAddress);
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
        // Create deployment directory if it doesn't exist
        string memory deploymentDir = "deployments";

        // Ensure the deployments directory exists
        try vm.createDir(deploymentDir, false) {} catch {}

        // Create filename based on network
        string memory filename =
            string(abi.encodePacked(deploymentDir, "/", getNetworkFilename(networkConfig.chainId), ".json"));

        // Create JSON with deployment addresses
        string memory json = string(
            abi.encodePacked(
                "{\n",
                '  "network": "',
                networkConfig.name,
                '",\n',
                '  "chainId": ',
                vm.toString(networkConfig.chainId),
                ",\n",
                '  "timestamp": ',
                vm.toString(block.timestamp),
                ",\n",
                '  "deployer": "',
                vm.toString(msg.sender),
                '",\n',
                '  "contracts": {\n',
                '    "EntryPoint": "',
                vm.toString(entryPointAddress),
                '",\n',
                '    "OmniAccountFactory": "',
                vm.toString(factoryAddress),
                '",\n',
                '    "SimplePaymaster": "',
                vm.toString(paymasterAddress),
                '"\n',
                "  }\n",
                "}"
            )
        );

        // Write to deployment file (creates file if it doesn't exist)
        try vm.writeFile(filename, json) {
            console.log("Deployment addresses saved to:", filename);
        } catch Error(string memory reason) {
            console.log("Failed to save deployment file:", reason);
            console.log("Contract addresses (save manually if needed):");
            console.log("EntryPoint:         ", entryPointAddress);
            console.log("OmniAccountFactory: ", factoryAddress);
            console.log("SimplePaymaster:    ", paymasterAddress);
        } catch {
            console.log("Failed to save deployment file (unknown error)");
            console.log("Contract addresses (save manually if needed):");
            console.log("EntryPoint:         ", entryPointAddress);
            console.log("OmniAccountFactory: ", factoryAddress);
            console.log("SimplePaymaster:    ", paymasterAddress);
        }
    }

    function getNetworkFilename(uint256 chainId) internal pure returns (string memory) {
        if (chainId == 1) return "mainnet";
        if (chainId == 11155111) return "sepolia";
        if (chainId == 56) return "bsc";
        if (chainId == 97) return "bsc-testnet";
        if (chainId == 137) return "polygon";
        if (chainId == 80001) return "mumbai";
        if (chainId == 1337) return "local";
        if (chainId == 31337) return "local";
        return string(abi.encodePacked("chain-", vm.toString(chainId)));
    }
}
