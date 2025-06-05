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

use crate::FromStr;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

const BNB_999_CLUB_MEMBER_LENGTH: usize = 3;
const BNB_9999_CLUB_MEMBER_LENGTH: usize = 4;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum BnbDigitDomainType {
	Bnb999ClubMember, // 000-999.bnb
	Bnb10kClubMember, // 0000-9999.bnb
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct DigitDomainTypeError;

impl BnbDigitDomainType {
	pub fn is_bnb_999_club_member(domain_name: &str) -> bool {
		if domain_name.len() == BNB_999_CLUB_MEMBER_LENGTH && domain_name.parse::<usize>().is_ok() {
			return true;
		}

		false
	}

	pub fn is_bnb_9999_club_member(domain_name: &str) -> bool {
		if domain_name.len() == BNB_9999_CLUB_MEMBER_LENGTH && domain_name.parse::<usize>().is_ok()
		{
			return true;
		}

		false
	}

	pub fn is_member(&self, domain_name: &str) -> bool {
		if let Ok(member) = BnbDigitDomainType::from_str(domain_name) {
			return *self == member;
		}

		false
	}
}

impl FromStr for BnbDigitDomainType {
	type Err = DigitDomainTypeError;

	fn from_str(domain_name: &str) -> Result<Self, Self::Err> {
		if BnbDigitDomainType::is_bnb_999_club_member(domain_name) {
			return Ok(BnbDigitDomainType::Bnb999ClubMember);
		}

		if BnbDigitDomainType::is_bnb_9999_club_member(domain_name) {
			return Ok(BnbDigitDomainType::Bnb10kClubMember);
		}

		Err(DigitDomainTypeError)
	}
}
