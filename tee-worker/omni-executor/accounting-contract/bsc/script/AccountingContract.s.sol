// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import {AccountingContract} from "../src/AccountingContract.sol";

contract DeployContract is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address initialAdmin = vm.envAddress("ADMIN_PUBLIC_KEY");
        address initialWorker = vm.envAddress("WORKER_PUBLIC_KEY");
        vm.startBroadcast(deployerPrivateKey);

        AccountingContract accountingContract = new AccountingContract(
            initialAdmin,
            initialWorker
        );

        console.log(
            "AccountingContract deployed at:",
            address(accountingContract)
        );

        console.log(
            "is ",
            vm.addr(deployerPrivateKey),
            " admin: ",
            accountingContract.isAdmin(vm.addr(deployerPrivateKey))
        );

        console.log(
            "is ",
            vm.addr(deployerPrivateKey),
            " worker: ",
            accountingContract.isWorker(vm.addr(deployerPrivateKey))
        );
        vm.stopBroadcast();
    }
}

// export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 && export CONTRACT=0x5FbDB2315678afecb367f032d93F642f64180aa3 && forge script script/AccountingContract.s.sol:DeployContract --rpc-url http://localhost:8545 --broadcast
contract DepositFunds is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        uint256 amount = vm.envUint("AMOUNT");
        vm.startBroadcast(deployerPrivateKey);

        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        accountingContract.depositFunds{value: amount * 10 ** 18}();
        vm.stopBroadcast();
    }
}

contract WithdrawFunds is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        uint256 amount = vm.envUint("AMOUNT");
        address beneficiary = vm.envAddress("BENEFICIARY_PUBLIC_KEY");
        vm.startBroadcast(deployerPrivateKey);

        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        accountingContract.withdrawFunds(
            payable(beneficiary),
            amount * 10 ** 18
        );
        vm.stopBroadcast();
    }
}

contract SetWorker is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        address workerAddress = vm.envAddress("WORKER_PUBLIC_KEY");
        vm.startBroadcast(deployerPrivateKey);

        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        accountingContract.setWorker(workerAddress);

        console.log(
            "Is worker ",
            workerAddress,
            " :",
            accountingContract.isWorker(workerAddress)
        );
        vm.stopBroadcast();
    }
}

contract IsAdmin is Script {
    function run() external view {
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        address adminAddress = vm.envAddress("ADMIN_PUBLIC_KEY");

        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        console.log(
            adminAddress,
            " is Admin: ",
            accountingContract.isAdmin(adminAddress)
        );
    }
}

contract IsWorker is Script {
    function run() external view {
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        address workerAddress = vm.envAddress("WORKER_PUBLIC_KEY");
        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        console.log(
            workerAddress,
            " is Worker: ",
            accountingContract.isWorker(workerAddress)
        );
    }
}

contract Balance is Script {
    function run() external view {
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        console.log(
            "Balance of the contract: ",
            accountingContract.getBalance()
        );
    }
}

contract GetUserRecords is Script {
    function run() external view {
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        address beneficiaryAddresss = vm.envAddress("BENEFICIARY_PUBLIC_KEY");

        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        uint256 nonce = accountingContract.getNonce(beneficiaryAddresss);

        for (uint256 i = 1; i <= nonce; i++) {
            uint256 amount = accountingContract.payouts(beneficiaryAddresss, i);
            if (amount != 0) {
                console.log("Payout Request #", i, ":", amount);
            }
        }
    }
}

contract ExecutePayment is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address contractAddress = vm.envAddress("CONTRACT_ADDRESS");
        address beneficiaryAddress = vm.envAddress("BENEFICIARY_PUBLIC_KEY");
        uint256 amount = vm.envUint("AMOUNT");

        address payable publicAddress = payable(beneficiaryAddress);
        vm.startBroadcast(deployerPrivateKey);

        AccountingContract accountingContract = AccountingContract(
            contractAddress
        );

        uint256 nonce = accountingContract.getNonce(beneficiaryAddress);

        accountingContract.executePayOutRequest(
            publicAddress,
            nonce + 1,
            amount * 10 ** 18
        );

        vm.stopBroadcast();
    }
}
