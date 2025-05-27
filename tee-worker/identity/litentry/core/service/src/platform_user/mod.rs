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

use lc_data_providers::DataProviderConfig;
use litentry_primitives::PlatformUserType;

use crate::*;

mod daren_market_user;
mod karat_dao_user;
mod magic_craft_staking_user;

pub fn is_user(
	platform_user_type: PlatformUserType,
	addresses: Vec<String>,
	data_provider_config: &DataProviderConfig,
) -> Result<bool, Error> {
	match platform_user_type {
		PlatformUserType::KaratDao => karat_dao_user::is_user(addresses, data_provider_config),
		PlatformUserType::MagicCraftStaking => {
			magic_craft_staking_user::is_user(addresses, data_provider_config)
		},
		PlatformUserType::DarenMarket => {
			daren_market_user::is_user(addresses, data_provider_config)
		},
	}
}
