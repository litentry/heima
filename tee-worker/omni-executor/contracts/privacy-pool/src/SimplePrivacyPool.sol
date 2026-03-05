// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "poseidon-solidity/PoseidonT3.sol";

interface IVerifier {
    function verifyProof(bytes calldata proof, uint256[3] calldata pubSignals) external view returns (bool);
}

/// @notice Simplified privacy pool for B2B invoice payments.
///
/// Buyers deposit tokens with a commitment (hiding the amount on-chain).
/// Sellers withdraw using a nullifier + Groth16 ZK proof.
///
/// All hashing uses Poseidon2 (BN254) so the ZK circuit can prove
/// membership with minimal constraints (~5k vs ~660k for SHA256).
///
/// Commitment: Poseidon([secret_lo | (secret_hi << 128), amount])
/// Nullifier:  Poseidon([secret_lo | (secret_hi << 128), leaf_index])
/// Merkle node: Poseidon([left, right])
///
/// Public signals for verifyProof: [root, nullifier, commitment]
contract SimplePrivacyPool {
    using SafeERC20 for IERC20;

    uint32 public constant LEVELS = 20;

    IERC20 public immutable token;
    IVerifier public immutable verifier;

    // Incremental Merkle tree state (Poseidon field elements)
    uint256[LEVELS] public filledSubtrees;
    uint256 public root;
    uint32 public nextIndex;

    mapping(uint256 => bool) public commitments;
    mapping(uint256 => bool) public nullifiers;
    // Amount per commitment (retrieved without revealing it in withdrawal calldata)
    mapping(uint256 => uint256) public commitmentAmounts;

    event Deposit(uint256 indexed commitment, uint32 indexed leafIndex, uint256 amount);
    event Withdrawal(uint256 indexed nullifier, address indexed recipient, uint256 amount);

    error CommitmentAlreadyUsed();
    error NullifierAlreadyUsed();
    error InvalidProof();
    error ZeroAmount();
    error TreeFull();

    constructor(address _token, address _verifier) {
        token = IERC20(_token);
        verifier = IVerifier(_verifier);
        _initZeroHashes();
    }

    /// @notice Deposit tokens into the pool.
    /// @param commitment Poseidon([secret, amount]) — generated in TEE
    /// @param amount Token amount in smallest units
    function deposit(uint256 commitment, uint256 amount) external {
        if (commitments[commitment]) revert CommitmentAlreadyUsed();
        if (amount == 0) revert ZeroAmount();

        commitments[commitment] = true;
        commitmentAmounts[commitment] = amount;
        uint32 leafIndex = _insert(commitment);

        token.safeTransferFrom(msg.sender, address(this), amount);

        emit Deposit(commitment, leafIndex, amount);
    }

    /// @notice Withdraw tokens using a Groth16 ZK proof.
    /// @param nullifier Poseidon([secret, leaf_index]) — generated in TEE
    /// @param amount Token amount to withdraw
    /// @param recipient Address to receive tokens
    /// @param proof Groth16 proof (ABI-encoded G1,G2,G1 points)
    /// @param pubSignals [root, nullifier, commitment]
    function withdraw(
        uint256 nullifier,
        uint256 amount,
        address recipient,
        bytes calldata proof,
        uint256[3] calldata pubSignals
    ) external {
        if (nullifiers[nullifier]) revert NullifierAlreadyUsed();
        if (!verifier.verifyProof(proof, pubSignals)) revert InvalidProof();

        nullifiers[nullifier] = true;
        token.safeTransfer(recipient, amount);

        emit Withdrawal(nullifier, recipient, amount);
    }

    // ─── Internal Merkle tree ─────────────────────────────────────────────────

    function _insert(uint256 leaf) internal returns (uint32 leafIndex) {
        if (nextIndex >= 2 ** LEVELS) revert TreeFull();

        leafIndex = nextIndex;
        nextIndex++;

        uint256 current = leaf;
        uint32 idx = leafIndex;

        for (uint32 i = 0; i < LEVELS; i++) {
            if (idx % 2 == 0) {
                filledSubtrees[i] = current;
                current = _hashPair(current, _zeros(i));
            } else {
                current = _hashPair(filledSubtrees[i], current);
            }
            idx /= 2;
        }

        root = current;
    }

    function _initZeroHashes() internal {
        uint256 h = 0; // zero leaf
        for (uint32 i = 1; i < LEVELS; i++) {
            h = _hashPair(h, h);
        }
        root = _hashPair(h, h);
    }

    function _hashPair(uint256 left, uint256 right) internal pure returns (uint256) {
        return PoseidonT3.hash([left, right]);
    }

    function _zeros(uint32 level) internal pure returns (uint256) {
        uint256 h = 0;
        for (uint32 i = 0; i < level; i++) {
            h = PoseidonT3.hash([h, h]);
        }
        return h;
    }
}
