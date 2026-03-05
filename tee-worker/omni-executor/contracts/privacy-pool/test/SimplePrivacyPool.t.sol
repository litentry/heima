// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/SimplePrivacyPool.sol";
import "../src/MockVerifier.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "poseidon-solidity/PoseidonT3.sol";

/// @dev Minimal ERC20 for testing
contract MockUSDC is ERC20 {
    constructor() ERC20("Mock USDC", "USDC") {}

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract SimplePrivacyPoolTest is Test {
    SimplePrivacyPool pool;
    MockVerifier verifier;
    MockUSDC usdc;

    address buyer = makeAddr("buyer");
    address seller = makeAddr("seller");

    // Poseidon-based commitment and nullifier for tests
    uint256 constant SECRET = 0xdeadbeef;
    uint256 constant AMOUNT = 1_000_000; // 1 USDC (6 decimals)
    uint256 COMMITMENT;
    uint256 NULLIFIER;

    function setUp() public {
        usdc = new MockUSDC();
        verifier = new MockVerifier();
        pool = new SimplePrivacyPool(address(usdc), address(verifier));

        // commitment = Poseidon(secret, amount)
        COMMITMENT = PoseidonT3.hash([SECRET, AMOUNT]);
        // nullifier = Poseidon(secret, leaf_index=0)
        NULLIFIER = PoseidonT3.hash([SECRET, uint256(0)]);

        usdc.mint(buyer, 100_000_000); // 100 USDC
        vm.prank(buyer);
        usdc.approve(address(pool), type(uint256).max);
    }

    // ─── Deposit tests ────────────────────────────────────────────────────────

    function test_deposit_success() public {
        vm.prank(buyer);
        pool.deposit(COMMITMENT, AMOUNT);

        assertTrue(pool.commitments(COMMITMENT));
        assertEq(pool.nextIndex(), 1);
        assertEq(usdc.balanceOf(address(pool)), AMOUNT);
    }

    function test_deposit_emits_event() public {
        vm.expectEmit(true, true, false, true);
        emit SimplePrivacyPool.Deposit(COMMITMENT, 0, AMOUNT);

        vm.prank(buyer);
        pool.deposit(COMMITMENT, AMOUNT);
    }

    function test_deposit_duplicate_commitment_reverts() public {
        vm.prank(buyer);
        pool.deposit(COMMITMENT, AMOUNT);

        vm.prank(buyer);
        vm.expectRevert(SimplePrivacyPool.CommitmentAlreadyUsed.selector);
        pool.deposit(COMMITMENT, AMOUNT);
    }

    function test_deposit_zero_amount_reverts() public {
        vm.prank(buyer);
        vm.expectRevert(SimplePrivacyPool.ZeroAmount.selector);
        pool.deposit(COMMITMENT, 0);
    }

    // ─── Withdrawal tests ─────────────────────────────────────────────────────

    function test_withdraw_success() public {
        vm.prank(buyer);
        pool.deposit(COMMITMENT, AMOUNT);

        uint256 sellerBalanceBefore = usdc.balanceOf(seller);

        pool.withdraw(NULLIFIER, AMOUNT, seller, "", [uint256(0), uint256(0), uint256(0)]);

        assertEq(usdc.balanceOf(seller), sellerBalanceBefore + AMOUNT);
        assertEq(usdc.balanceOf(address(pool)), 0);
        assertTrue(pool.nullifiers(NULLIFIER));
    }

    function test_withdraw_emits_event() public {
        vm.prank(buyer);
        pool.deposit(COMMITMENT, AMOUNT);

        vm.expectEmit(true, true, false, true);
        emit SimplePrivacyPool.Withdrawal(NULLIFIER, seller, AMOUNT);

        pool.withdraw(NULLIFIER, AMOUNT, seller, "", [uint256(0), uint256(0), uint256(0)]);
    }

    function test_withdraw_nullifier_reuse_reverts() public {
        vm.prank(buyer);
        pool.deposit(COMMITMENT, AMOUNT);

        // Mint more so second withdrawal doesn't revert on balance
        usdc.mint(address(pool), AMOUNT);

        pool.withdraw(NULLIFIER, AMOUNT, seller, "", [uint256(0), uint256(0), uint256(0)]);

        vm.expectRevert(SimplePrivacyPool.NullifierAlreadyUsed.selector);
        pool.withdraw(NULLIFIER, AMOUNT, seller, "", [uint256(0), uint256(0), uint256(0)]);
    }

    // ─── Merkle tree tests ────────────────────────────────────────────────────

    function test_multiple_deposits_increment_index() public {
        uint256 c1 = PoseidonT3.hash([uint256(1), AMOUNT]);
        uint256 c2 = PoseidonT3.hash([uint256(2), AMOUNT]);
        uint256 c3 = PoseidonT3.hash([uint256(3), AMOUNT]);

        vm.startPrank(buyer);
        pool.deposit(c1, AMOUNT);
        pool.deposit(c2, AMOUNT);
        pool.deposit(c3, AMOUNT);
        vm.stopPrank();

        assertEq(pool.nextIndex(), 3);
    }

    function test_root_changes_after_deposit() public {
        uint256 rootBefore = pool.root();

        vm.prank(buyer);
        pool.deposit(COMMITMENT, AMOUNT);

        assertNotEq(pool.root(), rootBefore);
    }
}
