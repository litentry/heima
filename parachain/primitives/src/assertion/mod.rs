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

pub mod achainable;
use achainable::AchainableParams;

pub mod bnb_domain;
use bnb_domain::BnbDigitDomainType;

pub mod contest;

pub mod evm_amount_holding;
use evm_amount_holding::EVMTokenType;

pub mod generic_discord_role;
use generic_discord_role::GenericDiscordRoleType;

pub mod network;
use network::{all_web3networks, BoundedWeb3Network, Web3Network};

pub mod oneblock;
use oneblock::OneBlockCourseType;

pub mod platform_user;
use platform_user::PlatformUserType;

pub mod soraquiz;

pub mod vip3;
use vip3::VIP3MembershipCardLevel;

pub mod web3_nft;
use web3_nft::Web3NftType;

pub mod web3_token;
use web3_token::Web3TokenType;

pub mod dynamic;
use dynamic::DynamicParams;

use crate::{vec, AccountId, ParameterString, Vec};

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

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
	A8(BoundedWeb3Network),                                 // litentry, polkadot, kusama, khala, ethereum
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

	#[codec(index = 27)]
	Dynamic(DynamicParams),

	#[codec(index = 28)]
	LinkedIdentities,
}

const A8_SUPPORTED_NETWORKS: [Web3Network; 5] = [
	Web3Network::Polkadot,
	Web3Network::Kusama,
	Web3Network::Litentry,
	Web3Network::Khala,
	Web3Network::Ethereum,
];

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
			Self::A4(..) => vec![Web3Network::Litentry, Web3Network::Ethereum],
			Self::A7(..) | Self::A14 => vec![Web3Network::Polkadot],
			// WBTC/ETH holder
			Self::A10(..)
			| Self::A11(..)
			| Self::VIP3MembershipCard(..)
			| Self::WeirdoGhostGangHolder => vec![Web3Network::Ethereum],
			// total tx over `networks`
			Self::A8(networks) => networks
				.into_iter()
				.filter(|network| A8_SUPPORTED_NETWORKS.contains(*network))
				.cloned()
				.collect::<Vec<_>>(),
			// Achainable Assertions
			Self::Achainable(arg) => arg.chains(),
			// OneBlock Assertion
			Self::OneBlock(..) => vec![Web3Network::Polkadot, Web3Network::Kusama],
			// SPACEID Assertions
			Self::BnbDomainHolding | Self::BnbDigitDomainClub(..) => vec![Web3Network::Bsc],
			// LITStaking
			Self::LITStaking => vec![Web3Network::Litentry],
			// EVM Amount Holding
			Self::EVMAmountHolding(_) | Self::CryptoSummary => {
				vec![Web3Network::Ethereum, Web3Network::Bsc]
			},
			// BRC20 Holder
			Self::BRC20AmountHolder => vec![Web3Network::BitcoinP2tr],
			//
			// general rules
			//
			// any web3 network is allowed
			Self::A1 | Self::A13(..) | Self::A20 => all_web3networks(),
			// no web3 network is allowed
			Self::A2(..)
			| Self::A3(..)
			| Self::A6
			| Self::GenericDiscordRole(..)
			| Self::LinkedIdentities => vec![],
			Self::TokenHoldingAmount(t_type) => t_type.get_supported_networks(),
			Self::PlatformUser(p_type) => p_type.get_supported_networks(),
			Self::NftHolder(t_type) => t_type.get_supported_networks(),
			Self::Dynamic(..) => all_web3networks(),
		}
	}

	// Used in `get_eligible_identities` to decide if we should pass identities through
	// and let assertion logic handle them
	#[allow(clippy::match_like_matches_macro)]
	pub fn skip_identity_filtering(&self) -> bool {
		match self {
			Self::A1 | Self::Dynamic(..) | Self::LinkedIdentities => true,
			_ => false,
		}
	}
}
