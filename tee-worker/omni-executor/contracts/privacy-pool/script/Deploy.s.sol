// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/DemoUSDC.sol";
import "../src/MockVerifier.sol";
import "../src/SimplePrivacyPool.sol";

/// @notice Deploy the full demo stack: DemoUSDC + MockVerifier + SimplePrivacyPool.
///
/// Setup:
///   cp .env.example .env   # fill in PRIVATE_KEY, optionally ARBISCAN_API_KEY
///
/// Deploy:
///   forge script script/Deploy.s.sol \
///     --rpc-url $ARBITRUM_SEPOLIA_RPC_URL \
///     --broadcast \
///     --verify            # optional: verifies on Arbiscan (needs ARBISCAN_API_KEY)
///
/// After deployment copy the printed addresses into .env, then:
///   export OE_PRIVACY_POOL_ADDRESS=<pool address>   (TEE server)
/// And update USDC_ADDRESS in the frontend HTML files.
contract DeployScript is Script {
    function run() external {
        vm.startBroadcast();

        DemoUSDC usdc = new DemoUSDC();
        console.log("DemoUSDC deployed at:         ", address(usdc));

        MockVerifier mockVerifier = new MockVerifier();
        console.log("MockVerifier deployed at:     ", address(mockVerifier));

        SimplePrivacyPool pool = new SimplePrivacyPool(address(usdc), address(mockVerifier));
        console.log("SimplePrivacyPool deployed at:", address(pool));

        vm.stopBroadcast();
    }
}
