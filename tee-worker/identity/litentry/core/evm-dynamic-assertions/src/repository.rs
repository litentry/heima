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

use crate::{
	sealing::io::{seal_state, unseal_state},
	AssertionId, SmartContractByteCode,
};
use lc_dynamic_assertion::AssertionLogicRepository;
use std::{
	collections::HashMap,
	format,
	string::{String, ToString},
	vec::Vec,
};

#[cfg(feature = "sgx")]
use std::sync::SgxMutex as Mutex;

#[cfg(feature = "std")]
use std::sync::Mutex;

pub type AssertionsMap = HashMap<AssertionId, (SmartContractByteCode, Vec<String>)>;

// Assertion repository backed by sealed file. Contains only latest state from parachain storage.
pub struct EvmAssertionRepository {
	path: String,
	state: Mutex<AssertionsMap>,
}

impl EvmAssertionRepository {
	pub fn new(path: &str) -> Result<Self, String> {
		let mut state = HashMap::new();
		let unsealed = unseal_state(path)
			.map_err(|e| format!("Could not unseal assertions state: {:?}", e))?;
		unsealed.into_iter().for_each(|(id, byte_code)| {
			state.insert(id, byte_code);
		});

		Ok(EvmAssertionRepository { state: state.into(), path: path.to_string() })
	}
}

impl AssertionLogicRepository for EvmAssertionRepository {
	type Id = AssertionId;
	type Item = (SmartContractByteCode, Vec<String>);

	fn get(&self, id: &Self::Id) -> Result<Option<Self::Item>, String> {
		Ok(self
			.state
			.lock()
			.map_err(|e| format!("Could not acquire lock on inner state: {:?}", e))?
			.get(id)
			.cloned())
	}

	fn save(&self, id: Self::Id, item: Self::Item) -> Result<(), String> {
		self.state
			.lock()
			.map_err(|e| format!("Could not acquire lock on inner state: {:?}", e))?
			.insert(id, item);
		// prepare data for encoding
		let unsealed_state: Vec<(AssertionId, Self::Item)> = self
			.state
			.lock()
			.map_err(|e| format!("Could not acquire lock on inner state: {:?}", e))?
			.iter()
			.map(|(key, val)| (*key, val.clone()))
			.collect();

		if let Err(e) = seal_state(&self.path, unsealed_state) {
			//clean up memory state
			self.state
				.lock()
				.map_err(|e| format!("Could not acquire lock on inner state: {:?}", e))?
				.remove(&id);
			return Err(format!("Could not seal assertions state: {:?}", e));
		}
		Ok(())
	}
}

#[cfg(feature = "sgx-test")]
pub mod sgx_tests {
	use crate::{repository::EvmAssertionRepository, sealing::io::seal_state};
	use ethabi::ethereum_types::H160;
	use itp_sgx_temp_dir::TempDir;
	use lc_dynamic_assertion::AssertionLogicRepository;
	use sgx_tstd::{string::ToString, vec, vec::Vec};

	pub fn restores_state_from_seal() {
		let seal_file_name = "test_sealed_assertion.bin";
		let temp_dir = TempDir::with_prefix("evm_assertion_seal_tests").unwrap();
		let seal_path = temp_dir.path().join(seal_file_name);

		let assertion_id = H160::default();
		let byte_code: Vec<u8> = [1; 67].to_vec();
		let secrets = vec!["secret_1".to_string()];

		seal_state(
			seal_path.to_str().unwrap(),
			vec![(assertion_id.clone(), (byte_code.clone(), secrets.clone()))],
		)
		.unwrap();

		let repository = EvmAssertionRepository::new(seal_path.to_str().unwrap()).unwrap();
		let assertion = repository.get(&assertion_id).unwrap().unwrap();
		assert_eq!(assertion.0, byte_code);
		assert_eq!(assertion.1, secrets);
	}
}
