// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {SimplePaymaster} from "../src/core/SimplePaymaster.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import {IPaymaster} from "../src/interfaces/IPaymaster.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";

contract SimplePaymasterTest is Test {
    SimplePaymaster public paymaster;
    EntryPoint public entryPoint;

    address bundler1 = makeAddr("bundler1");
    address bundler2 = makeAddr("bundler2");
    address unauthorizedBundler = makeAddr("unauthorizedBundler");
    address omniAccount = makeAddr("omniAccount");
    address owner = makeAddr("owner");

    event AuthorizedBundlerUpdated(address indexed bundler, bool authorized);
    event UserOpSponsored(address indexed account, uint256 actualGasCost);

    function setUp() public {
        entryPoint = new EntryPoint();

        vm.prank(owner);
        paymaster = new SimplePaymaster(entryPoint, bundler1);

        // Fund the paymaster
        vm.deal(address(paymaster), 10 ether);
        paymaster.deposit{value: 5 ether}();
    }

    function test_Constructor() public view {
        assertTrue(paymaster.authorizedBundlers(bundler1));
        assertEq(address(paymaster.entryPoint()), address(entryPoint));
        assertEq(paymaster.owner(), owner);
    }

    function test_ConstructorEmitsEvent() public {
        vm.expectEmit(true, false, false, true);
        emit AuthorizedBundlerUpdated(bundler2, true);

        vm.prank(owner);
        new SimplePaymaster(entryPoint, bundler2);
    }

    function test_SetAuthorizedBundler_Add() public {
        vm.expectEmit(true, false, false, true);
        emit AuthorizedBundlerUpdated(bundler2, true);

        vm.prank(owner);
        paymaster.setAuthorizedBundler(bundler2, true);

        assertTrue(paymaster.authorizedBundlers(bundler2));
    }

    function test_SetAuthorizedBundler_Remove() public {
        vm.expectEmit(true, false, false, true);
        emit AuthorizedBundlerUpdated(bundler1, false);

        vm.prank(owner);
        paymaster.setAuthorizedBundler(bundler1, false);

        assertFalse(paymaster.authorizedBundlers(bundler1));
    }

    function test_SetAuthorizedBundler_OnlyOwner() public {
        vm.prank(bundler1);
        vm.expectRevert();
        paymaster.setAuthorizedBundler(bundler2, true);
    }

    function test_ValidatePaymasterUserOp_AuthorizedBundler() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        // Simulate call from EntryPoint with authorized bundler as tx.origin
        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);

        assertEq(validationData, 0); // Success
        address decodedAccount = abi.decode(context, (address));
        assertEq(decodedAccount, omniAccount);
    }

    function test_ValidatePaymasterUserOp_UnauthorizedBundler() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        // Simulate call from EntryPoint with unauthorized bundler as tx.origin
        vm.prank(address(entryPoint), unauthorizedBundler);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);

        assertEq(validationData, 1); // Failure
        assertEq(context.length, 0);
    }

    function test_ValidatePaymasterUserOp_InsufficientFunds() public {
        // Withdraw all funds as owner
        uint256 currentDeposit = paymaster.getDeposit();
        vm.prank(owner);
        paymaster.withdrawTo(payable(owner), currentDeposit);

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);

        assertEq(validationData, 1); // Failure due to insufficient funds
        assertEq(context.length, 0);
    }

    function test_ValidatePaymasterUserOp_OnlyEntryPoint() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        // Try to call directly (not from EntryPoint)
        vm.prank(bundler1);
        vm.expectRevert("Sender not EntryPoint");
        paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);
    }

    function test_PostOp() public {
        bytes memory context = abi.encode(omniAccount);
        uint256 actualGasCost = 50000;
        uint256 actualUserOpFeePerGas = 1000000000;

        vm.expectEmit(true, false, false, true);
        emit UserOpSponsored(omniAccount, actualGasCost);

        vm.prank(address(entryPoint));
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, actualGasCost, actualUserOpFeePerGas);
    }

    function test_PostOp_OnlyEntryPoint() public {
        bytes memory context = abi.encode(omniAccount);

        vm.prank(bundler1);
        vm.expectRevert("Sender not EntryPoint");
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, 50000, 1000000000);
    }

    function test_Deposit() public {
        uint256 initialDeposit = paymaster.getDeposit();
        uint256 depositAmount = 1 ether;

        paymaster.deposit{value: depositAmount}();

        assertEq(paymaster.getDeposit(), initialDeposit + depositAmount);
    }

    function test_WithdrawTo() public {
        uint256 withdrawAmount = 1 ether;
        uint256 initialBalance = owner.balance;

        vm.prank(owner);
        paymaster.withdrawTo(payable(owner), withdrawAmount);

        assertEq(owner.balance, initialBalance + withdrawAmount);
        assertEq(paymaster.getDeposit(), 5 ether - withdrawAmount);
    }

    function test_WithdrawTo_OnlyOwner() public {
        vm.prank(bundler1);
        vm.expectRevert();
        paymaster.withdrawTo(payable(bundler1), 1 ether);
    }

    function test_WithdrawTo_ZeroAmount() public {
        vm.prank(owner);
        vm.expectRevert("Amount must be greater than 0");
        paymaster.withdrawTo(payable(owner), 0);
    }

    function test_Pause() public {
        vm.prank(owner);
        paymaster.pause();
        assertTrue(paymaster.paused());
    }

    function test_Pause_OnlyOwner() public {
        vm.prank(bundler1);
        vm.expectRevert();
        paymaster.pause();
    }

    function test_Unpause() public {
        // First pause
        vm.prank(owner);
        paymaster.pause();

        // Then unpause
        vm.prank(owner);
        paymaster.unpause();
        assertFalse(paymaster.paused());
    }

    function test_Unpause_OnlyOwner() public {
        // First pause as owner
        vm.prank(owner);
        paymaster.pause();

        // Try to unpause as non-owner
        vm.prank(bundler1);
        vm.expectRevert();
        paymaster.unpause();
    }

    function test_ValidatePaymasterUserOp_WhenPaused() public {
        // Pause the paymaster
        vm.prank(owner);
        paymaster.pause();

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        // Should revert when paused
        vm.prank(address(entryPoint), bundler1);
        vm.expectRevert();
        paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);
    }

    function test_ValidatePaymasterUserOp_WhenUnpaused() public {
        // Pause then unpause the paymaster
        vm.startPrank(owner);
        paymaster.pause();
        paymaster.unpause();
        vm.stopPrank();

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        // Should work normally when unpaused
        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);

        assertEq(validationData, 0); // Success
        address decodedAccount = abi.decode(context, (address));
        assertEq(decodedAccount, omniAccount);
    }

    function test_ReceiveEther() public {
        uint256 initialDeposit = paymaster.getDeposit();
        uint256 sendAmount = 1 ether;

        vm.deal(address(this), sendAmount);
        (bool success,) = address(paymaster).call{value: sendAmount}("");
        require(success, "Send failed");

        assertEq(paymaster.getDeposit(), initialDeposit + sendAmount);
    }

    function test_AddStake() public {
        uint32 unstakeDelay = 86400; // 1 day
        uint256 stakeAmount = 1 ether;

        // Fund the owner to have enough ETH for staking
        vm.deal(owner, stakeAmount);

        vm.prank(owner);
        paymaster.addStake{value: stakeAmount}(unstakeDelay);

        // Check stake was added (would need to check EntryPoint state)
    }

    function test_AddStake_OnlyOwner() public {
        // Fund bundler1 to have ETH for the failed attempt
        vm.deal(bundler1, 1 ether);

        vm.prank(bundler1);
        vm.expectRevert();
        paymaster.addStake{value: 1 ether}(86400);
    }

    function test_UnlockStake() public {
        // First add stake
        vm.deal(owner, 1 ether);
        vm.prank(owner);
        paymaster.addStake{value: 1 ether}(86400);

        // Then unlock
        vm.prank(owner);
        paymaster.unlockStake();
    }

    function test_UnlockStake_OnlyOwner() public {
        vm.prank(bundler1);
        vm.expectRevert();
        paymaster.unlockStake();
    }

    function test_WithdrawStake() public {
        address payable withdrawAddress = payable(makeAddr("withdrawTo"));
        uint32 unstakeDelay = 86400; // 1 day
        uint256 stakeAmount = 1 ether;

        // Fund owner, add stake, unlock it, and wait for delay
        vm.deal(owner, stakeAmount);

        vm.startPrank(owner);
        paymaster.addStake{value: stakeAmount}(unstakeDelay);
        paymaster.unlockStake();
        vm.stopPrank();

        // Fast forward time past the unstake delay
        vm.warp(block.timestamp + unstakeDelay + 1);

        vm.prank(owner);
        paymaster.withdrawStake(withdrawAddress);
    }

    function test_WithdrawStake_OnlyOwner() public {
        vm.prank(bundler1);
        vm.expectRevert();
        paymaster.withdrawStake(payable(bundler1));
    }

    function test_MultipleBundlers() public {
        // Add second bundler
        vm.prank(owner);
        paymaster.setAuthorizedBundler(bundler2, true);

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        // Both bundlers should be able to sponsor operations
        vm.prank(address(entryPoint), bundler1);
        (, uint256 validationData1) = paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);

        vm.prank(address(entryPoint), bundler2);
        (, uint256 validationData2) = paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);

        assertEq(validationData1, 0);
        assertEq(validationData2, 0);
    }

    function test_RemoveBundlerAccess() public {
        // Remove bundler1
        vm.prank(owner);
        paymaster.setAuthorizedBundler(bundler1, false);

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(omniAccount, "");
        bytes32 userOpHash = keccak256("test");
        uint256 maxCost = 1000000;

        // bundler1 should no longer be able to sponsor
        vm.prank(address(entryPoint), bundler1);
        (, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, userOpHash, maxCost);

        assertEq(validationData, 1); // Should fail
    }
}
