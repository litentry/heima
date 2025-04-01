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
use std::string::ToString;

use lc_common::abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls};
use lc_data_providers::{
	daren_market::{DarenMarketApi, DarenMarketClient},
	DataProviderConfig,
};

use crate::*;

pub fn is_user(
	addresses: Vec<String>,
	data_provider_config: &DataProviderConfig,
) -> Result<bool, Error> {
	let mut result = false;
	let mut client = DarenMarketClient::new(data_provider_config);

	loop_with_abort_strategy(
		addresses,
		|address| match client.user_verification(address.to_string(), true) {
			Ok(response) => {
				if response.created {
					result = true;
					Ok(LoopControls::Break)
				} else {
					Ok(LoopControls::Continue)
				}
			},
			Err(err) => Err(err.into_error_detail()),
		},
		AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
	)
	.map_err(|errors| errors[0].clone())?;

	Ok(result)
}
