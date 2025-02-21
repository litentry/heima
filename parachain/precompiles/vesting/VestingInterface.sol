// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface IVesting {
    /// @notice Used to unlock vest.
    /// @custom:selector 0x458efde3
    ///                  vest()
    function vest() external;
}