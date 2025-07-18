// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/Create2.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import "../interfaces/ISenderCreator.sol";
import "../core/ImplementationRegistry.sol";
import "./OmniAccount.sol";

contract OmniAccountFactoryV2 {
    // Fixed canonical implementation address for address calculation
    address public immutable CANONICAL_IMPLEMENTATION;
    
    // Implementation registry
    ImplementationRegistry public immutable registry;
    
    // SenderCreator contract
    ISenderCreator public immutable senderCreator;
    
    // Contract type identifier
    string public constant CONTRACT_TYPE = "OmniAccount";
    
    event AccountCreated(
        address indexed account,
        bytes32 indexed oa,
        bytes clientId,
        address indexed root,
        address implementation
    );
    
    constructor(
        IEntryPoint _entryPoint,
        ImplementationRegistry _registry,
        address _canonicalImplementation
    ) {
        require(address(_registry) != address(0), "Invalid registry");
        require(_canonicalImplementation != address(0), "Invalid canonical impl");
        
        registry = _registry;
        CANONICAL_IMPLEMENTATION = _canonicalImplementation;
        senderCreator = _entryPoint.senderCreator();
    }

    function createAccount(
        bytes32 oa,
        bytes memory clientId,
        address root
    ) public returns (OmniAccount ret) {
        require(msg.sender == address(senderCreator), "only SenderCreator");
        
        address addr = getAddress(oa, clientId, root);
        if (addr.code.length > 0) {
            ret = OmniAccount(payable(addr));
            
            // Check if there's a newer implementation available
            address latestImpl = registry.getImplementation(CONTRACT_TYPE);
            if (latestImpl != address(0) && latestImpl != CANONICAL_IMPLEMENTATION) {
                // Upgrade the existing account to the latest implementation
                ret.factoryUpgrade(latestImpl);
            }
            
            return ret;
        }
        
        // Use canonical implementation for consistency
        address currentImpl = CANONICAL_IMPLEMENTATION;
        
        // Create proxy with canonical implementation
        ret = OmniAccount(
            payable(
                new ERC1967Proxy{salt: oa}(
                    currentImpl,
                    abi.encodeCall(OmniAccount.initialize, (oa, clientId, root))
                )
            )
        );
        
        emit AccountCreated(address(ret), oa, clientId, root, currentImpl);
    }

    function getAddress(
        bytes32 oa,
        bytes memory clientId,
        address root
    ) public view returns (address) {
        return Create2.computeAddress(
            oa,
            keccak256(
                abi.encodePacked(
                    type(ERC1967Proxy).creationCode,
                    abi.encode(
                        CANONICAL_IMPLEMENTATION,
                        abi.encodeCall(OmniAccount.initialize, (oa, clientId, root))
                    )
                )
            )
        );
    }

    function getCurrentImplementation() external view returns (address) {
        return CANONICAL_IMPLEMENTATION;
    }
}
