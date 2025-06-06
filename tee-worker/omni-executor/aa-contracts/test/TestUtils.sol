// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";

library TestUtils {
    function prepare_evm_oa(address account) public returns (bytes32) {
        bytes1 oaType = 0x01;
        return (sha256(abi.encodePacked(oaType, account)));
    }

    function preparePackedOp(address sender, bytes memory initCode)
        internal
        pure
        returns (PackedUserOperation memory)
    {
        uint256 nonce = 0;
        bytes memory callData = "";
        bytes32 accountGasLimits = 0x0000000000000000000000000003d09000000000000000000000000000005b8d;
        uint256 preVerificationGas = 21000;
        bytes32 gasFees = 0x0000000000000000000000003b9aca00000000000000000000000000b2d05e00;
        bytes memory paymasterAndData = "";
        bytes memory signature = "";

        return (
            PackedUserOperation(
                sender,
                nonce,
                initCode,
                callData,
                accountGasLimits,
                preVerificationGas,
                gasFees,
                paymasterAndData,
                signature
            )
        );
    }
}
