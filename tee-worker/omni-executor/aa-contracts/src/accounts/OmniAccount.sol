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
import "../core/Primitives.sol";
import "../core/Helpers.sol";
import "./callback/TokenCallbackHandler.sol";

/**
 *  smart wallet account
 *  has an owner which is represented by sha256(bytes('evm')+client_id+hexencoded-address)
 *  has list of root signers who can sign messages and generate sessions
 *  has execute, eth handling methods
 */
contract OmniAccount is BaseAccount, TokenCallbackHandler, UUPSUpgradeable, Initializable {
    bytes32 public owner;
    bytes public clientId;
    mapping(address => bool) public rootSigners;

    IEntryPoint private immutable _entryPoint;

    event AccountInitialized(
        IEntryPoint indexed entryPoint, bytes32 indexed owner, bytes clientId, address indexed root
    );

    event RootSignerAdded(address root);
    event RootSignerRemoved(address root);

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
        // Directly from EOA owner, or through the account itself (which gets redirected through execute())
        require(_determineOa(msg.sender) == owner || msg.sender == address(this), "only owner");
    }

    /**
     * @dev The _entryPoint member is immutable, to reduce gas consumption.  To upgrade EntryPoint,
     * a new implementation of SimpleAccount must be deployed with the new EntryPoint address, then upgrading
     * the implementation by calling `upgradeTo()`
     * @param anOwner the owner (signer) of this account
     */
    function initialize(bytes32 anOwner, bytes memory aClientId, address aRoot) public virtual initializer {
        _initialize(anOwner, aClientId, aRoot);
    }

    function _initialize(bytes32 anOwner, bytes memory aClientId, address aRoot) internal virtual {
        owner = anOwner;
        rootSigners[aRoot] = true;
        clientId = aClientId;
        emit AccountInitialized(_entryPoint, owner, clientId, aRoot);
    }

    // Require the function call went through EntryPoint or be signed by [owner|root]
    function _requireForExecute() internal view virtual override {
        require(
            msg.sender == address(entryPoint()) || _determineOa(msg.sender) == owner || isRootSigner(msg.sender),
            "account: not Owner or EntryPoint or root"
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

        UserOpSigType sigType = UserOpSigType(uint8(userOp.signature[0]));
        bytes calldata sig = userOp.signature[1:];

        if (sigType == UserOpSigType.EOA) {
            return _validateEOA(userOpHash, sig);
        } else if (sigType == UserOpSigType.RootKey) {
            return _validateRootKey(userOpHash, sig);
        } else if (sigType == UserOpSigType.SessionKey) {
            return _validateSessionKey(userOpHash, sig);
        } else if (sigType == UserOpSigType.Passkey) {
            return _validatePasskey(userOpHash, sig);
        } else {
            revert("unsupported signature type");
        }
    }

    function _validateEOA(bytes32 userOpHash, bytes sig) internal returns (uint256 validationData) {
        require(sig.length == 64, "EOA signature length invalid");
        address signer = ECDSA.recover(userOpHash, sig);
        return owner == _determineOa(signer) ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function _validateRootKey(bytes32 userOpHash, bytes sig) internal returns (uint256 validationData) {
        require(sig.length == 64, "RootKey signature length invalid");
        address signer = ECDSA.recover(userOpHash, sig);
        return isRootSigner(signer) ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function _validateSessionKey(bytes32 userOpHash, bytes sig) internal returns (uint256 validationData) {
        require(sig.length == 160, "SessionKey signature length invalid");
        bytes memory sessionSig = sig[:64];
        address sessionKey = ECDSA.recover(userOpHash, sessionSig);
        uint256 sessionExpiration = uint256(bytes32(sig[64:96]));

        if (block.timestamp > sessionExpiration) {
            return SIG_VALIDATION_FAILED;
        }

        // validate sessionProof was signed by RootKey
        bytes memory sessionProof = sig[96:160];
        bytes32 sessionDigest = sha256(abi.encodePacked(sessionKey, sessionExpiration));
        address sessionProofSigner = ECDSA.recover(sessionDigest, sessionProof);

        return isRootSigner(sessionProofSigner) ? SIG_VALIDATION_SUCCESS : SIG_VALIDATION_FAILED;
    }

    function _validatePasskey(bytes32 userOpHash, bytes sig) internal returns (uint256 validationData) {
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

    function _authorizeUpgrade(address newImplementation) internal view override {
        (newImplementation);
        _onlyOwner();
    }
}
