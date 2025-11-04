// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.28;

/**
 * A module that writes to various storage slots to test storage integrity after upgrade.
 * This module uses a namespaced storage pattern (EIP-7201) to avoid colliding with account storage.
 *
 * IMPORTANT: Modules MUST use namespaced storage to avoid corrupting account state!
 * The account uses slots 0-8 for its core state, so we use a completely different namespace.
 */
contract StorageTestModule {
    // Use a unique storage namespace to avoid collisions with account storage
    bytes32 private constant MODULE_STORAGE_POSITION = keccak256("test.module.storage.v1");

    struct TestStruct {
        uint256 id;
        string name;
        uint256 value;
        bool active;
    }

    struct ModuleStorage {
        uint256 simpleValue;
        uint256[] dynamicArray;
        mapping(address => uint256) addressToValue;
        mapping(address => mapping(uint256 => uint256)) nestedMapping;
        mapping(uint256 => TestStruct) structStorage;
        string stringValue;
        bytes bytesValue;
    }

    // Events to track module execution
    event ValueSet(string key, uint256 value);
    event ArrayPushed(uint256 value);
    event MappingSet(address key, uint256 value);
    event NestedMappingSet(address key1, uint256 key2, uint256 value);
    event StructSet(uint256 id, string name, uint256 value);

    function _getStorage() private pure returns (ModuleStorage storage ms) {
        bytes32 position = MODULE_STORAGE_POSITION;
        assembly {
            ms.slot := position
        }
    }

    function setSimpleValue(uint256 _value) external {
        ModuleStorage storage ms = _getStorage();
        ms.simpleValue = _value;
        emit ValueSet("simpleValue", _value);
    }

    function getSimpleValue() external view returns (uint256) {
        return _getStorage().simpleValue;
    }

    function pushToArray(uint256 _value) external {
        ModuleStorage storage ms = _getStorage();
        ms.dynamicArray.push(_value);
        emit ArrayPushed(_value);
    }

    function getArrayLength() external view returns (uint256) {
        return _getStorage().dynamicArray.length;
    }

    function getArrayValue(uint256 index) external view returns (uint256) {
        return _getStorage().dynamicArray[index];
    }

    function setMapping(address _key, uint256 _value) external {
        ModuleStorage storage ms = _getStorage();
        ms.addressToValue[_key] = _value;
        emit MappingSet(_key, _value);
    }

    function getMapping(address _key) external view returns (uint256) {
        return _getStorage().addressToValue[_key];
    }

    function setNestedMapping(address _key1, uint256 _key2, uint256 _value) external {
        ModuleStorage storage ms = _getStorage();
        ms.nestedMapping[_key1][_key2] = _value;
        emit NestedMappingSet(_key1, _key2, _value);
    }

    function getNestedMapping(address _key1, uint256 _key2) external view returns (uint256) {
        return _getStorage().nestedMapping[_key1][_key2];
    }

    function setStruct(uint256 _id, string calldata _name, uint256 _value) external {
        ModuleStorage storage ms = _getStorage();
        ms.structStorage[_id] = TestStruct({
            id: _id,
            name: _name,
            value: _value,
            active: true
        });
        emit StructSet(_id, _name, _value);
    }

    function getStruct(uint256 _id) external view returns (uint256 id, string memory name, uint256 value, bool active) {
        TestStruct memory s = _getStorage().structStorage[_id];
        return (s.id, s.name, s.value, s.active);
    }

    function setString(string calldata _value) external {
        ModuleStorage storage ms = _getStorage();
        ms.stringValue = _value;
    }

    function getString() external view returns (string memory) {
        return _getStorage().stringValue;
    }

    function setBytes(bytes calldata _value) external {
        ModuleStorage storage ms = _getStorage();
        ms.bytesValue = _value;
    }

    function getBytes() external view returns (bytes memory) {
        return _getStorage().bytesValue;
    }

    function complexOperation(uint256 _value1, uint256 _value2, address _addr) external returns (uint256) {
        ModuleStorage storage ms = _getStorage();
        // Perform multiple storage operations
        ms.simpleValue = _value1 + _value2;
        ms.dynamicArray.push(_value1);
        ms.dynamicArray.push(_value2);
        ms.addressToValue[_addr] = _value1 * _value2;
        ms.nestedMapping[_addr][_value1] = _value2;

        return ms.simpleValue;
    }

    function incrementSimpleValue() external returns (uint256) {
        ModuleStorage storage ms = _getStorage();
        ms.simpleValue++;
        return ms.simpleValue;
    }
}
