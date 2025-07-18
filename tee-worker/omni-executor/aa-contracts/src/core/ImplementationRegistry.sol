// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

contract ImplementationRegistry {
    // Mapping from contract type to current implementation address
    mapping(string => address) private currentImplementations;
    
    // Reverse mapping from implementation address to contract type
    mapping(address => ContractInfo) private implementationInfo;
    
    struct ContractInfo {
        string contractType;
    }
    
    // Event definitions
    event ImplementationRegistered(
        string indexed contractType,
        address indexed implementation
    );
    
    constructor() {}

    function registerImplementation(
        string calldata contractType,
        address implementation
    ) external {
        require(implementation != address(0), "Invalid implementation address");
        require(bytes(contractType).length > 0, "Invalid contract type");
        require(implementation.code.length > 0, "Implementation must have code");
        
        // Check if already registered
        require(
            bytes(implementationInfo[implementation].contractType).length == 0,
            "Implementation already registered"
        );
        
        // Set implementation info
        implementationInfo[implementation] = ContractInfo({
            contractType: contractType
        });
        
        // Set as current implementation
        currentImplementations[contractType] = implementation;
        
        emit ImplementationRegistered(contractType, implementation);
    }

    function setCurrentImplementation(
        string calldata contractType,
        address implementation
    ) external {
        require(
            bytes(implementationInfo[implementation].contractType).length > 0,
            "Implementation not registered"
        );
        require(
            keccak256(bytes(implementationInfo[implementation].contractType)) == 
            keccak256(bytes(contractType)),
            "Contract type mismatch"
        );
        
        currentImplementations[contractType] = implementation;
    }

    function getImplementation(string calldata contractType) external view returns (address) {
        address implementation = currentImplementations[contractType];
        require(implementation != address(0), "No implementation registered");
        return implementation;
    }

    function getImplementationInfo(address implementation) external view returns (ContractInfo memory) {
        return implementationInfo[implementation];
    }

    function isValidImplementation(
        string calldata contractType,
        address implementation
    ) external view returns (bool) {
        ContractInfo memory info = implementationInfo[implementation];
        return bytes(info.contractType).length > 0 && 
               keccak256(bytes(info.contractType)) == keccak256(bytes(contractType));
    }
}