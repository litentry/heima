// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";

contract AccountingContract is Ownable {
    constructor(
        address initialOwner,
        address initialWorker
    ) Ownable(initialOwner) {
        require(initialWorker != address(0), "Worker can't be zero address");
        admin = initialOwner;
        worker = initialWorker;
    }

    address public admin;
    address public worker;
    mapping(address => uint256) public nonces;

    struct Payout {
        uint256 amount;
        bool paid;
    }

    mapping(address => mapping(uint256 => Payout)) public payouts;

    event AdminUpdated(address indexed oldAdmin, address indexed newAdmin);
    event WorkerUpdated(address indexed oldWorker, address indexed newWorker);
    event FundsDeposited(address indexed from, uint256 amount);
    event FundsWithdrawn(address indexed to, uint256 amount);
    event PayoutRequestCreated(
        address indexed beneficiary,
        uint256 amount,
        uint256 nonce
    );

    modifier onlyAdmin() {
        require(msg.sender == admin, "Unauthorized: not admin");
        _;
    }

    modifier onlyWorker() {
        require(msg.sender == worker, "Unauthorized: not worker");
        _;
    }

    function setAdmin(address newAdmin) external onlyOwner {
        require(newAdmin != address(0), "New admin cannot be zero address");
        emit AdminUpdated(admin, newAdmin);
        admin = newAdmin;
    }

    function setWorker(address newWorker) external onlyAdmin {
        require(newWorker != address(0), "New worker cannot be zero address");
        emit WorkerUpdated(worker, newWorker);
        worker = newWorker;
    }

    function depositFunds() external payable {
        require(msg.value > 0, "Amount must be greater than zero");
        emit FundsDeposited(msg.sender, msg.value);
    }

    function withdrawFunds(
        address payable beneficiary,
        uint256 amount
    ) external onlyAdmin {
        require(address(this).balance >= amount, "OutOfBalance");
        (bool sent, ) = beneficiary.call{value: amount}("");
        require(sent, "Withdraw failed");
        emit FundsWithdrawn(beneficiary, amount);
    }

    function createPayRequest(
        address payable beneficiary,
        uint256 nonce,
        uint256 amount
    ) external onlyWorker {
        require(beneficiary != address(0), "Invalid beneficiary");
        require(address(this).balance >= amount, "OutOfBalance");
        require(nonces[beneficiary] < nonce, "InvalidNonce");

        Payout memory request = payouts[beneficiary][nonce];
        require(request.amount == 0, "Already exists");

        (bool sent, ) = beneficiary.call{value: amount}("");
        require(sent, "Payout failed");

        payouts[beneficiary][nonce] = Payout({amount: amount, paid: true});
        nonces[beneficiary] = nonce;

        emit PayoutRequestCreated(beneficiary, amount, nonce);
    }

    function getNonce(address user) external view returns (uint256) {
        return nonces[user];
    }

    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
}
