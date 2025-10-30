// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {Create2Factory} from "../src/core/Create2Factory.sol";
import {Counter} from "../src/Counter.sol";

contract Create2FactoryTest is Test {
    Create2Factory public factory;

    address deployer1 = makeAddr("deployer1");
    address deployer2 = makeAddr("deployer2");

    event ContractDeployed(address indexed deployed, bytes32 indexed salt, address indexed deployer);

    function setUp() public {
        factory = new Create2Factory();
    }

    // ============ Constructor Tests ============

    function test_Constructor() public view {
        // Factory should be deployed successfully
        assertTrue(address(factory).code.length > 0);
    }

    // ============ Address Computation Tests ============

    function test_ComputeAddress() public view {
        bytes32 salt = keccak256("test-salt");
        bytes memory bytecode = type(Counter).creationCode;

        address predicted = factory.computeAddress(salt, bytecode);

        // Predicted address should be non-zero
        assertTrue(predicted != address(0));

        // Manual calculation should match
        bytes32 bytecodeHash = keccak256(bytecode);
        address expectedAddress = address(
            uint160(uint256(keccak256(abi.encodePacked(bytes1(0xff), address(factory), salt, bytecodeHash))))
        );

        assertEq(predicted, expectedAddress);
    }

    function test_ComputeAddressWithHash() public view {
        bytes32 salt = keccak256("test-salt");
        bytes memory bytecode = type(Counter).creationCode;
        bytes32 bytecodeHash = keccak256(bytecode);

        address predicted = factory.computeAddressWithHash(salt, bytecodeHash);

        // Should match computeAddress result
        address predictedDirect = factory.computeAddress(salt, bytecode);
        assertEq(predicted, predictedDirect);
    }

    function test_ComputeAddress_DifferentSalts() public view {
        bytes32 salt1 = keccak256("salt1");
        bytes32 salt2 = keccak256("salt2");
        bytes memory bytecode = type(Counter).creationCode;

        address predicted1 = factory.computeAddress(salt1, bytecode);
        address predicted2 = factory.computeAddress(salt2, bytecode);

        // Different salts should produce different addresses
        assertTrue(predicted1 != predicted2);
    }

    function test_ComputeAddress_DifferentBytecode() public view {
        bytes32 salt = keccak256("same-salt");
        bytes memory bytecode1 = type(Counter).creationCode;
        bytes memory bytecode2 = abi.encodePacked(type(Counter).creationCode, bytes32(0));

        address predicted1 = factory.computeAddress(salt, bytecode1);
        address predicted2 = factory.computeAddress(salt, bytecode2);

        // Different bytecode should produce different addresses
        assertTrue(predicted1 != predicted2);
    }

    // ============ Deployment Tests ============

    function test_Deploy() public {
        bytes32 salt = keccak256("test-deployment");
        bytes memory bytecode = type(Counter).creationCode;
        address predicted = factory.computeAddress(salt, bytecode);

        // Deploy should emit event
        vm.expectEmit(true, true, true, true);
        emit ContractDeployed(predicted, salt, address(this));

        address deployed = factory.deploy(salt, bytecode);

        // Deployed address should match prediction
        assertEq(deployed, predicted);

        // Contract should have code
        assertTrue(deployed.code.length > 0);

        // Should be marked as deployed
        assertTrue(factory.isDeployed(deployed));

        // Deployed contract should be functional
        Counter counter = Counter(deployed);
        counter.increment();
        assertEq(counter.number(), 1);
    }

    function test_Deploy_WithValue() public {
        // Deploy a simple contract that can receive ETH
        // Using minimal bytecode that accepts value and stores it
        bytes32 salt = keccak256("test-deployment-with-value");

        // Bytecode that just returns empty runtime (but can receive ETH)
        // PUSH1 0x00 DUP1 RETURN
        bytes memory bytecode = hex"60008060006000f3";

        uint256 valueToSend = 1 ether;
        vm.deal(address(this), valueToSend);

        address deployed = factory.deploy{value: valueToSend}(salt, bytecode);

        // Contract should have received ETH
        assertEq(deployed.balance, valueToSend);
    }

    function test_Deploy_MultipleDifferentContracts() public {
        // Deploy first contract
        bytes32 salt1 = keccak256("contract1");
        bytes memory bytecode = type(Counter).creationCode;
        address deployed1 = factory.deploy(salt1, bytecode);

        // Deploy second contract with different salt
        bytes32 salt2 = keccak256("contract2");
        address deployed2 = factory.deploy(salt2, bytecode);

        // Addresses should be different
        assertTrue(deployed1 != deployed2);

        // Both should be functional
        Counter(deployed1).increment();
        Counter(deployed2).increment();
        assertEq(Counter(deployed1).number(), 1);
        assertEq(Counter(deployed2).number(), 1);
    }

    function test_Deploy_FromDifferentSenders() public {
        bytes32 salt = keccak256("same-salt");
        bytes memory bytecode = type(Counter).creationCode;

        // First deployment from deployer1
        vm.prank(deployer1);
        address deployed1 = factory.deploy(salt, bytecode);

        // Second deployment from deployer2 with same salt should work
        // (different from predicted address though since it's already used)
        bytes32 salt2 = keccak256("different-salt");
        vm.prank(deployer2);
        address deployed2 = factory.deploy(salt2, bytecode);

        // Addresses should be different
        assertTrue(deployed1 != deployed2);
    }

    // ============ Redeployment Prevention Tests ============

    function test_Deploy_RevertRedeployment() public {
        bytes32 salt = keccak256("test-redeployment");
        bytes memory bytecode = type(Counter).creationCode;

        // First deployment
        factory.deploy(salt, bytecode);

        // Second deployment should revert
        vm.expectRevert(
            abi.encodeWithSelector(
                Create2Factory.AddressAlreadyDeployed.selector, factory.computeAddress(salt, bytecode)
            )
        );
        factory.deploy(salt, bytecode);
    }

    function test_Deploy_RevertIfAddressHasCode() public {
        // This test simulates the scenario where an address already has code
        // We'll deploy a contract first, then try to deploy again
        bytes32 salt = keccak256("existing-code");
        bytes memory bytecode = type(Counter).creationCode;

        // First deployment
        address deployed = factory.deploy(salt, bytecode);

        // Try to deploy again - should revert
        vm.expectRevert(abi.encodeWithSelector(Create2Factory.AddressAlreadyDeployed.selector, deployed));
        factory.deploy(salt, bytecode);
    }

    // ============ Salt Generation Tests ============

    function test_GenerateSalt() public view {
        string memory contractName = "TestContract";
        string memory version = "v1.0.0";
        address sender = deployer1;

        bytes32 salt = factory.generateSalt(contractName, version, sender);

        // Salt should be non-zero
        assertTrue(salt != bytes32(0));

        // Salt should be deterministic
        bytes32 salt2 = factory.generateSalt(contractName, version, sender);
        assertEq(salt, salt2);

        // Salt should include sender (different sender = different salt)
        bytes32 salt3 = factory.generateSalt(contractName, version, deployer2);
        assertTrue(salt != salt3);
    }

    function test_GenerateSalt_DifferentNames() public view {
        bytes32 salt1 = factory.generateSalt("Contract1", "v1.0.0", deployer1);
        bytes32 salt2 = factory.generateSalt("Contract2", "v1.0.0", deployer1);

        // Different names should produce different salts
        assertTrue(salt1 != salt2);
    }

    function test_GenerateSalt_DifferentVersions() public view {
        bytes32 salt1 = factory.generateSalt("TestContract", "v1.0.0", deployer1);
        bytes32 salt2 = factory.generateSalt("TestContract", "v2.0.0", deployer1);

        // Different versions should produce different salts
        assertTrue(salt1 != salt2);
    }

    function test_GenerateSalt_DifferentSenders() public view {
        bytes32 salt1 = factory.generateSalt("TestContract", "v1.0.0", deployer1);
        bytes32 salt2 = factory.generateSalt("TestContract", "v1.0.0", deployer2);

        // Different senders should produce different salts (frontrunning protection)
        assertTrue(salt1 != salt2);
    }

    function test_GenerateSimpleSalt() public view {
        string memory contractName = "TestContract";
        string memory version = "v1.0.0";

        bytes32 salt = factory.generateSimpleSalt(contractName, version);

        // Salt should be non-zero
        assertTrue(salt != bytes32(0));

        // Salt should be deterministic
        bytes32 salt2 = factory.generateSimpleSalt(contractName, version);
        assertEq(salt, salt2);

        // Simple salt should be same for all senders (no frontrunning protection)
        // This is implicit - the function doesn't take a sender parameter
    }

    function test_GenerateSimpleSalt_VsGenerateSalt() public view {
        string memory contractName = "TestContract";
        string memory version = "v1.0.0";

        bytes32 simpleSalt = factory.generateSimpleSalt(contractName, version);
        bytes32 protectedSalt = factory.generateSalt(contractName, version, deployer1);

        // Simple salt and protected salt should be different
        assertTrue(simpleSalt != protectedSalt);
    }

    // ============ Deployment Tracking Tests ============

    function test_IsDeployed() public {
        bytes32 salt = keccak256("tracking-test");
        bytes memory bytecode = type(Counter).creationCode;

        address predicted = factory.computeAddress(salt, bytecode);

        // Should not be deployed initially
        assertFalse(factory.isDeployed(predicted));

        // Deploy contract
        factory.deploy(salt, bytecode);

        // Should be marked as deployed
        assertTrue(factory.isDeployed(predicted));
    }

    function test_IsDeployed_RandomAddress() public {
        address randomAddr = makeAddr("random");

        // Random address should not be marked as deployed
        assertFalse(factory.isDeployed(randomAddr));
    }

    // ============ Integration Tests ============

    function test_EndToEnd_SenderProtectedDeployment() public {
        // Scenario: Deploy same contract to same address on multiple "chains" (test runs)
        // using sender-protected salt

        string memory contractName = "EntryPointV1";
        string memory version = "v1.0.0";

        // Generate sender-protected salt
        vm.prank(deployer1);
        bytes32 salt = factory.generateSalt(contractName, version, deployer1);

        // Predict address
        bytes memory bytecode = type(Counter).creationCode;
        address predicted = factory.computeAddress(salt, bytecode);

        // Deploy
        vm.prank(deployer1);
        address deployed = factory.deploy(salt, bytecode);

        // Verify
        assertEq(deployed, predicted);
        assertTrue(factory.isDeployed(deployed));

        // Verify only deployer1 can deploy to this address
        // (deployer2 would generate a different salt and thus different address)
        bytes32 salt2 = factory.generateSalt(contractName, version, deployer2);
        assertTrue(salt != salt2);
    }

    function test_EndToEnd_MultiChainDeployment() public {
        // Simulate deploying the same contract with same salt on multiple chains
        // In reality this would be different factory instances, but we can test the logic

        string memory contractName = "TestContract";
        string memory version = "v1.0.0";
        bytes memory bytecode = type(Counter).creationCode;

        // "Chain 1" deployment
        bytes32 salt1 = factory.generateSalt(contractName, version, deployer1);
        address predicted1 = factory.computeAddress(salt1, bytecode);
        vm.prank(deployer1);
        address deployed1 = factory.deploy(salt1, bytecode);

        assertEq(deployed1, predicted1);

        // In a real multi-chain scenario, deploying with the same salt and bytecode
        // from the same deployer on a different chain (different factory instance)
        // would yield the same address - but the factory address would need to be the same
        // That's why we deploy the factory with a fresh EOA on each chain
    }

    // ============ Fuzz Tests ============

    function testFuzz_ComputeAddress(bytes32 salt, bytes memory bytecode) public view {
        // Skip if bytecode is empty (invalid contract)
        vm.assume(bytecode.length > 0);

        address predicted = factory.computeAddress(salt, bytecode);

        // Should always return a non-zero address
        assertTrue(predicted != address(0));
    }

    function testFuzz_Deploy(bytes32 salt) public {
        // Use a simple valid bytecode
        bytes memory bytecode = type(Counter).creationCode;

        address predicted = factory.computeAddress(salt, bytecode);
        address deployed = factory.deploy(salt, bytecode);

        assertEq(deployed, predicted);
        assertTrue(factory.isDeployed(deployed));
    }

    function testFuzz_GenerateSalt(string memory name, string memory version, address sender) public view {
        vm.assume(sender != address(0));

        bytes32 salt = factory.generateSalt(name, version, sender);

        // Salt should be deterministic
        bytes32 salt2 = factory.generateSalt(name, version, sender);
        assertEq(salt, salt2);
    }

    // ============ Edge Cases ============

    function test_Deploy_MinimalBytecode() public {
        // Test with minimal valid bytecode that returns empty runtime code
        bytes32 salt = keccak256("minimal");
        // This bytecode: PUSH1 0x00 DUP1 RETURN (returns 0 bytes of runtime code)
        bytes memory bytecode = hex"60008060006000f3";

        address deployed = factory.deploy(salt, bytecode);

        assertTrue(deployed != address(0));
        // Empty runtime code is valid (contract gets deployed with no code)
        assertTrue(deployed.code.length == 0);
    }

    function test_Deploy_EmptyBytecode() public {
        // Empty creation bytecode actually succeeds in CREATE2 (creates empty contract)
        bytes32 salt = keccak256("empty");
        bytes memory bytecode = hex"";

        // This should succeed and create a contract with no code
        address deployed = factory.deploy(salt, bytecode);

        assertTrue(deployed != address(0));
        assertEq(deployed.code.length, 0);
    }
}
