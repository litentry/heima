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

import "../libraries/Http.sol";
import "../libraries/Utils.sol";
import "../libraries/Identities.sol";

library BlockchainInfoClient {
    function getMultiAddress(
        string[] memory accounts
    ) internal returns (bool, int64) {
        string memory url = "https://blockchain.info/multiaddr";
        string memory activeQueryParam = "";

        for (uint256 i = 0; i < accounts.length; i++) {
            activeQueryParam = string(
                abi.encodePacked(activeQueryParam, accounts[i])
            );
            if (i != accounts.length - 1) {
                activeQueryParam = string(
                    abi.encodePacked(activeQueryParam, "|")
                );
            }
        }

        url = string(
            abi.encodePacked(url, "?active=", activeQueryParam, "&n=", "0")
        );

        HttpHeader[] memory headers = new HttpHeader[](0);
        return Http.GetI64(url, "/wallet/final_balance", headers);
    }

    function isSupportedNetwork(uint32 network) internal pure returns (bool) {
        return
            network == Web3Networks.BitcoinP2tr ||
            network == Web3Networks.BitcoinP2pkh ||
            network == Web3Networks.BitcoinP2sh ||
            network == Web3Networks.BitcoinP2wpkh ||
            network == Web3Networks.BitcoinP2wsh;
    }

    function getTokenBalance(
        string[] memory accounts
    ) internal returns (uint256) {
        (bool balanceSuccess, int64 balance) = BlockchainInfoClient
            .getMultiAddress(accounts);
        if (balanceSuccess) {
            return uint256(uint64(balance));
        } else {
            return 0;
        }
    }
}
