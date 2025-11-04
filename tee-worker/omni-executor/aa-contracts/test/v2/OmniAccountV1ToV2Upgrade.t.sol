// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OmniAccountV1} from "../../src/accounts/OmniAccountV1.sol";
import {OmniAccountV2} from "../../src/accounts/OmniAccountV2.sol";
import {EntryPointV1} from "../../src/core/EntryPointV1.sol";
import {OwnerType} from "../../src/interfaces/OwnerType.sol";
import {Passkey} from "../../src/interfaces/Passkey.sol";
import {TestUtils} from "../TestUtils.sol";
import {StorageTestModule} from "./StorageTestModule.sol";

/**
 * Comprehensive test for upgrading OmniAccountV1 to V2 with full state verification.
 * Tests:
 * 1. V1 account setup with all fields populated
 * 2. Upgrade to V2
 * 3. Verify all V1 state is preserved
 * 4. Register and execute modules
 * 5. Verify module storage operations don't corrupt account storage
 * 6. Verify V1 functionality still works after upgrade
 */
contract OmniAccountV1ToV2Upgrade is Test {
    EntryPointV1 public entryPoint;
    OmniAccountV1 public accountV1;
    OmniAccountV2 public accountV2Implementation;
    StorageTestModule public module;

    // Test accounts
    address public owner = address(0x1111);
    address public rootSigner1 = address(0x2222);
    address public rootSigner2 = address(0x3333);
    address public rootSigner3 = address(0x4444);
    bytes public clientId = bytes("comprehensive_test_client_v1_to_v2");

    // Passkey signers
    Passkey.PublicKey public passkey1 = Passkey.PublicKey({x: 111111, y: 222222});
    Passkey.PublicKey public passkey2 = Passkey.PublicKey({x: 333333, y: 444444});

    // Storage state to verify
    bytes32 public expectedOwner;
    uint256 public depositAmount = 5 ether;

    function setUp() public {
        entryPoint = new EntryPointV1();

        // Deploy V1 implementation and create proxy
        OmniAccountV1 v1Implementation = new OmniAccountV1(entryPoint);
        expectedOwner = TestUtils.prepare_evm_oa(owner, clientId);

        accountV1 = OmniAccountV1(
            payable(
                new ERC1967Proxy{salt: expectedOwner}(
                    address(v1Implementation),
                    abi.encodeCall(OmniAccountV1.initialize, (expectedOwner, OwnerType.Evm, clientId, rootSigner1))
                )
            )
        );

        // Prepare V2 implementation for upgrade
        accountV2Implementation = new OmniAccountV2(entryPoint);

        // Prepare module
        module = new StorageTestModule();
    }

    function test_ComprehensiveV1ToV2Upgrade() public {
        // ============ STEP 1: Setup V1 Account with Full State ============

        // Add multiple root signers
        vm.prank(owner);
        accountV1.addRootSigner(rootSigner2);
        vm.prank(owner);
        accountV1.addRootSigner(rootSigner3);

        // Add passkey signers
        vm.prank(owner);
        accountV1.addPasskeySigner(passkey1);
        vm.prank(owner);
        accountV1.addPasskeySigner(passkey2);

        // Add deposit to EntryPoint
        vm.deal(address(accountV1), 10 ether);
        vm.prank(address(accountV1));
        accountV1.addDeposit{value: depositAmount}();

        // Verify V1 initial state
        assertEq(accountV1.owner(), expectedOwner, "V1 owner mismatch");
        assertEq(accountV1.clientId(), clientId, "V1 clientId mismatch");
        assertEq(uint256(accountV1.ownerType()), uint256(OwnerType.Evm), "V1 ownerType mismatch");
        assertTrue(accountV1.isRootSigner(rootSigner1), "rootSigner1 not registered");
        assertTrue(accountV1.isRootSigner(rootSigner2), "rootSigner2 not registered");
        assertTrue(accountV1.isRootSigner(rootSigner3), "rootSigner3 not registered");
        assertTrue(accountV1.passkeySigners(Passkey.toKey(passkey1)), "passkey1 not registered");
        assertTrue(accountV1.passkeySigners(Passkey.toKey(passkey2)), "passkey2 not registered");
        assertEq(accountV1.passkeySignerCount(), 2, "passkey count mismatch");
        assertEq(accountV1.getDeposit(), depositAmount, "deposit mismatch");
        assertEq(accountV1.version(), "1.0.0", "V1 version mismatch");

        // ============ STEP 2: Upgrade to V2 ============

        vm.prank(owner);
        UUPSUpgradeable(address(accountV1)).upgradeToAndCall(address(accountV2Implementation), "");

        // Cast to V2 interface
        OmniAccountV2 accountV2 = OmniAccountV2(payable(address(accountV1)));

        // ============ STEP 3: Verify All V1 State is Preserved ============

        assertEq(accountV2.owner(), expectedOwner, "V2 owner mismatch after upgrade");
        assertEq(accountV2.clientId(), clientId, "V2 clientId mismatch after upgrade");
        assertEq(uint256(accountV2.ownerType()), uint256(OwnerType.Evm), "V2 ownerType mismatch after upgrade");
        assertTrue(accountV2.isRootSigner(rootSigner1), "rootSigner1 lost after upgrade");
        assertTrue(accountV2.isRootSigner(rootSigner2), "rootSigner2 lost after upgrade");
        assertTrue(accountV2.isRootSigner(rootSigner3), "rootSigner3 lost after upgrade");
        assertTrue(accountV2.passkeySigners(Passkey.toKey(passkey1)), "passkey1 lost after upgrade");
        assertTrue(accountV2.passkeySigners(Passkey.toKey(passkey2)), "passkey2 lost after upgrade");
        assertEq(accountV2.passkeySignerCount(), 2, "passkey count changed after upgrade");
        assertEq(accountV2.getDeposit(), depositAmount, "deposit changed after upgrade");
        assertEq(accountV2.version(), "2.0.0", "V2 version mismatch after upgrade");

        // ============ STEP 4: Test New V2 Module Functionality ============

        // Register module
        vm.prank(owner);
        accountV2.registerModule(address(module));
        assertTrue(accountV2.isModuleRegistered(address(module)), "Module not registered");

        // Execute module functions that write to various storage slots
        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setSimpleValue(uint256)", 12345)
        );

        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("pushToArray(uint256)", 111)
        );

        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("pushToArray(uint256)", 222)
        );

        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setMapping(address,uint256)", address(0x5555), 9999)
        );

        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setNestedMapping(address,uint256,uint256)", address(0x6666), 42, 7777)
        );

        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setStruct(uint256,string,uint256)", 1, "test_struct", 8888)
        );

        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setString(string)", "hello_from_module")
        );

        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setBytes(bytes)", hex"deadbeef")
        );

        // Complex operation with multiple storage writes
        vm.prank(address(entryPoint));
        bytes memory complexResult = accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("complexOperation(uint256,uint256,address)", 100, 200, address(0x7777))
        );
        uint256 complexReturnValue = abi.decode(complexResult, (uint256));
        assertEq(complexReturnValue, 300, "Complex operation return value mismatch");

        // ============ STEP 5: Verify Account State is NOT Corrupted ============

        // All V1 state should remain intact
        assertEq(accountV2.owner(), expectedOwner, "Owner corrupted after module execution");
        assertEq(accountV2.clientId(), clientId, "ClientId corrupted after module execution");
        assertEq(uint256(accountV2.ownerType()), uint256(OwnerType.Evm), "OwnerType corrupted after module execution");
        assertTrue(accountV2.isRootSigner(rootSigner1), "rootSigner1 corrupted");
        assertTrue(accountV2.isRootSigner(rootSigner2), "rootSigner2 corrupted");
        assertTrue(accountV2.isRootSigner(rootSigner3), "rootSigner3 corrupted");
        assertTrue(accountV2.passkeySigners(Passkey.toKey(passkey1)), "passkey1 corrupted");
        assertTrue(accountV2.passkeySigners(Passkey.toKey(passkey2)), "passkey2 corrupted");
        assertEq(accountV2.passkeySignerCount(), 2, "Passkey count corrupted");
        assertEq(accountV2.getDeposit(), depositAmount, "Deposit corrupted");

        // Module registration state should be intact
        assertTrue(accountV2.isModuleRegistered(address(module)), "Module registration lost");

        // ============ STEP 6: Verify V1 Functionality Still Works ============

        // Add new root signer
        address newRootSigner = address(0x8888);
        vm.prank(owner);
        accountV2.addRootSigner(newRootSigner);
        assertTrue(accountV2.isRootSigner(newRootSigner), "Cannot add new root signer after upgrade");

        // Remove a root signer
        vm.prank(owner);
        accountV2.removeRootSigner(rootSigner3);
        assertFalse(accountV2.isRootSigner(rootSigner3), "Cannot remove root signer after upgrade");
        assertTrue(accountV2.isRootSigner(rootSigner1), "Other root signers affected");
        assertTrue(accountV2.isRootSigner(rootSigner2), "Other root signers affected");

        // Add new passkey signer
        Passkey.PublicKey memory newPasskey = Passkey.PublicKey({x: 555555, y: 666666});
        vm.prank(owner);
        accountV2.addPasskeySigner(newPasskey);
        assertTrue(accountV2.passkeySigners(Passkey.toKey(newPasskey)), "Cannot add new passkey after upgrade");
        assertEq(accountV2.passkeySignerCount(), 3, "Passkey count not updated");

        // Remove a passkey signer
        vm.prank(owner);
        accountV2.removePasskeySigner(passkey1);
        assertFalse(accountV2.passkeySigners(Passkey.toKey(passkey1)), "Cannot remove passkey after upgrade");
        assertEq(accountV2.passkeySignerCount(), 2, "Passkey count not updated after removal");

        // Test deposit operations
        vm.deal(address(accountV2), 10 ether);
        vm.prank(address(accountV2));
        accountV2.addDeposit{value: 2 ether}();
        assertEq(accountV2.getDeposit(), depositAmount + 2 ether, "Cannot add deposit after upgrade");

        // ============ STEP 7: Execute More Module Operations ============

        // Increment simple value multiple times
        for (uint256 i = 0; i < 5; i++) {
            vm.prank(address(entryPoint));
            accountV2.executeModuleCall(
                address(module),
                abi.encodeWithSignature("incrementSimpleValue()")
            );
        }

        // Push more values to array
        for (uint256 i = 0; i < 3; i++) {
            vm.prank(address(entryPoint));
            accountV2.executeModuleCall(
                address(module),
                abi.encodeWithSignature("pushToArray(uint256)", 1000 + i)
            );
        }

        // ============ STEP 8: Final State Verification ============

        // Verify all account state is still intact
        assertEq(accountV2.owner(), expectedOwner, "Final: Owner corrupted");
        assertEq(accountV2.clientId(), clientId, "Final: ClientId corrupted");
        assertTrue(accountV2.isRootSigner(rootSigner1), "Final: rootSigner1 corrupted");
        assertTrue(accountV2.isRootSigner(rootSigner2), "Final: rootSigner2 corrupted");
        assertTrue(accountV2.isRootSigner(newRootSigner), "Final: newRootSigner lost");
        assertFalse(accountV2.isRootSigner(rootSigner3), "Final: removed root signer reappeared");
        assertTrue(accountV2.passkeySigners(Passkey.toKey(passkey2)), "Final: passkey2 corrupted");
        assertTrue(accountV2.passkeySigners(Passkey.toKey(newPasskey)), "Final: newPasskey lost");
        assertFalse(accountV2.passkeySigners(Passkey.toKey(passkey1)), "Final: removed passkey reappeared");
        assertEq(accountV2.passkeySignerCount(), 2, "Final: passkey count incorrect");
        assertTrue(accountV2.isModuleRegistered(address(module)), "Final: module registration lost");

        // ============ STEP 9: Test Module Unregistration ============

        vm.prank(owner);
        accountV2.unregisterModule(address(module));
        assertFalse(accountV2.isModuleRegistered(address(module)), "Module still registered after unregistration");

        // Verify account state is still intact after unregistration
        assertEq(accountV2.owner(), expectedOwner, "Owner corrupted after module unregistration");
        assertEq(accountV2.passkeySignerCount(), 2, "Passkey count changed after module unregistration");

        // ============ STEP 10: Verify Module Cannot Be Executed After Unregistration ============

        vm.expectRevert("Module not registered");
        vm.prank(address(entryPoint));
        accountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setSimpleValue(uint256)", 99999)
        );
    }

    function test_V1ToV2UpgradeNonEvmAccount() public {
        // Test upgrade with non-EVM account type
        OmniAccountV1 v1Implementation = new OmniAccountV1(entryPoint);
        bytes32 substrateOwner = keccak256("substrate_owner");

        OmniAccountV1 substrateAccount = OmniAccountV1(
            payable(
                new ERC1967Proxy{salt: substrateOwner}(
                    address(v1Implementation),
                    abi.encodeCall(OmniAccountV1.initialize, (substrateOwner, OwnerType.Substrate, clientId, rootSigner1))
                )
            )
        );

        // Setup state
        vm.prank(rootSigner1);
        substrateAccount.addRootSigner(rootSigner2);

        // Verify V1 state
        assertEq(substrateAccount.owner(), substrateOwner);
        assertEq(uint256(substrateAccount.ownerType()), uint256(OwnerType.Substrate));
        assertTrue(substrateAccount.isRootSigner(rootSigner1));
        assertTrue(substrateAccount.isRootSigner(rootSigner2));

        // Upgrade
        vm.prank(rootSigner1);
        UUPSUpgradeable(address(substrateAccount)).upgradeToAndCall(address(accountV2Implementation), "");

        OmniAccountV2 substrateAccountV2 = OmniAccountV2(payable(address(substrateAccount)));

        // Verify state preserved
        assertEq(substrateAccountV2.owner(), substrateOwner, "Substrate owner lost");
        assertEq(uint256(substrateAccountV2.ownerType()), uint256(OwnerType.Substrate), "OwnerType changed");
        assertTrue(substrateAccountV2.isRootSigner(rootSigner1), "rootSigner1 lost");
        assertTrue(substrateAccountV2.isRootSigner(rootSigner2), "rootSigner2 lost");
        assertEq(substrateAccountV2.version(), "2.0.0", "Version not updated");

        // Test module functionality
        vm.prank(rootSigner1);
        substrateAccountV2.registerModule(address(module));
        assertTrue(substrateAccountV2.isModuleRegistered(address(module)));

        // Execute module
        vm.prank(address(entryPoint));
        substrateAccountV2.executeModuleCall(
            address(module),
            abi.encodeWithSignature("setSimpleValue(uint256)", 777)
        );

        // Verify account state intact
        assertEq(substrateAccountV2.owner(), substrateOwner, "Substrate owner corrupted");
        assertTrue(substrateAccountV2.isRootSigner(rootSigner1), "rootSigner1 corrupted");
    }
}
