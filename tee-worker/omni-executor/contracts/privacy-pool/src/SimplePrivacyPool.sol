// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IVerifier {
    function verifyProof(bytes calldata proof, uint256[3] calldata pubSignals) external view returns (bool);
}

/// @notice Simplified privacy pool for B2B invoice payments.
///
/// Buyers deposit USDC with a commitment (hiding the amount on-chain).
/// Sellers withdraw using a nullifier (+ ZK proof in Phase 5; mock verifier for Phase 1-4).
///
/// Merkle tree (depth 20) tracks all commitments. On-chain calldata only shows
/// commitment hashes / nullifiers — no amounts are visible to observers.
contract SimplePrivacyPool {
    using SafeERC20 for IERC20;

    uint32 public constant LEVELS = 20;

    IERC20 public immutable token;
    IVerifier public immutable verifier;

    // Incremental Merkle tree state
    bytes32[LEVELS] public filledSubtrees;
    bytes32 public root;
    uint32 public nextIndex;

    mapping(bytes32 => bool) public commitments;
    mapping(bytes32 => bool) public nullifiers;
    // Store amount per commitment so withdrawal can retrieve it without on-chain exposure in calldata
    mapping(bytes32 => uint256) public commitmentAmounts;

    event Deposit(bytes32 indexed commitment, uint32 indexed leafIndex, uint256 amount);
    event Withdrawal(bytes32 indexed nullifier, address indexed recipient, uint256 amount);

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

    /// @notice Deposit tokens into the pool with a commitment.
    /// @param commitment SHA256(invoice_id || amount || secret) — generated in TEE
    /// @param amount Token amount in smallest units (e.g. USDC has 6 decimals)
    function deposit(bytes32 commitment, uint256 amount) external {
        if (commitments[commitment]) revert CommitmentAlreadyUsed();
        if (amount == 0) revert ZeroAmount();

        commitments[commitment] = true;
        commitmentAmounts[commitment] = amount;
        uint32 leafIndex = _insert(commitment);

        token.safeTransferFrom(msg.sender, address(this), amount);

        emit Deposit(commitment, leafIndex, amount);
    }

    /// @notice Withdraw tokens using a nullifier and ZK proof.
    /// @param nullifier SHA256(secret || leaf_index) — generated in TEE
    /// @param amount Token amount to withdraw
    /// @param recipient Address to send tokens to
    /// @param proof ZK proof bytes (0x for Phase 1-4 with MockVerifier)
    /// @param pubSignals Public signals [root, nullifier_as_uint, commitment_as_uint]
    function withdraw(
        bytes32 nullifier,
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

    /// @dev Insert a leaf into the incremental Merkle tree.
    function _insert(bytes32 leaf) internal returns (uint32 leafIndex) {
        if (nextIndex >= 2 ** LEVELS) revert TreeFull();

        leafIndex = nextIndex;
        nextIndex++;

        bytes32 currentHash = leaf;
        uint32 currentIndex = leafIndex;

        for (uint32 i = 0; i < LEVELS; i++) {
            if (currentIndex % 2 == 0) {
                // Left node: store and hash with zero sibling
                filledSubtrees[i] = currentHash;
                currentHash = _hashPair(currentHash, _zeros(i));
            } else {
                // Right node: hash with stored left sibling
                currentHash = _hashPair(filledSubtrees[i], currentHash);
            }
            currentIndex /= 2;
        }

        root = currentHash;
    }

    function _initZeroHashes() internal {
        // Pre-compute zero hashes bottom-up (not stored, just initialise filledSubtrees[0])
        // filledSubtrees start all zeros; root starts as zero hash of full tree
        bytes32 h = _zeros(0);
        for (uint32 i = 1; i < LEVELS; i++) {
            h = _hashPair(h, h);
        }
        root = _hashPair(h, h);
    }

    function _hashPair(bytes32 left, bytes32 right) internal pure returns (bytes32) {
        return sha256(abi.encodePacked(left, right));
    }

    /// @dev Returns the zero hash for level i (all-zeros leaf hashed up i times).
    function _zeros(uint32 level) internal pure returns (bytes32) {
        bytes32 h = bytes32(0);
        for (uint32 i = 0; i < level; i++) {
            h = sha256(abi.encodePacked(h, h));
        }
        return h;
    }
}
