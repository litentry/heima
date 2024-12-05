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

//! Contains semi-generic type definitions to talk to the node without depending on an implementation of Runtime.
//!
//! You need to update this if you have a signed extension in your node that
//! is different from the integritee-node, e.g., if you use the `pallet_asset_tx_payment`.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use codec::{Decode, Encode, MaxEncodedLen};

pub use itp_types::parentchain::{
	AccountData, AccountId, AccountInfo, Address, Balance, Hash, Index, Signature as PairSignature,
};
pub use substrate_api_client::{
	ac_compose_macros::{compose_call, compose_extrinsic_offline},
	ac_node_api::{
		metadata::{InvalidMetadataError, Metadata, MetadataError},
		EventDetails, Events, StaticEvent,
	},
	ac_primitives::{
		config::{AssetRuntimeConfig, Config, DefaultRuntimeConfig},
		extrinsics::{
			AssetTip, CallIndex, ExtrinsicParams, GenericAdditionalParams, GenericAdditionalSigned,
			GenericExtrinsicParams, GenericSignedExtra, PlainTip, UncheckedExtrinsicV4,
		},
		serde_impls::StorageKey,
		signer::{SignExtrinsic, StaticExtrinsicSigner},
	},
	rpc::Request,
	storage_key, Api,
};

// traits from the api-client
pub mod traits {
	pub use substrate_api_client::{GetAccountInformation, GetChainInfo, GetStorage};
}

pub type ParentchainPlainTip = PlainTip<Balance>;
pub type ParentchainAssetTip = AssetTip<Balance>;

/// Configuration for the ExtrinsicParams.
///
/// Valid for the default integritee node
pub type ParentchainExtrinsicParams =
	GenericExtrinsicParams<DefaultRuntimeConfig, ParentchainPlainTip>;
pub type ParentchainAdditionalParams = GenericAdditionalParams<ParentchainPlainTip, Hash>;
pub use DefaultRuntimeConfig as ParentchainRuntimeConfig;

// Pay in asset fees.
//
// This needs to be used if the node uses the `pallet_asset_tx_payment`.
//pub type ParentchainExtrinsicParams =  GenericExtrinsicParams<AssetRuntimeConfig, AssetTip>;
// pub type ParentchainAdditionalParams = GenericAdditionalParams<AssetRuntimeConfig, Hash>;

pub type ParentchainUncheckedExtrinsic<Call> =
	UncheckedExtrinsicV4<Address, Call, PairSignature, ParentchainSignedExtra>;
pub type ParentchainSignedExtra = GenericSignedExtra<ParentchainPlainTip, Index>;
pub type ParentchainSignature = Signature<ParentchainSignedExtra>;

/// Signature type of the [UncheckedExtrinsicV4].
pub type Signature<SignedExtra> = Option<(Address, PairSignature, SignedExtra)>;

// The following types are copied from the substrate-api-client crate to be able to encode/decode them
/// Simplified TransactionStatus to allow the user to choose until when to watch an extrinsic.
// Indexes must match the substrate_api_client::TransactionStatus::as_u8
#[derive(Encode, Decode, Debug, PartialEq, PartialOrd, Eq, Copy, Clone, MaxEncodedLen)]
pub enum XtStatus {
	Ready = 1,
	Broadcast = 2,
	InBlock = 3,
	Retracted = 4,
	Finalized = 6,
}

impl From<XtStatus> for substrate_api_client::XtStatus {
	fn from(status: XtStatus) -> Self {
		match status {
			XtStatus::Ready => substrate_api_client::XtStatus::Ready,
			XtStatus::Broadcast => substrate_api_client::XtStatus::Broadcast,
			XtStatus::InBlock => substrate_api_client::XtStatus::InBlock,
			XtStatus::Retracted => substrate_api_client::XtStatus::Retracted,
			XtStatus::Finalized => substrate_api_client::XtStatus::Finalized,
		}
	}
}

/// Extrinsic report returned upon a submit_and_watch request.
/// Holds as much information as available.
#[derive(Encode, Decode, Debug, Clone, MaxEncodedLen)]
pub struct ExtrinsicReport<Hash: Decode> {
	// Hash of the extrinsic.
	pub extrinsic_hash: Hash,
	// Block hash of the block the extrinsic was included in.
	// Only available if watched until at least `InBlock`.
	pub block_hash: Option<Hash>,
	// Last known Transaction Status.
	pub status: TransactionStatus<Hash, Hash>,
}

impl<Hash: Decode> From<substrate_api_client::ExtrinsicReport<Hash>> for ExtrinsicReport<Hash> {
	fn from(report: substrate_api_client::ExtrinsicReport<Hash>) -> Self {
		Self {
			extrinsic_hash: report.extrinsic_hash,
			block_hash: report.block_hash,
			status: report.status.into(),
		}
	}
}

/// Possible transaction status events.
#[derive(Encode, Decode, Debug, Clone, PartialEq, MaxEncodedLen)]
pub enum TransactionStatus<Hash, BlockHash> {
	/// Transaction is part of the future queue.
	Future,
	/// Transaction is part of the ready queue.
	Ready,
	/// The transaction has been broadcast to the given peers.
	Broadcasted,
	/// Transaction has been included in block with given hash.
	InBlock(BlockHash),
	/// The block this transaction was included in has been retracted.
	Retracted(BlockHash),
	/// Maximum number of finality watchers has been reached,
	/// old watchers are being removed.
	FinalityTimeout(BlockHash),
	/// Transaction has been finalized by a finality-gadget, e.g GRANDPA
	Finalized(BlockHash),
	/// Transaction has been replaced in the pool, by another transaction
	/// that provides the same tags. (e.g. same (sender, nonce)).
	Usurped(Hash),
	/// Transaction has been dropped from the pool because of the limit.
	Dropped,
	/// Transaction is no longer valid in the current state.
	Invalid,
}

impl<Hash, BlockHash> TransactionStatus<Hash, BlockHash> {
	pub fn as_u8(&self) -> u8 {
		match self {
			TransactionStatus::Future => 0,
			TransactionStatus::Ready => 1,
			TransactionStatus::Broadcasted => 2,
			TransactionStatus::InBlock(_) => 3,
			TransactionStatus::Retracted(_) => 4,
			TransactionStatus::FinalityTimeout(_) => 5,
			TransactionStatus::Finalized(_) => 6,
			TransactionStatus::Usurped(_) => 7,
			TransactionStatus::Dropped => 8,
			TransactionStatus::Invalid => 9,
		}
	}
}

impl<Hash, BlockHash> From<substrate_api_client::TransactionStatus<Hash, BlockHash>>
	for TransactionStatus<Hash, BlockHash>
{
	fn from(status: substrate_api_client::TransactionStatus<Hash, BlockHash>) -> Self {
		match status {
			substrate_api_client::TransactionStatus::Future => TransactionStatus::Future,
			substrate_api_client::TransactionStatus::Ready => TransactionStatus::Ready,
			substrate_api_client::TransactionStatus::Broadcast(_) => TransactionStatus::Broadcasted,
			substrate_api_client::TransactionStatus::InBlock(block_hash) =>
				TransactionStatus::InBlock(block_hash),
			substrate_api_client::TransactionStatus::Retracted(block_hash) =>
				TransactionStatus::Retracted(block_hash),
			substrate_api_client::TransactionStatus::FinalityTimeout(block_hash) =>
				TransactionStatus::FinalityTimeout(block_hash),
			substrate_api_client::TransactionStatus::Finalized(block_hash) =>
				TransactionStatus::Finalized(block_hash),
			substrate_api_client::TransactionStatus::Usurped(hash) =>
				TransactionStatus::Usurped(hash),
			substrate_api_client::TransactionStatus::Dropped => TransactionStatus::Dropped,
			substrate_api_client::TransactionStatus::Invalid => TransactionStatus::Invalid,
		}
	}
}
// End of copied types

#[cfg(feature = "std")]
pub use api::*;

#[cfg(feature = "std")]
mod api {
	use super::ParentchainRuntimeConfig;
	use substrate_api_client::Api;

	pub use substrate_api_client::{
		api::Error as ApiClientError,
		rpc::{tungstenite_client::TungsteniteRpcClient, Error as RpcClientError},
	};

	pub type ParentchainApi = Api<ParentchainRuntimeConfig, TungsteniteRpcClient>;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_xt_status_index() {
		assert_eq!(1, XtStatus::Ready as u8);
		assert_eq!(2, XtStatus::Broadcast as u8);
		assert_eq!(3, XtStatus::InBlock as u8);
		assert_eq!(4, XtStatus::Retracted as u8);
		assert_eq!(6, XtStatus::Finalized as u8);
	}

	#[test]
	fn test_transaction_status_as_u8() {
		assert_eq!(0, TransactionStatus::<Hash, Hash>::Future.as_u8());
		assert_eq!(1, TransactionStatus::<Hash, Hash>::Ready.as_u8());
		assert_eq!(2, TransactionStatus::<Hash, Hash>::Broadcasted.as_u8());
		assert_eq!(3, TransactionStatus::<Hash, Hash>::InBlock(Hash::random()).as_u8());
		assert_eq!(4, TransactionStatus::<Hash, Hash>::Retracted(Hash::random()).as_u8());
		assert_eq!(5, TransactionStatus::<Hash, Hash>::FinalityTimeout(Hash::random()).as_u8());
		assert_eq!(6, TransactionStatus::<Hash, Hash>::Finalized(Hash::random()).as_u8());
		assert_eq!(7, TransactionStatus::<Hash, Hash>::Usurped(Hash::random()).as_u8());
		assert_eq!(8, TransactionStatus::<Hash, Hash>::Dropped.as_u8());
		assert_eq!(9, TransactionStatus::<Hash, Hash>::Invalid.as_u8());
	}

	#[test]
	fn test_xt_status_match_transaction_status_index() {
		assert_eq!(XtStatus::Ready as u8, TransactionStatus::<Hash, Hash>::Ready.as_u8());
		assert_eq!(XtStatus::Broadcast as u8, TransactionStatus::<Hash, Hash>::Broadcasted.as_u8());
		assert_eq!(
			XtStatus::InBlock as u8,
			TransactionStatus::<Hash, Hash>::InBlock(Hash::random()).as_u8()
		);
		assert_eq!(
			XtStatus::Retracted as u8,
			TransactionStatus::<Hash, Hash>::Retracted(Hash::random()).as_u8()
		);
		assert_eq!(
			XtStatus::Finalized as u8,
			TransactionStatus::<Hash, Hash>::Finalized(Hash::random()).as_u8()
		);
	}
}
