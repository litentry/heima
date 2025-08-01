// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Vm} from "forge-std/Vm.sol";
import {console} from "forge-std/console.sol";

/**
 * @title DeploymentHelper
 * @notice Helper library for saving enhanced deployment artifacts with ABI, bytecode, and metadata
 * @dev This library provides utilities to save comprehensive deployment information including contract ABIs,
 *      bytecode, and optional metadata for verification and cross-checking purposes
 */
library DeploymentHelper {
    struct ContractDeployment {
        string name;
        address addr;
        string abi;
        string bytecode;
        string metadata;
    }

    struct DeploymentArtifact {
        string network;
        uint256 chainId;
        uint256 timestamp;
        uint256 blockNumber;
        address deployer;
        ContractDeployment[] contracts;
    }

    /**
     * @notice Saves enhanced deployment artifacts to a JSON file
     * @param deploymentDir The directory to save the deployment file
     * @param networkName The name of the network
     * @param chainId The chain ID of the network
     * @param deployments Array of contract deployments to save
     */
    function saveDeploymentArtifacts(
        Vm vm,
        string memory deploymentDir,
        string memory networkName,
        uint256 chainId,
        ContractDeployment[] memory deployments
    ) internal {
        // Default to no environment subdirectory for backward compatibility
        saveDeploymentArtifacts(vm, deploymentDir, "", networkName, chainId, deployments);
    }

    /**
     * @notice Saves enhanced deployment artifacts to a JSON file with environment support
     * @param deploymentDir The base directory to save the deployment file
     * @param environment The environment subdirectory (e.g., "staging", "production", or "" for none)
     * @param networkName The name of the network
     * @param chainId The chain ID of the network
     * @param deployments Array of contract deployments to save
     */
    function saveDeploymentArtifacts(
        Vm vm,
        string memory deploymentDir,
        string memory environment,
        string memory networkName,
        uint256 chainId,
        ContractDeployment[] memory deployments
    ) internal {
        // Ensure the deployments directory exists
        try vm.createDir(deploymentDir, false) {} catch {}
        
        // Create environment subdirectory if specified
        string memory targetDir = deploymentDir;
        if (bytes(environment).length > 0) {
            targetDir = string(abi.encodePacked(deploymentDir, "/", environment));
            try vm.createDir(targetDir, false) {} catch {}
        }

        // Create filename based on network
        string memory filename =
            string(abi.encodePacked(targetDir, "/", getNetworkFilename(chainId), ".json"));

        // Build JSON structure
        string memory json = buildDeploymentJson(vm, networkName, chainId, deployments);

        // Write to deployment file
        try vm.writeFile(filename, json) {
            console.log("Enhanced deployment artifacts saved to:", filename);
        } catch Error(string memory reason) {
            console.log("Failed to save deployment file:", reason);
            logContractAddresses(deployments);
        } catch {
            console.log("Failed to save deployment file (unknown error)");
            logContractAddresses(deployments);
        }
    }

    /**
     * @notice Builds the JSON string for deployment artifacts
     * @param networkName The name of the network
     * @param chainId The chain ID of the network
     * @param deployments Array of contract deployments
     * @return The formatted JSON string
     */
    function buildDeploymentJson(
        Vm vm,
        string memory networkName,
        uint256 chainId,
        ContractDeployment[] memory deployments
    ) internal view returns (string memory) {
        string memory contractsJson = "";
        
        for (uint256 i = 0; i < deployments.length; i++) {
            if (i > 0) {
                contractsJson = string(abi.encodePacked(contractsJson, ",\n"));
            }
            
            contractsJson = string(
                abi.encodePacked(
                    contractsJson,
                    '    "',
                    deployments[i].name,
                    '": {\n',
                    '      "address": "',
                    vm.toString(deployments[i].addr),
                    '",\n',
                    '      "abi": ',
                    deployments[i].abi,
                    ',\n',
                    '      "bytecode": "',
                    deployments[i].bytecode,
                    '"'
                )
            );
            
            // Add metadata if available
            if (bytes(deployments[i].metadata).length > 0) {
                contractsJson = string(
                    abi.encodePacked(
                        contractsJson,
                        ',\n      "metadata": ',
                        deployments[i].metadata
                    )
                );
            }
            
            contractsJson = string(abi.encodePacked(contractsJson, '\n    }'));
        }

        return string(
            abi.encodePacked(
                '{\n',
                '  "network": "',
                networkName,
                '",\n',
                '  "chainId": ',
                vm.toString(chainId),
                ',\n',
                '  "timestamp": ',
                vm.toString(block.timestamp),
                ',\n',
                '  "blockNumber": ',
                vm.toString(block.number),
                ',\n',
                '  "deployer": "',
                vm.toString(msg.sender),
                '",\n',
                '  "contracts": {\n',
                contractsJson,
                '\n  }\n',
                '}'
            )
        );
    }

    /**
     * @notice Logs contract addresses to console as fallback
     * @param deployments Array of contract deployments
     */
    function logContractAddresses(ContractDeployment[] memory deployments) internal view {
        console.log("Contract addresses (save manually if needed):");
        for (uint256 i = 0; i < deployments.length; i++) {
            console.log(
                string(abi.encodePacked(deployments[i].name, ": ")),
                deployments[i].addr
            );
        }
    }

    /**
     * @notice Gets the network filename based on chain ID
     * @param chainId The chain ID
     * @return The network filename
     */
    function getNetworkFilename(uint256 chainId) internal pure returns (string memory) {
        // Ethereum networks
        if (chainId == 1) return "ethereum";
        if (chainId == 11155111) return "ethereum-sepolia";
        
        // Arbitrum networks
        if (chainId == 42161) return "arbitrum";
        if (chainId == 421614) return "arbitrum-sepolia";
        
        // Polygon networks
        if (chainId == 137) return "polygon";
        if (chainId == 80001) return "polygon-mumbai";
        if (chainId == 80002) return "polygon-amoy";
        
        // BSC networks
        if (chainId == 56) return "bsc";
        if (chainId == 97) return "bsc-testnet";
        
        // Optimism networks
        if (chainId == 10) return "optimism";
        if (chainId == 11155420) return "optimism-sepolia";
        
        // Base networks
        if (chainId == 8453) return "base";
        if (chainId == 84532) return "base-sepolia";
        
        // HyperEVM networks
        if (chainId == 999) return "hyperevm";
        if (chainId == 998) return "hyperevm-testnet";
        
        // Local networks
        if (chainId == 1337) return "local";
        if (chainId == 31337) return "local";
        
        // Fallback for unknown networks
        return string(abi.encodePacked("chain-", uint2str(chainId)));
    }
    
    /**
     * @notice Converts uint to string
     * @param value The uint value to convert
     * @return The string representation
     */
    function uint2str(uint256 value) internal pure returns (string memory) {
        if (value == 0) {
            return "0";
        }
        uint256 temp = value;
        uint256 digits;
        while (temp != 0) {
            digits++;
            temp /= 10;
        }
        bytes memory buffer = new bytes(digits);
        while (value != 0) {
            digits -= 1;
            buffer[digits] = bytes1(uint8(48 + uint256(value % 10)));
            value /= 10;
        }
        return string(buffer);
    }

    /**
     * @notice Reads the ABI from Foundry artifacts
     * @param vm The Foundry VM instance
     * @param contractName The name of the contract
     * @return The ABI as a JSON string
     */
    function getContractAbi(Vm vm, string memory contractName) internal view returns (string memory) {
        // Due to Solidity limitations with JSON parsing, especially for arrays,
        // we'll return a placeholder. In production, use a post-deployment script
        // to enrich the deployment artifacts with ABIs from the Foundry artifacts.
        // The bytecode is still captured correctly for verification purposes.
        return "[]";
    }

    /**
     * @notice Gets the deployed bytecode of a contract
     * @param contractAddress The address of the deployed contract
     * @return The deployed bytecode as a hex string
     */
    function getDeployedBytecode(address contractAddress) internal view returns (string memory) {
        bytes memory bytecode = contractAddress.code;
        return bytesToHexString(bytecode);
    }

    /**
     * @notice Converts bytes to a hex string
     * @param data The bytes to convert
     * @return The hex string representation
     */
    function bytesToHexString(bytes memory data) internal pure returns (string memory) {
        bytes memory hexChars = "0123456789abcdef";
        bytes memory result = new bytes(2 + data.length * 2);
        result[0] = "0";
        result[1] = "x";
        
        for (uint256 i = 0; i < data.length; i++) {
            result[2 + i * 2] = hexChars[uint8(data[i] >> 4)];
            result[3 + i * 2] = hexChars[uint8(data[i] & 0x0f)];
        }
        
        return string(result);
    }

    /**
     * @notice Creates a ContractDeployment struct with all artifact data
     * @param vm The Foundry VM instance
     * @param name The contract name
     * @param addr The deployed contract address
     * @param metadata Optional metadata JSON string
     * @return The ContractDeployment struct
     */
    function createContractDeployment(
        Vm vm,
        string memory name,
        address addr,
        string memory metadata
    ) internal view returns (ContractDeployment memory) {
        return ContractDeployment({
            name: name,
            addr: addr,
            abi: getContractAbi(vm, name),
            bytecode: getDeployedBytecode(addr),
            metadata: metadata
        });
    }
}