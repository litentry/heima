// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {UserOpSigner} from "../src/interfaces/UserOpSigner.sol";
import {OmniAccountFactoryV1} from "../src/accounts/OmniAccountFactoryV1.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import {TestUtils} from "./TestUtils.sol";

contract EntryPointTest is Test {
    EntryPointV1 public entryPoint;
    OmniAccountFactoryV1 public omniAccountFactory;
    address ownerAddress = 0x0000000000000000000000000000000000000000;
    address rootAddress = 0x0000000000000000000000000000000000000001;
    bytes clientId = bytes("test_client");
    bytes32 oa;

    function setUp() public {
        entryPoint = new EntryPointV1();
        omniAccountFactory = new OmniAccountFactoryV1(entryPoint);
        oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
    }

    function test_UserOpHash() public view {
        bytes32 expectedHash = 0x022c5458b7cbe770dbd845f7e6759a8711818cde05ade6ebaa799d1a6d4f174d;

        address sender = 0x922D6956C99E12DFeB3224DEA977D0939758A1Fe;
        bytes memory initCode = "";

        PackedUserOperation memory op = TestUtils.preparePackedOp(sender, initCode);
        assertEq(entryPoint.getUserOpHash(op), expectedHash);
    }

    function test_HandleOps() public {
        (address alice, uint256 alicePk) = makeAddrAndKey("alice");

        bytes32 aliceOa = TestUtils.prepare_evm_oa(alice, clientId);

        address factory = address(omniAccountFactory);
        address sender = omniAccountFactory.getAddress(aliceOa, OwnerType.Evm, clientId, rootAddress);

        fundAccountOnEntryPoint(sender, entryPoint);

        bytes memory initCode = abi.encodePacked(
            factory, abi.encodeCall(omniAccountFactory.createAccount, (aliceOa, OwnerType.Evm, clientId, rootAddress))
        );

        address payable beneficiary = payable(0x0000000000000000000000000000000000000002);

        PackedUserOperation[] memory ops = new PackedUserOperation[](1);
        PackedUserOperation memory packedOp = TestUtils.preparePackedOp(sender, initCode);
        bytes32 packedOpHash = entryPoint.getUserOpHash(packedOp);
        // sign userOp
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePk, packedOpHash);
        packedOp.signature = abi.encodePacked(uint8(UserOpSigner.Owner), r, s, v);

        ops[0] = packedOp;
        entryPoint.handleOps(ops, beneficiary);
    }

    function fundAccountOnEntryPoint(address to, EntryPointV1 ep) internal {
        (bool callSuccess,) = address(ep).call{value: 10000000000000000}(abi.encodeCall(ep.depositTo, (to)));
        require(callSuccess, "call failed");
    }
}
