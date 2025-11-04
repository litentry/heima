// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/* solhint-disable avoid-low-level-calls */
/* solhint-disable no-inline-assembly */
/* solhint-disable reason-string */

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import "../core/BaseAccount.sol";
import "../interfaces/OwnerType.sol";
import "../interfaces/UserOpSigner.sol";
import "../interfaces/Passkey.sol";
import "../core/Helpers.sol";
import "./callback/TokenCallbackHandler.sol";
import "../utils/Exec.sol";
import "../core/LibModuleManager.sol";

contract OmniAccountV2 is BaseAccount, TokenCallbackHandler, UUPSUpgradeable, Initializable {
    using Passkey for Passkey.PublicKey;
    using LibModuleManager for LibModuleManager.ModuleStorage;

    bytes32 public owner;
    bytes public clientId;
    OwnerType public ownerType;

    mapping(address => bool) public rootSigners;
    mapping(bytes32 => bool) public passkeySigners;
    uint256 public passkeySignerCount;

    IEntryPoint private immutable _entryPoint;

    bytes4 private constant ADD_ROOT_SIGNER_SELECTOR = bytes4(keccak256("addRootSigner(address)"));
    bytes4 private constant REMOVE_ROOT_SIGNER_SELECTOR = bytes4(keccak256("removeRootSigner(address)"));
    bytes4 private constant ADD_PASSKEY_SIGNER_SELECTOR = bytes4(keccak256("addPasskeySigner((uint256,uint256))"));
    bytes4 private constant REMOVE_PASSKEY_SIGNER_SELECTOR = bytes4(keccak256("removePasskeySigner((uint256,uint256))"));
    bytes4 private constant WITHDRAW_DEPOSIT_SELECTOR = bytes4(keccak256("withdrawDepositTo(address,uint256)"));
    bytes4 private constant UPGRADE_TO_AND_CALL_SELECTOR = bytes4(keccak256("upgradeToAndCall(address,bytes)"));
    bytes4 private constant REGISTER_MODULE_SELECTOR = bytes4(keccak256("registerModule(address)"));
    bytes4 private constant UNREGISTER_MODULE_SELECTOR = bytes4(keccak256("unregisterModule(address)"));

    event AccountInitialized(
        IEntryPoint indexed entryPoint, bytes32 indexed owner, OwnerType ownerType, bytes clientId, address indexed root
    );
    event RootSignerAdded(address root);
    event RootSignerRemoved(address root);
    event PasskeySignerAdded(Passkey.PublicKey pk);
    event PasskeySignerRemoved(Passkey.PublicKey pk);
    event ModuleRegistered(address indexed module);
    event ModuleUnregistered(address indexed module);

    modifier onlyOwner() {
        _onlyOwner();
        _;
    }

    function entryPoint() public view virtual override returns (IEntryPoint) {
        return _entryPoint;
    }

    receive() external payable {}

    constructor(IEntryPoint anEntryPoint) {
        _entryPoint = anEntryPoint;
        _disableInitializers();
    }

    function _onlyOwner() internal view {
        require(
            _determineOa(msg.sender) == owner || msg.sender == address(entryPoint())
            || (ownerType != OwnerType.Evm && passkeySignerCount == 0 && isRootSigner(msg.sender)),
            "only owner"
        );
    }

    function initialize(bytes32 anOwner, OwnerType anOwnerType, bytes memory aClientId, address aRoot)
    public
    virtual
    initializer
    {
        _initialize(anOwner, anOwnerType, aClientId, aRoot);
    }

    function _initialize(bytes32 anOwner, OwnerType anOwnerType, bytes memory aClientId, address aRoot)
    internal
    virtual
    {
        owner = anOwner;
        rootSigners[aRoot] = true;
        clientId = aClientId;
        ownerType = anOwnerType;
        emit AccountInitialized(_entryPoint, owner, ownerType, clientId, aRoot);
    }

    function _determineOa(address sender) internal view returns (bytes32) {
        bytes3 oaType = 0x65766d;
        return sha256(abi.encodePacked(clientId, oaType, sender));
    }

    function isRootSigner(address sender) public view returns (bool) {
        return rootSigners[sender];
    }

    function _validateSignature(PackedUserOperation calldata userOp, bytes32 userOpHash)
    internal
    virtual
    override
    returns (uint256 validationData)
    {
        require(userOp.signature.length >= 1, "signature too short");

        UserOpSigner signer = UserOpSigner(uint8(userOp.signature[0]));
        bytes calldata sig = userOp.signature[1:];

        if (signer == UserOpSigner.Owner) {
            return _validateOwner(userOpHash, sig);
        } else if (signer == UserOpSigner.RootKey) {
            if (ownerType != OwnerType.Evm && passkeySignerCount == 0) {
                return _validateRootKey(userOpHash, sig);
            }
            if (_isRestrictedCall(userOp.callData)) {
                return SIG_VALIDATION_FAILED;
            }
            return _validateRootKey(userOpHash, sig);
        } else if (signer == UserOpSigner.SessionKey) {
            if (_isRestrictedCall(userOp.callData)) {
                return SIG_VALIDATION_FAILED;
            }
            return _validateSessionKey(userOpHash, sig);
        } else if (signer == UserOpSigner.Passkey) {
            if (ownerType != OwnerType.Evm && passkeySignerCount > 0) {
                return _validatePasskey(userOpHash, sig);
            }
            if (_isRestrictedCall(userOp.callData)) {
                return SIG_VALIDATION_FAILED;
            }
            return _validatePasskey(userOpHash, sig);
        } else {
            revert("unsupported signer type");
        }
    }

    function _validateOwner(bytes32 userOpHash, bytes calldata sig) internal view returns (uint256 validationData) {
        require(sig.length == 65, "Owner signature length invalid");
        address signer = ECDSA.recover(userOpHash, sig);
        return owner == _determineOa(signer) ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function _validateRootKey(bytes32 userOpHash, bytes calldata sig) internal view returns (uint256 validationData) {
        require(sig.length == 65, "RootKey signature length invalid");
        address signer = ECDSA.recover(userOpHash, sig);
        return isRootSigner(signer) ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function _validateSessionKey(bytes32 userOpHash, bytes calldata sig)
    internal
    view
    returns (uint256 validationData)
    {
        require(sig.length == 162, "SessionKey signature length invalid");
        bytes memory sessionSig = sig[:65];
        address sessionKey = ECDSA.recover(userOpHash, sessionSig);
        uint256 sessionExpiration = uint256(bytes32(sig[65:97]));

        if (block.timestamp > sessionExpiration) {
            return SIG_VALIDATION_FAILED;
        }

        bytes memory sessionProof = sig[97:162];
        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        address sessionProofSigner = ECDSA.recover(sessionDigest, sessionProof);

        return isRootSigner(sessionProofSigner) ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function _validatePasskey(bytes32 userOpHash, bytes calldata sig) internal view returns (uint256 validationData) {
        (
            Passkey.PublicKey memory publicKey,
            Passkey.Signature memory passkeySignature,
            Passkey.Metadata memory metadata
        ) = abi.decode(sig, (Passkey.PublicKey, Passkey.Signature, Passkey.Metadata));

        if (!passkeySigners[publicKey.toKey()]) {
            return SIG_VALIDATION_FAILED;
        }

        bool isValid = Passkey.verify(userOpHash, metadata, passkeySignature, publicKey);

        return isValid ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function getDeposit() public view returns (uint256) {
        return entryPoint().balanceOf(address(this));
    }

    function getOwner() public view returns (bytes32) {
        return owner;
    }

    function addDeposit() public payable {
        entryPoint().depositTo{value: msg.value}(address(this));
    }

    function withdrawDepositTo(address payable withdrawAddress, uint256 amount) public onlyOwner {
        entryPoint().withdrawTo(withdrawAddress, amount);
    }

    function addRootSigner(address root) public onlyOwner {
        rootSigners[root] = true;
        emit RootSignerAdded(root);
    }

    function removeRootSigner(address root) public onlyOwner {
        rootSigners[root] = false;
        emit RootSignerRemoved(root);
    }

    function addPasskeySigner(Passkey.PublicKey memory pk) public onlyOwner {
        bytes32 key = pk.toKey();
        if (!passkeySigners[key]) {
            passkeySigners[key] = true;
            passkeySignerCount++;
            emit PasskeySignerAdded(pk);
        }
    }

    function removePasskeySigner(Passkey.PublicKey memory pk) public onlyOwner {
        bytes32 key = pk.toKey();
        if (passkeySigners[key]) {
            passkeySigners[key] = false;
            passkeySignerCount--;
            emit PasskeySignerRemoved(pk);
        }
    }

    function _authorizeUpgrade(address newImplementation) internal view override {
        (newImplementation);
        _onlyOwner();
    }

    function _isRestrictedCall(bytes calldata callData) internal pure returns (bool) {
        if (callData.length < 4) return false;

        bytes4 selector = bytes4(callData[0:4]);

        return selector == ADD_ROOT_SIGNER_SELECTOR || selector == REMOVE_ROOT_SIGNER_SELECTOR
        || selector == ADD_PASSKEY_SIGNER_SELECTOR || selector == REMOVE_PASSKEY_SIGNER_SELECTOR
        || selector == WITHDRAW_DEPOSIT_SELECTOR || selector == UPGRADE_TO_AND_CALL_SELECTOR
        || selector == REGISTER_MODULE_SELECTOR || selector == UNREGISTER_MODULE_SELECTOR;
    }

    function registerModule(address module) external onlyOwner {
        LibModuleManager.registerModule(module);
        emit ModuleRegistered(module);
    }

    function unregisterModule(address module) external onlyOwner {
        LibModuleManager.unregisterModule(module);
        emit ModuleUnregistered(module);
    }

    function isModuleRegistered(address module) external view returns (bool) {
        return LibModuleManager.isModuleRegistered(module);
    }

    function executeModuleCall(address module, bytes calldata data) external virtual returns (bytes memory) {
        _requireFromEntryPoint();
        require(LibModuleManager.isModuleRegistered(module), "Module not registered");

        bool ok = Exec.delegateCall(module, data, gasleft());
        if (!ok) {
            Exec.revertWithReturnData();
        }

        return Exec.getReturnData(0);
    }

    function version() public pure virtual returns (string memory) {
        return "2.0.0";
    }
}
