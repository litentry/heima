// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/core/EntryPointV1.sol";
import "../src/accounts/OmniAccountFactoryV1.sol";
import "../src/core/SimplePaymaster.sol";
import "../src/core/ERC20PaymasterV1.sol";
import "../src/core/Create2FactoryV1.sol";
import "./DeploymentHelper.sol";

/**
 * @title DeployContract
 * @notice Deployment script for deploying individual AA contracts using CREATE2
 * @dev This script deploys a single contract specified by the CONTRACT_NAME environment variable
 *
 * Prerequisites:
 *   1. Create2FactoryV1 must be deployed on the target network
 *   2. Factory address must be provided via CREATE2_FACTORY_ADDRESS environment variable
 *
 * Usage:
 *   CONTRACT_NAME=EntryPointV1 forge script script/DeployContract.s.sol:DeployContract --rpc-url <RPC_URL> --broadcast
 *   CONTRACT_NAME=OmniAccountFactoryV1 ENTRYPOINT_ADDRESS=0x... forge script script/DeployContract.s.sol:DeployContract --rpc-url <RPC_URL> --broadcast
 *
 * Environment Variables:
 *   CONTRACT_NAME - Name of the contract to deploy (required)
 *     Valid values: EntryPointV1, OmniAccountFactoryV1, SimplePaymaster, ERC20PaymasterV1
 *   CREATE2_FACTORY_ADDRESS - Address of the deployed Create2FactoryV1 (required)
 *   ENTRYPOINT_ADDRESS - Address of deployed EntryPoint (required for Factory/Paymaster contracts)
 *   INITIAL_BUNDLER - Address of initial bundler (optional, defaults to deployer)
 *   PAYMASTER_INITIAL_DEPOSIT - Initial deposit for paymaster in wei (optional, default: 1 ETH)
 *   SAVE_DEPLOYMENT_FILE - Save deployment artifacts to file (optional, default: false)
 *   DEPLOYMENT_ENV - Environment subdirectory for deployment files (optional)
 */
contract DeployContract is Script {
    // CREATE2 configuration
    Create2FactoryV1 public factory;
    string public contractName;

    // Configuration
    address public entryPointAddress;
    address public initialBundler;
    uint256 public paymasterInitialDeposit;
    bool public saveDeploymentFile;

    // Deployed contract address
    address public deployedAddress;

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

        // Load CREATE2 factory
        address factoryAddr = vm.envAddress("CREATE2_FACTORY_ADDRESS");
        require(factoryAddr != address(0), "CREATE2_FACTORY_ADDRESS not set");
        require(factoryAddr.code.length > 0, "CREATE2_FACTORY_ADDRESS is not a contract");
        factory = Create2FactoryV1(factoryAddr);

        // Load contract name
        contractName = vm.envString("CONTRACT_NAME");
        require(bytes(contractName).length > 0, "CONTRACT_NAME not set");

        // Load configuration
        loadConfiguration(deployer);

        // Get network info
        NetworkConfig memory networkConfig = getNetworkConfig();

        // Log deployment info
        console.log("=== Single Contract Deployment (CREATE2) ===");
        console.log("Contract:", contractName);
        console.log("Network:", networkConfig.name);
        console.log("Chain ID:", networkConfig.chainId);
        console.log("Deployer address:", deployer);
        console.log("Deployer balance:", deployer.balance / 1e18, "ETH");
        console.log("Block number:", block.number);
        console.log("Create2FactoryV1:", address(factory));
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

        // Validate dependencies
        validateDependencies();

        // Predict address
        predictAddress();

        // Deploy contract
        vm.startBroadcast(deployerPrivateKey);
        deployContract();
        vm.stopBroadcast();

        // Log deployment results
        logDeploymentResults(networkConfig);

        // Save deployment addresses to file (if enabled)
        if (saveDeploymentFile) {
            saveDeploymentAddress(networkConfig);
        } else {
            console.log("Deployment file saving disabled (set SAVE_DEPLOYMENT_FILE=true to enable)");
        }
    }

    function loadConfiguration(address deployer) internal {
        // Load EntryPoint address if required
        if (
            keccak256(bytes(contractName)) == keccak256("OmniAccountFactoryV1")
                || keccak256(bytes(contractName)) == keccak256("SimplePaymaster")
                || keccak256(bytes(contractName)) == keccak256("ERC20PaymasterV1")
        ) {
            entryPointAddress = vm.envAddress("ENTRYPOINT_ADDRESS");
            require(entryPointAddress != address(0), "ENTRYPOINT_ADDRESS required for this contract");
            require(entryPointAddress.code.length > 0, "ENTRYPOINT_ADDRESS must be a deployed contract");
        }

        // Load paymaster configuration
        if (
            keccak256(bytes(contractName)) == keccak256("SimplePaymaster")
                || keccak256(bytes(contractName)) == keccak256("ERC20PaymasterV1")
        ) {
            initialBundler = vm.envOr("INITIAL_BUNDLER", deployer);
            paymasterInitialDeposit = vm.envOr("PAYMASTER_INITIAL_DEPOSIT", uint256(1 ether));
        }

        // Load save deployment file flag
        saveDeploymentFile = vm.envOr("SAVE_DEPLOYMENT_FILE", false);

        console.log("Configuration:");
        console.log("- Contract:", contractName);
        if (entryPointAddress != address(0)) {
            console.log("- EntryPoint:", entryPointAddress);
        }
        if (initialBundler != address(0)) {
            console.log("- Initial bundler:", initialBundler);
            console.log("- Paymaster initial deposit:", paymasterInitialDeposit / 1e18, "ETH");
        }
        console.log("- Save deployment file:", saveDeploymentFile ? "Yes" : "No");
        console.log("");
    }

    function validateDependencies() internal view {
        // Validate contract name
        require(
            keccak256(bytes(contractName)) == keccak256("EntryPointV1")
                || keccak256(bytes(contractName)) == keccak256("OmniAccountFactoryV1")
                || keccak256(bytes(contractName)) == keccak256("SimplePaymaster")
                || keccak256(bytes(contractName)) == keccak256("ERC20PaymasterV1"),
            string(abi.encodePacked("Unknown contract name: ", contractName))
        );
    }

    function predictAddress() internal view {
        console.log("=== Predicted Address ===");
        bytes32 salt = factory.generateSalt(contractName);

        if (keccak256(bytes(contractName)) == keccak256("EntryPointV1")) {
            address predicted = factory.computeAddress(salt, abi.encodePacked(type(EntryPointV1).creationCode));
            console.log(string(abi.encodePacked(contractName, " (predicted):")), predicted);
        } else if (keccak256(bytes(contractName)) == keccak256("OmniAccountFactoryV1")) {
            address predicted = factory.computeAddress(
                salt,
                abi.encodePacked(type(OmniAccountFactoryV1).creationCode, abi.encode(IEntryPoint(entryPointAddress)))
            );
            console.log(string(abi.encodePacked(contractName, " (predicted):")), predicted);
        } else if (keccak256(bytes(contractName)) == keccak256("SimplePaymaster")) {
            address predicted = factory.computeAddress(
                salt,
                abi.encodePacked(
                    type(SimplePaymaster).creationCode, abi.encode(IEntryPoint(entryPointAddress), initialBundler)
                )
            );
            console.log(string(abi.encodePacked(contractName, " (predicted):")), predicted);
        } else if (keccak256(bytes(contractName)) == keccak256("ERC20PaymasterV1")) {
            address predicted = factory.computeAddress(
                salt,
                abi.encodePacked(
                    type(ERC20PaymasterV1).creationCode, abi.encode(IEntryPoint(entryPointAddress), initialBundler)
                )
            );
            console.log(string(abi.encodePacked(contractName, " (predicted):")), predicted);
        }
        console.log("");
    }

    function deployContract() internal {
        console.log(string(abi.encodePacked("Deploying ", contractName, " via CREATE2...")));

        bytes32 salt = factory.generateSalt(contractName);
        bytes memory bytecode;

        if (keccak256(bytes(contractName)) == keccak256("EntryPointV1")) {
            bytecode = abi.encodePacked(type(EntryPointV1).creationCode);
        } else if (keccak256(bytes(contractName)) == keccak256("OmniAccountFactoryV1")) {
            bytecode =
                abi.encodePacked(type(OmniAccountFactoryV1).creationCode, abi.encode(IEntryPoint(entryPointAddress)));
        } else if (keccak256(bytes(contractName)) == keccak256("SimplePaymaster")) {
            bytecode = abi.encodePacked(
                type(SimplePaymaster).creationCode, abi.encode(IEntryPoint(entryPointAddress), initialBundler)
            );
        } else if (keccak256(bytes(contractName)) == keccak256("ERC20PaymasterV1")) {
            bytecode = abi.encodePacked(
                type(ERC20PaymasterV1).creationCode, abi.encode(IEntryPoint(entryPointAddress), initialBundler)
            );
        }

        deployedAddress = factory.deploy(salt, bytecode);

        console.log(string(abi.encodePacked(contractName, " deployed at:")), deployedAddress);
        if (entryPointAddress != address(0)) {
            console.log("EntryPoint reference:", entryPointAddress);
        }
        if (initialBundler != address(0)) {
            console.log("Initial bundler:", initialBundler);
        }
        console.log("");

        // Initialize paymaster if configured
        if (
            (
                keccak256(bytes(contractName)) == keccak256("SimplePaymaster")
                    || keccak256(bytes(contractName)) == keccak256("ERC20PaymasterV1")
            ) && paymasterInitialDeposit > 0
        ) {
            initializePaymaster();
        }
    }

    function initializePaymaster() internal {
        console.log("Initializing Paymaster with deposit...");

        if (keccak256(bytes(contractName)) == keccak256("SimplePaymaster")) {
            SimplePaymaster paymaster = SimplePaymaster(payable(deployedAddress));
            paymaster.deposit{value: paymasterInitialDeposit}();
            console.log("Added deposit:", paymasterInitialDeposit / 1e18, "ETH");
        } else if (keccak256(bytes(contractName)) == keccak256("ERC20PaymasterV1")) {
            ERC20PaymasterV1 paymaster = ERC20PaymasterV1(payable(deployedAddress));
            paymaster.deposit{value: paymasterInitialDeposit}();
            console.log("Added deposit:", paymasterInitialDeposit / 1e18, "ETH");
        }

        console.log("Paymaster initialized");
        console.log("");
    }

    function getNetworkConfig() internal view returns (NetworkConfig memory) {
        uint256 chainId = block.chainid;

        if (chainId == 1) {
            return NetworkConfig("Ethereum Mainnet", 1, 0.01 ether);
        } else if (chainId == 11155111) {
            return NetworkConfig("Ethereum Sepolia", 11155111, 0.01 ether);
        } else if (chainId == 56) {
            return NetworkConfig("BSC Mainnet", 56, 0.01 ether);
        } else if (chainId == 97) {
            return NetworkConfig("BSC Testnet", 97, 0.05 ether);
        } else if (chainId == 8453) {
            return NetworkConfig("Base", 8453, 0.01 ether);
        } else if (chainId == 84532) {
            return NetworkConfig("Base Sepolia", 84532, 0.05 ether);
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

    function logDeploymentResults(NetworkConfig memory networkConfig) internal view {
        console.log("=== DEPLOYMENT COMPLETE (CREATE2) ===");
        console.log("");
        console.log("Contract:", contractName);
        console.log("Address:", deployedAddress);
        console.log("");
        console.log("Network:", networkConfig.name);
        console.log("Chain ID:", networkConfig.chainId);
        console.log("Deployment Date:", block.timestamp);
        console.log("Create2FactoryV1:", address(factory));
        console.log("");
        console.log("IMPORTANT: This address is deterministic across all chains!");
        console.log("The same contract name will always yield the same address.");
        console.log("");
    }

    function saveDeploymentAddress(NetworkConfig memory networkConfig) internal {
        string memory environment = "";
        try vm.envString("DEPLOYMENT_ENV") returns (string memory env) {
            environment = env;
        } catch {}

        DeploymentHelper.ContractDeployment[] memory deployments = new DeploymentHelper.ContractDeployment[](1);

        string memory metadata;
        if (entryPointAddress != address(0) && initialBundler != address(0)) {
            metadata = string(
                abi.encodePacked(
                    '{"initialBundler": "',
                    vm.toString(initialBundler),
                    '", "entryPoint": "',
                    vm.toString(entryPointAddress),
                    '", "deploymentMethod": "CREATE2"}'
                )
            );
        } else if (entryPointAddress != address(0)) {
            metadata = string(
                abi.encodePacked(
                    '{"entryPoint": "', vm.toString(entryPointAddress), '", "deploymentMethod": "CREATE2"}'
                )
            );
        } else {
            metadata = '{"deploymentMethod": "CREATE2"}';
        }

        deployments[0] = DeploymentHelper.createContractDeployment(vm, contractName, deployedAddress, metadata);

        DeploymentHelper.saveDeploymentArtifacts(
            vm, "deployments", environment, networkConfig.name, networkConfig.chainId, deployments
        );
    }
}
