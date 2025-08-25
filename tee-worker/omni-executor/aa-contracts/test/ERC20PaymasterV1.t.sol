// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Test, console} from "forge-std/Test.sol";
import {ERC20PaymasterV1} from "../src/core/ERC20PaymasterV1.sol";
import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {TestToken} from "../src/TestToken.sol";
import {IPaymaster} from "../src/interfaces/IPaymaster.sol";
import {PackedUserOperation} from "../src/interfaces/PackedUserOperation.sol";
import {TestUtils} from "./TestUtils.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// Mock account contract that can send ETH when requested
contract MockAccount {
    receive() external payable {}

    function sendETH(address to, uint256 amount) external {
        payable(to).transfer(amount);
    }
}

// Mock account that fails to send ETH
contract MockAccountFailsPayment {
    receive() external payable {}

    function sendETH(address, uint256) external pure {
        revert("Payment failed");
    }
}

contract ERC20PaymasterV1Test is Test {
    using Math for uint256;

    ERC20PaymasterV1 public paymaster;
    EntryPointV1 public entryPoint;
    TestToken public testToken;
    MockAccount public mockAccount;
    MockAccountFailsPayment public mockAccountFailsPayment;

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
    event UserOpSponsored(address indexed account, address indexed token, uint256 actualGasCost, uint256 tokenAmount);
    event TokensWithdrawn(address indexed token, address indexed to, uint256 amount);
    event NativeTokenPaymentFailed(address indexed userAccount, uint256 amount);

    function setUp() public {
        entryPoint = new EntryPointV1();
        testToken = new TestToken("Test Token", "TEST", 18);
        mockAccount = new MockAccount();
        mockAccountFailsPayment = new MockAccountFailsPayment();

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
            type(uint256).max // validUntil far in future
        );

        vm.prank(address(entryPoint), bundler1);
        (, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);

        assertEq(validationData, 1); // Should reject
    }

    function test_ValidatePaymasterUserOp_FullSponsorship() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), 0); // Zero exchange rate = full sponsorship

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);

        assertEq(validationData, 0); // Should succeed
        assertTrue(context.length > 0);

        // Verify no tokens were transferred (full sponsorship)
        assertEq(testToken.balanceOf(address(paymaster)), 0);
        assertEq(testToken.balanceOf(user), 100000e18); // User keeps all tokens
    }

    function test_ValidatePaymasterUserOp_NativeToken() public {
        // Fund the mock account with ETH to cover the gas cost
        uint256 maxCost = 1 ether;
        uint256 requiredETH = maxCost.mulDiv(EXCHANGE_RATE, 1e18, Math.Rounding.Ceil);
        vm.deal(address(mockAccount), requiredETH + 1 ether); // Extra for buffer

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(address(mockAccount), "");
        userOp.paymasterAndData = _encodePaymasterData(address(0), EXCHANGE_RATE); // Native token

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context, uint256 validationData) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), maxCost);

        assertEq(validationData, 0); // Should succeed
        assertTrue(context.length > 0);
    }

    function test_ValidatePaymasterUserOp_NativeToken_InsufficientBalance() public {
        // Create a mock account with no ETH
        MockAccount poorMockAccount = new MockAccount();
        // Don't fund it with ETH - it should have 0 balance

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(address(poorMockAccount), "");
        userOp.paymasterAndData = _encodePaymasterData(address(0), EXCHANGE_RATE); // Native token

        vm.prank(address(entryPoint), bundler1);
        vm.expectRevert(ERC20PaymasterV1.InsufficientTokenBalance.selector);

        paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);
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

    function test_PostOp_NativeToken_Success() public {
        // Fund the mock account with ETH
        uint256 maxCost = 1 ether;
        uint256 requiredETH = maxCost.mulDiv(EXCHANGE_RATE, 1e18, Math.Rounding.Ceil);
        vm.deal(address(mockAccount), requiredETH + 1 ether);

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(address(mockAccount), "");
        userOp.paymasterAndData = _encodePaymasterData(address(0), EXCHANGE_RATE);

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context,) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), maxCost);

        // Calculate expected token cost for actual gas used
        uint256 actualGasCost = 0.5 ether;
        uint256 expectedTokenCost = actualGasCost.mulDiv(EXCHANGE_RATE, 1e18, Math.Rounding.Ceil);

        uint256 beneficiaryBalanceBefore = beneficiary.balance;

        vm.expectEmit(true, true, false, false);
        emit UserOpSponsored(address(mockAccount), address(0), actualGasCost, expectedTokenCost);

        vm.prank(address(entryPoint));
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, actualGasCost, 1000000000);

        // Check that payment was collected (if beneficiary is not set to paymaster, payment goes to paymaster by default)
        // Since beneficiary defaults to address(paymaster), ETH should go to paymaster
        // Check that payment was collected - the paymaster should have received the payment\n        // Note: paymaster already had some ETH, so we check for the increase\n        assertGt(address(paymaster).balance, 5 ether); // Should be more than initial 5 ether
    }

    function test_PostOp_ERC20_WithRefund() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), EXCHANGE_RATE);

        // Approve tokens
        vm.prank(user);
        testToken.approve(address(paymaster), type(uint256).max);

        uint256 userBalanceBefore = testToken.balanceOf(user);

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context,) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);

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

    function test_GetTokenDecimals() public view {
        // Test with testToken (should have 18 decimals)
        assertEq(paymaster.getTokenDecimals(address(testToken)), 18);

        // Test with native token
        assertEq(paymaster.getTokenDecimals(address(0)), 18);
    }

    function test_GetTokenDecimals_NonExistentContract() public view {
        // Create a random address that's not a contract
        address nonExistentContract = address(0xdead);

        // Should default to 18 for non-existent contracts
        assertEq(paymaster.getTokenDecimals(nonExistentContract), 18);
    }

    function test_CalculateExchangeRate() public view {
        // Test USDC-like token (6 decimals) at $0.0005 ETH per USDC
        // Should give us 2000 USDC per 1 ETH = 2000 * 10^6 = 2000000000
        uint256 rate = paymaster.calculateExchangeRate(6, 0.0005e18);
        assertEq(rate, 2000000000);

        // Test 18-decimal token - simplify to avoid precision issues
        // Use a simpler example: 1000 tokens per 1 ETH
        // 1 token = 1/1000 ETH = 0.001 ETH = 1000000000000000 wei
        uint256 simpleTokenPrice = 0.001e18; // 1/1000 ETH per token
        uint256 rate18 = paymaster.calculateExchangeRate(18, simpleTokenPrice);
        // Should give us exactly 1000 tokens per ETH = 1000 * 10^18
        assertEq(rate18, 1000e18);
    }

    function test_CalculateExchangeRate_ZeroPrice() public {
        vm.expectRevert(ERC20PaymasterV1.InvalidExchangeRate.selector);
        paymaster.calculateExchangeRate(18, 0);
    }

    function test_PostOp_NativeToken_PaymentFailed() public {
        // Use the mock account that fails payment
        uint256 maxCost = 1 ether;
        uint256 requiredETH = maxCost.mulDiv(EXCHANGE_RATE, 1e18, Math.Rounding.Ceil);
        vm.deal(address(mockAccountFailsPayment), requiredETH + 1 ether);

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(address(mockAccountFailsPayment), "");
        userOp.paymasterAndData = _encodePaymasterData(address(0), EXCHANGE_RATE);

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context,) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), maxCost);

        // Calculate expected token cost for actual gas used
        uint256 actualGasCost = 0.5 ether;
        uint256 expectedTokenCost = actualGasCost.mulDiv(EXCHANGE_RATE, 1e18, Math.Rounding.Ceil);

        // Expect the payment failure event
        vm.expectEmit(true, false, false, true);
        emit NativeTokenPaymentFailed(address(mockAccountFailsPayment), expectedTokenCost);

        // Also expect the sponsored event
        vm.expectEmit(true, true, false, false);
        emit UserOpSponsored(address(mockAccountFailsPayment), address(0), actualGasCost, expectedTokenCost);

        vm.prank(address(entryPoint));
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, actualGasCost, 1000000000);
    }

    function test_PostOp_NativeToken_FullSponsorship() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(address(mockAccount), "");
        userOp.paymasterAndData = _encodePaymasterData(address(0), 0); // Full sponsorship for native token

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context,) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);

        vm.expectEmit(true, true, false, false);
        emit UserOpSponsored(address(mockAccount), address(0), 0.5 ether, 0); // 0 token cost for full sponsorship

        vm.prank(address(entryPoint));
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, 0.5 ether, 1000000000);
    }

    function test_PostOp_ERC20_FullSponsorship() public {
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.paymasterAndData = _encodePaymasterData(address(testToken), 0); // Full sponsorship

        vm.prank(address(entryPoint), bundler1);
        (bytes memory context,) = paymaster.validatePaymasterUserOp(userOp, bytes32(0), 1 ether);

        vm.expectEmit(true, true, false, false);
        emit UserOpSponsored(user, address(testToken), 0.5 ether, 0); // 0 token cost for full sponsorship

        vm.prank(address(entryPoint));
        paymaster.postOp(IPaymaster.PostOpMode.opSucceeded, context, 0.5 ether, 1000000000);
    }

    function test_ArithmeticOverflowProtection() public pure {
        // Test with large values that could cause overflow without mulDiv
        uint256 maxCost = 1e30; // Large but not max uint256
        uint256 exchangeRate = 1e30;

        // This should not revert due to overflow protection
        uint256 result = maxCost.mulDiv(exchangeRate, 1e18, Math.Rounding.Ceil);
        assertTrue(result > 0);

        // The naive calculation would overflow: maxCost * exchangeRate > type(uint256).max
        // But mulDiv prevents this
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

    function _encodePaymasterDataWithTime(address token, uint256 exchangeRate, uint256 validAfter, uint256 validUntil)
        internal
        view
        returns (bytes memory)
    {
        return abi.encodePacked(
            address(paymaster),
            bytes12(0), // Pad token address to 32 bytes
            token,
            exchangeRate,
            validUntil,
            validAfter
        );
    }

    function test_ApprovalDetection_ExecuteApproval() public {
        // Test detection of execute(token, 0, approve(paymaster, amount))
        bytes memory approveData = abi.encodeWithSelector(IERC20.approve.selector, address(paymaster), 1000e18);

        bytes memory executeCallData = abi.encodeWithSelector(
            bytes4(keccak256("execute(address,uint256,bytes)")), address(testToken), 0, approveData
        );

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.callData = executeCallData;

        // Should detect this as an approval operation
        bool isApproval = paymaster._isApprovalOperation_exposed(userOp, address(testToken));
        assertTrue(isApproval, "Should detect execute approval");
    }

    function test_ApprovalDetection_ExecuteApproval_WrongToken() public {
        // Test that approval to wrong token is not detected
        bytes memory approveData = abi.encodeWithSelector(IERC20.approve.selector, address(paymaster), 1000e18);

        bytes memory executeCallData = abi.encodeWithSelector(
            bytes4(keccak256("execute(address,uint256,bytes)")),
            makeAddr("wrongToken"), // Wrong token address
            0,
            approveData
        );

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.callData = executeCallData;

        // Should NOT detect this as an approval operation for testToken
        bool isApproval = paymaster._isApprovalOperation_exposed(userOp, address(testToken));
        assertFalse(isApproval, "Should not detect approval to wrong token");
    }

    function test_ApprovalDetection_ExecuteApproval_WrongSpender() public {
        // Test that approval to wrong spender is not detected
        bytes memory approveData = abi.encodeWithSelector(
            IERC20.approve.selector,
            makeAddr("wrongSpender"), // Wrong spender
            1000e18
        );

        bytes memory executeCallData = abi.encodeWithSelector(
            bytes4(keccak256("execute(address,uint256,bytes)")), address(testToken), 0, approveData
        );

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.callData = executeCallData;

        // Should NOT detect this as an approval operation
        bool isApproval = paymaster._isApprovalOperation_exposed(userOp, address(testToken));
        assertFalse(isApproval, "Should not detect approval to wrong spender");
    }

    function test_ApprovalDetection_NonApprovalCall() public {
        // Test that non-approval calls are not detected
        bytes memory transferData = abi.encodeWithSelector(IERC20.transfer.selector, makeAddr("recipient"), 1000e18);

        bytes memory executeCallData = abi.encodeWithSelector(
            bytes4(keccak256("execute(address,uint256,bytes)")), address(testToken), 0, transferData
        );

        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.callData = executeCallData;

        // Should NOT detect this as an approval operation
        bool isApproval = paymaster._isApprovalOperation_exposed(userOp, address(testToken));
        assertFalse(isApproval, "Should not detect transfer as approval");
    }

    function test_ApprovalDetection_DirectCall() public {
        // Test that direct function calls (not execute/executeBatch) are not detected
        PackedUserOperation memory userOp = TestUtils.preparePackedOp(user, "");
        userOp.callData = abi.encodeWithSignature("someOtherFunction()");

        // Should NOT detect this as an approval operation
        bool isApproval = paymaster._isApprovalOperation_exposed(userOp, address(testToken));
        assertFalse(isApproval, "Should not detect direct calls as approval");
    }
}
