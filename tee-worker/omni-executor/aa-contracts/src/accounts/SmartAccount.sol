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
 *  has an owner which is represented by sha256(bytes('evm')+client_id+hexencoded-address)
 *  has list of root signers who can sign messages and generate sessions
 *  has execute, eth handling methods
 */
contract SmartAccount is BaseAccount, TokenCallbackHandler, UUPSUpgradeable, Initializable {
    bytes32 public owner;
    bytes32 public clientId;
    mapping(address => bool) public rootSigners;

    IEntryPoint private immutable _entryPoint;

    event AccountInitialized(
        IEntryPoint indexed entryPoint, bytes32 indexed owner, bytes32 clientId, address indexed root
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
    function initialize(bytes32 anOwner, bytes32 aClientId, address aRoot) public virtual initializer {
        _initialize(anOwner, aClientId, aRoot);
    }

    function _initialize(bytes32 anOwner, bytes32 aClientId, address aRoot) internal virtual {
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
        bytes memory oaType = bytes("evm");
        return sha256(abi.encodePacked(oaType, clientId, sender));
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
        // Check if this is a deployment operation (account not initialized)
        if (owner == bytes32(0) && userOp.initCode.length > 0) {
            // During deployment, validate against the signer who will become the owner
            address deploySigner = ECDSA.recover(userOpHash, userOp.signature);
            
            // Extract factory address and decode the createAccount parameters
            if (userOp.initCode.length >= 24) { // Need at least 20 bytes for address + 4 for selector
                // The initCode format is: [20 bytes factory address][4 bytes selector][encoded params]
                // We need to extract the parameters starting from byte 24
                
                // Extract just the parameter bytes (skip factory address and selector)
                uint256 paramsLength = userOp.initCode.length - 24;
                bytes memory params = new bytes(paramsLength);
                
                // Copy parameter bytes from calldata
                for (uint256 i = 0; i < paramsLength; i++) {
                    params[i] = userOp.initCode[i + 24];
                }
                
                // Decode createAccount parameters (oa, clientId, root)
                (bytes32 expectedOa, , address expectedRoot) = abi.decode(
                    params,
                    (bytes32, bytes32, address)
                );
                
                // Verify the signer matches either:
                // 1. The address whose omniAccount equals expectedOa, OR
                // 2. The expectedRoot address
                if (_determineOa(deploySigner) == expectedOa || deploySigner == expectedRoot) {
                    return SIG_VALIDATION_SUCCESS;
                }
            }
            return SIG_VALIDATION_FAILED;
        }
        
        // Existing validation logic for initialized accounts
        // UserOpHash can be generated using eth_signTypedData_v4
        address signer = ECDSA.recover(userOpHash, userOp.signature);
        if (owner == _determineOa(signer) || isRootSigner(signer)) {
            return SIG_VALIDATION_SUCCESS;
        }
        if (signer == userOp.sessionAccount) {
            if (block.timestamp > userOp.sessionExpiration) {
                return SIG_VALIDATION_FAILED;
            }
            // validate session_proof was signed by root
            bytes32 sessionDigest = sha256(abi.encodePacked(userOp.sessionAccount, userOp.sessionExpiration));
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
