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
import "../core/Helpers.sol";
import "./callback/TokenCallbackHandler.sol";

/**
 *  smart wallet account
 *  has an owner which is represented by sha256(0x01+hexencoded-address)
 *  has and root address, this account can generate session keys that will pass signature validation
 *  has list of allowed signers who can sign messages
 *  has execute, eth handling methods
 */
contract SmartAccount is BaseAccount, TokenCallbackHandler, UUPSUpgradeable, Initializable {
    bytes32 public owner;
    bytes32 public clientId;
    mapping(address => bool) public rootSigners;
    mapping(address => bool) public allowedSigners;

    IEntryPoint private immutable _entryPoint;

    event AccountInitialized(
        IEntryPoint indexed entryPoint, bytes32 indexed owner, bytes32 clientId, address indexed root
    );

    event AllowedSignerAdded(address signer);
    event AllowedSignerRemoved(address signer);

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
    function initialize(bytes32 anOwner, bytes32 aClientId, address aRoot) public virtual initializer {
        _initialize(anOwner, aClientId, aRoot);
    }

    function _initialize(bytes32 anOwner, bytes32 aClientId, address aRoot) internal virtual {
        owner = anOwner;
        rootSigners[aRoot] = true;
        clientId = aClientId;
        emit AccountInitialized(_entryPoint, owner, clientId, aRoot);
    }

    // Require the function call went through EntryPoint or be signed by [owner|allowed signer|root]
    function _requireForExecute() internal view virtual override {
        require(
            msg.sender == address(entryPoint()) || _determineOa(msg.sender) == owner || isAllowedSigner(msg.sender)
                || isRootSigner(msg.sender),
            "account: not Owner or EntryPoint or allowed signer or root"
        );
    }

    /**
     * convert sender to oa bytes
     */
    function _determineOa(address sender) internal view returns (bytes32) {
        bytes1 oaType = 0x01;
        return sha256(abi.encodePacked(oaType, clientId, sender));
    }

    function isAllowedSigner(address sender) public view returns (bool) {
        return allowedSigners[sender];
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
        // UserOpHash can be generated using eth_signTypedData_v4
        address signer = ECDSA.recover(userOpHash, userOp.signature);
        if (owner == _determineOa(signer) || isAllowedSigner(signer) || isRootSigner(signer)) {
            return SIG_VALIDATION_SUCCESS;
        }
        if (signer == userOp.sessionAccount) {
            // validate session_proof was signed by root
            bytes32 sessionDigest = sha256(abi.encodePacked(userOp.sessionAccount));
            address sessionProofSigner = ECDSA.recover(sessionDigest, userOp.sessionAccountProof);

            if (isRootSigner(sessionProofSigner)) {
                return SIG_VALIDATION_SUCCESS;
            }
        }

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

    function addAllowedSigner(address signer) public onlyOwner {
        allowedSigners[signer] = true;
        emit AllowedSignerAdded(signer);
    }

    function removeAllowedSigner(address signer) public onlyOwner {
        allowedSigners[signer] = false;
        emit AllowedSignerRemoved(signer);
    }

    function _authorizeUpgrade(address newImplementation) internal view override {
        (newImplementation);
        _onlyOwner();
    }
}
