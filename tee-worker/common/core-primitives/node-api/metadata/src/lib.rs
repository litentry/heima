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

//! Handle SGX compatible substrate chain metadata.

#![cfg_attr(not(feature = "std"), no_std)]

use crate::{
	error::Result, pallet_balances::BalancesCallIndexes,
	pallet_evm_assertion::EvmAssertionsCallIndexes, pallet_imp::IMPCallIndexes,
	pallet_omni_account::OmniAccountCallIndexes, pallet_proxy::ProxyCallIndexes,
	pallet_system::SystemConstants, pallet_teebag::TeebagCallIndexes,
	pallet_timestamp::TimestampCallIndexes, pallet_utility::UtilityCallIndexes,
	pallet_vcmp::VCMPCallIndexes,
};
use codec::{Decode, Encode};
use sp_core::storage::StorageKey;

pub use crate::error::Error;
pub use itp_api_client_types::{Metadata, MetadataError};

pub mod error;
pub mod pallet_balances;
pub mod pallet_evm_assertion;
pub mod pallet_imp;
pub mod pallet_omni_account;
pub mod pallet_proxy;
pub mod pallet_system;
pub mod pallet_teebag;
pub mod pallet_utility;
pub mod pallet_vcmp;
pub mod runtime_call;

pub mod pallet_timestamp;

#[cfg(feature = "mocks")]
pub mod metadata_mocks;

pub trait NodeMetadataTrait:
	TeebagCallIndexes
	+ IMPCallIndexes
	+ VCMPCallIndexes
	+ SystemConstants
	+ UtilityCallIndexes
	+ ProxyCallIndexes
	+ BalancesCallIndexes
	+ TimestampCallIndexes
	+ EvmAssertionsCallIndexes
	+ OmniAccountCallIndexes
{
}

impl<
		T: TeebagCallIndexes
			+ IMPCallIndexes
			+ VCMPCallIndexes
			+ SystemConstants
			+ UtilityCallIndexes
			+ ProxyCallIndexes
			+ BalancesCallIndexes
			+ TimestampCallIndexes
			+ EvmAssertionsCallIndexes
			+ OmniAccountCallIndexes,
	> NodeMetadataTrait for T
{
}

impl TryFrom<NodeMetadata> for Metadata {
	type Error = crate::error::Error;

	fn try_from(value: NodeMetadata) -> core::result::Result<Self, Self::Error> {
		value.node_metadata.ok_or(Error::MetadataNotSet)
	}
}

#[derive(Default, Encode, Decode, Debug, Clone)]
pub struct NodeMetadata {
	node_metadata: Option<Metadata>,
	runtime_spec_version: u32,
	runtime_transaction_version: u32,
}

impl NodeMetadata {
	pub fn new(
		node_metadata: Metadata,
		runtime_spec_version: u32,
		runtime_transaction_version: u32,
	) -> Self {
		Self {
			node_metadata: Some(node_metadata),
			runtime_spec_version,
			runtime_transaction_version,
		}
	}

	pub fn get_metadata(&self) -> Option<&Metadata> {
		self.node_metadata.as_ref()
	}

	/// Return the substrate chain runtime version.
	pub fn get_runtime_version(&self) -> u32 {
		self.runtime_spec_version
	}

	/// Return the substrate chain runtime transaction version.
	pub fn get_runtime_transaction_version(&self) -> u32 {
		self.runtime_transaction_version
	}

	/// Generic call indexes:
	/// Get the array [pallet index, call index] corresponding to a pallet's call over the metadata.
	pub fn call_indexes(
		&self,
		pallet_name: &'static str,
		call_name: &'static str,
	) -> Result<[u8; 2]> {
		let pallet = match &self.node_metadata {
			None => return Err(Error::MetadataNotSet),
			Some(m) => m.pallet_by_name_err(pallet_name)?,
		};
		let call_index = pallet
			.call_variant_by_name(call_name)
			.ok_or(Error::NodeMetadata(MetadataError::CallNotFound(call_name)))?;
		Ok([pallet.index(), call_index.index])
	}

	/// Generic storages:
	/// Get the storage keys corresponding to a storage over the metadata:
	pub fn storage_value_key(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
	) -> Result<StorageKey> {
		match &self.node_metadata {
			None => Err(Error::MetadataNotSet),
			Some(m) => m
				.storage_value_key(storage_prefix, storage_key_name)
				.map(|key| key.into())
				.map_err(Error::NodeMetadata),
		}
	}

	pub fn storage_map_key<K: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		map_key: K,
	) -> Result<StorageKey> {
		match &self.node_metadata {
			None => Err(Error::MetadataNotSet),
			Some(m) => m
				.storage_map_key::<K>(storage_prefix, storage_key_name, map_key)
				.map(|key| key.into())
				.map_err(Error::NodeMetadata),
		}
	}

	pub fn storage_double_map_key<K: Encode, Q: Encode>(
		&self,
		storage_prefix: &'static str,
		storage_key_name: &'static str,
		first: K,
		second: Q,
	) -> Result<StorageKey> {
		match &self.node_metadata {
			None => Err(Error::MetadataNotSet),
			Some(m) => m
				.storage_double_map_key(storage_prefix, storage_key_name, first, second)
				.map(|key| key.into())
				.map_err(Error::NodeMetadata),
		}
	}
}
