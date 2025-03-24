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

use async_trait::async_trait;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::Intent;

pub struct CrossChainIntentExecutor {}

impl CrossChainIntentExecutor {
	pub fn new() -> Result<Self, ()> {
		Ok(Self {})
	}
}

#[async_trait]
impl IntentExecutor for CrossChainIntentExecutor {
	async fn execute(&self, intent: Intent) -> Result<(), ()> {
		match intent {
			Intent::CrossChainSwap(_swap_order) => {
				// TODO:
				// 1. Check if user has enough balance on the source chain
				// 2. Lock the balance on the source chain
				// 3. Swap assets:
				//    - Call accounting contract (e.g Swap SOL to TRUMP)
				//    - Call Binance convert via binance account (e.g Swap USDC to SOL)
				// 4. Send locked balance to binance account (refill)

				todo!("CrossChainSwap is not implemented yet");
			},
			_ => {
				log::error!("[CrossChainIntentExecutor]: Unsupported intent: {:?}", intent);
				return Err(());
			},
		}
	}
}
