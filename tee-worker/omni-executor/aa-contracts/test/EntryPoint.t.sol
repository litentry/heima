// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {SmartAccountFactory} from "../src/accounts/SmartAccountFactory.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";

contract EntryPointTest is Test {
    EntryPoint public entryPoint;
    SmartAccountFactory public smartAccountFactory;
    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes32 clientId = 0x0000000000000000000000000000000000000000000000000000000000000000;
    bytes32 oa;

    function setUp() public {
        entryPoint = new EntryPoint();
        smartAccountFactory = new SmartAccountFactory(entryPoint);
        oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
    }

    function test_UserOpHash() public view {
        bytes32 expectedHash = 0x994a57baf67923f51eee12d1e4efac5fa67d66a996d97988cb6892e0732e98b9;

        address sender = 0x922D6956C99E12DFeB3224DEA977D0939758A1Fe;
        bytes memory initCode = "";

        address sessionAccount = 0x0000000000000000000000000000000000000000;
        bytes memory sessionAccountProof = "";
        PackedUserOperation memory op =
            TestUtils.preparePackedOp(sender, initCode, sessionAccount, 2, sessionAccountProof);
        assertEq(entryPoint.getUserOpHash(op), expectedHash);
    }

    function test_HandleOps() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");

        bytes32 aliceOa = TestUtils.prepare_evm_oa(alice, clientId);

        address factory = address(smartAccountFactory);
        address sender = 0xE4bcd5D91207C36CCA09fB7BCB4A4FCF693A1089;

        fundAccountOnEntryPoint(sender, entryPoint);

        bytes memory initCode = abi.encodePacked(
            factory, abi.encodeCall(smartAccountFactory.createAccount, (aliceOa, clientId, rootAddress))
        );
        address payable beneficiary = payable(0x0000000000000000000000000000000000000002);

        PackedUserOperation[] memory ops = new PackedUserOperation[](1);
        address sessionAccount = 0x0000000000000000000000000000000000000000;
        bytes memory sessionAccountProof = "";
        PackedUserOperation memory packedOp =
            TestUtils.preparePackedOp(sender, initCode, sessionAccount, 2, sessionAccountProof);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePk, packedOpHash);
        packedOp.signature = abi.encodePacked(r, s, v);

        ops[0] = packedOp;
        entryPoint.handleOps(ops, beneficiary);
    }

    function fundAccountOnEntryPoint(address to, EntryPoint ep) internal {
        (bool callSuccess,) = address(ep).call{value: 1000000000000000}(abi.encodeCall(ep.depositTo, (to)));
        require(callSuccess, "call failed");
    }
}
