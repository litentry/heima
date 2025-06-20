// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {SmartAccountFactory} from "../src/accounts/SmartAccountFactory.sol";

contract DeployAA is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // Deploy EntryPoint
        EntryPoint entryPoint = new EntryPoint();
        console.log("EntryPoint deployed at:", address(entryPoint));

        // Deploy SmartAccountFactory
        SmartAccountFactory factory = new SmartAccountFactory(entryPoint);
        console.log("SmartAccountFactory deployed at:", address(factory));
        console.log("SmartAccount implementation at:", address(factory.accountImplementation()));
        console.log("SenderCreator at:", address(factory.senderCreator()));

        vm.stopBroadcast();
    }
}