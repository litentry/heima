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
library Eth {
    function getTokenRanges() internal pure returns (TokenInfoRanges memory) {
        uint256[] memory ranges = new uint256[](10);

        // all ranges multiplied by decimals_factor(100).
        // pub const ETH_AMOUNT_RANGE: [f64; 10] = [0.0, 0.01, 0.05, 0.2, 0.6, 1.2, 3.0, 8.0, 20.0, 50.0];
        ranges[0] = 0;
        ranges[1] = 1;
        ranges[2] = 5;
        ranges[3] = 20;
        ranges[4] = 60;
        ranges[5] = 120;
        ranges[6] = 300;
        ranges[7] = 800;
        ranges[8] = 2000;
        ranges[9] = 5000;

        return TokenInfoRanges(ranges, 2);
    }

    function getTokenNetworks()
        internal
        pure
        returns (TokenInfoNetwork[] memory)
    {
        TokenInfoNetwork[] memory networks = new TokenInfoNetwork[](2);
        networks[0] = TokenInfoNetwork(
            Web3Networks.Ethereum,
            "Native Token",
            DataProviderTypes.NoderealClient,
            18
        );
        networks[1] = TokenInfoNetwork(
            Web3Networks.Bsc,
            "0x2170ed0880ac9a755fd29b2688956bd959f933f8",
            DataProviderTypes.NoderealClient,
            18
        );

        return networks;
    }
}
