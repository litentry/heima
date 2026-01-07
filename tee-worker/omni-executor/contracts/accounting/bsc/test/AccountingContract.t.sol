// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/AccountingContract.sol";
import "@openzeppelin/contracts/access/extensions/AccessControlDefaultAdminRules.sol";


contract AccountingContractTest is Test {
    AccountingContract public accountingContract;
    address public admin = address(0xABCD);
    address public worker = address(0xBEEF);
    address public beneficiary = address(0xCAFE);

    function setUp() public {
        vm.prank(admin);
        accountingContract = new AccountingContract(admin, worker);
        vm.deal(admin, 10 ether);
        vm.deal(worker, 10 ether);
        vm.deal(beneficiary, 10 ether);
    }

    function testInitialState() public {
        assertEq(accountingContract.isAdmin(admin), true);
        assertEq(accountingContract.isWorker(worker), true);
    }

    function testDepositFunds() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 1 ether}();
        assertEq(address(accountingContract).balance, 1 ether);
    }

    function testWithdrawFunds() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 2 ether}();

        vm.prank(admin);
        accountingContract.withdrawFunds(payable(beneficiary), 1 ether);
        assertEq(address(accountingContract).balance, 1 ether);
    }

    function testWithdrawFundsUnauthorized() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 2 ether}();

        vm.prank(worker);
        vm.expectRevert(
            abi.encodeWithSignature(
                "AccessControlUnauthorizedAccount(address,bytes32)",
                address(worker),
                bytes32(0)
            )
        );
        accountingContract.withdrawFunds(payable(beneficiary), 1 ether);
    }

    function testWithdrawFundsOutOfFunds() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 1 ether}();

        vm.prank(admin);
        vm.expectRevert("OutOfBalance");
        accountingContract.withdrawFunds(payable(beneficiary), 2 ether);
    }

    function testSetAdmin() public {
        address newAdmin = address(0x1234);
        vm.prank(admin);
        accountingContract.beginDefaultAdminTransfer(newAdmin);
                
        vm.warp(block.timestamp + 15 seconds);  
        vm.roll(block.number + 1);         

        vm.prank(newAdmin); 
        accountingContract.acceptDefaultAdminTransfer();

        assertEq(accountingContract.isAdmin(newAdmin), true);
        assertEq(accountingContract.isAdmin(admin), false);
    }

    function testSetAdminUnauthorized() public {
        address newAdmin = address(0x1234);
        vm.prank(worker);
        vm.expectRevert(
            abi.encodeWithSignature(
                "AccessControlUnauthorizedAccount(address,bytes32)",
                address(worker),
                bytes32(0)
            )
        );
        accountingContract.beginDefaultAdminTransfer(newAdmin);
    }

    function testSetWorker() public {
        address newWorker = address(0x5678);
        vm.prank(admin);
        accountingContract.setWorker(newWorker);
        assertEq(accountingContract.worker(), newWorker);
    }

    function testSetWorkerUnauthorizedError() public {
        vm.prank(worker);
        vm.expectRevert(
            abi.encodeWithSignature(
                "AccessControlUnauthorizedAccount(address,bytes32)",
                address(worker),
                bytes32(0)
            )
        );
        accountingContract.setWorker(worker);
    }

    function testExecutePayOutRequest() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 1 ether}();

        uint256 initialBalance = beneficiary.balance;

        vm.prank(worker);
        accountingContract.executePayOutRequest(
            payable(beneficiary),
            1,
            0.5 ether
        );

        uint256 amount = accountingContract.payouts(
            beneficiary,
            1
        );
        assertEq(amount, 0.5 ether);
        assertEq(accountingContract.nonces(beneficiary), 1);
        assertEq(beneficiary.balance, initialBalance + 0.5 ether);
    }

    function testExecutePayOutRequestNonceInvalidError() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 1 ether}();

        vm.prank(worker);
        accountingContract.executePayOutRequest(
            payable(beneficiary),
            1,
            0.5 ether
        );

        vm.prank(worker);
        vm.expectRevert("InvalidNonce");
        accountingContract.executePayOutRequest(
            payable(beneficiary),
            1,
            0.5 ether
        );
    }

    function testExecutePayOutRequestOutOfFundsError() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 1 ether}();

        uint256 initialBalance = beneficiary.balance;

        vm.prank(worker);
        accountingContract.executePayOutRequest(
            payable(beneficiary),
            1,
            0.5 ether
        );

        vm.prank(worker);
        vm.expectRevert("OutOfBalance");
        accountingContract.executePayOutRequest(
            payable(beneficiary),
            1,
            1 ether
        );
    }

    function testExecutePayOutRequestUnauthorizedError() public {
        vm.prank(admin);
        accountingContract.depositFunds{value: 1 ether}();

        uint256 initialBalance = beneficiary.balance;

        vm.prank(beneficiary);
        vm.expectRevert(
            abi.encodeWithSignature(
                "AccessControlUnauthorizedAccount(address,bytes32)",
                address(beneficiary),
                keccak256("WORKER_ROLE")
            )
        );
        accountingContract.executePayOutRequest(
            payable(beneficiary),
            1,
            0.5 ether
        );
    }
}
