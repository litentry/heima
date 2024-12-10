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
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
	commitment_config::CommitmentConfig,
	pubkey::Pubkey,
	signer::{keypair::Keypair, EncodableKey, Signer},
	system_instruction,
	transaction::Transaction,
};

// Executes intents on Solana network.
pub struct SolanaIntentExecutor {
	rpc_client: RpcClient,
}

impl SolanaIntentExecutor {
	pub fn new(rpc_url: String) -> Result<Self, ()> {
		let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
		Ok(Self { rpc_client: client })
	}
}

#[async_trait]
impl IntentExecutor for SolanaIntentExecutor {
	async fn execute(&self, intent: Intent) -> Result<(), ()> {
		info!("Executing intent: {:?}", intent);
		// TODO: get key from key store
		let signer_key_pair = Keypair::read_from_file("dev-key.json")
			.map_err(|e| error!("Could not read key: {:?}", e))?;
		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| error!("Could not get block hash: {:?}", e))?;

		match intent {
			Intent::TransferSolana(to, amount) => {
				let to = Pubkey::new_from_array(to);
				let transfer_instruction =
					system_instruction::transfer(&signer_key_pair.pubkey(), &to, amount);
				let tx = Transaction::new_signed_with_payer(
					&[transfer_instruction],
					Some(&signer_key_pair.pubkey()),
					&[&signer_key_pair],
					block_hash,
				);
				let _signature = self
					.rpc_client
					.send_and_confirm_transaction(&tx)
					.await
					.map_err(|e| error!("Could not send transaction: {:?}", e))?;
			},
			_ => {
				error!("[SolanaIntentExecutor]: Unsupported intent: {:?}", intent);
				return Err(());
			},
		}

		Ok(())
	}
}
