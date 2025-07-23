// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Script.sol";
import "forge-std/console.sol";

/**
 * @title BaseMigration
 * @notice Base contract for all migration scripts
 * @dev All migration scripts should inherit from this contract
 */
abstract contract BaseMigration is Script {
    // Migration metadata - must be overridden in each migration
    function MIGRATION_ID() public pure virtual returns (string memory);
    function MIGRATION_NAME() public pure virtual returns (string memory);
    function DEPENDENCIES() public pure virtual returns (string[] memory);

    // Deployment configuration
    bytes32 public salt;
    
    /**
     * @notice Main entry point for migration execution
     */
    function run() external {
        console.log("=== STARTING MIGRATION", MIGRATION_ID(), "===");
        console.log("Migration name:", MIGRATION_NAME());
        console.log("Chain ID:", block.chainid);
        console.log("");

        // Load configuration
        _loadConfiguration();

        // Validate prerequisites
        _validatePrerequisites();

        // Get deployment parameters
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        console.log("Deployer:", deployer);
        console.log("");

        // Start broadcasting transactions
        vm.startBroadcast(deployerPrivateKey);

        // Execute the migration
        _executeMigration();

        vm.stopBroadcast();

        console.log("=== MIGRATION", MIGRATION_ID(), "COMPLETE ===");
    }

    /**
     * @notice Load migration configuration
     */
    function _loadConfiguration() internal {
        salt = vm.envOr("DEPLOYMENT_SALT", bytes32(0));
        console.log("Deployment salt:", vm.toString(salt));
    }

    /**
     * @notice Validate prerequisites
     */
    function _validatePrerequisites() internal view {
        string[] memory deps = DEPENDENCIES();
        if (deps.length > 0) {
            console.log("Dependencies:");
            for (uint i = 0; i < deps.length; i++) {
                console.log("-", deps[i]);
            }
        }
    }

    /**
     * @notice Execute the migration logic
     */
    function _executeMigration() internal virtual;
}