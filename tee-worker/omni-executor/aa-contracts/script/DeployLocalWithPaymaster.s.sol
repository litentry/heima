// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/core/EntryPointV1.sol";
import "../src/accounts/OmniAccountFactoryV1.sol";
import "../src/core/SimplePaymaster.sol";
import "../src/core/DemoPaymaster.sol";
import "../src/TestToken.sol";

contract DeployLocalWithPaymaster is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address omniExecutorSigner = vm.envAddress("OMNI_EXECUTOR_SIGNER");

        // Check for paymaster type environment variable
        string memory paymasterType = "simple"; // default
        try vm.envString("PAYMASTER_TYPE") returns (string memory envPaymasterType) {
            paymasterType = envPaymasterType;
        } catch {
            // Use default if not set
        }

        vm.startBroadcast(deployerPrivateKey);

        // Deploy EntryPointV1
        EntryPointV1 entryPoint = new EntryPointV1();
        console.log("EntryPointV1 deployed at:", address(entryPoint));

        // Deploy OmniAccountFactoryV1
        OmniAccountFactoryV1 factory = new OmniAccountFactoryV1(entryPoint);
        console.log("OmniAccountFactoryV1 deployed at:", address(factory));

        // Deploy appropriate paymaster based on environment variable
        if (keccak256(bytes(paymasterType)) == keccak256(bytes("demo"))) {
            // Deploy DemoPaymaster
            DemoPaymaster paymaster = new DemoPaymaster(entryPoint);
            console.log("DemoPaymaster deployed at:", address(paymaster));

            // Fund the paymaster with 0.1 ETH for demo purposes
            // Note: The paymaster's receive function will automatically deposit to EntryPointV1
            (bool success,) = address(paymaster).call{value: 0.1 ether}("");
            require(success, "Failed to fund paymaster");
            console.log("DemoPaymaster funded with 0.1 ETH");
        } else {
            // Deploy SimplePaymaster (default)
            SimplePaymaster paymaster = new SimplePaymaster(entryPoint, omniExecutorSigner);
            console.log("SimplePaymaster deployed at:", address(paymaster));
        }

        // Deploy Test Tokens
        TestToken testUSDC = new TestToken("Test USDC", "USDC", 6);
        console.log("Test USDC deployed at:", address(testUSDC));

        TestToken testUSDT = new TestToken("Test USDT", "USDT", 6);
        console.log("Test USDT deployed at:", address(testUSDT));

        // Mint some tokens to the deployer for testing
        address deployer = vm.addr(deployerPrivateKey);
        testUSDC.mint(deployer, 1000000 * 10 ** 6); // 1M USDC
        testUSDT.mint(deployer, 1000000 * 10 ** 6); // 1M USDT
        console.log("Minted 1M tokens to deployer:", deployer);

        vm.stopBroadcast();
    }
}
