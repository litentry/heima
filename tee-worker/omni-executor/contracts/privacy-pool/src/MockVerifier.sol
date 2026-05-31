// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @notice Mock verifier that accepts any proof — used for Phase 1-4 before real Groth16 circuit
contract MockVerifier {
    function verifyProof(bytes calldata, uint256[3] calldata) external pure returns (bool) {
        return true;
    }
}
