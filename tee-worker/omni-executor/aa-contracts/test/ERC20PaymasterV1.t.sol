// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {ERC20PaymasterV1} from "../src/core/ERC20PaymasterV1.sol";
import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {TestToken} from "../src/TestToken.sol";
import {IPaymaster} from "../src/interfaces/IPaymaster.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";

contract ERC20PaymasterV1Test is Test {
    ERC20PaymasterV1 public paymaster;
    EntryPointV1 public entryPoint;
    TestToken public testToken;

    address bundler1 = makeAddr("bundler1");
    address bundler2 = makeAddr("bundler2");
    address unauthorizedBundler = makeAddr("unauthorizedBundler");
    address user = makeAddr("user");
    address beneficiary = makeAddr("beneficiary");
    address owner = makeAddr("owner");

    uint256 constant EXCHANGE_RATE = 1500e18; // 1500 tokens per 1 ETH
    uint256 constant VALID_UNTIL = type(uint256).max; // Far future
    uint256 constant VALID_AFTER = 0; // No restriction

    event AuthorizedBundlerUpdated(address indexed bundler, bool authorized);
    event BeneficiaryUpdated(address indexed oldBeneficiary, address indexed newBeneficiary);
    event UserOpSponsored(
        address indexed account,
        address indexed token,
        uint256 actualGasCost,
        uint256 tokenAmount
    );
    event TokensWithdrawn(address indexed token, address indexed to, uint256 amount);

    function setUp() public {
        entryPoint = new EntryPointV1();
        testToken = new TestToken("Test Token", "TEST", 18);

        vm.prank(owner);
        paymaster = new ERC20PaymasterV1(entryPoint, bundler1);

        // Fund the paymaster with ETH
        vm.deal(address(paymaster), 10 ether);
        paymaster.deposit{value: 5 ether}();

        // Mint tokens to user
        testToken.mint(user, 100000e18);
    }

    function test_Constructor() public view {
        assertTrue(paymaster.authorizedBundlers(bundler1));
        assertEq(address(paymaster.entryPoint()), address(entryPoint));
        assertEq(paymaster.owner(), owner);
        assertEq(paymaster.beneficiary(), address(paymaster));
        assertEq(paymaster.version(), "1.0.0");
    }

    function test_ConstructorSetsInitialState() public {
        vm.prank(owner);
        ERC20PaymasterV1 newPaymaster = new ERC20PaymasterV1(entryPoint, bundler2);
        
        // Check that initial state is set correctly
        assertTrue(newPaymaster.authorizedBundlers(bundler2));
        assertEq(newPaymaster.beneficiary(), address(newPaymaster));
        assertEq(newPaymaster.owner(), owner);
        assertEq(address(newPaymaster.entryPoint()), address(entryPoint));
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
        vm.prank(unauthorizedBundler);
        vm.expectRevert();
        paymaster.setAuthorizedBundler(bundler2, true);
    }

    function test_SetBeneficiary() public {
        vm.expectEmit(true, true, false, true);
        emit BeneficiaryUpdated(address(paymaster), beneficiary);

        vm.prank(owner);
        paymaster.setBeneficiary(beneficiary);

        assertEq(paymaster.beneficiary(), beneficiary);
    }

    function test_SetBeneficiary_OnlyOwner() public {
        vm.prank(unauthorizedBundler);
        vm.expectRevert();
        paymaster.setBeneficiary(beneficiary);
    }

    function test_SetBeneficiary_InvalidAddress() public {
        vm.prank(owner);
        vm.expectRevert("Invalid beneficiary");
        paymaster.setBeneficiary(address(0));
    }

    function test_ValidatePaymasterUserOp_UnauthorizedBundler() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);
        
        vm.prank(address(entryPoint), unauthorizedBundler);
        vm.expectRevert(ERC20PaymasterV1.UnauthorizedBundler.selector);
        
        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
    }

    function test_ValidatePaymasterUserOp_InsufficientDeposit() public {
        // Withdraw all funds from paymaster
        uint256 currentDeposit = paymaster.getDeposit();
        vm.prank(owner);
        paymaster.withdrawTo(payable(owner), currentDeposit);

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);
        
        vm.prank(address(entryPoint), bundler1);
        vm.expectRevert(ERC20PaymasterV1.InsufficientDeposit.selector);
        
        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
    }

    function test_ValidatePaymasterUserOp_InvalidPaymasterData() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = abi.encodePacked(address(paymaster)); // Too short
        
        vm.prank(address(entryPoint), bundler1);
        vm.expectRevert(ERC20PaymasterV1.InvalidPaymasterData.selector);
        
        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
    }

    function test_ValidatePaymasterUserOp_TimestampValidation() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        
        // Test validAfter in future
        userOp.paymasterAndData = _encodePaymasterDataWithTime(
            address(testToken), 
            EXCHANGE_RATE, 
            block.timestamp + 1000, // validAfter in future
            type(uint256).max  // validUntil far in future
        );
        
        vm.prank(address(entryPoint), bundler1);
        (, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
        
        assertEq(validationData, 1); // Should reject
    }

    function test_ValidatePaymasterUserOp_InvalidExchangeRate() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), 0); // Zero exchange rate
        
        vm.prank(address(entryPoint), bundler1);
        vm.expectRevert(ERC20PaymasterV1.InvalidExchangeRate.selector);
        
        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
    }

    function test_ValidatePaymasterUserOp_NativeToken() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(0), EXCHANGE_RATE); // Native token
        
        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
        
        assertEq(validationData, 0); // Should succeed
        assertTrue(context.length > 0);
    }

    function test_ValidatePaymasterUserOp_ERC20_InsufficientBalance() public {
        address poorUser = makeAddr("poorUser"); // Use a new user with no tokens
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(poorUser, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);
        
        // Verify poorUser has no tokens
        assertEq(testToken.balanceOf(poorUser), 0);
        
        vm.prank(address(entryPoint), bundler1);
        vm.expectRevert(ERC20PaymasterV1.InsufficientTokenBalance.selector);
        
        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
    }

    function test_ValidatePaymasterUserOp_ERC20_InsufficientAllowance() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);
        
        vm.prank(address(entryPoint), bundler1);
        vm.expectRevert(ERC20PaymasterV1.InsufficientTokenAllowance.selector);
        
        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
    }

    function test_ValidatePaymasterUserOp_ERC20_Success() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);
        
        // Approve tokens
        vm.prank(user);
        testToken.approve(address(paymaster), type(uint256).max);
        
        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
        
        assertEq(validationData, 0); // Should succeed
        assertTrue(context.length > 0);
        
        // Check tokens were transferred to beneficiary
        assertGt(testToken.balanceOf(address(paymaster)), 0);
    }

    function test_PostOp_NativeToken() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(0), EXCHANGE_RATE);
        
        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, ) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
        
        vm.expectEmit(true, true, false, false);
        emit UserOpSponsored(user, address(0), 0.5 ether, 0);
        
        vm.prank(address(entryPoint));
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, 0.5 ether, 1000000000);
    }

    function test_PostOp_ERC20_WithRefund() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);
        
        // Approve tokens
        vm.prank(user);
        testToken.approve(address(paymaster), type(uint256).max);
        
        uint256 userBalanceBefore = testToken.balanceOf(user);
        
        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, ) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
        
        uint256 userBalanceAfterValidation = testToken.balanceOf(user);
        uint256 chargedAmount = userBalanceBefore - userBalanceAfterValidation;
        
        // Simulate actual gas cost being less than max cost
        uint256 actualGasCost = 0.5 ether;
        uint256 expectedActualTokenCost = (actualGasCost * EXCHANGE_RATE) / 1e18;
        // Calculate expected refund but don't use it (refund is handled internally)
        
        vm.expectEmit(true, true, false, false);
        emit UserOpSponsored(user, address(testToken), actualGasCost, expectedActualTokenCost);
        
        vm.prank(address(entryPoint));
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, actualGasCost, 1000000000);
        
        // Check refund was issued
        uint256 userBalanceAfterRefund = testToken.balanceOf(user);
        assertEq(userBalanceAfterRefund, userBalanceBefore - expectedActualTokenCost);
    }

    function test_WithdrawTokens_ERC20() public {
        // First, get some tokens in the paymaster
        vm.prank(user);
        testToken.approve(address(paymaster), type(uint256).max);
        
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);
        
        vm.prank(address(entryPoint), bundler1);
        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
        
        uint256 paymasterBalance = testToken.balanceOf(address(paymaster));
        assertGt(paymasterBalance, 0);
        
        vm.expectEmit(true, true, false, true);
        emit TokensWithdrawn(address(testToken), owner, paymasterBalance);
        
        vm.prank(owner);
        paymaster.withdrawTokens(address(testToken), owner, paymasterBalance);
        
        assertEq(testToken.balanceOf(owner), paymasterBalance);
        assertEq(testToken.balanceOf(address(paymaster)), 0);
    }

    function test_WithdrawTokens_ETH() public {
        uint256 amount = 1 ether;
        vm.deal(address(paymaster), amount);
        
        vm.expectEmit(true, true, false, true);
        emit TokensWithdrawn(address(0), owner, amount);
        
        vm.prank(owner);
        paymaster.withdrawTokens(address(0), owner, amount);
        
        assertEq(owner.balance, amount);
    }

    function test_WithdrawTokens_OnlyOwner() public {
        vm.prank(unauthorizedBundler);
        vm.expectRevert();
        paymaster.withdrawTokens(address(testToken), unauthorizedBundler, 1000);
    }

    function test_WithdrawTokens_BeneficiaryNotContract() public {
        vm.prank(owner);
        paymaster.setBeneficiary(beneficiary);
        
        vm.prank(owner);
        vm.expectRevert("Beneficiary is not this contract");
        paymaster.withdrawTokens(address(testToken), owner, 1000);
    }

    function test_ReceiveETH() public {
        uint256 balanceBefore = paymaster.getDeposit();
        
        vm.deal(user, 1 ether);
        (bool success,) = address(paymaster).call{value: 1 ether}("");
        assertTrue(success);
        
        assertEq(paymaster.getDeposit(), balanceBefore + 1 ether);
    }

    // Helper functions
    function _encodePaymasterData(address token, uint256 exchangeRate) internal view returns (bytes memory) {
        return _encodePaymasterDataWithTime(token, exchangeRate, VALID_AFTER, VALID_UNTIL);
    }

    function _encodePaymasterDataWithTime(
        address token, 
        uint256 exchangeRate, 
        uint256 validAfter, 
        uint256 validUntil
    ) internal view returns (bytes memory) {
        return abi.encodePacked(
            address(paymaster),
            bytes12(0), // Pad token address to 32 bytes
            token,
            exchangeRate,
            validUntil,
            validAfter
        );
    }
}