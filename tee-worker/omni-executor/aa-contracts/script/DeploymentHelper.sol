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
        uint256 blockNumber;
    }

    struct DeploymentArtifact {
        string network;
        uint256 chainId;
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
        try vm.createDir(deploymentDir, true) {} catch {}

        // Create environment subdirectory if specified
        string memory targetDir = deploymentDir;
        if (bytes(environment).length > 0) {
            targetDir = string(abi.encodePacked(deploymentDir, "/", environment));
            try vm.createDir(targetDir, true) {} catch {}
        }

        // Create filename based on network
        string memory filename = string(abi.encodePacked(targetDir, "/", getNetworkFilename(chainId), ".json"));

        // Check if file already exists and merge if needed
        string memory json;
        try vm.readFile(filename) returns (string memory existingContent) {
            // File exists, try to merge the new contracts with existing ones
            json = mergeDeploymentJson(vm, existingContent, networkName, chainId, deployments);
            console.log("Appending new contracts to existing deployment file:", filename);
        } catch {
            // File doesn't exist, create new JSON
            json = buildDeploymentJson(vm, networkName, chainId, deployments);
            console.log("Creating new deployment file:", filename);
        }

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
                    '      "blockNumber": ',
                    vm.toString(deployments[i].blockNumber),
                    ",\n",
                    '      "abi": ',
                    deployments[i].abi,
                    ",\n",
                    '      "bytecode": "',
                    deployments[i].bytecode,
                    '"'
                )
            );

            // Add metadata if available
            if (bytes(deployments[i].metadata).length > 0) {
                contractsJson =
                    string(abi.encodePacked(contractsJson, ',\n      "metadata": ', deployments[i].metadata));
            }

            contractsJson = string(abi.encodePacked(contractsJson, "\n    }"));
        }

        return string(
            abi.encodePacked(
                "{\n",
                '  "network": "',
                networkName,
                '",\n',
                '  "chainId": ',
                vm.toString(chainId),
                ",\n",
                '  "deployer": "',
                vm.toString(msg.sender),
                '",\n',
                '  "contracts": {\n',
                contractsJson,
                "\n  }\n",
                "}"
            )
        );
    }

    /**
     * @notice Merges new deployment contracts with existing deployment JSON
     * Uses VM methods for JSON parsing but preserves existing contract order and appends new contracts
     * @param vm The Foundry VM instance
     * @param existingJson The existing JSON content from the file
     * @param networkName The name of the network
     * @param chainId The chain ID of the network
     * @param newDeployments Array of new contract deployments to add
     * @return The merged JSON string
     */
    function mergeDeploymentJson(
        Vm vm,
        string memory existingJson,
        string memory networkName,
        uint256 chainId,
        ContractDeployment[] memory newDeployments
    ) internal view returns (string memory) {
        try vm.parseJson(existingJson, ".contracts") returns (bytes memory) {
            // Successfully parsed - extract existing contracts and append new ones

            // Extract existing contract names - try all known contract names and see which exist
            ContractDeployment[] memory existingDeployments = new ContractDeployment[](0);

            string[] memory allKnownNames = getCommonContractNames();
            for (uint256 i = 0; i < allKnownNames.length; i++) {
                string memory contractName = allKnownNames[i];
                string memory contractPath = string(abi.encodePacked(".contracts.", contractName));

                try vm.parseJsonAddress(existingJson, string(abi.encodePacked(contractPath, ".address"))) returns (
                    address contractAddr
                ) {
                    // This contract exists! Extract its properties
                    uint256 blockNum;
                    try vm.parseJsonUint(existingJson, string(abi.encodePacked(contractPath, ".blockNumber"))) returns (
                        uint256 bn
                    ) {
                        blockNum = bn;
                    } catch {
                        blockNum = block.number; // fallback
                    }

                    string memory abiString;
                    try vm.parseJsonString(existingJson, string(abi.encodePacked(contractPath, ".abi"))) returns (
                        string memory abiStr
                    ) {
                        abiString = abiStr;
                    } catch {
                        abiString = "[]"; // fallback
                    }

                    string memory bytecode;
                    try vm.parseJsonString(existingJson, string(abi.encodePacked(contractPath, ".bytecode"))) returns (
                        string memory bc
                    ) {
                        bytecode = bc;
                    } catch {
                        bytecode = "0x"; // fallback
                    }

                    string memory metadata;
                    try vm.parseJsonString(existingJson, string(abi.encodePacked(contractPath, ".metadata"))) returns (
                        string memory md
                    ) {
                        metadata = md;
                    } catch {
                        metadata = ""; // no metadata
                    }

                    // Add to existing deployments array
                    ContractDeployment[] memory newExisting = new ContractDeployment[](existingDeployments.length + 1);
                    for (uint256 j = 0; j < existingDeployments.length; j++) {
                        newExisting[j] = existingDeployments[j];
                    }
                    newExisting[existingDeployments.length] = ContractDeployment({
                        name: contractName,
                        addr: contractAddr,
                        abi: abiString,
                        bytecode: bytecode,
                        metadata: metadata,
                        blockNumber: blockNum
                    });
                    existingDeployments = newExisting;
                } catch {
                    // Contract doesn't exist, skip it
                    continue;
                }
            }

            // Now combine existing deployments with new deployments
            ContractDeployment[] memory allDeployments =
                new ContractDeployment[](existingDeployments.length + newDeployments.length);
            for (uint256 i = 0; i < existingDeployments.length; i++) {
                allDeployments[i] = existingDeployments[i];
            }
            for (uint256 i = 0; i < newDeployments.length; i++) {
                allDeployments[existingDeployments.length + i] = newDeployments[i];
            }

            // Build final JSON with all contracts (existing first, new last)
            return buildDeploymentJson(vm, networkName, chainId, allDeployments);
        } catch {
            console.log("JSON parsing failed, creating new deployment file");
            return buildDeploymentJson(vm, networkName, chainId, newDeployments);
        }
    }

    /**
     * @notice Returns common contract names to check for when merging
     * @return Array of common contract names
     */
    function getCommonContractNames() internal pure returns (string[] memory) {
        string[] memory names = new string[](20);
        names[0] = "EntryPoint";
        names[1] = "SimpleAccount";
        names[2] = "SimpleAccountFactory";
        names[3] = "SimplePaymaster";
        names[4] = "ERC20Paymaster";
        names[5] = "ERC20PaymasterV1";
        names[6] = "TokenPaymaster";
        names[7] = "VerifyingPaymaster";
        names[8] = "StakeManager";
        names[9] = "OmniAccount";
        names[10] = "OmniAccountFactory";
        names[11] = "Multicall";
        names[12] = "Create2Factory";
        names[13] = "ProxyFactory";
        names[14] = "UpgradeableBeacon";
        names[15] = "Implementation";
        names[16] = "Proxy";
        names[17] = "Registry";
        names[18] = "Forwarder";
        names[19] = "Aggregator";
        return names;
    }

    /**
     * @notice Logs contract addresses to console as fallback
     * @param deployments Array of contract deployments
     */
    function logContractAddresses(ContractDeployment[] memory deployments) internal pure {
        console.log("Contract addresses (save manually if needed):");
        for (uint256 i = 0; i < deployments.length; i++) {
            console.log(string(abi.encodePacked(deployments[i].name, ": ")), deployments[i].addr);
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
     * @return The ABI as a JSON string
     */
    function getContractAbi(Vm, /* vm */ string memory /* contractName */ ) internal pure returns (string memory) {
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
     * @notice Creates a ContractDeployment struct with accurate deployment block from broadcast file
     * @dev This function attempts to read the actual deployment block from Foundry broadcast files.
     *      If broadcast files are not available (e.g., local testing), it gracefully falls back to block.number.
     * @param vm The Foundry VM instance
     * @param name The contract name
     * @param addr The deployed contract address
     * @param metadata Optional metadata JSON string
     * @return The ContractDeployment struct with accurate block number
     */
    function createContractDeployment(Vm vm, string memory name, address addr, string memory metadata)
        internal
        view
        returns (ContractDeployment memory)
    {
        // Try to get actual deployment block from broadcast file
        uint256 actualBlockNumber = getActualDeploymentBlockSafe(vm, addr);

        return ContractDeployment({
            name: name,
            addr: addr,
            abi: getContractAbi(vm, name),
            bytecode: getDeployedBytecode(addr),
            metadata: metadata,
            blockNumber: actualBlockNumber
        });
    }

    /**
     * @notice Gets the actual deployment block number automatically detecting script and chain
     * @dev This function tries common script names and uses current chain ID
     * @param vm The Foundry VM instance
     * @param contractAddress The deployed contract address
     * @return The actual block number where the contract was deployed
     */
    function getActualDeploymentBlockSafe(Vm vm, address contractAddress) internal view returns (uint256) {
        // Try common script names in order of likelihood
        string[5] memory commonScripts =
            ["Deploy.s.sol", "DeployLocal.s.sol", "DeployLocalWithPaymaster.s.sol", "script.s.sol", "Script.s.sol"];

        uint256 currentChainId = block.chainid;

        for (uint256 i = 0; i < commonScripts.length; i++) {
            uint256 blockNum = getActualDeploymentBlock(vm, contractAddress, commonScripts[i], currentChainId);
            if (blockNum != block.number) {
                // Found actual deployment block (not fallback)
                return blockNum;
            }
        }

        // All attempts failed, use fallback
        return block.number;
    }

    /**
     * @notice Gets the actual deployment block number from broadcast file
     * @param vm The Foundry VM instance
     * @param contractAddress The deployed contract address
     * @param scriptName The script name (e.g., "Deploy.s.sol")
     * @param chainId The chain ID
     * @return The actual block number where the contract was deployed
     */
    function getActualDeploymentBlock(Vm vm, address contractAddress, string memory scriptName, uint256 chainId)
        public
        view
        returns (uint256)
    {
        try vm.projectRoot() returns (string memory root) {
            string memory broadcastPath =
                string(abi.encodePacked(root, "/broadcast/", scriptName, "/", vm.toString(chainId), "/run-latest.json"));

            try vm.readFile(broadcastPath) returns (string memory json) {
                return parseBlockNumberFromBroadcast(vm, json, contractAddress);
            } catch {
                console.log("Warning: Could not read broadcast file, using current block number");
                return block.number;
            }
        } catch {
            console.log("Warning: Could not get project root, using current block number");
            return block.number;
        }
    }

    /**
     * @notice Parses the actual deployment block from broadcast JSON
     * @param vm The Foundry VM instance
     * @param broadcastJson The broadcast JSON content
     * @param contractAddress The contract address to find
     * @return The block number where the contract was deployed
     */
    function parseBlockNumberFromBroadcast(Vm vm, string memory broadcastJson, address contractAddress)
        internal
        view
        returns (uint256)
    {
        // Try to find the deployment transaction for this contract address
        string memory addressLower = toLowerCase(vm.toString(contractAddress));

        // Parse receipts array length
        try vm.parseJsonUint(broadcastJson, ".receipts") returns (uint256) {
            // If .receipts is a uint, it means the structure is different, fall back
            return block.number;
        } catch {
            // .receipts should be an array, try to get its length
            try vm.parseJson(broadcastJson, ".receipts") returns (bytes memory /* receiptsData */ ) {
                // Look for transactions that created contracts
                // We'll check the first few receipts for contract creation (to field is null or empty)
                for (uint256 i = 0; i < 20; i++) {
                    // Check up to 20 transactions
                    try vm.parseJsonString(
                        broadcastJson, string(abi.encodePacked(".receipts[", vm.toString(i), "].to"))
                    ) returns (string memory toAddr) {
                        // If 'to' is null/empty, this is a contract creation transaction
                        if (bytes(toAddr).length == 0 || keccak256(bytes(toAddr)) == keccak256(bytes("null"))) {
                            try vm.parseJsonString(
                                broadcastJson,
                                string(abi.encodePacked(".receipts[", vm.toString(i), "].contractAddress"))
                            ) returns (string memory contractAddr) {
                                if (keccak256(bytes(toLowerCase(contractAddr))) == keccak256(bytes(addressLower))) {
                                    // Found our contract! Get its block number
                                    try vm.parseJsonString(
                                        broadcastJson,
                                        string(abi.encodePacked(".receipts[", vm.toString(i), "].blockNumber"))
                                    ) returns (string memory blockHex) {
                                        return hexStringToUint(blockHex);
                                    } catch {
                                        return block.number;
                                    }
                                }
                            } catch {
                                continue;
                            }
                        }
                    } catch {
                        // If receipt[i] doesn't exist, we've reached the end
                        break;
                    }
                }
                return block.number;
            } catch {
                return block.number;
            }
        }
    }

    /**
     * @notice Converts a hex string to uint256
     * @param hexStr The hex string (with or without 0x prefix)
     * @return The uint256 value
     */
    function hexStringToUint(string memory hexStr) public pure returns (uint256) {
        bytes memory hexBytes = bytes(hexStr);
        uint256 startIndex = 0;

        // Skip "0x" prefix if present
        if (hexBytes.length >= 2 && hexBytes[0] == "0" && (hexBytes[1] == "x" || hexBytes[1] == "X")) {
            startIndex = 2;
        }

        uint256 result = 0;
        for (uint256 i = startIndex; i < hexBytes.length; i++) {
            result = result * 16;
            uint8 digit = uint8(hexBytes[i]);

            if (digit >= 48 && digit <= 57) {
                // 0-9
                result += digit - 48;
            } else if (digit >= 65 && digit <= 70) {
                // A-F
                result += digit - 55;
            } else if (digit >= 97 && digit <= 102) {
                // a-f
                result += digit - 87;
            }
            // Invalid hex characters are ignored
        }

        return result;
    }

    /**
     * @notice Converts a string to lowercase
     * @param str The input string
     * @return The lowercase string
     */
    function toLowerCase(string memory str) public pure returns (string memory) {
        bytes memory strBytes = bytes(str);
        bytes memory result = new bytes(strBytes.length);

        for (uint256 i = 0; i < strBytes.length; i++) {
            if (strBytes[i] >= 0x41 && strBytes[i] <= 0x5A) {
                // Convert A-Z to a-z
                result[i] = bytes1(uint8(strBytes[i]) + 32);
            } else {
                result[i] = strBytes[i];
            }
        }

        return string(result);
    }
}
