// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

// SPDX-License-Identifier: GPL-3.0-or-later

pragma solidity ^0.8.8;

import "../../libraries/Identities.sol";
import "../Constants.sol";

library Mcrt {
    function getTokenRanges() internal pure returns (uint256[] memory) {
        uint256[] memory ranges = new uint256[](6);
        ranges[0] = 0 * Constants.decimals_factor;
        ranges[1] = 2000 * Constants.decimals_factor;
        ranges[2] = 10000 * Constants.decimals_factor;
        ranges[3] = 50000 * Constants.decimals_factor;
        ranges[4] = 150000 * Constants.decimals_factor;
        ranges[5] = 500000 * Constants.decimals_factor;

        return ranges;
    }

    function getTokenInfo() internal pure returns (TokenInfo[] memory) {
        TokenInfo[] memory tokenInfoList = new TokenInfo[](3);
        tokenInfoList[0] = TokenInfo(
            Web3Networks.Ethereum,
            "0xde16ce60804a881e9f8c4ebb3824646edecd478d",
            DataProviderTypes.NoderealClient,
            18
        );
        tokenInfoList[1] = TokenInfo(
            Web3Networks.Bsc,
            "0x4b8285aB433D8f69CB48d5Ad62b415ed1a221e4f",
            DataProviderTypes.NoderealClient,
            18
        );
        tokenInfoList[2] = TokenInfo(
            Web3Networks.Solana,
            "FADm4QuSUF1K526LvTjvbJjKzeeipP6bj5bSzp3r6ipq",
            DataProviderTypes.MoralisClient,
            18
        );
        return tokenInfoList;
    }
}
