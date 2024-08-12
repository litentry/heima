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

library Link {
    function getTokenRanges() internal pure returns (uint256[] memory) {
        uint256[] memory ranges = new uint256[](10);
        ranges[0] = 0 * Constants.decimals_factor;
        ranges[1] = 1 * Constants.decimals_factor;
        ranges[2] = 50 * Constants.decimals_factor;
        ranges[3] = 100 * Constants.decimals_factor;
        ranges[4] = 200 * Constants.decimals_factor;
        ranges[5] = 500 * Constants.decimals_factor;
        ranges[6] = 800 * Constants.decimals_factor;
        ranges[7] = 1200 * Constants.decimals_factor;
        ranges[8] = 1600 * Constants.decimals_factor;
        ranges[9] = 3000 * Constants.decimals_factor;

        return ranges;
    }

    function getTokenInfo() internal pure returns (TokenInfo[] memory) {
        TokenInfo[] memory tokenInfoList = new TokenInfo[](2);
        tokenInfoList[0] = TokenInfo(
            Web3Networks.Bsc,
            "0xf8a0bf9cf54bb92f17374d9e9a321e6a111a51bd",
            DataProviderTypes.NoderealClient,
            18
        );
        tokenInfoList[1] = TokenInfo(
            Web3Networks.Ethereum,
            "0x514910771af9ca656af840dff83e8264ecf986ca",
            DataProviderTypes.NoderealClient,
            18
        );
        return tokenInfoList;
    }
}
