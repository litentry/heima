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
// along with Litentry. If not, see <https://www.gnu.org/licenses/>.

use litentry_primitives::{AchainableParams, Assertion};
use std::string::{String, ToString};

const BASE_URL: &str = "https://raw.githubusercontent.com/litentry/vc-jsonschema/main/dist/schemas";
const NOT_IMPLEMENTED: &str =
	"https://raw.githubusercontent.com/litentry/vc-jsonschema/main/dist/schemas/0-base.json";

/// Returns the respective JSON Schema for the given assertion and its credential.
/// JSON Schemas can be found at https://github.com/litentry/vc-jsonschema
pub fn get_schema_url(assertion: &Assertion) -> Option<String> {
	match assertion {
		Assertion::A1 => Some(format!("{BASE_URL}/1-basic-identity-verification/1-1-0.json")),

		Assertion::A2(_) => Some(format!("{BASE_URL}/2-litentry-discord-member/1-1-0.json")),

		Assertion::A3(_, _, _) => Some(format!("{BASE_URL}/3-active-discord-id-hubber/1-1-0.json")),

		Assertion::A4(_) => Some(format!("{BASE_URL}/4-token-holding-time/1-1-0.json")),

		Assertion::A6 => Some(format!("{BASE_URL}/6-twitter-follower-amount/1-1-1.json")),

		Assertion::A7(_) => Some(format!("{BASE_URL}/4-token-holding-time/1-1-0.json")),

		Assertion::A8(_) => {
			Some(format!("{BASE_URL}/7-evm-substrate-transaction-count/1-1-0.json"))
		},

		Assertion::A10(_) => Some(format!("{BASE_URL}/4-token-holding-time/1-1-0.json")),

		Assertion::A11(_) => Some(format!("{BASE_URL}/4-token-holding-time/1-1-0.json")),

		Assertion::A13(_) => {
			Some(format!("{BASE_URL}/8-decoded-2023-basic-special-badge/1-1-0.json"))
		},

		Assertion::A14 => {
			Some(format!("{BASE_URL}/9-polkadot-governance-participation-proof/1-1-0.json"))
		},

		Assertion::Achainable(params) => match params {
			AchainableParams::AmountHolding(_) => {
				Some(format!("{BASE_URL}/17-token-holding-amount/1-1-2.json"))
			},

			AchainableParams::AmountToken(_) => {
				Some(format!("{BASE_URL}/17-token-holding-amount/1-1-2.json"))
			},

			AchainableParams::Amount(_) => Some(format!("{BASE_URL}/11-token-holder/1-1-0.json")),

			AchainableParams::Basic(_) => Some(format!("{BASE_URL}/11-token-holder/1-1-0.json")),

			AchainableParams::ClassOfYear(_) => {
				Some(format!("{BASE_URL}/10-account-class-of-year/1-1-0.json"))
			},

			AchainableParams::Mirror(_) => {
				Some(format!("{BASE_URL}/22-mirror-contributor/1-1-0.json"))
			},

			// The following assertions are Unused and produce no specific claims. They Generates
			// generic JSON Credentials
			AchainableParams::Amounts(_) => Some(NOT_IMPLEMENTED.to_string()),
			AchainableParams::BetweenPercents(_) => Some(NOT_IMPLEMENTED.to_string()),
			AchainableParams::Date(_) => Some(NOT_IMPLEMENTED.to_string()),
			AchainableParams::DateInterval(_) => Some(NOT_IMPLEMENTED.to_string()),
			AchainableParams::DatePercent(_) => Some(NOT_IMPLEMENTED.to_string()),
			AchainableParams::Token(_) => Some(NOT_IMPLEMENTED.to_string()),
		},

		Assertion::A20 => Some(format!("{BASE_URL}/12-idhub-evm-version-early-bird/1-1-0.json")),

		Assertion::OneBlock(_) => {
			Some(format!("{BASE_URL}/13-oneblock-student-phase-12/1-1-0.json"))
		},

		Assertion::GenericDiscordRole(_) => {
			Some(format!("{BASE_URL}/14-generic-discord-role/1-1-0.json"))
		},

		Assertion::BnbDomainHolding => {
			Some(format!("{BASE_URL}/15-bnb-domain-holding-amount/1-1-1.json"))
		},

		Assertion::BnbDigitDomainClub(_) => {
			Some(format!("{BASE_URL}/16-bnb-3d-4d-club-domain-holding-amount/1-1-1.json"))
		},

		Assertion::VIP3MembershipCard(_) => {
			Some(format!("{BASE_URL}/19-vip3-card-holder/1-1-0.json"))
		},

		Assertion::WeirdoGhostGangHolder => {
			Some(format!("{BASE_URL}/18-weirdoghostgang-holder/1-1-0.json"))
		},

		Assertion::LITStaking => Some(format!("{BASE_URL}/17-token-holding-amount/1-1-2.json")),

		Assertion::EVMAmountHolding(_) => {
			Some(format!("{BASE_URL}/21-evm-holding-amount/1-1-2.json"))
		},

		Assertion::BRC20AmountHolder => {
			Some(format!("{BASE_URL}/20-token-holding-amount-list/1-1-1.json"))
		},

		Assertion::CryptoSummary => Some(format!("{BASE_URL}/23-crypto-summary/1-1-0.json")),

		Assertion::PlatformUser(_) => Some(format!("{BASE_URL}/24-platform-user/1-1-2.json")),

		Assertion::NftHolder(_) => Some(format!("{BASE_URL}/26-nft-holder/1-1-2.json")),

		Assertion::TokenHoldingAmount(_) => {
			Some(format!("{BASE_URL}/25-token-holding-amount/1-1-4.json"))
		},

		Assertion::Dynamic(..) => None,

		Assertion::LinkedIdentities => Some(format!("{BASE_URL}/27-linked-identities/1-0-0.json")),
	}
}
