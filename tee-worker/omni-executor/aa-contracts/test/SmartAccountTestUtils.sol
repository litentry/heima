// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import {Counter} from "../src/Counter.sol";
import {Vm} from "forge-std/Vm.sol";
import {SmartAccount} from "../src/accounts/SmartAccount.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPoint} from "../src/core/EntryPoint.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {TestUtils} from "./TestUtils.sol";

library SmartAccountTestUtils {
    function setUp(address ownerAddress, address rootAddress) external returns (Counter, EntryPoint, SmartAccount) {
        Counter counter = new Counter();
        EntryPoint entryPoint = new EntryPoint();
        SmartAccount accountImpl = new SmartAccount(entryPoint);
        bytes32 oa = TestUtils.prepare_evm_oa(ownerAddress);
        SmartAccount account = SmartAccount(
            payable(
                new ERC1967Proxy{salt: oa}(
                    address(accountImpl), abi.encodeCall(SmartAccount.initialize, (oa, rootAddress))
                )
            )
        );

        return (counter, entryPoint, account);
    }

    function performExecuteTestAs(Vm vm, address asAccount, SmartAccount account, Counter counter) internal {
        uint256 number = counter.number();
        vm.prank(asAccount);
        account.execute(address(counter), 0, abi.encodeWithSignature("increment()"));
        vm.assertEq(number + 1, counter.number());
    }

    function performExecuteBatchTestAs(Vm vm, address asAccount, SmartAccount account, Counter counter) internal {
        uint256 number = counter.number();
        vm.prank(asAccount);
        BaseAccount.Call[] memory calls = new BaseAccount.Call[](2);
        calls[0] = BaseAccount.Call(address(counter), 0, abi.encodeWithSignature("increment()"));
        calls[1] = BaseAccount.Call(address(counter), 0, abi.encodeWithSignature("increment()"));
        account.executeBatch(calls);
        vm.assertEq(number + 2, counter.number());
    }
}
