/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

//! Interface for direct access to a workers rpc.

use crate::{direct_client::DirectApi, error::Result};
use codec::Decode;
use frame_metadata::RuntimeMetadataPrefixed;
use itp_api_client_types::Metadata;
use itp_stf_primitives::types::ShardIdentifier;
use itp_types::MrEnclave;
use litentry_primitives::Identity;
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use std::{sync::mpsc::Sender as MpscSender, thread::JoinHandle};

#[derive(Clone, Default)]
pub struct DirectClientMock {
	rsa_pubkey: Rsa3072PubKey,
	mu_ra_url: String,
	untrusted_worker_url: String,
	metadata: String,
	nonce: u32,
}

impl DirectClientMock {
	pub fn new(
		rsa_pubkey: Rsa3072PubKey,
		mu_ra_url: String,
		untrusted_worker_url: String,
		metadata: String,
		nonce: u32,
	) -> Self {
		Self { rsa_pubkey, mu_ra_url, untrusted_worker_url, metadata, nonce }
	}

	pub fn with_rsa_pubkey(mut self, key: Rsa3072PubKey) -> Self {
		self.rsa_pubkey = key;
		self
	}

	pub fn with_mu_ra_url(mut self, url: &str) -> Self {
		self.mu_ra_url = url.to_string();
		self
	}

	pub fn with_untrusted_worker_url(mut self, url: &str) -> Self {
		self.untrusted_worker_url = url.to_string();
		self
	}

	pub fn with_metadata(mut self, metadata: String) -> Self {
		self.metadata = metadata;
		self
	}

	pub fn with_nonce(mut self, nonce: u32) -> Self {
		self.nonce = nonce;
		self
	}
}

impl DirectApi for DirectClientMock {
	fn get(&self, _request: &str) -> Result<String> {
		Ok("Hello_world".to_string())
	}

	fn watch(&self, _request: String, _sender: MpscSender<String>) -> JoinHandle<()> {
		unimplemented!()
	}

	fn get_rsa_pubkey(&self) -> Result<Rsa3072PubKey> {
		Ok(self.rsa_pubkey)
	}

	fn get_mu_ra_url(&self) -> Result<String> {
		Ok(self.mu_ra_url.clone())
	}

	fn get_untrusted_worker_url(&self) -> Result<String> {
		Ok(self.untrusted_worker_url.clone())
	}

	fn get_state_metadata(&self) -> Result<Metadata> {
		let metadata = RuntimeMetadataPrefixed::decode(&mut self.metadata.as_bytes())?;
		Metadata::try_from(metadata).map_err(|e| e.into())
	}

	fn send(&self, _request: &str) -> Result<()> {
		unimplemented!()
	}

	fn import_sidechain_blocks(&self, _blocks_encoded: String) -> Result<()> {
		Ok(())
	}

	fn close(&self) -> Result<()> {
		unimplemented!()
	}

	fn get_state_metadata_raw(&self) -> Result<String> {
		unimplemented!()
	}

	fn get_next_nonce(&self, _shard: &ShardIdentifier, _account: &Identity) -> Result<u32> {
		Ok(self.nonce)
	}

	fn get_state_mrenclave(&self) -> Result<MrEnclave> {
		unimplemented!()
	}
}
