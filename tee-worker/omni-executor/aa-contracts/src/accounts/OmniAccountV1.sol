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

/**
 *  smart wallet account
 *  has an owner which is represented by sha256(bytes('evm')+client_id+hexencoded-address)
 *  has list of root signers who can sign messages and generate sessions
 *  has execute, eth handling methods
 */
contract OmniAccountV1 is BaseAccount, TokenCallbackHandler, UUPSUpgradeable, Initializable {
    using Passkey for Passkey.PublicKey;

    bytes32 public owner;
    bytes public clientId;
    OwnerType public ownerType;

    mapping(address => bool) public rootSigners;
    mapping(bytes32 => bool) public passkeySigners;
    uint256 public passkeySignerCount;

    IEntryPoint private immutable _entryPoint;

    // Selectors for restricted functions that only owner should call
    bytes4 private constant ADD_ROOT_SIGNER_SELECTOR = bytes4(keccak256("addRootSigner(address)"));
    bytes4 private constant REMOVE_ROOT_SIGNER_SELECTOR = bytes4(keccak256("removeRootSigner(address)"));
    bytes4 private constant ADD_PASSKEY_SIGNER_SELECTOR = bytes4(keccak256("addPasskeySigner((uint256,uint256))"));
    bytes4 private constant REMOVE_PASSKEY_SIGNER_SELECTOR = bytes4(keccak256("removePasskeySigner((uint256,uint256))"));
    bytes4 private constant WITHDRAW_DEPOSIT_SELECTOR = bytes4(keccak256("withdrawDepositTo(address,uint256)"));
    bytes4 private constant UPGRADE_TO_AND_CALL_SELECTOR = bytes4(keccak256("upgradeToAndCall(address,bytes)"));

    event AccountInitialized(
        IEntryPoint indexed entryPoint, bytes32 indexed owner, OwnerType ownerType, bytes clientId, address indexed root
    );

    event RootSignerAdded(address root);
    event RootSignerRemoved(address root);
    event PasskeySignerAdded(Passkey.PublicKey pk);
    event PasskeySignerRemoved(Passkey.PublicKey pk);

    modifier onlyOwner() {
        _onlyOwner();
        _;
    }

    /// @inheritdoc BaseAccount
    function entryPoint() public view virtual override returns (IEntryPoint) {
        return _entryPoint;
    }

    // solhint-disable-next-line no-empty-blocks
    receive() external payable {}

    constructor(IEntryPoint anEntryPoint) {
        _entryPoint = anEntryPoint;
        _disableInitializers();
    }

    function _onlyOwner() internal view {
        // Directly from EOA owner, through the account itself (which gets redirected through execute()),
        // or from EntryPoint (safe because _validateSignature ensures only owner-signed UserOps can call restricted functions)
        // For non-EVM owner types:
        // - If passkey signers exist, they have priority (root signers cannot call onlyOwner)
        // - If no passkey signers exist, root signers can call onlyOwner
        require(
            _determineOa(msg.sender) == owner || msg.sender == address(this) || msg.sender == address(entryPoint())
                || (ownerType != OwnerType.Evm && passkeySignerCount == 0 && isRootSigner(msg.sender)),
            "only owner"
        );
    }

    /**
     * @dev The _entryPoint member is immutable, to reduce gas consumption.  To upgrade EntryPoint,
     * a new implementation of SimpleAccount must be deployed with the new EntryPoint address, then upgrading
     * the implementation by calling `upgradeTo()`
     * @param anOwner the owner (signer) of this account
     */
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

    // Require the function call went through EntryPoint or be signed by owner
    function _requireForExecute() internal view virtual override {
        require(
            msg.sender == address(entryPoint()) || _determineOa(msg.sender) == owner, "account: not Owner or EntryPoint"
        );
    }

    /**
     * convert sender to oa bytes
     */
    function _determineOa(address sender) internal view returns (bytes32) {
        // bytes("evm");
        bytes3 oaType = 0x65766d;
        return sha256(abi.encodePacked(clientId, oaType, sender));
    }

    function isRootSigner(address sender) public view returns (bool) {
        return rootSigners[sender];
    }

    /// implement template method of BaseAccount
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
            // For non-EVM owner types, allow root signers to call restricted functions only if no passkey signers exist
            if (ownerType != OwnerType.Evm && passkeySignerCount == 0) {
                // Root signers can act as owners for non-EVM accounts without passkey signers
                return _validateRootKey(userOpHash, sig);
            }
            // Otherwise, check if the UserOp is trying to call a restricted function
            if (_isRestrictedCall(userOp.callData)) {
                return SIG_VALIDATION_FAILED;
            }
            return _validateRootKey(userOpHash, sig);
        } else if (signer == UserOpSigner.SessionKey) {
            // Session keys should also be restricted from sensitive operations
            if (_isRestrictedCall(userOp.callData)) {
                return SIG_VALIDATION_FAILED;
            }
            return _validateSessionKey(userOpHash, sig);
        } else if (signer == UserOpSigner.Passkey) {
            // For non-EVM owner types with passkey signers, allow passkeys to call restricted functions
            if (ownerType != OwnerType.Evm && passkeySignerCount > 0) {
                // Passkey signers can act as owners for non-EVM accounts
                return _validatePasskey(userOpHash, sig);
            }
            // Otherwise, check if the UserOp is trying to call a restricted function
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

        // validate sessionProof was signed by RootKey
        bytes memory sessionProof = sig[97:162];
        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        address sessionProofSigner = ECDSA.recover(sessionDigest, sessionProof);

        return isRootSigner(sessionProofSigner) ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function _validatePasskey(bytes32, /* userOpHash */ bytes calldata /* sig */ )
        internal
        pure
        returns (uint256 validationData)
    {
        // TODO
        return SIG_VALIDATION_FAILED;
    }

    /**
     * check current account deposit in the entryPoint
     */
    function getDeposit() public view returns (uint256) {
        return entryPoint().balanceOf(address(this));
    }

    function getOwner() public view returns (bytes32) {
        return owner;
    }

    /**
     * deposit more funds for this account in the entryPoint
     */
    function addDeposit() public payable {
        entryPoint().depositTo{value: msg.value}(address(this));
    }

    /**
     * withdraw value from the account's deposit
     * @param withdrawAddress target to send to
     * @param amount to withdraw
     */
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

    /// Check if the callData contains a call to a restricted function
    function _isRestrictedCall(bytes calldata callData) internal pure returns (bool) {
        if (callData.length < 4) return false;

        bytes4 selector = bytes4(callData[0:4]);

        // Check against all restricted selectors
        return selector == ADD_ROOT_SIGNER_SELECTOR || selector == REMOVE_ROOT_SIGNER_SELECTOR
            || selector == ADD_PASSKEY_SIGNER_SELECTOR || selector == REMOVE_PASSKEY_SIGNER_SELECTOR
            || selector == WITHDRAW_DEPOSIT_SELECTOR || selector == UPGRADE_TO_AND_CALL_SELECTOR
            || _isExecuteWithRestrictedCall(selector, callData);
    }

    /// Check if execute/executeBatch contains calls to restricted functions
    function _isExecuteWithRestrictedCall(bytes4 selector, bytes calldata callData) internal pure returns (bool) {
        // Check execute(address,uint256,bytes)
        if (selector == bytes4(keccak256("execute(address,uint256,bytes)"))) {
            if (callData.length < 100) return false; // Not enough data

            // Extract the inner calldata from execute
            // Skip 4 (selector) + 32 (address) + 32 (value) + 32 (offset) + 32 (length)
            uint256 innerDataLength = abi.decode(callData[100:132], (uint256));
            if (callData.length < 132 + innerDataLength) return false;

            bytes calldata innerData = callData[132:132 + innerDataLength];
            return _isRestrictedCall(innerData);
        }

        // Check executeBatch(Call[])
        if (selector == bytes4(keccak256("executeBatch((address,uint256,bytes)[])"))) {
            // This is more complex to parse, so for simplicity we could restrict all executeBatch from root
            return true;
        }

        return false;
    }

    function version() public pure returns (string memory) {
        return "1.0.0";
    }
}
