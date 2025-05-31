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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{traits::ConstU32, BoundedVec};

pub type DynamicContractParams = BoundedVec<u8, ConstU32<1024>>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct DynamicParams {
	// smart contract code identifier
	pub smart_contract_id: H160,
	// abi encoded smart contract params
	pub smart_contract_params: Option<DynamicContractParams>,
	// true to return contract log
	pub return_log: bool,
}
