// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {IStakeManager} from "../src/interfaces/IStakeManager.sol";

contract StakeManagerTest is Test {
    EntryPoint public entryPoint;
    address public alice;
    address public bob;
    address payable public withdrawAddress;

    uint256 constant STAKE_AMOUNT = 1 ether;
    uint256 constant DEPOSIT_AMOUNT = 0.5 ether;
    uint32 constant UNSTAKE_DELAY = 86400; // 1 day

    event Deposited(address indexed account, uint256 totalDeposit);
    event Withdrawn(address indexed account, address withdrawAddress, uint256 amount);
    event StakeLocked(address indexed account, uint256 totalStaked, uint256 unstakeDelaySec);
    event StakeUnlocked(address indexed account, uint256 withdrawTime);
    event StakeWithdrawn(address indexed account, address withdrawAddress, uint256 amount);

    function setUp() public {
        entryPoint = new EntryPoint();
        alice = makeAddr("alice");
        bob = makeAddr("bob");
        withdrawAddress = payable(makeAddr("withdrawAddress"));

        // Fund test accounts
        vm.deal(alice, 10 ether);
        vm.deal(bob, 10 ether);
    }

    // ============ Core Staking Tests ============

    function test_AddStake() public {
        vm.startPrank(alice);

        vm.expectEmit(true, false, false, true);
        emit StakeLocked(alice, STAKE_AMOUNT, UNSTAKE_DELAY);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, STAKE_AMOUNT);
        assertEq(info.unstakeDelaySec, UNSTAKE_DELAY);
        assertTrue(info.staked);
        assertEq(info.withdrawTime, 0);

        vm.stopPrank();
    }

    function test_AddStakeWithInsufficientValue() public {
        vm.prank(alice);
        vm.expectRevert("no stake specified");
        entryPoint.addStake{value: 0}(UNSTAKE_DELAY);
    }

    function test_AddStakeMultipleTimes() public {
        vm.startPrank(alice);

        // First stake
        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        // Second stake with same delay
        uint256 additionalStake = 0.5 ether;
        entryPoint.addStake{value: additionalStake}(UNSTAKE_DELAY);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, STAKE_AMOUNT + additionalStake);
        assertEq(info.unstakeDelaySec, UNSTAKE_DELAY);

        vm.stopPrank();
    }

    function test_AddStakeWithZeroDelay() public {
        vm.prank(alice);
        vm.expectRevert("must specify unstake delay");
        entryPoint.addStake{value: STAKE_AMOUNT}(0);
    }

    function test_AddStakeWithDecreasingDelay() public {
        vm.startPrank(alice);

        // First stake with longer delay
        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        // Try to add stake with shorter delay
        vm.expectRevert("cannot decrease unstake time");
        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY / 2);

        vm.stopPrank();
    }

    function test_UnlockStake() public {
        vm.startPrank(alice);

        // First stake
        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        uint256 expectedWithdrawTime = block.timestamp + UNSTAKE_DELAY;

        vm.expectEmit(true, false, false, true);
        emit StakeUnlocked(alice, expectedWithdrawTime);

        entryPoint.unlockStake();

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.withdrawTime, expectedWithdrawTime);
        assertFalse(info.staked);

        vm.stopPrank();
    }

    function test_UnlockStakeWhenNotStaked() public {
        vm.prank(alice);
        vm.expectRevert("not staked");
        entryPoint.unlockStake();
    }

    function test_UnlockStakeWhenAlreadyUnlocking() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        vm.expectRevert("already unstaking");
        entryPoint.unlockStake();

        vm.stopPrank();
    }

    function test_WithdrawStake() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        // Fast forward time
        vm.warp(block.timestamp + UNSTAKE_DELAY + 1);

        uint256 balanceBefore = withdrawAddress.balance;

        vm.expectEmit(true, true, false, true);
        emit StakeWithdrawn(alice, withdrawAddress, STAKE_AMOUNT);

        entryPoint.withdrawStake(withdrawAddress);

        assertEq(withdrawAddress.balance, balanceBefore + STAKE_AMOUNT);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, 0);
        assertEq(info.unstakeDelaySec, 0);
        assertEq(info.withdrawTime, 0);

        vm.stopPrank();
    }

    function test_WithdrawStakeBeforeDelay() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        // Try to withdraw before delay
        vm.expectRevert("Stake withdrawal is not due");
        entryPoint.withdrawStake(withdrawAddress);

        vm.stopPrank();
    }

    function test_WithdrawStakeAfterDelay() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        // Fast forward exactly to withdrawal time
        vm.warp(block.timestamp + UNSTAKE_DELAY);

        entryPoint.withdrawStake(withdrawAddress);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, 0);

        vm.stopPrank();
    }

    function test_WithdrawStakeWhenNotUnlocked() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        vm.expectRevert("must call unlockStake() first");
        entryPoint.withdrawStake(withdrawAddress);

        vm.stopPrank();
    }

    function test_WithdrawStakeWithNoStake() public {
        vm.prank(alice);
        vm.expectRevert("No stake to withdraw");
        entryPoint.withdrawStake(withdrawAddress);
    }

    // ============ Stake Information Tests ============

    function test_GetDepositInfo() public {
        vm.prank(alice);
        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, STAKE_AMOUNT);
        assertEq(info.unstakeDelaySec, UNSTAKE_DELAY);
        assertTrue(info.staked);
        assertEq(info.withdrawTime, 0);
    }

    function test_GetDepositInfoForUnstakedAccount() public view {
        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, 0);
        assertEq(info.unstakeDelaySec, 0);
        assertFalse(info.staked);
        assertEq(info.withdrawTime, 0);
    }

    function test_BalanceOf() public {
        vm.prank(alice);
        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);

        assertEq(entryPoint.balanceOf(alice), DEPOSIT_AMOUNT);
    }

    function test_BalanceOfAfterWithdrawal() public {
        vm.startPrank(alice);

        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);
        uint256 withdrawAmount = DEPOSIT_AMOUNT / 2;
        entryPoint.withdrawTo(withdrawAddress, withdrawAmount);

        assertEq(entryPoint.balanceOf(alice), DEPOSIT_AMOUNT - withdrawAmount);

        vm.stopPrank();
    }

    // ============ Deposit Tests ============

    function test_DepositTo() public {
        vm.expectEmit(true, false, false, true);
        emit Deposited(alice, DEPOSIT_AMOUNT);

        vm.prank(alice);
        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);

        assertEq(entryPoint.balanceOf(alice), DEPOSIT_AMOUNT);
    }

    function test_DepositToMultipleTimes() public {
        vm.startPrank(alice);

        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);
        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);

        assertEq(entryPoint.balanceOf(alice), DEPOSIT_AMOUNT * 2);

        vm.stopPrank();
    }

    function test_WithdrawTo() public {
        vm.startPrank(alice);

        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);

        uint256 withdrawAmount = DEPOSIT_AMOUNT / 2;
        uint256 balanceBefore = withdrawAddress.balance;

        vm.expectEmit(true, true, false, true);
        emit Withdrawn(alice, withdrawAddress, withdrawAmount);

        entryPoint.withdrawTo(withdrawAddress, withdrawAmount);

        assertEq(withdrawAddress.balance, balanceBefore + withdrawAmount);
        assertEq(entryPoint.balanceOf(alice), DEPOSIT_AMOUNT - withdrawAmount);

        vm.stopPrank();
    }

    function test_WithdrawToExceedsBalance() public {
        vm.startPrank(alice);

        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);

        vm.expectRevert("Withdraw amount too large");
        entryPoint.withdrawTo(withdrawAddress, DEPOSIT_AMOUNT + 1);

        vm.stopPrank();
    }

    // ============ Time-based Tests ============

    function test_UnstakeDelayEnforcement() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        // Try to withdraw 1 second before delay expires
        vm.warp(block.timestamp + UNSTAKE_DELAY - 1);

        vm.expectRevert("Stake withdrawal is not due");
        entryPoint.withdrawStake(withdrawAddress);

        vm.stopPrank();
    }

    function test_UnstakeDelayAfterTimeWarp() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        // Fast forward way past the delay
        vm.warp(block.timestamp + UNSTAKE_DELAY * 10);

        entryPoint.withdrawStake(withdrawAddress);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, 0);

        vm.stopPrank();
    }

    function test_MultipleUnstakeRequests() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        // Try to unlock again
        vm.expectRevert("already unstaking");
        entryPoint.unlockStake();

        vm.stopPrank();
    }

    function test_ExtendUnstakeDelay() public {
        vm.startPrank(alice);

        // First stake with shorter delay
        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        // Add more stake with longer delay
        uint32 longerDelay = UNSTAKE_DELAY * 2;
        entryPoint.addStake{value: STAKE_AMOUNT}(longerDelay);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.unstakeDelaySec, longerDelay);
        assertEq(info.stake, STAKE_AMOUNT * 2);

        vm.stopPrank();
    }

    // ============ Edge Cases & Security Tests ============

    function test_StakeOverflow() public {
        vm.startPrank(alice);

        // Try to stake maximum uint112 + 1
        uint256 maxStake = type(uint112).max;
        vm.deal(alice, maxStake + 1 ether);

        entryPoint.addStake{value: maxStake}(UNSTAKE_DELAY);

        vm.expectRevert("stake overflow");
        entryPoint.addStake{value: 1 ether}(UNSTAKE_DELAY);

        vm.stopPrank();
    }

    function test_ZeroStakeAmount() public {
        vm.prank(alice);
        vm.expectRevert("no stake specified");
        entryPoint.addStake{value: 0}(UNSTAKE_DELAY);
    }

    function test_MaxStakeAmount() public {
        uint256 maxStake = type(uint112).max;
        vm.deal(alice, maxStake);

        vm.prank(alice);
        entryPoint.addStake{value: maxStake}(UNSTAKE_DELAY);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertEq(info.stake, maxStake);
    }

    function test_StakeAfterUnlock() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        // Add more stake after unlock - should reset to staked state
        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);

        IStakeManager.DepositInfo memory info = entryPoint.getDepositInfo(alice);
        assertTrue(info.staked);
        assertEq(info.withdrawTime, 0);
        assertEq(info.stake, STAKE_AMOUNT * 2);

        vm.stopPrank();
    }

    function test_MultipleWithdrawAttempts() public {
        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        vm.warp(block.timestamp + UNSTAKE_DELAY + 1);

        entryPoint.withdrawStake(withdrawAddress);

        // Try to withdraw again
        vm.expectRevert("No stake to withdraw");
        entryPoint.withdrawStake(withdrawAddress);

        vm.stopPrank();
    }

    function test_ReceiveFunction() public {
        uint256 sendAmount = 1 ether;

        vm.prank(alice);
        (bool success,) = address(entryPoint).call{value: sendAmount}("");
        require(success, "Send failed");

        assertEq(entryPoint.balanceOf(alice), sendAmount);
    }

    function test_DepositToOtherAccount() public {
        vm.prank(alice);
        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(bob);

        assertEq(entryPoint.balanceOf(bob), DEPOSIT_AMOUNT);
        assertEq(entryPoint.balanceOf(alice), 0);
    }

    function test_WithdrawFailure() public {
        // Create a contract that rejects payments
        RejectPayments rejectContract = new RejectPayments();

        vm.startPrank(alice);

        entryPoint.addStake{value: STAKE_AMOUNT}(UNSTAKE_DELAY);
        entryPoint.unlockStake();

        vm.warp(block.timestamp + UNSTAKE_DELAY + 1);

        vm.expectRevert("failed to withdraw stake");
        entryPoint.withdrawStake(payable(address(rejectContract)));

        vm.stopPrank();
    }

    function test_DepositWithdrawFailure() public {
        // Create a contract that rejects payments
        RejectPayments rejectContract = new RejectPayments();

        vm.startPrank(alice);

        entryPoint.depositTo{value: DEPOSIT_AMOUNT}(alice);

        vm.expectRevert("failed to withdraw");
        entryPoint.withdrawTo(payable(address(rejectContract)), DEPOSIT_AMOUNT);

        vm.stopPrank();
    }
}

// Helper contract that rejects all payments
contract RejectPayments {
    receive() external payable {
        revert("Payment rejected");
    }
}
