// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/Create2.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import "../interfaces/ISenderCreator.sol";
import "../interfaces/OwnerType.sol";
import "./OmniAccountV2.sol";

contract OmniAccountFactoryV2 {
    OmniAccountV2 public immutable accountImplementation;
    ISenderCreator public immutable senderCreator;

    event AccountCreated(address indexed account, bytes32 indexed oa, OwnerType oaType, address root);

    constructor(IEntryPoint _entryPoint) {
        accountImplementation = new OmniAccountV2(_entryPoint);
        senderCreator = _entryPoint.senderCreator();
    }

    function createAccount(bytes32 oa, OwnerType oaType, bytes memory clientId, address root)
    public
    returns (OmniAccountV2 ret)
    {
        require(msg.sender == address(senderCreator), "only callable from SenderCreator");
        address addr = getAddress(oa, oaType, clientId, root);
        uint256 codeSize = addr.code.length;
        if (codeSize > 0) {
            return OmniAccountV2(payable(addr));
        }
        ret = OmniAccountV2(
            payable(
                new ERC1967Proxy{salt: oa}(
                    address(accountImplementation),
                    abi.encodeCall(OmniAccountV2.initialize, (oa, oaType, clientId, root))
                )
            )
        );
        emit AccountCreated(address(ret), oa, oaType, root);
    }

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
                        abi.encodeCall(OmniAccountV2.initialize, (oa, oaType, clientId, root))
                    )
                )
            )
        );
    }

    function version() public pure returns (string memory) {
        return "2.0.0";
    }
}
