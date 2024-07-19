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
import "./BRC20.sol";

library Btcs {
	function getTokenRanges() internal pure returns (uint256[] memory) {
		uint256[] memory ranges = new uint256[](9);
		ranges[0] = 0 * Constants.decimals_factor;
		ranges[1] = 1 * Constants.decimals_factor;
		ranges[2] = 5 * Constants.decimals_factor;
		ranges[3] = 20 * Constants.decimals_factor;
		ranges[4] = 50 * Constants.decimals_factor;
		ranges[5] = 100 * Constants.decimals_factor;
		ranges[6] = 200 * Constants.decimals_factor;
		ranges[7] = 500 * Constants.decimals_factor;
		ranges[8] = 800 * Constants.decimals_factor;
		return ranges;
	}
	function getTokenInfo() internal pure returns (TokenInfo[] memory) {
		uint32[] memory networks = BRC20.getDefaultTokenNetworks();
		TokenInfo[] memory tokenInfoList = new TokenInfo[](networks.length);
		for (uint i = 0; i < networks.length; i++) {
			tokenInfoList[i] = TokenInfo(networks[i], "");
		}

		return tokenInfoList;
	}
}
