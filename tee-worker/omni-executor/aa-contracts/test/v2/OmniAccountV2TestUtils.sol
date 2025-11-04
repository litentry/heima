// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

import {Counter} from "../../src/Counter.sol";
import {Vm} from "forge-std/Vm.sol";
import {OmniAccountV2 as OmniAccount} from "../../src/accounts/OmniAccountV2.sol";
import {BaseAccount} from "../../src/core/BaseAccount.sol";
import {EntryPointV1 as EntryPoint} from "../../src/core/EntryPointV1.sol";
import {OwnerType} from "../../src/interfaces/OwnerType.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {TestUtils} from "../TestUtils.sol";

library OmniAccountV2TestUtils {
    function setUpWithOwnerType(address ownerAddress, bytes memory clientId, address rootAddress, OwnerType ownerType)
        external
        returns (Counter, EntryPoint, OmniAccount)
    {
        Counter counter = new Counter();
        EntryPoint entryPoint = new EntryPoint();
        OmniAccount accountImpl = new OmniAccount(entryPoint);
        bytes32 oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
        OmniAccount account = OmniAccount(
            payable(
                new ERC1967Proxy{salt: oa}(
                    address(accountImpl), abi.encodeCall(OmniAccount.initialize, (oa, ownerType, clientId, rootAddress))
                )
            )
        );

        return (counter, entryPoint, account);
    }

    function setUp(address ownerAddress, bytes memory clientId, address rootAddress)
        external
        returns (Counter, EntryPoint, OmniAccount)
    {
        Counter counter = new Counter();
        EntryPoint entryPoint = new EntryPoint();
        OmniAccount accountImpl = new OmniAccount(entryPoint);
        bytes32 oa = TestUtils.prepare_evm_oa(ownerAddress, clientId);
        OmniAccount account = OmniAccount(
            payable(
                new ERC1967Proxy{salt: oa}(
                    address(accountImpl),
                    abi.encodeCall(OmniAccount.initialize, (oa, OwnerType.Evm, clientId, rootAddress))
                )
            )
        );

        return (counter, entryPoint, account);
    }

    function performExecuteTestAs(Vm vm, address asAccount, OmniAccount account, Counter counter) internal {
        uint256 number = counter.number();
        vm.prank(asAccount);
        account.execute(address(counter), 0, abi.encodeWithSignature("increment()"));
        vm.assertEq(number + 1, counter.number());
    }

    function performExecuteBatchTestAs(Vm vm, address asAccount, OmniAccount account, Counter counter) internal {
        uint256 number = counter.number();
        vm.prank(asAccount);
        BaseAccount.Call[] memory calls = new BaseAccount.Call[](2);
        calls[0] = BaseAccount.Call(address(counter), 0, abi.encodeWithSignature("increment()"));
        calls[1] = BaseAccount.Call(address(counter), 0, abi.encodeWithSignature("increment()"));
        account.executeBatch(calls);
        vm.assertEq(number + 2, counter.number());
    }
}
