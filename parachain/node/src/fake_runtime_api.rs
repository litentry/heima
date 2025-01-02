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

//! These are used to provide a type that implements these runtime APIs without requiring to import
//! the native runtimes.

use core_primitives::{AccountId, AuraId, Balance, Nonce};
use frame_support::weights::Weight;
use polkadot_primitives::Block;
use sp_core::{OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	traits::Block as BlockT,
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Permill,
};

#[allow(dead_code)]
pub struct Runtime;

sp_api::impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> sp_version::RuntimeVersion {
			unimplemented!()
		}

		fn execute_block(_: Block) {
			unimplemented!()
		}

		fn initialize_block(_: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			unimplemented!()
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			unimplemented!()
		}

		fn metadata_at_version(_: u32) -> Option<OpaqueMetadata> {
			unimplemented!()
		}

		fn metadata_versions() -> Vec<u32> {
			unimplemented!()
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			unimplemented!()
		}

		fn authorities() -> Vec<AuraId> {
			unimplemented!()
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			_: <Block as BlockT>::Hash,
			_: cumulus_primitives_aura::Slot,
		) -> bool {
			unimplemented!()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(_: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			unimplemented!()
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			unimplemented!()
		}

		fn inherent_extrinsics(_: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			unimplemented!()
		}

		fn check_inherents(_: Block, _: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			unimplemented!()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			_: TransactionSource,
			_: <Block as BlockT>::Extrinsic,
			_: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			unimplemented!()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(_seed: Option<Vec<u8>>) -> Vec<u8> {
			unimplemented!()
		}

		fn decode_session_keys(_encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			unimplemented!()
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			_: <Block as BlockT>::Extrinsic,
			_: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			unimplemented!()
		}
		fn query_fee_details(
			_: <Block as BlockT>::Extrinsic,
			_: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			unimplemented!()
		}
		fn query_weight_to_fee(_: Weight) -> Balance {
			unimplemented!()
		}
		fn query_length_to_fee(_: u32) -> Balance {
			unimplemented!()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(_: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			unimplemented!()
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(_: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			unimplemented!()
		}

		fn execute_block(
			_: Block,
			_: bool,
			_: bool,
			_: frame_try_runtime::TryStateSelect,
		) -> Weight {
			unimplemented!()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(_: AccountId) -> Nonce {
			unimplemented!()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(_: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			unimplemented!()
		}

		fn dispatch_benchmark(
			_: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			unimplemented!()
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(_: Vec<u8>) -> sp_genesis_builder::Result {
			unimplemented!()
		}

		fn get_preset(_id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			unimplemented!()
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			unimplemented!()
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			unimplemented!()
		}

		fn account_basic(_address: H160) -> pallet_evm::Account {
			unimplemented!()
		}

		fn gas_price() -> U256 {
			unimplemented!()
		}

		fn account_code_at(_address: H160) -> Vec<u8> {
			unimplemented!()
		}

		fn author() -> H160 {
			unimplemented!()
		}

		fn storage_at(_address: H160, _index: U256) -> H256 {
			unimplemented!()
		}

		fn call(
			_from: H160,
			_to: H160,
			_data: Vec<u8>,
			_value: U256,
			_gas_limit: U256,
			_max_fee_per_gas: Option<U256>,
			_max_priority_fee_per_gas: Option<U256>,
			_nonce: Option<U256>,
			_estimate: bool,
			_access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			unimplemented!()
		}

		fn create(
			_from: H160,
			_data: Vec<u8>,
			_value: U256,
			_gas_limit: U256,
			_max_fee_per_gas: Option<U256>,
			_max_priority_fee_per_gas: Option<U256>,
			_nonce: Option<U256>,
			_estimate: bool,
			_access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			unimplemented!()
		}

		fn current_transaction_statuses() -> Option<Vec<fp_rpc::TransactionStatus>> {
			unimplemented!()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			unimplemented!()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			unimplemented!()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<fp_rpc::TransactionStatus>>,
		) {
			unimplemented!()
		}

		fn extrinsic_filter(_xts: Vec<<Block as BlockT>::Extrinsic>) -> Vec<pallet_ethereum::Transaction> {
			unimplemented!()
		}

		fn elasticity() -> Option<Permill> {
			unimplemented!()
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(_xts: Vec<<Block as BlockT>::Extrinsic>) -> (Option<pallet_ethereum::Block>, Option<Vec<fp_rpc::TransactionStatus>>) {
			unimplemented!()
		}

		fn initialize_pending_block(_header: &<Block as BlockT>::Header) {
			unimplemented!()
		}
	}

	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(_transaction: pallet_ethereum::Transaction) -> <Block as BlockT>::Extrinsic {
			unimplemented!()
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(_header: &<Block as BlockT>::Header) {
			unimplemented!()
		}
	}

	impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
		fn trace_transaction(_extrinsics: Vec<<Block as BlockT>::Extrinsic>, _traced_transaction: &pallet_ethereum::Transaction, _header: &<Block as BlockT>::Header) -> Result<(), sp_runtime::DispatchError> {
			unimplemented!()
		}

		fn trace_block(_extrinsics: Vec<<Block as BlockT>::Extrinsic>, _known_transactions: Vec<H256>, _header: &<Block as BlockT>::Header) -> Result<(), sp_runtime::DispatchError> {
			unimplemented!()
		}

		fn trace_call(
			_header: &<Block as BlockT>::Header,
			_from: H160,
			_to: H160,
			_data: Vec<u8>,
			_value: U256,
			_gas_limit: U256,
			_max_fee_per_gas: Option<U256>,
			_max_priority_fee_per_gas: Option<U256>,
			_nonce: Option<U256>,
			_access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<(), sp_runtime::DispatchError> {
			unimplemented!()
		}
	}

	impl moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block> for Runtime {
		fn extrinsic_filter(_xts_ready: Vec<<Block as BlockT>::Extrinsic>, _xts_future: Vec<<Block as BlockT>::Extrinsic>) -> moonbeam_rpc_primitives_txpool::TxPoolResponse {
			unimplemented!()
		}
	}
}
