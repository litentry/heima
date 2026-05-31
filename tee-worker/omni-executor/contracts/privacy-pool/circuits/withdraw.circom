pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/mux1.circom";

// Incremental Merkle tree membership proof using Poseidon hashing.
// Verifies that `leaf` is at position `leaf_index` in a tree of depth `levels`
// with the given `root`, using the supplied sibling path.
template MerkleProof(levels) {
    signal input leaf;
    signal input leaf_index;
    signal input path_elements[levels];

    signal output root;

    // Decompose leaf_index into bits (path_indices)
    signal bits[levels];
    var idx = leaf_index;
    for (var i = 0; i < levels; i++) {
        bits[i] <-- (idx >> i) & 1;
        bits[i] * (1 - bits[i]) === 0; // bits[i] is binary
    }

    component hashers[levels];
    component muxes[levels];
    signal hashes[levels + 1];
    hashes[0] <== leaf;

    for (var i = 0; i < levels; i++) {
        // bits[i] == 0: current node is left child → hash(current, sibling)
        // bits[i] == 1: current node is right child → hash(sibling, current)
        muxes[i] = MultiMux1(2);
        muxes[i].c[0][0] <== hashes[i];       // left when bit=0
        muxes[i].c[0][1] <== path_elements[i]; // left when bit=1
        muxes[i].c[1][0] <== path_elements[i]; // right when bit=0
        muxes[i].c[1][1] <== hashes[i];        // right when bit=1
        muxes[i].s <== bits[i];

        hashers[i] = Poseidon(2);
        hashers[i].inputs[0] <== muxes[i].out[0];
        hashers[i].inputs[1] <== muxes[i].out[1];
        hashes[i + 1] <== hashers[i].out;
    }

    root <== hashes[levels];
}

// Privacy pool withdrawal circuit (Groth16, BN254).
//
// Public inputs:  root, nullifier, commitment
// Private inputs: secret_lo, secret_hi, leaf_index, amount, path_elements[levels]
//
// Proves:
//   1. commitment = Poseidon(secret_lo | (secret_hi << 128), amount)
//   2. nullifier  = Poseidon(secret_lo | (secret_hi << 128), leaf_index)
//   3. commitment is a leaf at leaf_index in the Merkle tree with the given root
//
// Secret is split into two 128-bit halves to stay within the BN254 scalar field
// while preserving full 256-bit entropy. Combined as: secret = secret_lo + secret_hi * 2^128.
template PrivacyPoolWithdraw(levels) {
    // Private inputs
    signal input secret_lo;          // lower 128 bits of the 256-bit secret
    signal input secret_hi;          // upper 128 bits of the 256-bit secret
    signal input leaf_index;         // position in the Merkle tree (uint32)
    signal input amount;             // token amount (private — bound by commitment)
    signal input path_elements[levels]; // Merkle siblings

    // Public inputs
    signal input root;
    signal input nullifier;
    signal input commitment;

    // Combine secret halves into a single field element
    // secret = secret_lo + secret_hi * 2^128
    signal secret;
    secret <== secret_lo + secret_hi * (1 << 128);

    // Verify commitment = Poseidon(secret, amount)
    component commit_hash = Poseidon(2);
    commit_hash.inputs[0] <== secret;
    commit_hash.inputs[1] <== amount;
    commit_hash.out === commitment;

    // Verify nullifier = Poseidon(secret, leaf_index)
    component null_hash = Poseidon(2);
    null_hash.inputs[0] <== secret;
    null_hash.inputs[1] <== leaf_index;
    null_hash.out === nullifier;

    // Verify Merkle membership
    component merkle = MerkleProof(levels);
    merkle.leaf <== commitment;
    merkle.leaf_index <== leaf_index;
    for (var i = 0; i < levels; i++) {
        merkle.path_elements[i] <== path_elements[i];
    }
    merkle.root === root;
}

component main {public [root, nullifier, commitment]} = PrivacyPoolWithdraw(20);
