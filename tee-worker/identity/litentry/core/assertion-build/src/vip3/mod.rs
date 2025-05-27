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

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

pub mod card;

use crate::*;
use lc_common::abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls};
use lc_data_providers::{
	vip3::{VIP3Client, VIP3QuerySet},
	DataProviderConfig,
};
use litentry_primitives::VIP3MembershipCardLevel;

pub struct VIP3SBTInfo {
	pub client: VIP3Client,
}

impl VIP3SBTInfo {
	pub fn new(
		data_provider_config: &DataProviderConfig,
	) -> core::result::Result<VIP3SBTInfo, ErrorDetail> {
		let client = VIP3Client::new(data_provider_config);

		Ok(VIP3SBTInfo { client })
	}
}

pub trait VIP3SBTLogicInterface {
	fn has_card_level(
		&mut self,
		addresses: Vec<String>,
		level: &VIP3MembershipCardLevel,
	) -> core::result::Result<bool, ErrorDetail>;
}

impl VIP3SBTLogicInterface for VIP3SBTInfo {
	fn has_card_level(
		&mut self,
		addresses: Vec<String>,
		level: &VIP3MembershipCardLevel,
	) -> core::result::Result<bool, ErrorDetail> {
		debug!("HAS specific card level");

		let mut result = false;

		loop_with_abort_strategy(
			addresses,
			|address| match self.client.sbt_info(address) {
				Ok(info) => {
					if info.data.level == level.to_level() {
						result = true;
						Ok(LoopControls::Break)
					} else {
						Ok(LoopControls::Continue)
					}
				},
				Err(e) => Err(e.into_error_detail()),
			},
			AbortStrategy::FailFast::<fn(&_) -> bool>,
		)
		.map_err(|errors| errors[0].clone())?;

		Ok(result)
	}
}
