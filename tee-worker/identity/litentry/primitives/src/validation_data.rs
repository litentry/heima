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

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use crate::LitentryMultiSignature;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::ConstU32, BoundedVec};

// The size limit value should be 128 otherwise the message size will exceed the limit while link identity.
pub type ValidationString = BoundedVec<u8, ConstU32<128>>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TwitterValidationData {
	PublicTweet { tweet_id: ValidationString },
	OAuth2 { code: ValidationString, state: ValidationString, redirect_uri: ValidationString },
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DiscordValidationData {
	PublicMessage {
		channel_id: ValidationString,
		message_id: ValidationString,
		guild_id: ValidationString,
	},
	OAuth2 {
		code: ValidationString,
		redirect_uri: ValidationString,
	},
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct EmailValidationData {
	pub email: ValidationString,
	pub verification_code: ValidationString,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Web3CommonValidationData {
	pub message: ValidationString, // or String if under std
	pub signature: LitentryMultiSignature,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[allow(non_camel_case_types)]
pub enum Web2ValidationData {
	#[codec(index = 0)]
	Twitter(TwitterValidationData),
	#[codec(index = 1)]
	Discord(DiscordValidationData),
	#[codec(index = 2)]
	Email(EmailValidationData),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[allow(non_camel_case_types)]
pub enum Web3ValidationData {
	#[codec(index = 0)]
	Substrate(Web3CommonValidationData),
	#[codec(index = 1)]
	Evm(Web3CommonValidationData),
	#[codec(index = 2)]
	Bitcoin(Web3CommonValidationData),
	#[codec(index = 3)]
	Solana(Web3CommonValidationData),
}

impl Web3ValidationData {
	pub fn message(&self) -> &ValidationString {
		match self {
			Self::Substrate(data) => &data.message,
			Self::Evm(data) => &data.message,
			Self::Bitcoin(data) => &data.message,
			Self::Solana(data) => &data.message,
		}
	}

	pub fn signature(&self) -> &LitentryMultiSignature {
		match self {
			Self::Substrate(data) => &data.signature,
			Self::Evm(data) => &data.signature,
			Self::Bitcoin(data) => &data.signature,
			Self::Solana(data) => &data.signature,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ValidationData {
	#[codec(index = 0)]
	Web2(Web2ValidationData),
	#[codec(index = 1)]
	Web3(Web3ValidationData),
}
