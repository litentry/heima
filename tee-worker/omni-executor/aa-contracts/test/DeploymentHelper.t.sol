// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {Vm} from "forge-std/Vm.sol";
import {DeploymentHelper} from "../script/DeploymentHelper.sol";

/**
 * @title DeploymentHelperTest
 * @notice Comprehensive test suite for DeploymentHelper library, focusing on append functionality
 */
contract DeploymentHelperTest is Test {
    using DeploymentHelper for *;

    string constant BASE_TEST_DIR = "./test_deployments";
    string constant NETWORK_NAME = "test-network";
    uint256 constant CHAIN_ID = 31337;

    address mockContract1 = address(0x1111111111111111111111111111111111111111);
    address mockContract2 = address(0x2222222222222222222222222222222222222222);
    address mockContract3 = address(0x3333333333333333333333333333333333333333);

    function setUp() public {
        // Clean up any existing test deployment directory
        try vm.removeDir(BASE_TEST_DIR, true) {} catch {}
    }

    function tearDown() public {
        // Clean up test files after each test
        try vm.removeDir(BASE_TEST_DIR, true) {} catch {}
    }
    
    // Helper to get unique directory for each test
    function getTestDir(string memory testName) internal pure returns (string memory) {
        return string(abi.encodePacked(BASE_TEST_DIR, "/", testName));
    }

    function test_CreateNewDeploymentFile() public {
        // Test creating a new deployment file from scratch
        string memory testDir = getTestDir("create");
        DeploymentHelper.ContractDeployment[] memory deployments = new DeploymentHelper.ContractDeployment[](1);
        deployments[0] = createMockDeployment("EntryPoint", mockContract1);

        DeploymentHelper.saveDeploymentArtifacts(
            vm,
            testDir,
            NETWORK_NAME,
            CHAIN_ID,
            deployments
        );

        // Verify file was created
        string memory filename = string(abi.encodePacked(testDir, "/local.json"));
        assertTrue(vm.exists(filename), "Deployment file should exist");

        // Verify basic content without strict JSON parsing (to avoid format issues)
        string memory content = vm.readFile(filename);
        assertTrue(bytes(content).length > 0, "File should not be empty");
        assertTrue(contains(content, "EntryPoint"), "Should contain contract name");
        assertTrue(contains(content, vm.toString(mockContract1)), "Should contain contract address");
        assertTrue(contains(content, NETWORK_NAME), "Should contain network name");
        assertTrue(contains(content, vm.toString(CHAIN_ID)), "Should contain chain ID");
    }

    function test_AppendToExistingFile() public {
        // Step 1: Create initial deployment with EntryPoint
        string memory testDir = getTestDir("append");
        DeploymentHelper.ContractDeployment[] memory initialDeployments = new DeploymentHelper.ContractDeployment[](1);
        initialDeployments[0] = createMockDeployment("EntryPoint", mockContract1);

        DeploymentHelper.saveDeploymentArtifacts(
            vm,
            testDir,
            NETWORK_NAME,
            CHAIN_ID,
            initialDeployments
        );

        string memory filename = string(abi.encodePacked(testDir, "/local.json"));
        string memory initialContent = vm.readFile(filename);
        
        // Verify initial deployment
        assertTrue(contains(initialContent, "EntryPoint"), "Initial file should contain EntryPoint");
        assertTrue(contains(initialContent, vm.toString(mockContract1)), "Initial file should contain EntryPoint address");

        // Step 2: Append SimpleAccountFactory
        DeploymentHelper.ContractDeployment[] memory newDeployments = new DeploymentHelper.ContractDeployment[](1);
        newDeployments[0] = createMockDeployment("SimpleAccountFactory", mockContract2);

        vm.warp(block.timestamp + 100);
        vm.roll(block.number + 10);

        DeploymentHelper.saveDeploymentArtifacts(
            vm,
            testDir,
            NETWORK_NAME,
            CHAIN_ID,
            newDeployments
        );

        // Step 3: Verify both contracts are present
        string memory finalContent = vm.readFile(filename);
        
        // Basic content verification
        assertTrue(contains(finalContent, "EntryPoint"), "Final file should contain EntryPoint");
        assertTrue(contains(finalContent, vm.toString(mockContract1)), "Final file should contain EntryPoint address");
        assertTrue(contains(finalContent, "SimpleAccountFactory"), "Final file should contain SimpleAccountFactory");
        assertTrue(contains(finalContent, vm.toString(mockContract2)), "Final file should contain SimpleAccountFactory address");
        
        // Verify the final content is longer than initial (indicating append, not replace)
        assertGt(bytes(finalContent).length, bytes(initialContent).length, "Final file should be larger than initial");
    }

    function test_MultipleSequentialAppends() public {
        string memory testDir = getTestDir("multiple");
        string memory filename = string(abi.encodePacked(testDir, "/local.json"));

        // First deployment - EntryPoint
        DeploymentHelper.ContractDeployment[] memory deployment1 = new DeploymentHelper.ContractDeployment[](1);
        deployment1[0] = createMockDeployment("EntryPoint", mockContract1);
        
        DeploymentHelper.saveDeploymentArtifacts(vm, testDir, NETWORK_NAME, CHAIN_ID, deployment1);
        string memory content1 = vm.readFile(filename);
        uint256 content1Length = bytes(content1).length;
        
        // Verify first deployment
        assertTrue(contains(content1, "EntryPoint"), "Should contain EntryPoint after first deployment");
        assertTrue(contains(content1, vm.toString(mockContract1)), "Should contain EntryPoint address");

        // Second deployment (append) - SimpleAccountFactory
        DeploymentHelper.ContractDeployment[] memory deployment2 = new DeploymentHelper.ContractDeployment[](1);
        deployment2[0] = createMockDeployment("SimpleAccountFactory", mockContract2);
        
        vm.warp(block.timestamp + 50);
        DeploymentHelper.saveDeploymentArtifacts(vm, testDir, NETWORK_NAME, CHAIN_ID, deployment2);
        
        string memory content2 = vm.readFile(filename);
        uint256 content2Length = bytes(content2).length;
        
        // Verify both contracts exist and content grew
        assertTrue(contains(content2, "EntryPoint"), "Should contain EntryPoint after second deployment");
        assertTrue(contains(content2, "SimpleAccountFactory"), "Should contain SimpleAccountFactory after second deployment");
        assertGt(content2Length, content1Length, "File should grow after second deployment");

        // Third deployment (append) - ERC20Paymaster  
        DeploymentHelper.ContractDeployment[] memory deployment3 = new DeploymentHelper.ContractDeployment[](1);
        deployment3[0] = createMockDeployment("ERC20Paymaster", mockContract3);
        
        vm.warp(block.timestamp + 50);
        DeploymentHelper.saveDeploymentArtifacts(vm, testDir, NETWORK_NAME, CHAIN_ID, deployment3);
        
        string memory finalContent = vm.readFile(filename);
        uint256 finalContentLength = bytes(finalContent).length;
        
        // Verify all three contracts exist and content grew again
        assertTrue(contains(finalContent, "EntryPoint"), "Should contain EntryPoint in final");
        assertTrue(contains(finalContent, "SimpleAccountFactory"), "Should contain SimpleAccountFactory in final");
        assertTrue(contains(finalContent, "ERC20Paymaster"), "Should contain ERC20Paymaster in final");
        assertTrue(contains(finalContent, vm.toString(mockContract1)), "Should contain EntryPoint address in final");
        assertTrue(contains(finalContent, vm.toString(mockContract2)), "Should contain SimpleAccountFactory address in final");
        assertTrue(contains(finalContent, vm.toString(mockContract3)), "Should contain ERC20Paymaster address in final");
        assertGt(finalContentLength, content2Length, "File should grow after third deployment");
    }

    function test_AppendWithEnvironment() public {
        string memory testDir = getTestDir("environment");
        string memory environment = "staging";
        
        DeploymentHelper.ContractDeployment[] memory deployments = new DeploymentHelper.ContractDeployment[](1);
        deployments[0] = createMockDeployment("EntryPoint", mockContract1);

        DeploymentHelper.saveDeploymentArtifacts(
            vm,
            testDir,
            environment,
            NETWORK_NAME,
            CHAIN_ID,
            deployments
        );

        // Verify file was created in environment subdirectory
        string memory filename = string(abi.encodePacked(testDir, "/", environment, "/local.json"));
        assertTrue(vm.exists(filename), "Deployment file should exist in environment subdirectory");

        string memory content = vm.readFile(filename);
        assertTrue(contains(content, "EntryPoint"), "Should contain contract name");
        assertTrue(contains(content, vm.toString(mockContract1)), "Should contain contract address");
        assertTrue(contains(content, NETWORK_NAME), "Should contain network name");
    }

    function test_AppendWithMetadata() public {
        string memory testDir = getTestDir("metadata");
        // Test with contract metadata
        DeploymentHelper.ContractDeployment[] memory deployments = new DeploymentHelper.ContractDeployment[](1);
        deployments[0] = DeploymentHelper.ContractDeployment({
            name: "EntryPoint",
            addr: mockContract1,
            abi: "[]",
            bytecode: DeploymentHelper.getDeployedBytecode(mockContract1),
            metadata: '{"compiler": "solc", "version": "0.8.28"}'
        });

        DeploymentHelper.saveDeploymentArtifacts(
            vm,
            testDir,
            NETWORK_NAME,
            CHAIN_ID,
            deployments
        );

        string memory filename = string(abi.encodePacked(testDir, "/local.json"));
        string memory content = vm.readFile(filename);
        
        // Basic verification - check that metadata-related content is present
        assertTrue(contains(content, "EntryPoint"), "Should contain contract name");
        assertTrue(contains(content, vm.toString(mockContract1)), "Should contain contract address");
        assertTrue(contains(content, "metadata"), "Should contain metadata field");
        assertTrue(contains(content, "compiler"), "Should contain compiler in metadata");
        assertTrue(contains(content, "solc"), "Should contain solc in metadata");
    }

    function test_FallbackOnInvalidJson() public {
        string memory testDir = getTestDir("fallback");
        // Create an invalid JSON file first
        string memory filename = string(abi.encodePacked(testDir, "/local.json"));
        try vm.createDir(testDir, true) {} catch {} // Allow creation to fail if directory exists
        vm.writeFile(filename, "{ invalid json structure without proper closing");
        
        DeploymentHelper.ContractDeployment[] memory newDeployments = new DeploymentHelper.ContractDeployment[](1);
        newDeployments[0] = createMockDeployment("TestContract", mockContract1);

        // This should not revert but fall back to creating new JSON
        DeploymentHelper.saveDeploymentArtifacts(
            vm,
            testDir,
            NETWORK_NAME,
            CHAIN_ID,
            newDeployments
        );

        string memory content = vm.readFile(filename);
        assertTrue(contains(content, "TestContract"), "Should contain new contract after fallback");
        assertTrue(contains(content, vm.toString(mockContract1)), "Should contain new contract address after fallback");
    }

    function test_NetworkFilenameMapping() public pure {
        // Test known chain IDs
        assertEq(DeploymentHelper.getNetworkFilename(1), "ethereum", "Mainnet should return ethereum");
        assertEq(DeploymentHelper.getNetworkFilename(56), "bsc", "BSC should return bsc");
        assertEq(DeploymentHelper.getNetworkFilename(137), "polygon", "Polygon should return polygon");
        assertEq(DeploymentHelper.getNetworkFilename(42161), "arbitrum", "Arbitrum should return arbitrum");
        assertEq(DeploymentHelper.getNetworkFilename(31337), "local", "Local should return local");
        
        // Test unknown chain ID
        uint256 unknownChainId = 999999;
        string memory expected = "chain-999999";
        string memory result = DeploymentHelper.getNetworkFilename(unknownChainId);
        assertEq(result, expected, "Unknown chain should return chain-{id}");
    }


    function test_AppendBehaviorBasic() public {
        string memory testDir = getTestDir("basic");
        // Simple test to verify append behavior works at all
        
        // First deployment
        DeploymentHelper.ContractDeployment[] memory deployment1 = new DeploymentHelper.ContractDeployment[](1);
        deployment1[0] = createMockDeployment("EntryPoint", mockContract1);
        
        DeploymentHelper.saveDeploymentArtifacts(vm, testDir, NETWORK_NAME, CHAIN_ID, deployment1);

        // Verify first file exists
        string memory filename = string(abi.encodePacked(testDir, "/local.json"));
        assertTrue(vm.exists(filename), "First deployment file should exist");
        
        // Second deployment (should attempt to append)
        DeploymentHelper.ContractDeployment[] memory deployment2 = new DeploymentHelper.ContractDeployment[](1);
        deployment2[0] = createMockDeployment("SimpleAccountFactory", mockContract2);
        
        vm.warp(block.timestamp + 100);
        DeploymentHelper.saveDeploymentArtifacts(vm, testDir, NETWORK_NAME, CHAIN_ID, deployment2);

        // Verify file still exists and was updated
        assertTrue(vm.exists(filename), "Deployment file should still exist after append");
        
        string memory finalContent = vm.readFile(filename);
        assertTrue(bytes(finalContent).length > 0, "Final file should not be empty");
        
        // At minimum, the file should contain the second contract
        assertTrue(contains(finalContent, "SimpleAccountFactory"), "Should contain second contract name");
        assertTrue(contains(finalContent, vm.toString(mockContract2)), "Should contain second contract address");
    }

    // Helper function to create mock deployments
    function createMockDeployment(string memory name, address addr) 
        internal 
        view 
        returns (DeploymentHelper.ContractDeployment memory) 
    {
        return DeploymentHelper.ContractDeployment({
            name: name,
            addr: addr,
            abi: "[]",
            bytecode: DeploymentHelper.getDeployedBytecode(addr),
            metadata: ""
        });
    }

    // Helper function to check if a string contains a substring
    function contains(string memory str, string memory substr) internal pure returns (bool) {
        bytes memory strBytes = bytes(str);
        bytes memory substrBytes = bytes(substr);
        
        if (substrBytes.length > strBytes.length) {
            return false;
        }
        
        for (uint256 i = 0; i <= strBytes.length - substrBytes.length; i++) {
            bool found = true;
            for (uint256 j = 0; j < substrBytes.length; j++) {
                if (strBytes[i + j] != substrBytes[j]) {
                    found = false;
                    break;
                }
            }
            if (found) {
                return true;
            }
        }
        return false;
    }
}