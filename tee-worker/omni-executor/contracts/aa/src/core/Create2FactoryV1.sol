// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

/**
 * @title Create2FactoryV1
 * @notice Factory contract for deterministic contract deployments using CREATE2
 * @dev Provides deterministic address generation across multiple EVM chains
 *      This contract should be deployed using a fresh EOA on each network
 */
contract Create2FactoryV1 {
    /// @notice Emitted when a contract is successfully deployed
    /// @param deployed The address of the newly deployed contract
    /// @param salt The salt used for deployment
    /// @param deployer The address that initiated the deployment
    event ContractDeployed(address indexed deployed, bytes32 indexed salt, address indexed deployer);

    /// @notice Thrown when attempting to deploy to an address that already has code
    error AddressAlreadyDeployed(address target);

    /// @notice Thrown when the deployment fails
    error DeploymentFailed();

    /// @notice Mapping to track deployed addresses to prevent redeployment
    mapping(address => bool) public deployments;

    /**
     * @notice Deploys a contract using CREATE2
     * @param salt The salt for deterministic address generation
     * @param bytecode The creation bytecode of the contract to deploy
     * @return deployed The address of the deployed contract
     */
    function deploy(bytes32 salt, bytes memory bytecode) external payable returns (address deployed) {
        // Predict the deployment address
        deployed = computeAddress(salt, bytecode);

        // Prevent redeployment to the same address
        if (deployments[deployed]) {
            revert AddressAlreadyDeployed(deployed);
        }

        // Check if address already has code (additional safety check)
        if (deployed.code.length > 0) {
            revert AddressAlreadyDeployed(deployed);
        }

        // Deploy the contract using CREATE2
        assembly {
            deployed := create2(callvalue(), add(bytecode, 0x20), mload(bytecode), salt)
        }

        // Check if deployment was successful
        if (deployed == address(0)) {
            revert DeploymentFailed();
        }

        // Mark address as deployed
        deployments[deployed] = true;

        emit ContractDeployed(deployed, salt, msg.sender);

        return deployed;
    }

    /**
     * @notice Computes the address where a contract will be deployed
     * @param salt The salt for deterministic address generation
     * @param bytecode The creation bytecode of the contract
     * @return predicted The predicted address of the contract
     */
    function computeAddress(bytes32 salt, bytes memory bytecode) public view returns (address predicted) {
        bytes32 bytecodeHash = keccak256(bytecode);
        predicted =
            address(uint160(uint256(keccak256(abi.encodePacked(bytes1(0xff), address(this), salt, bytecodeHash)))));
    }

    /**
     * @notice Computes the address where a contract will be deployed (using bytecode hash)
     * @param salt The salt for deterministic address generation
     * @param bytecodeHash The keccak256 hash of the creation bytecode
     * @return predicted The predicted address of the contract
     */
    function computeAddressWithHash(bytes32 salt, bytes32 bytecodeHash) public view returns (address predicted) {
        predicted =
            address(uint160(uint256(keccak256(abi.encodePacked(bytes1(0xff), address(this), salt, bytecodeHash)))));
    }

    /**
     * @notice Checks if a contract has been deployed at the given address via this factory
     * @param target The address to check
     * @return deployed True if the address was deployed via this factory
     */
    function isDeployed(address target) external view returns (bool) {
        return deployments[target];
    }

    /**
     * @notice Generates a deterministic salt for deployment
     * @param contractName The name/identifier of the contract
     * @return salt The generated salt
     * @dev This generates a purely deterministic salt based only on contract name.
     *      The same contract name will always produce the same address across all chains.
     *      This means addresses are predictable and consistent for all deployers.
     */
    function generateSalt(string memory contractName) external pure returns (bytes32 salt) {
        salt = keccak256(abi.encode(contractName));
    }
}
