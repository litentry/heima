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

//! All the different crypto schemes that we use in sgx

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use rand_sgx as rand;
	pub use serde_json_sgx as serde_json;
}

pub mod aes;
pub mod aes256;
pub mod ecdsa;
pub mod ed25519;
pub mod ed25519_derivation;
pub mod error;
pub mod key_repository;
pub mod rsa3072;
pub mod schnorr;
pub mod traits;

pub use self::{aes::*, aes256::*, ecdsa::*, ed25519::*, rsa3072::*};
pub use error::*;
pub use traits::*;

#[cfg(feature = "mocks")]
pub mod mocks;

#[cfg(feature = "test")]
pub mod tests {
	pub use super::ed25519::sgx_tests::{
		ed25529_sealing_works, using_get_ed25519_repository_twice_initializes_key_only_once,
	};

	pub use super::rsa3072::sgx_tests::{
		rsa3072_sealing_works, using_get_rsa3072_repository_twice_initializes_key_only_once,
	};

	pub use super::aes::sgx_tests::{
		aes_sealing_works, using_get_aes_repository_twice_initializes_key_only_once,
	};

	pub use super::aes256::sgx_tests::{
		aes256_creating_repository_with_same_path_and_prefix_but_new_key_results_in_new_key,
		aes256_creating_repository_with_same_path_and_prefix_results_in_same_key,
	};

	pub use super::ecdsa::sgx_tests::{
		ecdsa_creating_repository_with_same_path_and_prefix_but_new_key_results_in_new_key,
		ecdsa_creating_repository_with_same_path_and_prefix_results_in_same_key,
		ecdsa_seal_init_should_create_new_key_if_not_present,
		ecdsa_seal_init_should_not_change_key_if_exists_and_not_provided,
		ecdsa_seal_init_should_seal_provided_key,
		ecdsa_seal_init_with_key_should_change_current_key,
		ecdsa_sign_should_produce_valid_signature,
	};

	pub use super::schnorr::sgx_tests::{
		schnorr_creating_repository_with_same_path_and_prefix_but_new_key_results_in_new_key,
		schnorr_creating_repository_with_same_path_and_prefix_results_in_same_key,
		schnorr_seal_init_should_create_new_key_if_not_present,
		schnorr_seal_init_should_not_change_key_if_exists_and_not_provided,
		schnorr_seal_init_should_seal_provided_key,
		schnorr_seal_init_with_key_should_change_key_current_key,
	};
}
