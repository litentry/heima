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
    bytes32 oa;

    function setUp() public {
        entryPoint = new EntryPoint();
        smartAccountFactory = new SmartAccountFactory(entryPoint);
        oa = TestUtils.prepare_evm_oa(ownerAddress);
    }

    function test_UserOpHash() public view {
        bytes32 expectedHash = 0x994a57baf67923f51eee12d1e4efac5fa67d66a996d97988cb6892e0732e98b9;

        address sender = 0x922D6956C99E12DFeB3224DEA977D0939758A1Fe;
        bytes memory initCode = "";

        PackedUserOperation memory op = TestUtils.preparePackedOp(sender, initCode);
        assertEq(entryPoint.getUserOpHash(op), expectedHash);
    }

    function test_HandleOps() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");

        bytes32 aliceOa = TestUtils.prepare_evm_oa(alice);

        address factory = address(smartAccountFactory);
        address sender = 0x0eAfeE130Ab1F6261885eE7080f9e8B2513111d4;        

        fundAccountOnEntryPoint(sender, entryPoint);

        bytes memory initCode =
            abi.encodePacked(factory, abi.encodeCall(smartAccountFactory.createAccount, (aliceOa, rootAddress)));
        address payable beneficiary = payable(0x0000000000000000000000000000000000000002);

        entryPoint.getSenderAddress(initCode);

        PackedUserOperation[] memory ops = new PackedUserOperation[](1);
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePk, packedOpHash);
        packedOp.signature = abi.encodePacked(r, s, v);

        ops[0] = packedOp;
        entryPoint.handleOps(ops, beneficiary);
    }

    function fundAccountOnEntryPoint(address to, EntryPoint ep) internal {
        (bool callSuccess, bytes memory data) =
            address(ep).call{value: 1000000000000000}(abi.encodeCall(ep.depositTo, (to)));
    }
}
