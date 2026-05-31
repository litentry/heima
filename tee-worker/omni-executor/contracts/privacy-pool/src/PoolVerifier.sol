// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./Verifier.sol";

/// @notice Adapts the snarkjs-generated Groth16Verifier to the IVerifier interface
/// used by SimplePrivacyPool.
///
/// Proof encoding (bytes calldata proof):
///   ABI-encode(uint256[2] pA, uint256[2][2] pB, uint256[2] pC)
///   = 6 * 32 + 4 * 32 = 320 bytes total
///
/// Public signals order: [root, nullifier, commitment]
contract PoolVerifier {
    Groth16Verifier public immutable groth16;

    constructor() {
        groth16 = new Groth16Verifier();
    }

    /// @param proof ABI-encoded (uint256[2] pA, uint256[2][2] pB, uint256[2] pC)
    /// @param pubSignals [root, nullifier, commitment]
    function verifyProof(bytes calldata proof, uint256[3] calldata pubSignals) external view returns (bool) {
        (uint256[2] memory pA, uint256[2][2] memory pB, uint256[2] memory pC) =
            abi.decode(proof, (uint256[2], uint256[2][2], uint256[2]));
        return groth16.verifyProof(pA, pB, pC, pubSignals);
    }
}
