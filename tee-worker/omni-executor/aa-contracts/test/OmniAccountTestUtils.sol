// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Counter} from "../src/Counter.sol";
import {Vm} from "forge-std/Vm.sol";
import {OmniAccountV1} from "../src/accounts/OmniAccountV1.sol";
import {BaseAccount} from "../src/core/BaseAccount.sol";
import {EntryPointV1} from "../src/core/EntryPointV1.sol";
import {OwnerType} from "../src/interfaces/OwnerType.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {TestUtils} from "./TestUtils.sol";

library OmniAccountTestUtils {
    function setUp(address ownerAddress, bytes memory clientId, address rootAddress)
        external
        returns (Counter, EntryPointV1, OmniAccountV1)
    {
        Counter counter = new Counter();
        EntryPointV1 entryPoint = new EntryPointV1();
        OmniAccountV1 accountImpl = new OmniAccountV1(entryPoint);
        bytes32 oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
        OmniAccountV1 account = OmniAccountV1(
            payable(
                new ERC1967Proxy{salt: oa}(
                    address(accountImpl),
                    abi.encodeCall(OmniAccountV1.initialize, (oa, OwnerType.Evm, clientId, rootAddress))
                )
            )
        );

        return (counter, entryPoint, account);
    }

    function performExecuteTestAs(Vm vm, address asAccount, OmniAccountV1 account, Counter counter) internal {
        uint256 number = counter.number();
        vm.prank(asAccount);
        account.execute(address(counter), 0, abi.encodeWithSignature("increment()"));
        vm.assertEq(number + 1, counter.number());
    }

    function performExecuteBatchTestAs(Vm vm, address asAccount, OmniAccountV1 account, Counter counter) internal {
        uint256 number = counter.number();
        vm.prank(asAccount);
        BaseAccount.Call[] memory calls = new BaseAccount.Call[](2);
        calls[0] = BaseAccount.Call(address(counter), 0, abi.encodeWithSignature("increment()"));
        calls[1] = BaseAccount.Call(address(counter), 0, abi.encodeWithSignature("increment()"));
        account.executeBatch(calls);
        vm.assertEq(number + 2, counter.number());
    }
}
