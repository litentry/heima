// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/AccountingContract.sol";

contract AccountingContractTest is Test {
    AccountingContract public accountingContract;
    address public owner = address(0xABCD);
    address public worker = address(0xBEEF);
    address public beneficiary = address(0xCAFE);

    function setUp() public {
        vm.prank(owner);
        accountingContract = new AccountingContract(owner, worker);
        vm.deal(owner, 10 ether);
        vm.deal(worker, 10 ether);
        vm.deal(beneficiary, 10 ether);
    }

    function testInitialState() public {
        assertEq(accountingContract.admin(), owner);
        assertEq(accountingContract.worker(), worker);
    }

    function testDepositFunds() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 1 ether}();
        assertEq(address(accountingContract).balance, 1 ether);
    }

    function testWithdrawFunds() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 2 ether}();

        vm.prank(owner);
        accountingContract.withdrawFunds(payable(beneficiary), 1 ether);
        assertEq(address(accountingContract).balance, 1 ether);
    }

    function testWithdrawFundsUnauthorized() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 2 ether}();

        vm.prank(worker);
        vm.expectRevert("Unauthorized: not admin");
        accountingContract.withdrawFunds(payable(beneficiary), 1 ether);
    }

    function testWithdrawFundsOutOfFunds() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 1 ether}();

        vm.prank(owner);
        vm.expectRevert("OutOfBalance");
        accountingContract.withdrawFunds(payable(beneficiary), 2 ether);
    }

    function testSetAdmin() public {
        address newAdmin = address(0x1234);
        vm.prank(owner);
        accountingContract.setAdmin(newAdmin);
        assertEq(accountingContract.admin(), newAdmin);
    }

    function testSetAdminUnauthorized() public {
        address newAdmin = address(0x1234);
        vm.prank(worker);
        vm.expectRevert(
            abi.encodeWithSignature(
                "OwnableUnauthorizedAccount(address)",
                address(worker)
            )
        );
        accountingContract.setAdmin(newAdmin);
    }

    function testSetWorker() public {
        address newWorker = address(0x5678);
        vm.prank(owner);
        accountingContract.setWorker(newWorker);
        assertEq(accountingContract.worker(), newWorker);
    }

    function testSetWorkerUnauthorizedError() public {
        vm.prank(worker);
        vm.expectRevert("Unauthorized: not admin");
        accountingContract.setWorker(worker);
    }

    function testCreatePayRequest() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 1 ether}();

        uint256 initialBalance = beneficiary.balance;

        vm.prank(worker);
        accountingContract.createPayRequest(payable(beneficiary), 1, 0.5 ether);

        (uint256 amount, bool paid) = accountingContract.payouts(
            beneficiary,
            1
        );
        assertEq(amount, 0.5 ether);
        assertEq(paid, true);
        assertEq(accountingContract.nonces(beneficiary), 1);
        assertEq(beneficiary.balance, initialBalance + 0.5 ether);
    }

    function testCreatePayRequestNonceInvalidError() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 1 ether}();

        vm.prank(worker);
        accountingContract.createPayRequest(payable(beneficiary), 1, 0.5 ether);

        vm.prank(worker);
        vm.expectRevert("InvalidNonce");
        accountingContract.createPayRequest(payable(beneficiary), 1, 0.5 ether);
    }

    function testCreatePayRequestOutOfFundsError() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 1 ether}();

        uint256 initialBalance = beneficiary.balance;

        vm.prank(worker);
        accountingContract.createPayRequest(payable(beneficiary), 1, 0.5 ether);

        vm.prank(worker);
        vm.expectRevert("OutOfBalance");
        accountingContract.createPayRequest(payable(beneficiary), 1, 1 ether);
    }

    function testPayRequestUnauthorizedError() public {
        vm.prank(owner);
        accountingContract.depositFunds{value: 1 ether}();

        uint256 initialBalance = beneficiary.balance;

        vm.prank(beneficiary);
        vm.expectRevert("Unauthorized: not worker");
        accountingContract.createPayRequest(payable(beneficiary), 1, 0.5 ether);
    }
}
