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

import "@openzeppelin/contracts/utils/Strings.sol";
import "../libraries/AssertionLogic.sol";
import "../libraries/Identities.sol";
import "../DynamicAssertion.sol";
import "./Constants.sol";
import "../libraries/StringShift.sol";

abstract contract TokenHoldingAmount is DynamicAssertion {
    mapping(string => uint256[]) internal tokenRanges;
    function execute(
        Identity[] memory identities,
        string[] memory secrets,
        bytes memory params
    )
        public
        override
        returns (
            string memory,
            string memory,
            string[] memory,
            string memory,
            bool
        )
    {
        string
            memory description = "The amount of a particular token you are holding";
        string memory assertion_type = "Token Holding Amount";
        schema_url = "https://raw.githubusercontent.com/litentry/vc-jsonschema/main/dist/schemas/25-token-holding-amount/1-1-3.json";

        string memory tokenLowercaseName = abi.decode(params, (string));

        require(
            tokenRanges[tokenLowercaseName].length > 0,
            "Token not supported or not found"
        );

        uint256 balance = queryTotalBalance(
            identities,
            secrets,
            tokenLowercaseName
        );

        (uint256 index, uint256 min, int256 max) = calculateRange(
            balance,
            tokenRanges[tokenLowercaseName]
        );

        string[] memory assertions = assembleAssertions(
            min,
            max,
            tokenLowercaseName
        );

        bool result = index > 0 || balance > 0;

        return (description, assertion_type, assertions, schema_url, result);
    }

    function queryTotalBalance(
        Identity[] memory identities,
        string[] memory secrets,
        string memory tokenName
    ) internal virtual returns (uint256) {
        uint256 total_balance = 0;
        uint256 identitiesLength = identities.length;

        for (uint256 i = 0; i < identitiesLength; i++) {
            Identity memory identity = identities[i];
            uint256 networksLength = identity.networks.length;
            for (uint32 j = 0; j < networksLength; j++) {
                uint32 network = identity.networks[j];
                if (isSupportedNetwork(tokenName, network)) {
                    total_balance += queryBalance(
                        identity,
                        network,
                        secrets,
                        tokenName
                    );
                }
            }
        }

        return total_balance;
    }

    function calculateRange(
        uint256 balance,
        uint256[] memory ranges
    ) private view returns (uint256, uint256, int256) {
        uint256 index = ranges.length - 1;
        uint256 min = 0;
        int256 max = 0;

        for (uint32 i = 1; i < ranges.length; i++) {
            if (
                balance * Constants.decimals_factor <
                ranges[i] * 10 ** getTokenDecimals()
            ) {
                index = i - 1;
                break;
            }
        }

        if (index == ranges.length - 1) {
            min = ranges[index];
            max = -1;
        } else {
            min = ranges[index];
            max = int256(ranges[index + 1]);
        }
        return (index, min, max);
    }

    function assembleAssertions(
        uint256 min,
        int256 max,
        string memory tokenName
    ) private pure returns (string[] memory) {
        string memory variable = "$holding_amount";
        AssertionLogic.CompositeCondition memory cc = AssertionLogic
            .CompositeCondition(
                new AssertionLogic.Condition[](max > 0 ? 3 : 2),
                true
            );
        AssertionLogic.andOp(
            cc,
            0,
            "$token",
            AssertionLogic.Op.Equal,
            tokenName
        );
        AssertionLogic.andOp(
            cc,
            1,
            variable,
            AssertionLogic.Op.GreaterEq,
            StringShift.toShiftedString(min, Constants.decimals_factor)
        );
        if (max > 0) {
            AssertionLogic.andOp(
                cc,
                2,
                variable,
                AssertionLogic.Op.LessThan,
                StringShift.toShiftedString(
                    uint256(max),
                    Constants.decimals_factor
                )
            );
        }

        string[] memory assertions = new string[](1);
        assertions[0] = AssertionLogic.toString(cc);

        return assertions;
    }

    function getTokenDecimals() internal view virtual returns (uint8);

    function isSupportedNetwork(
        string memory tokenName,
        uint32 network
    ) internal view virtual returns (bool);

    function queryBalance(
        Identity memory identity,
        uint32 network,
        string[] memory secrets,
        string memory tokenName
    ) internal virtual returns (uint256);
}
