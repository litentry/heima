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

// This file includes the predefined rulesets and the corresponding parameters
// when requesting VCs.

use crate::{
	all_web3networks, AccountId, BnbDigitDomainType, BoundedWeb3Network, EVMTokenType,
	GenericDiscordRoleType, OneBlockCourseType, PlatformUserType, VIP3MembershipCardLevel,
	Web3Network, Web3NftType, Web3TokenType,
};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::ConstU32, BoundedVec};
use sp_std::{vec, vec::Vec};

pub type ParameterString = BoundedVec<u8, ConstU32<64>>;

macro_rules! AchainableRequestParams {
	($type_name:ident, {$( $field_name:ident : $field_type:ty , )* }) => {
		#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
		pub struct $type_name {
			pub name: ParameterString,
			pub chain: BoundedWeb3Network,
			$( pub $field_name: $field_type ),*
		}
	};
}

AchainableRequestParams!(AchainableAmountHolding, {
	amount: ParameterString,
	date: ParameterString,
	token: Option<ParameterString>,
});

AchainableRequestParams!(AchainableAmountToken, {
	amount: ParameterString,
	token: Option<ParameterString>,
});

AchainableRequestParams!(AchainableAmount, {
	amount: ParameterString,
});

AchainableRequestParams!(AchainableAmounts, {
	amount1: ParameterString,
	amount2: ParameterString,
});

AchainableRequestParams!(AchainableBasic, {});

AchainableRequestParams!(AchainableBetweenPercents, {
	greater_than_or_equal_to: ParameterString,
	less_than_or_equal_to: ParameterString,
});

AchainableRequestParams!(AchainableClassOfYear, {});

AchainableRequestParams!(AchainableDateInterval, {
	start_date: ParameterString,
	end_date: ParameterString,
});

AchainableRequestParams!(AchainableDatePercent, {
	token: ParameterString,
	date: ParameterString,
	percent: ParameterString,
});

AchainableRequestParams!(AchainableDate, {
	date: ParameterString,
});

AchainableRequestParams!(AchainableToken, {
	token: ParameterString,
});

AchainableRequestParams!(AchainableMirror, {
	post_quantity: Option<ParameterString>,
});

#[rustfmt::skip]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum AchainableParams {
	#[codec(index = 0)]
	AmountHolding(AchainableAmountHolding),
	#[codec(index = 1)]
	AmountToken(AchainableAmountToken),
	#[codec(index = 2)]
	Amount(AchainableAmount),
	#[codec(index = 3)]
	Amounts(AchainableAmounts),
	#[codec(index = 4)]
	Basic(AchainableBasic),
	#[codec(index = 5)]
	BetweenPercents(AchainableBetweenPercents),
	#[codec(index = 6)]
	ClassOfYear(AchainableClassOfYear),
	#[codec(index = 7)]
	DateInterval(AchainableDateInterval),
	#[codec(index = 8)]
	DatePercent(AchainableDatePercent),
	#[codec(index = 9)]
	Date(AchainableDate),
	#[codec(index = 10)]
	Token(AchainableToken),
	#[codec(index = 11)]
	Mirror(AchainableMirror),
}

impl AchainableParams {
	pub fn name(&self) -> ParameterString {
		match self {
			AchainableParams::AmountHolding(p) => p.name.clone(),
			AchainableParams::AmountToken(p) => p.name.clone(),
			AchainableParams::Amount(p) => p.name.clone(),
			AchainableParams::Amounts(p) => p.name.clone(),
			AchainableParams::Basic(p) => p.name.clone(),
			AchainableParams::BetweenPercents(p) => p.name.clone(),
			AchainableParams::ClassOfYear(p) => p.name.clone(),
			AchainableParams::DateInterval(p) => p.name.clone(),
			AchainableParams::DatePercent(p) => p.name.clone(),
			AchainableParams::Date(p) => p.name.clone(),
			AchainableParams::Token(p) => p.name.clone(),
			AchainableParams::Mirror(p) => p.name.clone(),
		}
	}

	pub fn chains(&self) -> Vec<Web3Network> {
		match self {
			AchainableParams::AmountHolding(arg) => arg.chain.to_vec(),
			AchainableParams::AmountToken(arg) => arg.chain.to_vec(),
			AchainableParams::Amount(arg) => arg.chain.to_vec(),
			AchainableParams::Amounts(arg) => arg.chain.to_vec(),
			AchainableParams::Basic(arg) => arg.chain.to_vec(),
			AchainableParams::BetweenPercents(arg) => arg.chain.to_vec(),
			AchainableParams::ClassOfYear(arg) => arg.chain.to_vec(),
			AchainableParams::DateInterval(arg) => arg.chain.to_vec(),
			AchainableParams::DatePercent(arg) => arg.chain.to_vec(),
			AchainableParams::Date(arg) => arg.chain.to_vec(),
			AchainableParams::Token(arg) => arg.chain.to_vec(),
			AchainableParams::Mirror(arg) => arg.chain.to_vec(),
		}
	}
}

#[rustfmt::skip]
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum Assertion {
	#[codec(index = 0)]
	A1,
	#[codec(index = 1)]
	A2(ParameterString),                                    // (guild_id)
	#[codec(index = 2)]
	A3(ParameterString, ParameterString, ParameterString),  // (guild_id, channel_id, role_id)
	#[codec(index = 3)]
	A4(ParameterString),                                    // (minimum_amount)
	#[codec(index = 4)]
	A6,
	#[codec(index = 5)]
	A7(ParameterString),                                    // (minimum_amount)
	#[codec(index = 6)]
	A8(BoundedWeb3Network),                                 // litentry, litmus, polkadot, kusama, khala, ethereum
	#[codec(index = 7)]
	A10(ParameterString),                                   // (minimum_amount)
	#[codec(index = 8)]
	A11(ParameterString),                                   // (minimum_amount)

	// ----- begin polkadot decoded 2023 -----
	#[codec(index = 9)]
	A13(AccountId),                                         // (participant_account), can only be requested by delegatee
	#[codec(index = 10)]
	A14,
	// for Holder assertions we'll reuse A4/A7
	// ----- end polkadot decoded 2023 -----
	#[codec(index = 11)]
	Achainable(AchainableParams),

	// For EVM Version Early Bird
	#[codec(index = 12)]
	A20,

	// For OneBlock
	#[codec(index = 13)]
	OneBlock(OneBlockCourseType),

	// GenericDiscordRole
	#[codec(index = 14)]
	GenericDiscordRole(GenericDiscordRoleType),  // (generic_discord_role_type)

	// ----- begin SPACEID -----
	#[codec(index = 16)]
	BnbDomainHolding,

	#[codec(index = 17)]
	BnbDigitDomainClub(BnbDigitDomainType),
	// ----- end SPACEID -----

	#[codec(index = 18)]
	VIP3MembershipCard(VIP3MembershipCardLevel),

	#[codec(index = 19)]
	WeirdoGhostGangHolder,

	#[codec(index = 20)]
	LITStaking,

	#[codec(index = 21)]
	EVMAmountHolding(EVMTokenType),  // (evm_token_type)

	#[codec(index = 22)]
	BRC20AmountHolder,

	#[codec(index = 23)]
	CryptoSummary,

	#[codec(index = 24)]
	TokenHoldingAmount(Web3TokenType),

	#[codec(index = 25)]
	PlatformUser(PlatformUserType),

	#[codec(index = 26)]
	NftHolder(Web3NftType),
}

impl Assertion {
	// Given an assertion enum type, retrieve the supported web3 networks.
	// So we limit the network types on the assertion definition level.
	//
	// The final networks used for assertion building are the common set of:
	// - "assertion supported networks" which are defined here, and
	// - "identity networks" which are defined by the user and stored in `IdentityContext`
	//
	// returns a vector of `Web3Network` guarantees it's a subnet of
	// the broader `Web3Network` (see network.rs)
	pub fn get_supported_web3networks(&self) -> Vec<Web3Network> {
		match self {
			// LIT holder, not including `LitentryRococo` as it's not supported by any data provider
			Self::A4(..) => vec![Web3Network::Litentry, Web3Network::Litmus, Web3Network::Ethereum],
			// DOT holder
			Self::A7(..) => vec![Web3Network::Polkadot],
			// WBTC/ETH holder
			Self::A10(..) |
			Self::A11(..) |
			Self::VIP3MembershipCard(..) |
			Self::WeirdoGhostGangHolder => vec![Web3Network::Ethereum],
			// total tx over `networks`
			Self::A8(network) => network.to_vec(),
			// polkadot paticipation
			Self::A14 => vec![Web3Network::Polkadot],
			// Achainable Assertions
			Self::Achainable(arg) => arg.chains(),
			// OneBlock Assertion
			Self::OneBlock(..) => vec![Web3Network::Polkadot, Web3Network::Kusama],
			// SPACEID Assertions
			Self::BnbDomainHolding | Self::BnbDigitDomainClub(..) => vec![Web3Network::Bsc],
			// LITStaking
			Self::LITStaking => vec![Web3Network::Litentry],
			// EVM Amount Holding
			Self::EVMAmountHolding(_) | Self::CryptoSummary =>
				vec![Web3Network::Ethereum, Web3Network::Bsc],
			// BRC20 Holder
			Self::BRC20AmountHolder => vec![Web3Network::BitcoinP2tr],
			//
			// general rules
			//
			// any web3 network is allowed
			Self::A1 | Self::A13(..) | Self::A20 => all_web3networks(),
			// no web3 network is allowed
			Self::A2(..) | Self::A3(..) | Self::A6 | Self::GenericDiscordRole(..) => vec![],
			Self::TokenHoldingAmount(t_type) => t_type.get_supported_networks(),
			Self::PlatformUser(p_type) => p_type.get_supported_networks(),
			Self::NftHolder(t_type) => t_type.get_supported_networks(),
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum AmountHoldingTimeType {
	#[codec(index = 0)]
	LIT,
	#[codec(index = 1)]
	DOT,
	#[codec(index = 2)]
	WBTC,
	#[codec(index = 3)]
	ETH,
}
