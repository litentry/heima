// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

library LibModuleManager {
    bytes32 constant MODULE_STORAGE_POSITION = keccak256("omni.account.module.storage");

    struct ModuleStorage {
        mapping(address => bool) registeredModules;
    }

    function moduleStorage() internal pure returns (ModuleStorage storage ms) {
        bytes32 position = MODULE_STORAGE_POSITION;
        assembly {
            ms.slot := position
        }
    }

    function isModuleRegistered(address module) internal view returns (bool) {
        return moduleStorage().registeredModules[module];
    }

    function registerModule(address module) internal {
        require(module != address(0), "Invalid module address");
        require(!isModuleRegistered(module), "Module already registered");

        uint256 codeSize;
        assembly {
            codeSize := extcodesize(module)
        }
        require(codeSize > 0, "Module must be a contract");

        moduleStorage().registeredModules[module] = true;
    }

    function unregisterModule(address module) internal {
        require(isModuleRegistered(module), "Module not registered");
        moduleStorage().registeredModules[module] = false;
    }
}
