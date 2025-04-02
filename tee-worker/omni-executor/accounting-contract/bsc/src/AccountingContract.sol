// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/extensions/AccessControlDefaultAdminRules.sol";

contract AccountingContract is AccessControlDefaultAdminRules, ReentrancyGuard {
    constructor(address initialAdmin, address initialWorker) AccessControlDefaultAdminRules(0, initialAdmin) {
        require(initialWorker != address(0), "Worker can't be zero address");
        require(initialAdmin != address(0), "Owner can't be zero address");
        worker = initialWorker;
        _grantRole(WORKER_ROLE, initialWorker);
    }

    bytes32 public constant WORKER_ROLE = keccak256("WORKER_ROLE");
    address public worker;
    mapping(address => uint256) public nonces;

    mapping(address => mapping(uint256 => uint256)) public payouts;

    event AdminUpdated(address indexed oldAdmin, address indexed newAdmin);
    event WorkerUpdated(address indexed oldWorker, address indexed newWorker);
    event FundsDeposited(address indexed from, uint256 amount);
    event FundsWithdrawn(address indexed to, uint256 amount);
    event PayoutRequestExecuted(
        address indexed beneficiary,
        uint256 amount,
        uint256 nonce
    );

    function isAdmin(address account) public view returns (bool) {
        return hasRole(DEFAULT_ADMIN_ROLE, account);
    }

    function setWorker(
        address newWorker
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newWorker != address(0), "New worker cannot be zero address");
        emit WorkerUpdated(worker, newWorker);
        grantRole(WORKER_ROLE, newWorker);
        revokeRole(WORKER_ROLE, worker);
        worker = newWorker;
    }

    function isWorker(address account) public view returns (bool) {
        return hasRole(WORKER_ROLE, account);
    }

    function depositFunds() external payable {
        require(msg.value > 0, "Amount must be greater than zero");
        emit FundsDeposited(msg.sender, msg.value);
    }

    function withdrawFunds(
        address payable beneficiary,
        uint256 amount
    ) external onlyRole(DEFAULT_ADMIN_ROLE) nonReentrant {
        require(address(this).balance >= amount, "OutOfBalance");

        emit FundsWithdrawn(beneficiary, amount);

        (bool sent, ) = beneficiary.call{value: amount}("");
        require(sent, "Withdraw failed");
    }

    function executePayOutRequest(
        address payable beneficiary,
        uint256 nonce,
        uint256 amount
    ) external onlyRole(WORKER_ROLE) nonReentrant {
        require(beneficiary != address(0), "Invalid beneficiary");
        require(address(this).balance >= amount, "OutOfBalance");
        require(nonces[beneficiary] < nonce, "InvalidNonce");

        require(payouts[beneficiary][nonce] == 0, "Already exists");
        payouts[beneficiary][nonce] = amount;
        nonces[beneficiary] = nonce;

        emit PayoutRequestExecuted(beneficiary, amount, nonce);

        (bool sent, ) = beneficiary.call{value: amount}("");
        require(sent, "Payout failed");
    }

    function getNonce(address user) external view returns (uint256) {
        return nonces[user];
    }

    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
}
