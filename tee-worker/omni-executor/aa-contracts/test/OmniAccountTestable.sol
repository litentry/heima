// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.13;

import "../src/accounts/OmniAccount.sol";

/**
 * @title OmniAccountTestable
 * @dev A test-only version of OmniAccount that exposes internal functions for testing
 */
contract OmniAccountTestable is OmniAccount {
    constructor(IEntryPoint anEntryPoint) OmniAccount(anEntryPoint) {}

    /**
     * @dev Expose the internal _validatePasskey function for direct testing
     */
    function validatePasskeyPublic(bytes32 userOpHash, bytes calldata sig)
        external
        view
        returns (uint256 validationData)
    {
        return _validatePasskey(userOpHash, sig);
    }

    /**
     * @dev Expose the internal _validateOwner function for direct testing
     */
    function validateOwnerPublic(bytes32 userOpHash, bytes calldata sig)
        external
        view
        returns (uint256 validationData)
    {
        return _validateOwner(userOpHash, sig);
    }

    /**
     * @dev Expose the internal _validateRootKey function for direct testing
     */
    function validateRootKeyPublic(bytes32 userOpHash, bytes calldata sig)
        external
        view
        returns (uint256 validationData)
    {
        return _validateRootKey(userOpHash, sig);
    }

    /**
     * @dev Expose the internal _validateSessionKey function for direct testing
     */
    function validateSessionKeyPublic(bytes32 userOpHash, bytes calldata sig)
        external
        view
        returns (uint256 validationData)
    {
        return _validateSessionKey(userOpHash, sig);
    }
}