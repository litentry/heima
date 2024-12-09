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
use executor_core::primitives::Intent;
use log::{error, info};
use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair, system_instruction::transfer};

// Executes intents on Solana network.
pub struct SolanaIntentExecutor {
	rpc_url: String,
}

impl SolanaIntentExecutor {
	pub fn new(rpc_url: &str) -> Result<Self, ()> {
		Ok(Self { rpc_url: rpc_url.to_string() })
	}
}

#[async_trait]
impl IntentExecutor for SolanaIntentExecutor {
	async fn execute(&self, intent: Intent) -> Result<(), ()> {
		info!("Executing intent: {:?}", intent);

		// let signer =

		Ok(())
	}
}
