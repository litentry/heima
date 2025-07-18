// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {OmniAccountFactoryV2} from "../src/accounts/OmniAccountFactoryV2.sol";
import {OmniAccount} from "../src/accounts/OmniAccount.sol";
import {ImplementationRegistry} from "../src/core/ImplementationRegistry.sol";
import {IEntryPoint} from "../src/interfaces/IEntryPoint.sol";
import {TestUtils} from "./TestUtils.sol";

contract OmniAccountV2 is OmniAccount {    
    constructor(IEntryPoint _entryPoint) OmniAccount(_entryPoint) {}

    function getVersion() override external pure returns (string memory) {
        return "2.0.0";
    }
}

contract OmniAccountV3 is OmniAccount {    
    constructor(IEntryPoint _entryPoint) OmniAccount(_entryPoint) {}

    function getVersion() override external pure returns (string memory) {
        return "3.0.0";
    }
}

contract OmniAccountFactoryV2Test is Test {
    EntryPoint public entryPoint;
    ImplementationRegistry public registry;
    OmniAccountFactoryV2 public factoryV2;
    
    // Implementation contracts for testing
     OmniAccount public omniAccountV1;
     OmniAccountV2 public omniAccountV2;
     OmniAccountV3 public omniAccountV3;
    
    // Test parameters
    address public owner = makeAddr("owner");
    address public rootAddress = makeAddr("root");
    bytes public clientId = bytes("test_client_v2");
    bytes32 public oa;
    
    string public constant CONTRACT_TYPE = "OmniAccount";
    
    event AccountCreated(
        address indexed account,
        bytes32 indexed oa,
        bytes clientId,
        address indexed root,
        address implementation
    );
    
    event ImplementationRegistered(
        string indexed contractType,
        address indexed implementation
    );
    
    function setUp() public {
        // Deploy base contracts
        entryPoint = new EntryPoint();
        registry = new ImplementationRegistry();
        
        // Deploy different versions of implementation contracts
         omniAccountV1 = new OmniAccount(entryPoint);
         omniAccountV2 = new OmniAccountV2(entryPoint);
         omniAccountV3 = new OmniAccountV3(entryPoint);
        
        // Register first version as canonical implementation
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV1)
        );
        
        // Deploy OmniAccountFactoryV2, using V1 as canonical implementation
        factoryV2 = new OmniAccountFactoryV2(
            entryPoint,
            registry,
            address(omniAccountV1)
        );
        
        // Prepare test data
        oa = TestUtils.prepare_evm_oa(owner, clientId);
    }
    
    function test_RegistryInitialState() public {
        assertEq(registry.getImplementation(CONTRACT_TYPE), address(omniAccountV1));
    }
    
    function test_RegisterNewImplementation() public {
        vm.expectEmit(true, true, true, true);
        emit ImplementationRegistered(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        assertEq(registry.getImplementation(CONTRACT_TYPE), address(omniAccountV2));
    }
    
    function test_MultipleImplementationsRegistration() public {
        // Register V2
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        // Verify current implementation has been updated
        assertEq(registry.getImplementation(CONTRACT_TYPE), address(omniAccountV2));
        
        // Register V3
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV3)
        );
        
        // Verify current implementation has been updated again
        assertEq(registry.getImplementation(CONTRACT_TYPE), address(omniAccountV3));
    }
    
    function test_FactoryInitialState() public {
        assertEq(address(factoryV2.registry()), address(registry));
        assertEq(factoryV2.CANONICAL_IMPLEMENTATION(), address(omniAccountV1));
        assertEq(factoryV2.getCurrentImplementation(), address(omniAccountV1));
    }
    
    function test_AddressStabilityAcrossVersions() public {
        // Calculate address using V1
        address addressV1 = factoryV2.getAddress(oa, clientId, rootAddress);
        
        // Register V2
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        // Address should remain unchanged (due to fixed canonical implementation)
        address addressV2 = factoryV2.getAddress(oa, clientId, rootAddress);
        assertEq(addressV1, addressV2);
        
        // Address remains unchanged after registering V3
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV3)
        );
        
        address addressV3 = factoryV2.getAddress(oa, clientId, rootAddress);
        assertEq(addressV1, addressV3);
    }
    
    function test_CreateAccountWithLatestImplementation() public {
        address senderCreator = address(entryPoint.senderCreator());
        address expectedAddr = factoryV2.getAddress(oa, clientId, rootAddress);
        
        // Use SenderCreator as caller
        vm.prank(senderCreator);
        vm.expectEmit(true, true, true, true, address(factoryV2));
        emit AccountCreated(
            expectedAddr,
            oa,
            clientId,
            rootAddress,
            address(omniAccountV1)
        );
        
        OmniAccount account1 = factoryV2.createAccount(oa, clientId, rootAddress);
        address accountAddress = address(account1);
        
        // Verify account address
        assertEq(accountAddress, expectedAddr);
        assertEq(account1.getVersion(), "1.0.0");
        
        // Register V2
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        // Calling createAccount again should return the same address
        vm.prank(senderCreator);
        OmniAccount account2 = factoryV2.createAccount(oa, clientId, rootAddress);
        assertEq(address(account2), accountAddress);
        assertEq(account2.getVersion(), "2.0.0");
    }
    
    function test_CreateNewAccountWithUpgradedImplementation() public {
        address senderCreator = address(entryPoint.senderCreator());
        
        // Register V2
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        // Create new account with different parameters, should still use canonical implementation (V1)
        bytes32 newOa = TestUtils.prepare_evm_oa(makeAddr("newOwner"), bytes("new_client"));
        
        address expectedAddr = factoryV2.getAddress(newOa, bytes("new_client"), rootAddress);
        
        vm.prank(senderCreator);
        vm.expectEmit(true, true, true, true, address(factoryV2));
        emit AccountCreated(
            expectedAddr,
            newOa,
            bytes("new_client"),
            rootAddress,
            address(omniAccountV1)  // Always uses canonical implementation
        );
        
        OmniAccount newAccount = factoryV2.createAccount(
            newOa,
            bytes("new_client"),
            rootAddress
        );
        
        assertEq(
            address(newAccount),
            expectedAddr
        );
    }
    
    function test_FactoryQueriesAfterUpgrade() public {
        // Register multiple implementations
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV3)
        );
        
        // Verify registry has latest implementation
        assertEq(registry.getImplementation(CONTRACT_TYPE), address(omniAccountV3));
        
        // Verify factory still uses canonical implementation
        assertEq(factoryV2.getCurrentImplementation(), address(omniAccountV1));
    }
    
    function test_RevertWhenNotSenderCreator() public {
        vm.prank(makeAddr("unauthorized"));
        vm.expectRevert("only SenderCreator");
        factoryV2.createAccount(oa, clientId, rootAddress);
    }
    
    function test_RevertOnInvalidRegistryAddress() public {
        vm.expectRevert("Invalid registry");
        new OmniAccountFactoryV2(
            entryPoint,
            ImplementationRegistry(address(0)),
            address(omniAccountV1)
        );
    }
    
    function test_RevertOnInvalidCanonicalImplementation() public {
        vm.expectRevert("Invalid canonical impl");
        new OmniAccountFactoryV2(
            entryPoint,
            registry,
            address(0)
        );
    }
    
    function test_FullUpgradeWorkflow() public {
        address senderCreator = address(entryPoint.senderCreator());
        
        // 1. Create account using canonical implementation (V1)
        vm.prank(senderCreator);
        OmniAccount account = factoryV2.createAccount(oa, clientId, rootAddress);
        address accountAddress = address(account);
        
        // 2. Register V2
        registry.registerImplementation(
            CONTRACT_TYPE,
            address(omniAccountV2)
        );
        
        // 3. Verify registry current implementation is V2
        assertEq(registry.getImplementation(CONTRACT_TYPE), address(omniAccountV2));
        
        // 4. Factory still uses canonical implementation for new accounts
        assertEq(factoryV2.getCurrentImplementation(), address(omniAccountV1));
        
        // 5. Creating new account should still use canonical implementation (V1)
        bytes32 newOa = TestUtils.prepare_evm_oa(makeAddr("newUser"), bytes("new_client"));
        vm.prank(senderCreator);
        OmniAccount newAccount = factoryV2.createAccount(
            newOa,
            bytes("new_client"),
            rootAddress
        );
        
        // 6. Verify address stability
        assertEq(
            address(newAccount),
            factoryV2.getAddress(newOa, bytes("new_client"), rootAddress)
        );
        
        // 7. Original account address remains unchanged
        vm.prank(senderCreator);
        OmniAccount sameAccount = factoryV2.createAccount(oa, clientId, rootAddress);
        assertEq(address(sameAccount), accountAddress);
    }
}