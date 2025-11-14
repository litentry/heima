// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/Create2.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import "../interfaces/ISenderCreator.sol";
import "../interfaces/OwnerType.sol";
import "./OmniAccountV1.sol";

/**
 * A sample factory contract for Account
 * A UserOperations "initCode" holds the address of the factory, and a method call (to createAccount, in this sample factory).
 * The factory's createAccount returns the target account address even if it is already installed.
 * This way, the entryPoint.getSenderAddress() can be called either before or after the account is created.
 */
contract OmniAccountFactoryV1 {
    OmniAccountV1 public immutable accountImplementation;
    ISenderCreator public immutable senderCreator;

    constructor(IEntryPoint _entryPoint) {
        accountImplementation = new OmniAccountV1(_entryPoint);
        senderCreator = _entryPoint.senderCreator();
    }

    /**
     * create an account, and return its address.
     * returns the address even if the account is already deployed.
     * Note that during UserOperation execution, this method is called only if the account is not deployed.
     * This method returns an existing account address so that entryPoint.getSenderAddress() would work even after account creation
     */
    function createAccount(bytes32 oa, OwnerType oaType, bytes memory clientId, address root)
        public
        returns (OmniAccountV1 ret)
    {
        require(msg.sender == address(senderCreator), "only callable from SenderCreator");
        address addr = getAddress(oa, oaType, clientId, root);
        uint256 codeSize = addr.code.length;
        if (codeSize > 0) {
            return OmniAccountV1(payable(addr));
        }
        ret = OmniAccountV1(
            payable(
                new ERC1967Proxy{salt: oa}(
                    address(accountImplementation),
                    abi.encodeCall(OmniAccountV1.initialize, (oa, oaType, clientId, root))
                )
            )
        );
    }

    /**
     * calculate the counterfactual address of this account as it would be returned by createAccount()
     */
    function getAddress(bytes32 oa, OwnerType oaType, bytes memory clientId, address root)
        public
        view
        returns (address)
    {
        return Create2.computeAddress(
            oa,
            keccak256(
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        address(accountImplementation),
                        abi.encodeCall(OmniAccountV1.initialize, (oa, oaType, clientId, root))
                    )
                )
            )
        );
    }

    function version() public pure returns (string memory) {
        return "1.0.0";
    }
}
