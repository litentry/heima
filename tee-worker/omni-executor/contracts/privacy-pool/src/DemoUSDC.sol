// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @notice Demo USDC for testnet — anyone can mint any amount.
/// 18 decimals (standard ERC20 default).
contract DemoUSDC is ERC20 {
    constructor() ERC20("Demo USDC", "USDC") {}

    /// @notice Mint tokens to yourself.
    function mint(uint256 amount) external {
        _mint(msg.sender, amount);
    }

    /// @notice Mint tokens to any address (useful for scripted demos).
    function mintFor(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
