// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/DemoUSDC.sol";
import "../src/PoolVerifier.sol";
import "../src/SimplePrivacyPool.sol";

/// @notice Deploy the full demo stack: DemoUSDC + PoolVerifier (Groth16) + SimplePrivacyPool.
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

        PoolVerifier verifier = new PoolVerifier();
        console.log("PoolVerifier deployed at:     ", address(verifier));
        console.log("Groth16Verifier deployed at:  ", address(verifier.groth16()));

        SimplePrivacyPool pool = new SimplePrivacyPool(address(usdc), address(verifier));
        console.log("SimplePrivacyPool deployed at:", address(pool));

        vm.stopBroadcast();
    }
}
