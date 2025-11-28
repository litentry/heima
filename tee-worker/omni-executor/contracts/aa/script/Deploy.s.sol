// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/core/EntryPointV1.sol";
import "../src/accounts/OmniAccountFactoryV1.sol";
import "../src/core/SimplePaymaster.sol";
import "../src/core/ERC20PaymasterV1.sol";
import "./DeploymentHelper.sol";

/**
 * @title Deploy
 * @notice Universal deployment script for Account Abstraction contracts
 * @dev This script can deploy any combination of EntryPointV1, OmniAccountFactoryV1, SimplePaymaster, and ERC20PaymasterV1 contracts on any EVM network
 *      All contract deployments are configurable via environment variables, allowing for flexible deployment scenarios
 */
contract Deploy is Script {
    // Configuration - can be overridden via environment variables
    uint256 public paymasterInitialDeposit;
    bool public shouldDeployEntryPoint;
    bool public shouldDeployFactory;
    bool public shouldDeploySimplePaymaster;
    bool public shouldDeployERC20Paymaster;
    address public existingEntryPointAddress;
    address public initialBundler;
    bool public saveDeploymentFile;

    // Contract addresses will be stored here after deployment
    address public entryPointAddress;
    address public factoryAddress;
    address public paymasterAddress;
    address public erc20PaymasterAddress;

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

        // Validate dependencies
        validateDependencies();

        // Deploy contracts in dependency order with confirmation between each
        if (shouldDeployEntryPoint) {
            vm.startBroadcast(deployerPrivateKey);
            deployEntryPoint();
            vm.stopBroadcast();
            console.log("Waiting for EntryPoint deployment confirmation...");
            // Wait for transaction confirmation (forge script handles this automatically when --broadcast is used)
        }

        if (shouldDeployFactory) {
            vm.startBroadcast(deployerPrivateKey);
            deployFactory();
            vm.stopBroadcast();
            console.log("Waiting for Factory deployment confirmation...");
        }

        // Deploy Simple paymaster if configured
        if (shouldDeploySimplePaymaster) {
            vm.startBroadcast(deployerPrivateKey);
            deployPaymaster();
            vm.stopBroadcast();
            console.log("Waiting for SimplePaymaster deployment confirmation...");

            // Initialize simple paymaster if configured
            if (paymasterInitialDeposit > 0) {
                vm.startBroadcast(deployerPrivateKey);
                initializePaymaster();
                vm.stopBroadcast();
                console.log("Waiting for SimplePaymaster initialization confirmation...");
            }
        }

        // Deploy ERC20 paymaster if configured
        if (shouldDeployERC20Paymaster) {
            vm.startBroadcast(deployerPrivateKey);
            deployERC20Paymaster();
            vm.stopBroadcast();
            console.log("Waiting for ERC20Paymaster deployment confirmation...");

            // Initialize ERC20 paymaster if configured
            if (paymasterInitialDeposit > 0) {
                vm.startBroadcast(deployerPrivateKey);
                initializeERC20Paymaster();
                vm.stopBroadcast();
                console.log("Waiting for ERC20Paymaster initialization confirmation...");
            }
        }

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
        // Load deployment flags (defaults for backward compatibility)
        shouldDeployEntryPoint = vm.envOr("DEPLOY_ENTRYPOINT", true);
        shouldDeployFactory = vm.envOr("DEPLOY_FACTORY", true);
        shouldDeploySimplePaymaster = vm.envOr("DEPLOY_SIMPLE_PAYMASTER", true);
        shouldDeployERC20Paymaster = vm.envOr("DEPLOY_ERC20_PAYMASTER", false);

        // Load existing EntryPoint address if not deploying new one
        if (!shouldDeployEntryPoint) {
            existingEntryPointAddress = vm.envAddress("ENTRYPOINT_ADDRESS");
            require(existingEntryPointAddress != address(0), "ENTRYPOINT_ADDRESS required when DEPLOY_ENTRYPOINT=false");
            require(existingEntryPointAddress.code.length > 0, "ENTRYPOINT_ADDRESS must be a deployed contract");
            entryPointAddress = existingEntryPointAddress;
        }

        // Load paymaster deposit amount (default: 1 ETH, can be 0 to skip initialization)
        paymasterInitialDeposit = vm.envOr("PAYMASTER_INITIAL_DEPOSIT", uint256(1 ether));

        // Load initial bundler (default: deployer address)
        address deployer = msg.sender;
        initialBundler = vm.envOr("INITIAL_BUNDLER", deployer);

        // Load save deployment file flag (default: false for testing, true for production)
        saveDeploymentFile = vm.envOr("SAVE_DEPLOYMENT_FILE", false);

        console.log("Configuration:");
        console.log("- Deploy EntryPoint:", shouldDeployEntryPoint ? "Yes" : "No");
        if (!shouldDeployEntryPoint) {
            console.log("- Existing EntryPoint:", entryPointAddress);
        }
        console.log("- Deploy Factory:", shouldDeployFactory ? "Yes" : "No");
        console.log("- Deploy Simple paymaster:", shouldDeploySimplePaymaster ? "Yes" : "No");
        console.log("- Deploy ERC20 paymaster:", shouldDeployERC20Paymaster ? "Yes" : "No");
        console.log("- Paymaster initial deposit:", paymasterInitialDeposit / 1e18, "ETH");
        console.log("- Initial bundler:", initialBundler);
        console.log("- Save deployment file:", saveDeploymentFile ? "Yes" : "No");
        console.log("");
    }

    function validateDependencies() internal view {
        // Validate that EntryPoint is available for contracts that depend on it
        if (
            (shouldDeployFactory || shouldDeploySimplePaymaster || shouldDeployERC20Paymaster)
                && !shouldDeployEntryPoint
        ) {
            require(entryPointAddress != address(0), "EntryPoint address required for Factory/Paymaster deployment");
        }
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

    function deployERC20Paymaster() internal {
        console.log("Deploying ERC20PaymasterV1...");

        ERC20PaymasterV1 erc20Paymaster = new ERC20PaymasterV1(IEntryPoint(entryPointAddress), initialBundler);
        erc20PaymasterAddress = address(erc20Paymaster);

        console.log("ERC20PaymasterV1 deployed at:", erc20PaymasterAddress);
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

    function initializeERC20Paymaster() internal {
        console.log("Initializing ERC20 Paymaster with deposit...");

        ERC20PaymasterV1 erc20Paymaster = ERC20PaymasterV1(payable(erc20PaymasterAddress));

        // Add stake and deposit for the ERC20 paymaster - for now all goes in deposit
        uint256 stakeAmount = 0;
        uint256 depositAmount = paymasterInitialDeposit;

        if (stakeAmount > 0) {
            erc20Paymaster.addStake{value: stakeAmount}(1 days);
            console.log("Added stake:", stakeAmount / 1e18, "ETH");
        }

        if (depositAmount > 0) {
            erc20Paymaster.deposit{value: depositAmount}();
            console.log("Added deposit:", depositAmount / 1e18, "ETH");
        }

        console.log("ERC20 Paymaster initialized");
        console.log("");
    }

    function logDeploymentResults(NetworkConfig memory networkConfig) internal view {
        console.log("=== DEPLOYMENT COMPLETE ===");
        console.log("");
        console.log("Contract Addresses:");
        if (shouldDeployEntryPoint) {
            console.log("EntryPointV1:         ", entryPointAddress);
        } else {
            console.log("EntryPointV1 (existing):", entryPointAddress);
        }
        if (shouldDeployFactory) {
            console.log("OmniAccountFactoryV1: ", factoryAddress);
        }
        if (shouldDeploySimplePaymaster) {
            console.log("SimplePaymaster:      ", paymasterAddress);
        }
        if (shouldDeployERC20Paymaster) {
            console.log("ERC20PaymasterV1:     ", erc20PaymasterAddress);
        }
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
        uint256 deploymentCount = 0;
        if (shouldDeployEntryPoint) deploymentCount++;
        if (shouldDeployFactory) deploymentCount++;
        if (shouldDeploySimplePaymaster) deploymentCount++;
        if (shouldDeployERC20Paymaster) deploymentCount++;

        require(deploymentCount > 0, "No contracts were deployed");
        DeploymentHelper.ContractDeployment[] memory deployments =
            new DeploymentHelper.ContractDeployment[](deploymentCount);

        uint256 currentIndex = 0;

        // Add EntryPoint deployment with ABI and bytecode if deployed
        if (shouldDeployEntryPoint) {
            deployments[currentIndex] = DeploymentHelper.createContractDeployment(
                vm,
                "EntryPointV1",
                entryPointAddress,
                "" // No additional metadata for now
            );
            currentIndex++;
        }

        // Add OmniAccountFactory deployment with ABI and bytecode if deployed
        if (shouldDeployFactory) {
            deployments[currentIndex] = DeploymentHelper.createContractDeployment(
                vm,
                "OmniAccountFactoryV1",
                factoryAddress,
                string(abi.encodePacked('{"entryPoint": "', vm.toString(entryPointAddress), '"}'))
            );
            currentIndex++;
        }

        // Add SimplePaymaster deployment with ABI and bytecode if deployed
        if (shouldDeploySimplePaymaster) {
            deployments[currentIndex] = DeploymentHelper.createContractDeployment(
                vm,
                "SimplePaymaster",
                paymasterAddress,
                string(
                    abi.encodePacked(
                        '{"initialBundler": "',
                        vm.toString(initialBundler),
                        '", "entryPoint": "',
                        vm.toString(entryPointAddress),
                        '"}'
                    )
                )
            );
            currentIndex++;
        }

        // Add ERC20PaymasterV1 deployment if deployed
        if (shouldDeployERC20Paymaster) {
            deployments[currentIndex] = DeploymentHelper.createContractDeployment(
                vm,
                "ERC20PaymasterV1",
                erc20PaymasterAddress,
                string(
                    abi.encodePacked(
                        '{"initialBundler": "',
                        vm.toString(initialBundler),
                        '", "entryPoint": "',
                        vm.toString(entryPointAddress),
                        '"}'
                    )
                )
            );
        }

        // Save enhanced deployment artifacts with environment support
        DeploymentHelper.saveDeploymentArtifacts(
            vm, "deployments", environment, networkConfig.name, networkConfig.chainId, deployments
        );
    }
}
