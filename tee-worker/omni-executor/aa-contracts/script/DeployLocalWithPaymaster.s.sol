// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/core/EntryPointV1.sol";
import "../src/accounts/OmniAccountFactoryV1.sol";
import "../src/core/SimplePaymaster.sol";
import "../src/core/DemoPaymaster.sol";
import "../src/TestToken.sol";
import "./DeploymentHelper.sol";

contract DeployLocalWithPaymaster is Script {
    // Store deployed addresses for artifact generation
    address public entryPointAddress;
    address public factoryAddress;
    address public paymasterAddress;
    address public testUSDCAddress;
    address public testUSDTAddress;
    string public deployedPaymasterType;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address omniExecutorSigner = vm.envAddress("OMNI_EXECUTOR_SIGNER");

        // Check if we should save deployment file
        bool saveDeploymentFile = vm.envOr("SAVE_DEPLOYMENT_FILE", false);

        // Check for paymaster type environment variable
        string memory paymasterType = "simple"; // default
        try vm.envString("PAYMASTER_TYPE") returns (string memory envPaymasterType) {
            paymasterType = envPaymasterType;
        } catch {
            // Use default if not set
        }
        deployedPaymasterType = paymasterType;

        vm.startBroadcast(deployerPrivateKey);

        // Deploy EntryPointV1
        EntryPointV1 entryPoint = new EntryPointV1();
        entryPointAddress = address(entryPoint);
        console.log("EntryPointV1 deployed at:", entryPointAddress);

        // Deploy OmniAccountFactoryV1
        OmniAccountFactoryV1 factory = new OmniAccountFactoryV1(entryPoint);
        factoryAddress = address(factory);
        console.log("OmniAccountFactoryV1 deployed at:", factoryAddress);

        // Deploy appropriate paymaster based on environment variable
        if (keccak256(bytes(paymasterType)) == keccak256(bytes("demo"))) {
            // Deploy DemoPaymaster
            DemoPaymaster paymaster = new DemoPaymaster(entryPoint);
            paymasterAddress = address(paymaster);
            console.log("DemoPaymaster deployed at:", paymasterAddress);

            // Fund the paymaster with 0.1 ETH for demo purposes
            // Note: The paymaster's receive function will automatically deposit to EntryPointV1
            (bool success,) = paymasterAddress.call{value: 0.1 ether}("");
            require(success, "Failed to fund paymaster");
            console.log("DemoPaymaster funded with 0.1 ETH");
        } else {
            // Deploy SimplePaymaster (default)
            SimplePaymaster paymaster = new SimplePaymaster(entryPoint, omniExecutorSigner);
            paymasterAddress = address(paymaster);
            console.log("SimplePaymaster deployed at:", paymasterAddress);
        }

        // Deploy Test Tokens
        TestToken testUSDC = new TestToken("Test USDC", "USDC", 6);
        testUSDCAddress = address(testUSDC);
        console.log("Test USDC deployed at:", testUSDCAddress);

        TestToken testUSDT = new TestToken("Test USDT", "USDT", 6);
        testUSDTAddress = address(testUSDT);
        console.log("Test USDT deployed at:", testUSDTAddress);

        // Mint some tokens to the deployer for testing
        address deployer = vm.addr(deployerPrivateKey);
        testUSDC.mint(deployer, 1000000 * 10 ** 6); // 1M USDC
        testUSDT.mint(deployer, 1000000 * 10 ** 6); // 1M USDT
        console.log("Minted 1M tokens to deployer:", deployer);

        vm.stopBroadcast();

        // Save deployment artifacts if enabled
        if (saveDeploymentFile) {
            saveDeploymentArtifacts();
        } else {
            console.log("Deployment file saving disabled (set SAVE_DEPLOYMENT_FILE=true to enable)");
        }
    }

    function saveDeploymentArtifacts() internal {
        // Get deployment environment (default to "local" for local deployments)
        string memory environment = "local";
        try vm.envString("DEPLOYMENT_ENV") returns (string memory env) {
            environment = env;
        } catch {
            // Default to local
        }

        // Determine number of contracts based on paymaster type
        uint256 numContracts = keccak256(bytes(deployedPaymasterType)) == keccak256(bytes("demo")) ? 5 : 5;
        DeploymentHelper.ContractDeployment[] memory deployments =
            new DeploymentHelper.ContractDeployment[](numContracts);

        // Core contracts
        deployments[0] = DeploymentHelper.createContractDeployment(vm, "EntryPointV1", entryPointAddress, "");

        deployments[1] = DeploymentHelper.createContractDeployment(vm, "OmniAccountFactoryV1", factoryAddress, "");

        // Paymaster (with metadata about type and config)
        string memory paymasterName =
            keccak256(bytes(deployedPaymasterType)) == keccak256(bytes("demo")) ? "DemoPaymaster" : "SimplePaymaster";

        string memory paymasterMetadata = keccak256(bytes(deployedPaymasterType)) == keccak256(bytes("demo"))
            ? '{"type": "demo", "initialFunding": "0.1 ETH"}'
            : string(
                abi.encodePacked(
                    '{"type": "simple", "bundler": "', vm.toString(vm.envAddress("OMNI_EXECUTOR_SIGNER")), '"}'
                )
            );

        deployments[2] =
            DeploymentHelper.createContractDeployment(vm, paymasterName, paymasterAddress, paymasterMetadata);

        // Test tokens
        deployments[3] = DeploymentHelper.createContractDeployment(
            vm, "TestToken", testUSDCAddress, '{"symbol": "USDC", "decimals": 6, "initialMint": "1000000"}'
        );

        deployments[4] = DeploymentHelper.createContractDeployment(
            vm, "TestToken", testUSDTAddress, '{"symbol": "USDT", "decimals": 6, "initialMint": "1000000"}'
        );

        // Save artifacts with environment support
        DeploymentHelper.saveDeploymentArtifacts(
            vm, "deployments", environment, "Local Network", block.chainid, deployments
        );
    }
}
