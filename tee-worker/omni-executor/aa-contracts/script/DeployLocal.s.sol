// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/core/EntryPoint.sol";
import "../src/accounts/OmniAccountFactory.sol";
import "../src/core/SimplePaymaster.sol";
import "../src/TestToken.sol";

contract DeployLocal is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address omniExecutorSigner = vm.envAddress("OMNI_EXECUTOR_SIGNER");

        vm.startBroadcast(deployerPrivateKey);

        // Deploy EntryPoint
        EntryPoint entryPoint = new EntryPoint();
        console.log("EntryPoint deployed at:", address(entryPoint));

        // Deploy OmniAccountFactory
        OmniAccountFactory factory = new OmniAccountFactory(entryPoint);
        console.log("OmniAccountFactory deployed at:", address(factory));

        // Deploy SimplePaymaster
        SimplePaymaster paymaster = new SimplePaymaster(entryPoint, omniExecutorSigner);
        console.log("SimplePaymaster deployed at:", address(paymaster));

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
