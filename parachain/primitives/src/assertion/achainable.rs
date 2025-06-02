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
	assertion::network::{BoundedWeb3Network, Web3Network},
	ParameterString, Vec,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

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
