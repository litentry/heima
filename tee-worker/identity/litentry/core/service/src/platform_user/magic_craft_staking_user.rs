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

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use core::result::Result;

use lc_data_providers::{
	magic_craft::{MagicCraftApi, MagicCraftClient},
	DataProviderConfig,
};

use crate::*;

pub fn is_user(
	addresses: Vec<String>,
	data_provider_config: &DataProviderConfig,
) -> Result<bool, Error> {
	let mut is_user = false;
	let mut client = MagicCraftClient::new(data_provider_config);
	for address in addresses {
		match client.user_verification(address, true) {
			Ok(response) => {
				is_user = response.user;
				if is_user {
					break;
				}
			},
			Err(err) => return Err(err.into_error_detail()),
		}
	}
	Ok(is_user)
}
