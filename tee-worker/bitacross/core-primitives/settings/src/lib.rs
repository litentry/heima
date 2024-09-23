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

//! Common settings for the worker and the enclave. It is strictly `no_std`

#![no_std]

pub mod files {
	// used by worker
	pub static ENCLAVE_TOKEN: &str = "enclave.token";
	pub static ENCLAVE_FILE: &str = "enclave.signed.so";
	pub static SHIELDING_KEY_FILE: &str = "enclave-shielding-pubkey.json";
	pub static SIGNING_KEY_FILE: &str = "enclave-signing-pubkey.bin";

	// used by enclave
	/// Path to the light-client db for the Integritee parentchain.
	pub const LITENTRY_PARENTCHAIN_LIGHT_CLIENT_DB_PATH: &str = "litentry_lcdb";

	/// Path to the light-client db for the Target A parentchain.
	pub const TARGET_A_PARENTCHAIN_LIGHT_CLIENT_DB_PATH: &str = "target_a_lcdb";

	/// Path to the light-client db for the Target B parentchain.
	pub const TARGET_B_PARENTCHAIN_LIGHT_CLIENT_DB_PATH: &str = "target_b_lcdb";

	// bitacross
	pub const RELAYER_REGISTRY_FILE: &str = "relayer_registry_sealed.bin";

	pub const ENCLAVE_REGISTRY_FILE: &str = "enclave_registry_sealed.bin";

	pub const SIGNER_REGISTRY_FILE: &str = "signer_registry_sealed.bin";

	pub const RA_DUMP_CERT_DER_FILE: &str = "ra_dump_cert.der";

	// used by worker and enclave
	pub const SHARDS_PATH: &str = "shards";

	#[cfg(not(feature = "development"))]
	pub static RA_SPID_FILE: &str = "spid_production.txt";
	#[cfg(not(feature = "development"))]
	pub static RA_API_KEY_FILE: &str = "key_production.txt";

	#[cfg(feature = "development")]
	pub static RA_SPID_FILE: &str = "spid.txt";
	#[cfg(feature = "development")]
	pub static RA_API_KEY_FILE: &str = "key.txt";

	pub const SPID_MIN_LENGTH: usize = 32;
	pub const STATE_SNAPSHOTS_CACHE_SIZE: usize = 4;
}

/// Settings concerning the worker
pub mod worker {
	// the maximum size of any extrinsic that the enclave will ever generate in B
	pub const EXTRINSIC_MAX_SIZE: usize = 13_000;
	// the maximum size of the header
	pub const HEADER_MAX_SIZE: usize = 512;
	// maximum size of shielding key
	pub const SHIELDING_KEY_SIZE: usize = 8192;
	// maximum size of signing key
	pub const SIGNING_KEY_SIZE: usize = 32;
	// size of the MR enclave
	pub const MR_ENCLAVE_SIZE: usize = 32;
	// Should be set to a value that ensures that the enclave can register itself
	// and that the worker can start.
	pub const REGISTERING_FEE_FACTOR_FOR_INIT_FUNDS: u128 = 10;
	// Should be set to a value that ensures that at least 2 sidechain blocks are finalized per
	// parentchain block.
	pub const BLOCK_NUMBER_FINALIZATION_DIFF: u64 = 20;
}

pub mod sidechain {
	use core::time::Duration;

	pub static SLOT_DURATION: Duration = Duration::from_millis(6000);
}
