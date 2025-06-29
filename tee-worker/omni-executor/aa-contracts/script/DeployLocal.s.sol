// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/core/EntryPoint.sol";
import "../src/accounts/OmniAccountFactory.sol";
import "../src/core/SimplePaymaster.sol";

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
        
        vm.stopBroadcast();
    }
}