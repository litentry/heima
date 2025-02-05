#[allow(dead_code, unused_imports, non_camel_case_types, unreachable_patterns)]
#[allow(clippy::all)]
#[allow(rustdoc::broken_intra_doc_links)]
pub mod api {
	#[allow(unused_imports)]
	mod root_mod {
		pub use super::*;
	}
	pub static PALLETS: [&str; 2usize] = ["OmniAccount", "Teebag"];
	pub static RUNTIME_APIS: [&str; 18usize] = [
		"Core",
		"Metadata",
		"BlockBuilder",
		"TaggedTransactionQueue",
		"OffchainWorkerApi",
		"SessionKeys",
		"AuraApi",
		"AuraUnincludedSegmentApi",
		"AccountNonceApi",
		"TransactionPaymentApi",
		"TransactionPaymentCallApi",
		"CollectCollationInfo",
		"EthereumRuntimeRPCApi",
		"ConvertTransactionRuntimeApi",
		"DebugRuntimeApi",
		"TxPoolRuntimeApi",
		"OmniAccountApi",
		"GenesisBuilder",
	];
	#[doc = r" The error type that is returned when there is a runtime issue."]
	pub type DispatchError = runtime_types::sp_runtime::DispatchError;
	#[doc = r" The outer event enum."]
	pub type Event = runtime_types::paseo_parachain_runtime::RuntimeEvent;
	#[doc = r" The outer extrinsic enum."]
	pub type Call = runtime_types::paseo_parachain_runtime::RuntimeCall;
	#[doc = r" The outer error enum represents the DispatchError's Module variant."]
	pub type Error = runtime_types::paseo_parachain_runtime::RuntimeError;
	pub fn constants() -> ConstantsApi {
		ConstantsApi
	}
	pub fn storage() -> StorageApi {
		StorageApi
	}
	pub fn tx() -> TransactionApi {
		TransactionApi
	}
	pub fn apis() -> runtime_apis::RuntimeApi {
		runtime_apis::RuntimeApi
	}
	pub mod runtime_apis {
		use super::root_mod;
		use super::runtime_types;
		use subxt::ext::subxt_core::ext::codec::Encode;
		pub struct RuntimeApi;
		impl RuntimeApi {
			pub fn core(&self) -> core::Core {
				core::Core
			}
			pub fn metadata(&self) -> metadata::Metadata {
				metadata::Metadata
			}
			pub fn block_builder(&self) -> block_builder::BlockBuilder {
				block_builder::BlockBuilder
			}
			pub fn tagged_transaction_queue(
				&self,
			) -> tagged_transaction_queue::TaggedTransactionQueue {
				tagged_transaction_queue::TaggedTransactionQueue
			}
			pub fn offchain_worker_api(&self) -> offchain_worker_api::OffchainWorkerApi {
				offchain_worker_api::OffchainWorkerApi
			}
			pub fn session_keys(&self) -> session_keys::SessionKeys {
				session_keys::SessionKeys
			}
			pub fn aura_api(&self) -> aura_api::AuraApi {
				aura_api::AuraApi
			}
			pub fn aura_unincluded_segment_api(
				&self,
			) -> aura_unincluded_segment_api::AuraUnincludedSegmentApi {
				aura_unincluded_segment_api::AuraUnincludedSegmentApi
			}
			pub fn account_nonce_api(&self) -> account_nonce_api::AccountNonceApi {
				account_nonce_api::AccountNonceApi
			}
			pub fn transaction_payment_api(
				&self,
			) -> transaction_payment_api::TransactionPaymentApi {
				transaction_payment_api::TransactionPaymentApi
			}
			pub fn transaction_payment_call_api(
				&self,
			) -> transaction_payment_call_api::TransactionPaymentCallApi {
				transaction_payment_call_api::TransactionPaymentCallApi
			}
			pub fn collect_collation_info(&self) -> collect_collation_info::CollectCollationInfo {
				collect_collation_info::CollectCollationInfo
			}
			pub fn ethereum_runtime_rpc_api(
				&self,
			) -> ethereum_runtime_rpc_api::EthereumRuntimeRPCApi {
				ethereum_runtime_rpc_api::EthereumRuntimeRPCApi
			}
			pub fn convert_transaction_runtime_api(
				&self,
			) -> convert_transaction_runtime_api::ConvertTransactionRuntimeApi {
				convert_transaction_runtime_api::ConvertTransactionRuntimeApi
			}
			pub fn debug_runtime_api(&self) -> debug_runtime_api::DebugRuntimeApi {
				debug_runtime_api::DebugRuntimeApi
			}
			pub fn tx_pool_runtime_api(&self) -> tx_pool_runtime_api::TxPoolRuntimeApi {
				tx_pool_runtime_api::TxPoolRuntimeApi
			}
			pub fn omni_account_api(&self) -> omni_account_api::OmniAccountApi {
				omni_account_api::OmniAccountApi
			}
			pub fn genesis_builder(&self) -> genesis_builder::GenesisBuilder {
				genesis_builder::GenesisBuilder
			}
		}
		pub mod core {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " The `Core` runtime api that every Substrate runtime needs to implement."]
			pub struct Core;
			impl Core {
				#[doc = " Returns the version of the runtime."]
				pub fn version(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::Version,
					types::version::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"Core",
						"version",
						types::Version {},
						[
							76u8, 202u8, 17u8, 117u8, 189u8, 237u8, 239u8, 237u8, 151u8, 17u8,
							125u8, 159u8, 218u8, 92u8, 57u8, 238u8, 64u8, 147u8, 40u8, 72u8, 157u8,
							116u8, 37u8, 195u8, 156u8, 27u8, 123u8, 173u8, 178u8, 102u8, 136u8,
							6u8,
						],
					)
				}
				#[doc = " Execute the given block."]
				pub fn execute_block(
					&self,
					block: types::execute_block::Block,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::ExecuteBlock,
					types::execute_block::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"Core",
						"execute_block",
						types::ExecuteBlock { block },
						[
							71u8, 186u8, 53u8, 183u8, 85u8, 95u8, 61u8, 56u8, 159u8, 137u8, 111u8,
							34u8, 39u8, 238u8, 212u8, 113u8, 200u8, 93u8, 118u8, 143u8, 186u8,
							16u8, 79u8, 249u8, 95u8, 210u8, 240u8, 0u8, 220u8, 57u8, 121u8, 186u8,
						],
					)
				}
				#[doc = " Initialize a block with the given header and return the runtime executive mode."]
				pub fn initialize_block(
					&self,
					header: types::initialize_block::Header,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::InitializeBlock,
					types::initialize_block::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"Core",
						"initialize_block",
						types::InitializeBlock { header },
						[
							132u8, 169u8, 113u8, 112u8, 80u8, 139u8, 113u8, 35u8, 41u8, 81u8, 36u8,
							35u8, 37u8, 202u8, 29u8, 207u8, 205u8, 229u8, 145u8, 7u8, 133u8, 94u8,
							25u8, 108u8, 233u8, 86u8, 234u8, 29u8, 236u8, 57u8, 56u8, 186u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod version {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::sp_version::RuntimeVersion;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Version {}
				pub mod execute_block {
					use super::runtime_types;
					pub type Block = runtime_types :: sp_runtime :: generic :: block :: Block < runtime_types :: sp_runtime :: generic :: header :: Header < :: core :: primitive :: u32 > , runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub mod output {
						use super::runtime_types;
						pub type Output = ();
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ExecuteBlock {
					pub block: execute_block::Block,
				}
				pub mod initialize_block {
					use super::runtime_types;
					pub type Header =
						runtime_types::sp_runtime::generic::header::Header<::core::primitive::u32>;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::sp_runtime::ExtrinsicInclusionMode;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct InitializeBlock {
					pub header: initialize_block::Header,
				}
			}
		}
		pub mod metadata {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " The `Metadata` api trait that returns metadata for the runtime."]
			pub struct Metadata;
			impl Metadata {
				#[doc = " Returns the metadata of a runtime."]
				pub fn metadata(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::Metadata,
					types::metadata::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"Metadata",
						"metadata",
						types::Metadata {},
						[
							231u8, 24u8, 67u8, 152u8, 23u8, 26u8, 188u8, 82u8, 229u8, 6u8, 185u8,
							27u8, 175u8, 68u8, 83u8, 122u8, 69u8, 89u8, 185u8, 74u8, 248u8, 87u8,
							217u8, 124u8, 193u8, 252u8, 199u8, 186u8, 196u8, 179u8, 179u8, 96u8,
						],
					)
				}
				#[doc = " Returns the metadata at a given version."]
				#[doc = ""]
				#[doc = " If the given `version` isn't supported, this will return `None`."]
				#[doc = " Use [`Self::metadata_versions`] to find out about supported metadata version of the runtime."]
				pub fn metadata_at_version(
					&self,
					version: types::metadata_at_version::Version,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::MetadataAtVersion,
					types::metadata_at_version::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"Metadata",
						"metadata_at_version",
						types::MetadataAtVersion { version },
						[
							131u8, 53u8, 212u8, 234u8, 16u8, 25u8, 120u8, 252u8, 153u8, 153u8,
							216u8, 28u8, 54u8, 113u8, 52u8, 236u8, 146u8, 68u8, 142u8, 8u8, 10u8,
							169u8, 131u8, 142u8, 204u8, 38u8, 48u8, 108u8, 134u8, 86u8, 226u8,
							61u8,
						],
					)
				}
				#[doc = " Returns the supported metadata versions."]
				#[doc = ""]
				#[doc = " This can be used to call `metadata_at_version`."]
				pub fn metadata_versions(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::MetadataVersions,
					types::metadata_versions::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"Metadata",
						"metadata_versions",
						types::MetadataVersions {},
						[
							23u8, 144u8, 137u8, 91u8, 188u8, 39u8, 231u8, 208u8, 252u8, 218u8,
							224u8, 176u8, 77u8, 32u8, 130u8, 212u8, 223u8, 76u8, 100u8, 190u8,
							82u8, 94u8, 190u8, 8u8, 82u8, 244u8, 225u8, 179u8, 85u8, 176u8, 56u8,
							16u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod metadata {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::sp_core::OpaqueMetadata;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Metadata {}
				pub mod metadata_at_version {
					use super::runtime_types;
					pub type Version = ::core::primitive::u32;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							::core::option::Option<runtime_types::sp_core::OpaqueMetadata>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct MetadataAtVersion {
					pub version: metadata_at_version::Version,
				}
				pub mod metadata_versions {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u32>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct MetadataVersions {}
			}
		}
		pub mod block_builder {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " The `BlockBuilder` api trait that provides the required functionality for building a block."]
			pub struct BlockBuilder;
			impl BlockBuilder {
				#[doc = " Apply the given extrinsic."]
				#[doc = ""]
				#[doc = " Returns an inclusion outcome which specifies if this extrinsic is included in"]
				#[doc = " this block or not."]
				pub fn apply_extrinsic(
					&self,
					extrinsic: types::apply_extrinsic::Extrinsic,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::ApplyExtrinsic,
					types::apply_extrinsic::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"BlockBuilder",
						"apply_extrinsic",
						types::ApplyExtrinsic { extrinsic },
						[
							93u8, 122u8, 238u8, 143u8, 82u8, 1u8, 206u8, 107u8, 167u8, 31u8, 22u8,
							6u8, 113u8, 221u8, 141u8, 95u8, 189u8, 75u8, 186u8, 177u8, 5u8, 149u8,
							68u8, 91u8, 171u8, 82u8, 234u8, 147u8, 165u8, 141u8, 13u8, 254u8,
						],
					)
				}
				#[doc = " Finish the current block."]
				pub fn finalize_block(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::FinalizeBlock,
					types::finalize_block::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"BlockBuilder",
						"finalize_block",
						types::FinalizeBlock {},
						[
							244u8, 207u8, 24u8, 33u8, 13u8, 69u8, 9u8, 249u8, 145u8, 143u8, 122u8,
							96u8, 197u8, 55u8, 64u8, 111u8, 238u8, 224u8, 34u8, 201u8, 27u8, 146u8,
							232u8, 99u8, 191u8, 30u8, 114u8, 16u8, 32u8, 220u8, 58u8, 62u8,
						],
					)
				}
				#[doc = " Generate inherent extrinsics. The inherent data will vary from chain to chain."]
				pub fn inherent_extrinsics(
					&self,
					inherent: types::inherent_extrinsics::Inherent,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::InherentExtrinsics,
					types::inherent_extrinsics::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"BlockBuilder",
						"inherent_extrinsics",
						types::InherentExtrinsics { inherent },
						[
							38u8, 7u8, 22u8, 96u8, 164u8, 159u8, 19u8, 22u8, 13u8, 1u8, 194u8,
							99u8, 146u8, 217u8, 97u8, 235u8, 104u8, 166u8, 20u8, 158u8, 193u8,
							203u8, 97u8, 247u8, 30u8, 246u8, 196u8, 113u8, 166u8, 78u8, 194u8,
							163u8,
						],
					)
				}
				#[doc = " Check that the inherents are valid. The inherent data will vary from chain to chain."]
				pub fn check_inherents(
					&self,
					block: types::check_inherents::Block,
					data: types::check_inherents::Data,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::CheckInherents,
					types::check_inherents::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"BlockBuilder",
						"check_inherents",
						types::CheckInherents { block, data },
						[
							188u8, 160u8, 23u8, 43u8, 139u8, 48u8, 47u8, 82u8, 226u8, 24u8, 53u8,
							86u8, 114u8, 2u8, 206u8, 95u8, 174u8, 219u8, 60u8, 86u8, 142u8, 164u8,
							171u8, 101u8, 249u8, 40u8, 171u8, 123u8, 137u8, 219u8, 122u8, 9u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod apply_extrinsic {
					use super::runtime_types;
					pub type Extrinsic = runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > ;
					pub mod output {
						use super::runtime_types;
						pub type Output = :: core :: result :: Result < :: core :: result :: Result < () , runtime_types :: sp_runtime :: DispatchError > , runtime_types :: sp_runtime :: transaction_validity :: TransactionValidityError > ;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ApplyExtrinsic {
					pub extrinsic: apply_extrinsic::Extrinsic,
				}
				pub mod finalize_block {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::sp_runtime::generic::header::Header<
							::core::primitive::u32,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct FinalizeBlock {}
				pub mod inherent_extrinsics {
					use super::runtime_types;
					pub type Inherent = runtime_types::sp_inherents::InherentData;
					pub mod output {
						use super::runtime_types;
						pub type Output = :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct InherentExtrinsics {
					pub inherent: inherent_extrinsics::Inherent,
				}
				pub mod check_inherents {
					use super::runtime_types;
					pub type Block = runtime_types :: sp_runtime :: generic :: block :: Block < runtime_types :: sp_runtime :: generic :: header :: Header < :: core :: primitive :: u32 > , runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub type Data = runtime_types::sp_inherents::InherentData;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::sp_inherents::CheckInherentsResult;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CheckInherents {
					pub block: check_inherents::Block,
					pub data: check_inherents::Data,
				}
			}
		}
		pub mod tagged_transaction_queue {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " The `TaggedTransactionQueue` api trait for interfering with the transaction queue."]
			pub struct TaggedTransactionQueue;
			impl TaggedTransactionQueue {
				#[doc = " Validate the transaction."]
				#[doc = ""]
				#[doc = " This method is invoked by the transaction pool to learn details about given transaction."]
				#[doc = " The implementation should make sure to verify the correctness of the transaction"]
				#[doc = " against current state. The given `block_hash` corresponds to the hash of the block"]
				#[doc = " that is used as current state."]
				#[doc = ""]
				#[doc = " Note that this call may be performed by the pool multiple times and transactions"]
				#[doc = " might be verified in any possible order."]
				pub fn validate_transaction(
					&self,
					source: types::validate_transaction::Source,
					tx: types::validate_transaction::Tx,
					block_hash: types::validate_transaction::BlockHash,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::ValidateTransaction,
					types::validate_transaction::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TaggedTransactionQueue",
						"validate_transaction",
						types::ValidateTransaction { source, tx, block_hash },
						[
							57u8, 96u8, 17u8, 117u8, 142u8, 93u8, 131u8, 36u8, 26u8, 196u8, 204u8,
							140u8, 147u8, 186u8, 55u8, 141u8, 61u8, 41u8, 59u8, 202u8, 64u8, 185u8,
							138u8, 194u8, 64u8, 61u8, 128u8, 199u8, 19u8, 92u8, 119u8, 187u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod validate_transaction {
					use super::runtime_types;
					pub type Source =
						runtime_types::sp_runtime::transaction_validity::TransactionSource;
					pub type Tx = runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > ;
					pub type BlockHash = ::subxt::ext::subxt_core::utils::H256;
					pub mod output {
						use super::runtime_types;
						pub type Output = :: core :: result :: Result < runtime_types :: sp_runtime :: transaction_validity :: ValidTransaction , runtime_types :: sp_runtime :: transaction_validity :: TransactionValidityError > ;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ValidateTransaction {
					pub source: validate_transaction::Source,
					pub tx: validate_transaction::Tx,
					pub block_hash: validate_transaction::BlockHash,
				}
			}
		}
		pub mod offchain_worker_api {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " The offchain worker api."]
			pub struct OffchainWorkerApi;
			impl OffchainWorkerApi {
				#[doc = " Starts the off-chain task for given block header."]
				pub fn offchain_worker(
					&self,
					header: types::offchain_worker::Header,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::OffchainWorker,
					types::offchain_worker::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"OffchainWorkerApi",
						"offchain_worker",
						types::OffchainWorker { header },
						[
							10u8, 135u8, 19u8, 153u8, 33u8, 216u8, 18u8, 242u8, 33u8, 140u8, 4u8,
							223u8, 200u8, 130u8, 103u8, 118u8, 137u8, 24u8, 19u8, 127u8, 161u8,
							29u8, 184u8, 111u8, 222u8, 111u8, 253u8, 73u8, 45u8, 31u8, 79u8, 60u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod offchain_worker {
					use super::runtime_types;
					pub type Header =
						runtime_types::sp_runtime::generic::header::Header<::core::primitive::u32>;
					pub mod output {
						use super::runtime_types;
						pub type Output = ();
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct OffchainWorker {
					pub header: offchain_worker::Header,
				}
			}
		}
		pub mod session_keys {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " Session keys runtime api."]
			pub struct SessionKeys;
			impl SessionKeys {
				#[doc = " Generate a set of session keys with optionally using the given seed."]
				#[doc = " The keys should be stored within the keystore exposed via runtime"]
				#[doc = " externalities."]
				#[doc = ""]
				#[doc = " The seed needs to be a valid `utf8` string."]
				#[doc = ""]
				#[doc = " Returns the concatenated SCALE encoded public keys."]
				pub fn generate_session_keys(
					&self,
					seed: types::generate_session_keys::Seed,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::GenerateSessionKeys,
					types::generate_session_keys::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"SessionKeys",
						"generate_session_keys",
						types::GenerateSessionKeys { seed },
						[
							96u8, 171u8, 164u8, 166u8, 175u8, 102u8, 101u8, 47u8, 133u8, 95u8,
							102u8, 202u8, 83u8, 26u8, 238u8, 47u8, 126u8, 132u8, 22u8, 11u8, 33u8,
							190u8, 175u8, 94u8, 58u8, 245u8, 46u8, 80u8, 195u8, 184u8, 107u8, 65u8,
						],
					)
				}
				#[doc = " Decode the given public session keys."]
				#[doc = ""]
				#[doc = " Returns the list of public raw public keys + key type."]
				pub fn decode_session_keys(
					&self,
					encoded: types::decode_session_keys::Encoded,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::DecodeSessionKeys,
					types::decode_session_keys::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"SessionKeys",
						"decode_session_keys",
						types::DecodeSessionKeys { encoded },
						[
							57u8, 242u8, 18u8, 51u8, 132u8, 110u8, 238u8, 255u8, 39u8, 194u8, 8u8,
							54u8, 198u8, 178u8, 75u8, 151u8, 148u8, 176u8, 144u8, 197u8, 87u8,
							29u8, 179u8, 235u8, 176u8, 78u8, 252u8, 103u8, 72u8, 203u8, 151u8,
							248u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod generate_session_keys {
					use super::runtime_types;
					pub type Seed = ::core::option::Option<
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					>;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct GenerateSessionKeys {
					pub seed: generate_session_keys::Seed,
				}
				pub mod decode_session_keys {
					use super::runtime_types;
					pub type Encoded =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<(
								::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
								runtime_types::sp_core::crypto::KeyTypeId,
							)>,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct DecodeSessionKeys {
					pub encoded: decode_session_keys::Encoded,
				}
			}
		}
		pub mod aura_api {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " API necessary for block authorship with aura."]
			pub struct AuraApi;
			impl AuraApi {
				#[doc = " Returns the slot duration for Aura."]
				#[doc = ""]
				#[doc = " Currently, only the value provided by this type at genesis will be used."]
				pub fn slot_duration(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::SlotDuration,
					types::slot_duration::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"AuraApi",
						"slot_duration",
						types::SlotDuration {},
						[
							233u8, 210u8, 132u8, 172u8, 100u8, 125u8, 239u8, 92u8, 114u8, 82u8,
							7u8, 110u8, 179u8, 196u8, 10u8, 19u8, 211u8, 15u8, 174u8, 2u8, 91u8,
							73u8, 133u8, 100u8, 205u8, 201u8, 191u8, 60u8, 163u8, 122u8, 215u8,
							10u8,
						],
					)
				}
				#[doc = " Return the current set of authorities."]
				pub fn authorities(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::Authorities,
					types::authorities::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"AuraApi",
						"authorities",
						types::Authorities {},
						[
							35u8, 244u8, 24u8, 155u8, 95u8, 1u8, 221u8, 159u8, 33u8, 144u8, 213u8,
							26u8, 13u8, 21u8, 136u8, 72u8, 45u8, 47u8, 15u8, 51u8, 235u8, 10u8,
							6u8, 219u8, 9u8, 246u8, 50u8, 252u8, 49u8, 77u8, 64u8, 182u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod slot_duration {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::sp_consensus_slots::SlotDuration;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct SlotDuration {}
				pub mod authorities {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::sp_consensus_aura::sr25519::app_sr25519::Public,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Authorities {}
			}
		}
		pub mod aura_unincluded_segment_api {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " This runtime API is used to inform potential block authors whether they will"]
			#[doc = " have the right to author at a slot, assuming they have claimed the slot."]
			#[doc = ""]
			#[doc = " In particular, this API allows Aura-based parachains to regulate their \"unincluded segment\","]
			#[doc = " which is the section of the head of the chain which has not yet been made available in the"]
			#[doc = " relay chain."]
			#[doc = ""]
			#[doc = " When the unincluded segment is short, Aura chains will allow authors to create multiple"]
			#[doc = " blocks per slot in order to build a backlog. When it is saturated, this API will limit"]
			#[doc = " the amount of blocks that can be created."]
			pub struct AuraUnincludedSegmentApi;
			impl AuraUnincludedSegmentApi {
				#[doc = " Whether it is legal to extend the chain, assuming the given block is the most"]
				#[doc = " recently included one as-of the relay parent that will be built against, and"]
				#[doc = " the given slot."]
				#[doc = ""]
				#[doc = " This should be consistent with the logic the runtime uses when validating blocks to"]
				#[doc = " avoid issues."]
				#[doc = ""]
				#[doc = " When the unincluded segment is empty, i.e. `included_hash == at`, where at is the block"]
				#[doc = " whose state we are querying against, this must always return `true` as long as the slot"]
				#[doc = " is more recent than the included block itself."]
				pub fn can_build_upon(
					&self,
					included_hash: types::can_build_upon::IncludedHash,
					slot: types::can_build_upon::Slot,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::CanBuildUpon,
					types::can_build_upon::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"AuraUnincludedSegmentApi",
						"can_build_upon",
						types::CanBuildUpon { included_hash, slot },
						[
							255u8, 59u8, 225u8, 229u8, 189u8, 250u8, 48u8, 150u8, 92u8, 226u8,
							221u8, 202u8, 143u8, 145u8, 107u8, 112u8, 151u8, 146u8, 136u8, 155u8,
							118u8, 174u8, 52u8, 178u8, 14u8, 89u8, 194u8, 157u8, 110u8, 103u8,
							92u8, 72u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod can_build_upon {
					use super::runtime_types;
					pub type IncludedHash = ::subxt::ext::subxt_core::utils::H256;
					pub type Slot = runtime_types::sp_consensus_slots::Slot;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::primitive::bool;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CanBuildUpon {
					pub included_hash: can_build_upon::IncludedHash,
					pub slot: can_build_upon::Slot,
				}
			}
		}
		pub mod account_nonce_api {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " The API to query account nonce."]
			pub struct AccountNonceApi;
			impl AccountNonceApi {
				#[doc = " Get current account nonce of given `AccountId`."]
				pub fn account_nonce(
					&self,
					account: types::account_nonce::Account,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::AccountNonce,
					types::account_nonce::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"AccountNonceApi",
						"account_nonce",
						types::AccountNonce { account },
						[
							231u8, 82u8, 7u8, 227u8, 131u8, 2u8, 215u8, 252u8, 173u8, 82u8, 11u8,
							103u8, 200u8, 25u8, 114u8, 116u8, 79u8, 229u8, 152u8, 150u8, 236u8,
							37u8, 101u8, 26u8, 220u8, 146u8, 182u8, 101u8, 73u8, 55u8, 191u8,
							171u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod account_nonce {
					use super::runtime_types;
					pub type Account = ::subxt::ext::subxt_core::utils::AccountId32;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::primitive::u32;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AccountNonce {
					pub account: account_nonce::Account,
				}
			}
		}
		pub mod transaction_payment_api {
			use super::root_mod;
			use super::runtime_types;
			pub struct TransactionPaymentApi;
			impl TransactionPaymentApi {
				pub fn query_info(
					&self,
					uxt: types::query_info::Uxt,
					len: types::query_info::Len,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryInfo,
					types::query_info::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentApi",
						"query_info",
						types::QueryInfo { uxt, len },
						[
							18u8, 21u8, 49u8, 170u8, 31u8, 239u8, 200u8, 100u8, 31u8, 197u8, 68u8,
							91u8, 138u8, 180u8, 24u8, 255u8, 153u8, 200u8, 167u8, 148u8, 66u8,
							89u8, 215u8, 13u8, 119u8, 183u8, 48u8, 15u8, 187u8, 50u8, 226u8, 111u8,
						],
					)
				}
				pub fn query_fee_details(
					&self,
					uxt: types::query_fee_details::Uxt,
					len: types::query_fee_details::Len,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryFeeDetails,
					types::query_fee_details::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentApi",
						"query_fee_details",
						types::QueryFeeDetails { uxt, len },
						[
							244u8, 136u8, 121u8, 182u8, 139u8, 54u8, 166u8, 70u8, 32u8, 165u8,
							61u8, 193u8, 186u8, 109u8, 2u8, 165u8, 42u8, 11u8, 113u8, 159u8, 21u8,
							62u8, 14u8, 60u8, 212u8, 124u8, 63u8, 118u8, 183u8, 92u8, 186u8, 52u8,
						],
					)
				}
				pub fn query_weight_to_fee(
					&self,
					weight: types::query_weight_to_fee::Weight,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryWeightToFee,
					types::query_weight_to_fee::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentApi",
						"query_weight_to_fee",
						types::QueryWeightToFee { weight },
						[
							206u8, 243u8, 189u8, 83u8, 231u8, 244u8, 247u8, 52u8, 126u8, 208u8,
							224u8, 5u8, 163u8, 108u8, 254u8, 114u8, 214u8, 156u8, 227u8, 217u8,
							211u8, 198u8, 121u8, 164u8, 110u8, 54u8, 181u8, 146u8, 50u8, 146u8,
							146u8, 23u8,
						],
					)
				}
				pub fn query_length_to_fee(
					&self,
					length: types::query_length_to_fee::Length,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryLengthToFee,
					types::query_length_to_fee::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentApi",
						"query_length_to_fee",
						types::QueryLengthToFee { length },
						[
							92u8, 132u8, 29u8, 119u8, 66u8, 11u8, 196u8, 224u8, 129u8, 23u8, 249u8,
							12u8, 32u8, 28u8, 92u8, 50u8, 188u8, 101u8, 203u8, 229u8, 248u8, 216u8,
							130u8, 150u8, 212u8, 161u8, 81u8, 254u8, 116u8, 89u8, 162u8, 48u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod query_info {
					use super::runtime_types;
					pub type Uxt = runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > ;
					pub type Len = ::core::primitive::u32;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							runtime_types::pallet_transaction_payment::types::RuntimeDispatchInfo<
								::core::primitive::u128,
								runtime_types::sp_weights::weight_v2::Weight,
							>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryInfo {
					pub uxt: query_info::Uxt,
					pub len: query_info::Len,
				}
				pub mod query_fee_details {
					use super::runtime_types;
					pub type Uxt = runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > ;
					pub type Len = ::core::primitive::u32;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							runtime_types::pallet_transaction_payment::types::FeeDetails<
								::core::primitive::u128,
							>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryFeeDetails {
					pub uxt: query_fee_details::Uxt,
					pub len: query_fee_details::Len,
				}
				pub mod query_weight_to_fee {
					use super::runtime_types;
					pub type Weight = runtime_types::sp_weights::weight_v2::Weight;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::primitive::u128;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryWeightToFee {
					pub weight: query_weight_to_fee::Weight,
				}
				pub mod query_length_to_fee {
					use super::runtime_types;
					pub type Length = ::core::primitive::u32;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::primitive::u128;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryLengthToFee {
					pub length: query_length_to_fee::Length,
				}
			}
		}
		pub mod transaction_payment_call_api {
			use super::root_mod;
			use super::runtime_types;
			pub struct TransactionPaymentCallApi;
			impl TransactionPaymentCallApi {
				#[doc = " Query information of a dispatch class, weight, and fee of a given encoded `Call`."]
				pub fn query_call_info(
					&self,
					call: types::query_call_info::Call,
					len: types::query_call_info::Len,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryCallInfo,
					types::query_call_info::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentCallApi",
						"query_call_info",
						types::QueryCallInfo { call, len },
						[
							30u8, 138u8, 178u8, 236u8, 178u8, 94u8, 119u8, 15u8, 47u8, 252u8,
							133u8, 50u8, 71u8, 85u8, 213u8, 75u8, 128u8, 119u8, 251u8, 136u8, 15u8,
							6u8, 205u8, 28u8, 67u8, 78u8, 77u8, 231u8, 42u8, 218u8, 233u8, 88u8,
						],
					)
				}
				#[doc = " Query fee details of a given encoded `Call`."]
				pub fn query_call_fee_details(
					&self,
					call: types::query_call_fee_details::Call,
					len: types::query_call_fee_details::Len,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryCallFeeDetails,
					types::query_call_fee_details::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentCallApi",
						"query_call_fee_details",
						types::QueryCallFeeDetails { call, len },
						[
							91u8, 199u8, 237u8, 1u8, 171u8, 236u8, 142u8, 3u8, 244u8, 143u8, 170u8,
							142u8, 154u8, 104u8, 239u8, 96u8, 59u8, 15u8, 61u8, 176u8, 184u8,
							155u8, 115u8, 244u8, 91u8, 148u8, 16u8, 143u8, 218u8, 155u8, 213u8,
							50u8,
						],
					)
				}
				#[doc = " Query the output of the current `WeightToFee` given some input."]
				pub fn query_weight_to_fee(
					&self,
					weight: types::query_weight_to_fee::Weight,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryWeightToFee,
					types::query_weight_to_fee::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentCallApi",
						"query_weight_to_fee",
						types::QueryWeightToFee { weight },
						[
							117u8, 91u8, 94u8, 22u8, 248u8, 212u8, 15u8, 23u8, 97u8, 116u8, 64u8,
							228u8, 83u8, 123u8, 87u8, 77u8, 97u8, 7u8, 98u8, 181u8, 6u8, 165u8,
							114u8, 141u8, 164u8, 113u8, 126u8, 88u8, 174u8, 171u8, 224u8, 35u8,
						],
					)
				}
				#[doc = " Query the output of the current `LengthToFee` given some input."]
				pub fn query_length_to_fee(
					&self,
					length: types::query_length_to_fee::Length,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::QueryLengthToFee,
					types::query_length_to_fee::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TransactionPaymentCallApi",
						"query_length_to_fee",
						types::QueryLengthToFee { length },
						[
							246u8, 40u8, 4u8, 160u8, 152u8, 94u8, 170u8, 53u8, 205u8, 122u8, 5u8,
							69u8, 70u8, 25u8, 128u8, 156u8, 119u8, 134u8, 116u8, 147u8, 14u8,
							164u8, 65u8, 140u8, 86u8, 13u8, 250u8, 218u8, 89u8, 95u8, 234u8, 228u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod query_call_info {
					use super::runtime_types;
					pub type Call = runtime_types::paseo_parachain_runtime::RuntimeCall;
					pub type Len = ::core::primitive::u32;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							runtime_types::pallet_transaction_payment::types::RuntimeDispatchInfo<
								::core::primitive::u128,
								runtime_types::sp_weights::weight_v2::Weight,
							>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryCallInfo {
					pub call: query_call_info::Call,
					pub len: query_call_info::Len,
				}
				pub mod query_call_fee_details {
					use super::runtime_types;
					pub type Call = runtime_types::paseo_parachain_runtime::RuntimeCall;
					pub type Len = ::core::primitive::u32;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							runtime_types::pallet_transaction_payment::types::FeeDetails<
								::core::primitive::u128,
							>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryCallFeeDetails {
					pub call: query_call_fee_details::Call,
					pub len: query_call_fee_details::Len,
				}
				pub mod query_weight_to_fee {
					use super::runtime_types;
					pub type Weight = runtime_types::sp_weights::weight_v2::Weight;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::primitive::u128;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryWeightToFee {
					pub weight: query_weight_to_fee::Weight,
				}
				pub mod query_length_to_fee {
					use super::runtime_types;
					pub type Length = ::core::primitive::u32;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::primitive::u128;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryLengthToFee {
					pub length: query_length_to_fee::Length,
				}
			}
		}
		pub mod collect_collation_info {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " Runtime api to collect information about a collation."]
			pub struct CollectCollationInfo;
			impl CollectCollationInfo {
				#[doc = " Collect information about a collation."]
				#[doc = ""]
				#[doc = " The given `header` is the header of the built block for that"]
				#[doc = " we are collecting the collation info for."]
				pub fn collect_collation_info(
					&self,
					header: types::collect_collation_info::Header,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::CollectCollationInfo,
					types::collect_collation_info::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"CollectCollationInfo",
						"collect_collation_info",
						types::CollectCollationInfo { header },
						[
							56u8, 138u8, 105u8, 91u8, 216u8, 40u8, 255u8, 98u8, 86u8, 138u8, 185u8,
							155u8, 80u8, 141u8, 85u8, 48u8, 252u8, 235u8, 178u8, 231u8, 111u8,
							216u8, 71u8, 20u8, 33u8, 202u8, 24u8, 215u8, 214u8, 132u8, 51u8, 166u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod collect_collation_info {
					use super::runtime_types;
					pub type Header =
						runtime_types::sp_runtime::generic::header::Header<::core::primitive::u32>;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::cumulus_primitives_core::CollationInfo;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CollectCollationInfo {
					pub header: collect_collation_info::Header,
				}
			}
		}
		pub mod ethereum_runtime_rpc_api {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " API necessary for Ethereum-compatibility layer."]
			pub struct EthereumRuntimeRPCApi;
			impl EthereumRuntimeRPCApi {
				#[doc = " Returns runtime defined pallet_evm::ChainId."]
				pub fn chain_id(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::ChainId,
					types::chain_id::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"chain_id",
						types::ChainId {},
						[
							119u8, 123u8, 11u8, 231u8, 92u8, 198u8, 90u8, 85u8, 52u8, 55u8, 105u8,
							101u8, 142u8, 11u8, 27u8, 201u8, 110u8, 228u8, 114u8, 128u8, 217u8,
							236u8, 146u8, 229u8, 7u8, 81u8, 102u8, 18u8, 115u8, 141u8, 216u8,
							164u8,
						],
					)
				}
				#[doc = " Returns pallet_evm::Accounts by address."]
				pub fn account_basic(
					&self,
					address: types::account_basic::Address,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::AccountBasic,
					types::account_basic::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"account_basic",
						types::AccountBasic { address },
						[
							253u8, 214u8, 163u8, 33u8, 163u8, 38u8, 64u8, 14u8, 118u8, 135u8, 77u8,
							215u8, 45u8, 161u8, 43u8, 40u8, 140u8, 131u8, 209u8, 187u8, 171u8,
							129u8, 42u8, 206u8, 175u8, 50u8, 208u8, 186u8, 216u8, 109u8, 166u8,
							123u8,
						],
					)
				}
				#[doc = " Returns FixedGasPrice::min_gas_price"]
				pub fn gas_price(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::GasPrice,
					types::gas_price::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"gas_price",
						types::GasPrice {},
						[
							109u8, 203u8, 181u8, 107u8, 184u8, 171u8, 218u8, 171u8, 23u8, 235u8,
							214u8, 209u8, 233u8, 89u8, 99u8, 63u8, 16u8, 175u8, 44u8, 31u8, 241u8,
							122u8, 126u8, 146u8, 146u8, 131u8, 11u8, 84u8, 129u8, 203u8, 249u8,
							235u8,
						],
					)
				}
				#[doc = " For a given account address, returns pallet_evm::AccountCodes."]
				pub fn account_code_at(
					&self,
					address: types::account_code_at::Address,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::AccountCodeAt,
					types::account_code_at::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"account_code_at",
						types::AccountCodeAt { address },
						[
							140u8, 192u8, 250u8, 89u8, 84u8, 152u8, 124u8, 241u8, 255u8, 64u8,
							238u8, 86u8, 13u8, 46u8, 252u8, 66u8, 62u8, 109u8, 46u8, 229u8, 126u8,
							188u8, 64u8, 24u8, 75u8, 157u8, 217u8, 154u8, 102u8, 207u8, 181u8,
							254u8,
						],
					)
				}
				#[doc = " Returns the converted FindAuthor::find_author authority id."]
				pub fn author(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::Author,
					types::author::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"author",
						types::Author {},
						[
							101u8, 105u8, 12u8, 234u8, 72u8, 53u8, 32u8, 56u8, 226u8, 189u8, 216u8,
							228u8, 196u8, 14u8, 147u8, 191u8, 144u8, 203u8, 195u8, 85u8, 162u8,
							67u8, 86u8, 196u8, 116u8, 129u8, 57u8, 102u8, 223u8, 129u8, 222u8,
							71u8,
						],
					)
				}
				#[doc = " For a given account address and index, returns pallet_evm::AccountStorages."]
				pub fn storage_at(
					&self,
					address: types::storage_at::Address,
					index: types::storage_at::Index,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::StorageAt,
					types::storage_at::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"storage_at",
						types::StorageAt { address, index },
						[
							19u8, 29u8, 54u8, 70u8, 199u8, 223u8, 39u8, 4u8, 126u8, 126u8, 81u8,
							102u8, 223u8, 60u8, 2u8, 155u8, 60u8, 167u8, 0u8, 219u8, 98u8, 236u8,
							145u8, 115u8, 202u8, 87u8, 183u8, 131u8, 230u8, 189u8, 228u8, 91u8,
						],
					)
				}
				pub fn call(
					&self,
					from: types::call::From,
					to: types::call::To,
					data: types::call::Data,
					value: types::call::Value,
					gas_limit: types::call::GasLimit,
					max_fee_per_gas: types::call::MaxFeePerGas,
					max_priority_fee_per_gas: types::call::MaxPriorityFeePerGas,
					nonce: types::call::Nonce,
					estimate: types::call::Estimate,
					access_list: types::call::AccessList,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::Call,
					types::call::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"call",
						types::Call {
							from,
							to,
							data,
							value,
							gas_limit,
							max_fee_per_gas,
							max_priority_fee_per_gas,
							nonce,
							estimate,
							access_list,
						},
						[
							42u8, 174u8, 110u8, 248u8, 167u8, 56u8, 238u8, 141u8, 64u8, 147u8,
							191u8, 107u8, 1u8, 132u8, 120u8, 22u8, 189u8, 52u8, 90u8, 121u8, 5u8,
							88u8, 147u8, 123u8, 53u8, 198u8, 212u8, 4u8, 141u8, 168u8, 56u8, 111u8,
						],
					)
				}
				pub fn create(
					&self,
					from: types::create::From,
					data: types::create::Data,
					value: types::create::Value,
					gas_limit: types::create::GasLimit,
					max_fee_per_gas: types::create::MaxFeePerGas,
					max_priority_fee_per_gas: types::create::MaxPriorityFeePerGas,
					nonce: types::create::Nonce,
					estimate: types::create::Estimate,
					access_list: types::create::AccessList,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::Create,
					types::create::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"create",
						types::Create {
							from,
							data,
							value,
							gas_limit,
							max_fee_per_gas,
							max_priority_fee_per_gas,
							nonce,
							estimate,
							access_list,
						},
						[
							251u8, 141u8, 86u8, 140u8, 222u8, 79u8, 227u8, 54u8, 84u8, 205u8, 69u8,
							204u8, 211u8, 29u8, 110u8, 132u8, 59u8, 139u8, 181u8, 48u8, 7u8, 158u8,
							39u8, 175u8, 40u8, 230u8, 79u8, 254u8, 115u8, 179u8, 136u8, 120u8,
						],
					)
				}
				#[doc = " Return the current block."]
				pub fn current_block(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::CurrentBlock,
					types::current_block::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"current_block",
						types::CurrentBlock {},
						[
							223u8, 174u8, 72u8, 237u8, 254u8, 149u8, 89u8, 18u8, 134u8, 21u8,
							192u8, 75u8, 29u8, 248u8, 230u8, 130u8, 137u8, 203u8, 81u8, 96u8,
							179u8, 133u8, 232u8, 183u8, 62u8, 196u8, 175u8, 136u8, 181u8, 66u8,
							230u8, 115u8,
						],
					)
				}
				#[doc = " Return the current receipt."]
				pub fn current_receipts(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::CurrentReceipts,
					types::current_receipts::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"current_receipts",
						types::CurrentReceipts {},
						[
							174u8, 101u8, 240u8, 50u8, 204u8, 64u8, 207u8, 220u8, 200u8, 196u8,
							248u8, 93u8, 30u8, 53u8, 208u8, 184u8, 110u8, 47u8, 19u8, 200u8, 35u8,
							70u8, 26u8, 66u8, 181u8, 129u8, 237u8, 115u8, 81u8, 169u8, 213u8,
							137u8,
						],
					)
				}
				#[doc = " Return the current transaction status."]
				pub fn current_transaction_statuses(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::CurrentTransactionStatuses,
					types::current_transaction_statuses::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"current_transaction_statuses",
						types::CurrentTransactionStatuses {},
						[
							104u8, 186u8, 112u8, 255u8, 185u8, 24u8, 178u8, 204u8, 141u8, 184u8,
							235u8, 60u8, 152u8, 200u8, 223u8, 154u8, 105u8, 193u8, 211u8, 82u8,
							77u8, 157u8, 122u8, 36u8, 86u8, 224u8, 150u8, 32u8, 48u8, 176u8, 129u8,
							33u8,
						],
					)
				}
				pub fn current_all(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::CurrentAll,
					types::current_all::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"current_all",
						types::CurrentAll {},
						[
							140u8, 52u8, 171u8, 209u8, 255u8, 183u8, 119u8, 234u8, 65u8, 156u8,
							128u8, 33u8, 217u8, 78u8, 181u8, 48u8, 162u8, 34u8, 92u8, 253u8, 132u8,
							67u8, 54u8, 1u8, 254u8, 44u8, 219u8, 112u8, 205u8, 2u8, 219u8, 225u8,
						],
					)
				}
				#[doc = " Receives a `Vec<OpaqueExtrinsic>` and filters all the ethereum transactions."]
				pub fn extrinsic_filter(
					&self,
					xts: types::extrinsic_filter::Xts,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::ExtrinsicFilter,
					types::extrinsic_filter::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"extrinsic_filter",
						types::ExtrinsicFilter { xts },
						[
							34u8, 199u8, 221u8, 81u8, 136u8, 126u8, 202u8, 122u8, 232u8, 184u8,
							41u8, 211u8, 199u8, 24u8, 250u8, 176u8, 199u8, 94u8, 233u8, 221u8,
							104u8, 94u8, 198u8, 52u8, 218u8, 125u8, 104u8, 91u8, 178u8, 133u8,
							195u8, 113u8,
						],
					)
				}
				#[doc = " Return the elasticity multiplier."]
				pub fn elasticity(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::Elasticity,
					types::elasticity::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"elasticity",
						types::Elasticity {},
						[
							231u8, 129u8, 189u8, 25u8, 156u8, 180u8, 55u8, 76u8, 115u8, 115u8,
							97u8, 87u8, 113u8, 235u8, 49u8, 153u8, 82u8, 254u8, 238u8, 66u8, 99u8,
							243u8, 179u8, 91u8, 186u8, 122u8, 112u8, 79u8, 0u8, 240u8, 239u8, 97u8,
						],
					)
				}
				#[doc = " Used to determine if gas limit multiplier for non-transactional calls (eth_call/estimateGas)"]
				#[doc = " is supported."]
				pub fn gas_limit_multiplier_support(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::GasLimitMultiplierSupport,
					types::gas_limit_multiplier_support::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"gas_limit_multiplier_support",
						types::GasLimitMultiplierSupport {},
						[
							156u8, 34u8, 145u8, 114u8, 36u8, 174u8, 36u8, 65u8, 158u8, 45u8, 203u8,
							4u8, 160u8, 170u8, 43u8, 227u8, 204u8, 213u8, 197u8, 230u8, 190u8,
							119u8, 166u8, 243u8, 225u8, 138u8, 134u8, 169u8, 187u8, 95u8, 245u8,
							102u8,
						],
					)
				}
				#[doc = " Return the pending block."]
				pub fn pending_block(
					&self,
					xts: types::pending_block::Xts,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::PendingBlock,
					types::pending_block::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"pending_block",
						types::PendingBlock { xts },
						[
							173u8, 229u8, 156u8, 7u8, 102u8, 234u8, 114u8, 87u8, 242u8, 104u8,
							174u8, 57u8, 202u8, 118u8, 125u8, 215u8, 117u8, 85u8, 137u8, 215u8,
							13u8, 146u8, 45u8, 197u8, 223u8, 59u8, 146u8, 30u8, 14u8, 209u8, 45u8,
							198u8,
						],
					)
				}
				#[doc = " Initialize the pending block."]
				#[doc = " The behavior should be the same as the runtime api Core_initialize_block but"]
				#[doc = " for a \"pending\" block."]
				#[doc = " If your project don't need to have a different behavior to initialize \"pending\" blocks,"]
				#[doc = " you can copy your Core_initialize_block implementation."]
				pub fn initialize_pending_block(
					&self,
					header: types::initialize_pending_block::Header,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::InitializePendingBlock,
					types::initialize_pending_block::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"EthereumRuntimeRPCApi",
						"initialize_pending_block",
						types::InitializePendingBlock { header },
						[
							46u8, 174u8, 182u8, 191u8, 229u8, 174u8, 228u8, 72u8, 119u8, 110u8,
							223u8, 19u8, 112u8, 12u8, 174u8, 88u8, 159u8, 126u8, 4u8, 166u8, 107u8,
							244u8, 243u8, 224u8, 84u8, 131u8, 40u8, 164u8, 133u8, 52u8, 76u8,
							114u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod chain_id {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::primitive::u64;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ChainId {}
				pub mod account_basic {
					use super::runtime_types;
					pub type Address = ::subxt::ext::subxt_core::utils::H160;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::evm::backend::Basic;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AccountBasic {
					pub address: account_basic::Address,
				}
				pub mod gas_price {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types::primitive_types::U256;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct GasPrice {}
				pub mod account_code_at {
					use super::runtime_types;
					pub type Address = ::subxt::ext::subxt_core::utils::H160;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AccountCodeAt {
					pub address: account_code_at::Address,
				}
				pub mod author {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::subxt::ext::subxt_core::utils::H160;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Author {}
				pub mod storage_at {
					use super::runtime_types;
					pub type Address = ::subxt::ext::subxt_core::utils::H160;
					pub type Index = runtime_types::primitive_types::U256;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::subxt::ext::subxt_core::utils::H256;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct StorageAt {
					pub address: storage_at::Address,
					pub index: storage_at::Index,
				}
				pub mod call {
					use super::runtime_types;
					pub type From = ::subxt::ext::subxt_core::utils::H160;
					pub type To = ::subxt::ext::subxt_core::utils::H160;
					pub type Data =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type Value = runtime_types::primitive_types::U256;
					pub type GasLimit = runtime_types::primitive_types::U256;
					pub type MaxFeePerGas =
						::core::option::Option<runtime_types::primitive_types::U256>;
					pub type MaxPriorityFeePerGas =
						::core::option::Option<runtime_types::primitive_types::U256>;
					pub type Nonce = ::core::option::Option<runtime_types::primitive_types::U256>;
					pub type Estimate = ::core::primitive::bool;
					pub type AccessList = ::core::option::Option<
						::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::utils::H160,
							::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::H256,
							>,
						)>,
					>;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::result::Result<
							runtime_types::fp_evm::ExecutionInfoV2<
								::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
							>,
							runtime_types::sp_runtime::DispatchError,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Call {
					pub from: call::From,
					pub to: call::To,
					pub data: call::Data,
					pub value: call::Value,
					pub gas_limit: call::GasLimit,
					pub max_fee_per_gas: call::MaxFeePerGas,
					pub max_priority_fee_per_gas: call::MaxPriorityFeePerGas,
					pub nonce: call::Nonce,
					pub estimate: call::Estimate,
					pub access_list: call::AccessList,
				}
				pub mod create {
					use super::runtime_types;
					pub type From = ::subxt::ext::subxt_core::utils::H160;
					pub type Data =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type Value = runtime_types::primitive_types::U256;
					pub type GasLimit = runtime_types::primitive_types::U256;
					pub type MaxFeePerGas =
						::core::option::Option<runtime_types::primitive_types::U256>;
					pub type MaxPriorityFeePerGas =
						::core::option::Option<runtime_types::primitive_types::U256>;
					pub type Nonce = ::core::option::Option<runtime_types::primitive_types::U256>;
					pub type Estimate = ::core::primitive::bool;
					pub type AccessList = ::core::option::Option<
						::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::utils::H160,
							::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::H256,
							>,
						)>,
					>;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::result::Result<
							runtime_types::fp_evm::ExecutionInfoV2<
								::subxt::ext::subxt_core::utils::H160,
							>,
							runtime_types::sp_runtime::DispatchError,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Create {
					pub from: create::From,
					pub data: create::Data,
					pub value: create::Value,
					pub gas_limit: create::GasLimit,
					pub max_fee_per_gas: create::MaxFeePerGas,
					pub max_priority_fee_per_gas: create::MaxPriorityFeePerGas,
					pub nonce: create::Nonce,
					pub estimate: create::Estimate,
					pub access_list: create::AccessList,
				}
				pub mod current_block {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::option::Option<
							runtime_types::ethereum::block::Block<
								runtime_types::ethereum::transaction::TransactionV2,
							>,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CurrentBlock {}
				pub mod current_receipts {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<
								runtime_types::ethereum::receipt::ReceiptV3,
							>,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CurrentReceipts {}
				pub mod current_transaction_statuses {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<
								runtime_types::fp_rpc::TransactionStatus,
							>,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CurrentTransactionStatuses {}
				pub mod current_all {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = (
							::core::option::Option<
								runtime_types::ethereum::block::Block<
									runtime_types::ethereum::transaction::TransactionV2,
								>,
							>,
							::core::option::Option<
								::subxt::ext::subxt_core::alloc::vec::Vec<
									runtime_types::ethereum::receipt::ReceiptV3,
								>,
							>,
							::core::option::Option<
								::subxt::ext::subxt_core::alloc::vec::Vec<
									runtime_types::fp_rpc::TransactionStatus,
								>,
							>,
						);
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CurrentAll {}
				pub mod extrinsic_filter {
					use super::runtime_types;
					pub type Xts = :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::ethereum::transaction::TransactionV2,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ExtrinsicFilter {
					pub xts: extrinsic_filter::Xts,
				}
				pub mod elasticity {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::option::Option<
							runtime_types::sp_arithmetic::per_things::Permill,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Elasticity {}
				pub mod gas_limit_multiplier_support {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ();
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct GasLimitMultiplierSupport {}
				pub mod pending_block {
					use super::runtime_types;
					pub type Xts = :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub mod output {
						use super::runtime_types;
						pub type Output = (
							::core::option::Option<
								runtime_types::ethereum::block::Block<
									runtime_types::ethereum::transaction::TransactionV2,
								>,
							>,
							::core::option::Option<
								::subxt::ext::subxt_core::alloc::vec::Vec<
									runtime_types::fp_rpc::TransactionStatus,
								>,
							>,
						);
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct PendingBlock {
					pub xts: pending_block::Xts,
				}
				pub mod initialize_pending_block {
					use super::runtime_types;
					pub type Header =
						runtime_types::sp_runtime::generic::header::Header<::core::primitive::u32>;
					pub mod output {
						use super::runtime_types;
						pub type Output = ();
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct InitializePendingBlock {
					pub header: initialize_pending_block::Header,
				}
			}
		}
		pub mod convert_transaction_runtime_api {
			use super::root_mod;
			use super::runtime_types;
			pub struct ConvertTransactionRuntimeApi;
			impl ConvertTransactionRuntimeApi {
				pub fn convert_transaction(
					&self,
					transaction: types::convert_transaction::Transaction,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::ConvertTransaction,
					types::convert_transaction::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"ConvertTransactionRuntimeApi",
						"convert_transaction",
						types::ConvertTransaction { transaction },
						[
							38u8, 3u8, 65u8, 131u8, 104u8, 95u8, 86u8, 42u8, 177u8, 179u8, 245u8,
							253u8, 51u8, 174u8, 207u8, 232u8, 181u8, 68u8, 71u8, 91u8, 213u8, 63u8,
							48u8, 83u8, 142u8, 178u8, 236u8, 64u8, 185u8, 64u8, 188u8, 123u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod convert_transaction {
					use super::runtime_types;
					pub type Transaction = runtime_types::ethereum::transaction::TransactionV2;
					pub mod output {
						use super::runtime_types;
						pub type Output = runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > ;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ConvertTransaction {
					pub transaction: convert_transaction::Transaction,
				}
			}
		}
		pub mod debug_runtime_api {
			use super::root_mod;
			use super::runtime_types;
			pub struct DebugRuntimeApi;
			impl DebugRuntimeApi {
				pub fn trace_transaction(
					&self,
					extrinsics: types::trace_transaction::Extrinsics,
					transaction: types::trace_transaction::Transaction,
					header: types::trace_transaction::Header,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::TraceTransaction,
					types::trace_transaction::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"DebugRuntimeApi",
						"trace_transaction",
						types::TraceTransaction { extrinsics, transaction, header },
						[
							57u8, 146u8, 198u8, 214u8, 208u8, 194u8, 42u8, 132u8, 64u8, 147u8,
							65u8, 156u8, 77u8, 44u8, 224u8, 14u8, 235u8, 85u8, 203u8, 49u8, 212u8,
							160u8, 75u8, 245u8, 186u8, 65u8, 33u8, 231u8, 69u8, 130u8, 90u8, 62u8,
						],
					)
				}
				pub fn trace_block(
					&self,
					extrinsics: types::trace_block::Extrinsics,
					known_transactions: types::trace_block::KnownTransactions,
					header: types::trace_block::Header,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::TraceBlock,
					types::trace_block::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"DebugRuntimeApi",
						"trace_block",
						types::TraceBlock { extrinsics, known_transactions, header },
						[
							116u8, 185u8, 157u8, 239u8, 105u8, 189u8, 60u8, 43u8, 88u8, 62u8,
							160u8, 121u8, 36u8, 157u8, 77u8, 84u8, 69u8, 226u8, 11u8, 70u8, 43u8,
							127u8, 227u8, 212u8, 128u8, 84u8, 164u8, 190u8, 234u8, 2u8, 105u8,
							212u8,
						],
					)
				}
				pub fn trace_call(
					&self,
					header: types::trace_call::Header,
					from: types::trace_call::From,
					to: types::trace_call::To,
					data: types::trace_call::Data,
					value: types::trace_call::Value,
					gas_limit: types::trace_call::GasLimit,
					max_fee_per_gas: types::trace_call::MaxFeePerGas,
					max_priority_fee_per_gas: types::trace_call::MaxPriorityFeePerGas,
					nonce: types::trace_call::Nonce,
					access_list: types::trace_call::AccessList,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::TraceCall,
					types::trace_call::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"DebugRuntimeApi",
						"trace_call",
						types::TraceCall {
							header,
							from,
							to,
							data,
							value,
							gas_limit,
							max_fee_per_gas,
							max_priority_fee_per_gas,
							nonce,
							access_list,
						},
						[
							150u8, 218u8, 67u8, 248u8, 174u8, 8u8, 184u8, 178u8, 138u8, 34u8,
							177u8, 4u8, 74u8, 161u8, 37u8, 237u8, 15u8, 237u8, 99u8, 249u8, 3u8,
							47u8, 130u8, 70u8, 107u8, 91u8, 218u8, 200u8, 33u8, 80u8, 19u8, 184u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod trace_transaction {
					use super::runtime_types;
					pub type Extrinsics = :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub type Transaction = runtime_types::ethereum::transaction::TransactionV2;
					pub type Header =
						runtime_types::sp_runtime::generic::header::Header<::core::primitive::u32>;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TraceTransaction {
					pub extrinsics: trace_transaction::Extrinsics,
					pub transaction: trace_transaction::Transaction,
					pub header: trace_transaction::Header,
				}
				pub mod trace_block {
					use super::runtime_types;
					pub type Extrinsics = :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub type KnownTransactions = ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::utils::H256,
					>;
					pub type Header =
						runtime_types::sp_runtime::generic::header::Header<::core::primitive::u32>;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TraceBlock {
					pub extrinsics: trace_block::Extrinsics,
					pub known_transactions: trace_block::KnownTransactions,
					pub header: trace_block::Header,
				}
				pub mod trace_call {
					use super::runtime_types;
					pub type Header =
						runtime_types::sp_runtime::generic::header::Header<::core::primitive::u32>;
					pub type From = ::subxt::ext::subxt_core::utils::H160;
					pub type To = ::subxt::ext::subxt_core::utils::H160;
					pub type Data =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type Value = runtime_types::primitive_types::U256;
					pub type GasLimit = runtime_types::primitive_types::U256;
					pub type MaxFeePerGas =
						::core::option::Option<runtime_types::primitive_types::U256>;
					pub type MaxPriorityFeePerGas =
						::core::option::Option<runtime_types::primitive_types::U256>;
					pub type Nonce = ::core::option::Option<runtime_types::primitive_types::U256>;
					pub type AccessList = ::core::option::Option<
						::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::utils::H160,
							::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::H256,
							>,
						)>,
					>;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TraceCall {
					pub header: trace_call::Header,
					pub from: trace_call::From,
					pub to: trace_call::To,
					pub data: trace_call::Data,
					pub value: trace_call::Value,
					pub gas_limit: trace_call::GasLimit,
					pub max_fee_per_gas: trace_call::MaxFeePerGas,
					pub max_priority_fee_per_gas: trace_call::MaxPriorityFeePerGas,
					pub nonce: trace_call::Nonce,
					pub access_list: trace_call::AccessList,
				}
			}
		}
		pub mod tx_pool_runtime_api {
			use super::root_mod;
			use super::runtime_types;
			pub struct TxPoolRuntimeApi;
			impl TxPoolRuntimeApi {
				pub fn extrinsic_filter(
					&self,
					xt_ready: types::extrinsic_filter::XtReady,
					xt_future: types::extrinsic_filter::XtFuture,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::ExtrinsicFilter,
					types::extrinsic_filter::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"TxPoolRuntimeApi",
						"extrinsic_filter",
						types::ExtrinsicFilter { xt_ready, xt_future },
						[
							134u8, 154u8, 125u8, 22u8, 74u8, 169u8, 248u8, 124u8, 232u8, 73u8,
							189u8, 109u8, 48u8, 207u8, 50u8, 60u8, 91u8, 154u8, 214u8, 247u8, 26u8,
							71u8, 174u8, 96u8, 248u8, 6u8, 145u8, 202u8, 130u8, 220u8, 66u8, 127u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod extrinsic_filter {
					use super::runtime_types;
					pub type XtReady = :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub type XtFuture = :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < runtime_types :: fp_self_contained :: unchecked_extrinsic :: UncheckedExtrinsic < :: subxt :: ext :: subxt_core :: utils :: MultiAddress < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , () > , runtime_types :: paseo_parachain_runtime :: RuntimeCall , runtime_types :: sp_runtime :: MultiSignature , (runtime_types :: frame_system :: extensions :: check_non_zero_sender :: CheckNonZeroSender , runtime_types :: frame_system :: extensions :: check_spec_version :: CheckSpecVersion , runtime_types :: frame_system :: extensions :: check_tx_version :: CheckTxVersion , runtime_types :: frame_system :: extensions :: check_genesis :: CheckGenesis , runtime_types :: frame_system :: extensions :: check_mortality :: CheckMortality , runtime_types :: frame_system :: extensions :: check_nonce :: CheckNonce , runtime_types :: frame_system :: extensions :: check_weight :: CheckWeight , runtime_types :: pallet_transaction_payment :: ChargeTransactionPayment ,) > > ;
					pub mod output {
						use super::runtime_types;
						pub type Output =
							runtime_types::moonbeam_rpc_primitives_txpool::TxPoolResponse;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ExtrinsicFilter {
					pub xt_ready: extrinsic_filter::XtReady,
					pub xt_future: extrinsic_filter::XtFuture,
				}
			}
		}
		pub mod omni_account_api {
			use super::root_mod;
			use super::runtime_types;
			pub struct OmniAccountApi;
			impl OmniAccountApi {
				pub fn omni_account(
					&self,
					id: types::omni_account::Id,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::OmniAccount,
					types::omni_account::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"OmniAccountApi",
						"omni_account",
						types::OmniAccount { id },
						[
							165u8, 192u8, 142u8, 204u8, 201u8, 38u8, 26u8, 251u8, 252u8, 190u8,
							7u8, 232u8, 65u8, 172u8, 151u8, 250u8, 180u8, 66u8, 21u8, 94u8, 94u8,
							17u8, 125u8, 144u8, 183u8, 127u8, 114u8, 120u8, 217u8, 221u8, 245u8,
							2u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod omni_account {
					use super::runtime_types;
					pub type Id = runtime_types::core_primitives::identity::Identity;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::subxt::ext::subxt_core::utils::AccountId32;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct OmniAccount {
					pub id: omni_account::Id,
				}
			}
		}
		pub mod genesis_builder {
			use super::root_mod;
			use super::runtime_types;
			#[doc = " API to interact with RuntimeGenesisConfig for the runtime"]
			pub struct GenesisBuilder;
			impl GenesisBuilder {
				#[doc = " Build `RuntimeGenesisConfig` from a JSON blob not using any defaults and store it in the"]
				#[doc = " storage."]
				#[doc = ""]
				#[doc = " In the case of a FRAME-based runtime, this function deserializes the full `RuntimeGenesisConfig` from the given JSON blob and"]
				#[doc = " puts it into the storage. If the provided JSON blob is incorrect or incomplete or the"]
				#[doc = " deserialization fails, an error is returned."]
				#[doc = ""]
				#[doc = " Please note that provided JSON blob must contain all `RuntimeGenesisConfig` fields, no"]
				#[doc = " defaults will be used."]
				pub fn build_state(
					&self,
					json: types::build_state::Json,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::BuildState,
					types::build_state::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"GenesisBuilder",
						"build_state",
						types::BuildState { json },
						[
							203u8, 233u8, 104u8, 116u8, 111u8, 131u8, 201u8, 235u8, 117u8, 116u8,
							140u8, 185u8, 93u8, 25u8, 155u8, 210u8, 56u8, 49u8, 23u8, 32u8, 253u8,
							92u8, 149u8, 241u8, 85u8, 245u8, 137u8, 45u8, 209u8, 189u8, 81u8, 2u8,
						],
					)
				}
				#[doc = " Returns a JSON blob representation of the built-in `RuntimeGenesisConfig` identified by"]
				#[doc = " `id`."]
				#[doc = ""]
				#[doc = " If `id` is `None` the function returns JSON blob representation of the default"]
				#[doc = " `RuntimeGenesisConfig` struct of the runtime. Implementation must provide default"]
				#[doc = " `RuntimeGenesisConfig`."]
				#[doc = ""]
				#[doc = " Otherwise function returns a JSON representation of the built-in, named"]
				#[doc = " `RuntimeGenesisConfig` preset identified by `id`, or `None` if such preset does not"]
				#[doc = " exists. Returned `Vec<u8>` contains bytes of JSON blob (patch) which comprises a list of"]
				#[doc = " (potentially nested) key-value pairs that are intended for customizing the default"]
				#[doc = " runtime genesis config. The patch shall be merged (rfc7386) with the JSON representation"]
				#[doc = " of the default `RuntimeGenesisConfig` to create a comprehensive genesis config that can"]
				#[doc = " be used in `build_state` method."]
				pub fn get_preset(
					&self,
					id: types::get_preset::Id,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::GetPreset,
					types::get_preset::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"GenesisBuilder",
						"get_preset",
						types::GetPreset { id },
						[
							43u8, 153u8, 23u8, 52u8, 113u8, 161u8, 227u8, 122u8, 169u8, 135u8,
							119u8, 8u8, 128u8, 33u8, 143u8, 235u8, 13u8, 173u8, 58u8, 121u8, 178u8,
							223u8, 66u8, 217u8, 22u8, 244u8, 168u8, 113u8, 202u8, 186u8, 241u8,
							124u8,
						],
					)
				}
				#[doc = " Returns a list of identifiers for available builtin `RuntimeGenesisConfig` presets."]
				#[doc = ""]
				#[doc = " The presets from the list can be queried with [`GenesisBuilder::get_preset`] method. If"]
				#[doc = " no named presets are provided by the runtime the list is empty."]
				pub fn preset_names(
					&self,
				) -> ::subxt::ext::subxt_core::runtime_api::payload::StaticPayload<
					types::PresetNames,
					types::preset_names::output::Output,
				> {
					::subxt::ext::subxt_core::runtime_api::payload::StaticPayload::new_static(
						"GenesisBuilder",
						"preset_names",
						types::PresetNames {},
						[
							150u8, 117u8, 54u8, 129u8, 221u8, 130u8, 186u8, 71u8, 13u8, 140u8,
							77u8, 180u8, 141u8, 37u8, 22u8, 219u8, 149u8, 218u8, 186u8, 206u8,
							80u8, 42u8, 165u8, 41u8, 99u8, 184u8, 73u8, 37u8, 125u8, 188u8, 167u8,
							122u8,
						],
					)
				}
			}
			pub mod types {
				use super::runtime_types;
				pub mod build_state {
					use super::runtime_types;
					pub type Json =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::result::Result<
							(),
							::subxt::ext::subxt_core::alloc::string::String,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct BuildState {
					pub json: build_state::Json,
				}
				pub mod get_preset {
					use super::runtime_types;
					pub type Id =
						::core::option::Option<::subxt::ext::subxt_core::alloc::string::String>;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct GetPreset {
					pub id: get_preset::Id,
				}
				pub mod preset_names {
					use super::runtime_types;
					pub mod output {
						use super::runtime_types;
						pub type Output = ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::alloc::string::String,
						>;
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct PresetNames {}
			}
		}
	}
	pub fn custom() -> CustomValuesApi {
		CustomValuesApi
	}
	pub struct CustomValuesApi;
	impl CustomValuesApi {}
	pub struct ConstantsApi;
	impl ConstantsApi {
		pub fn omni_account(&self) -> omni_account::constants::ConstantsApi {
			omni_account::constants::ConstantsApi
		}
		pub fn teebag(&self) -> teebag::constants::ConstantsApi {
			teebag::constants::ConstantsApi
		}
	}
	pub struct StorageApi;
	impl StorageApi {
		pub fn omni_account(&self) -> omni_account::storage::StorageApi {
			omni_account::storage::StorageApi
		}
		pub fn teebag(&self) -> teebag::storage::StorageApi {
			teebag::storage::StorageApi
		}
	}
	pub struct TransactionApi;
	impl TransactionApi {
		pub fn omni_account(&self) -> omni_account::calls::TransactionApi {
			omni_account::calls::TransactionApi
		}
		pub fn teebag(&self) -> teebag::calls::TransactionApi {
			teebag::calls::TransactionApi
		}
	}
	#[doc = r" check whether the metadata provided is aligned with this statically generated code."]
	pub fn is_codegen_valid_for(metadata: &::subxt::ext::subxt_core::Metadata) -> bool {
		let runtime_metadata_hash = metadata
			.hasher()
			.only_these_pallets(&PALLETS)
			.only_these_runtime_apis(&RUNTIME_APIS)
			.hash();
		runtime_metadata_hash
			== [
				181u8, 110u8, 158u8, 48u8, 59u8, 238u8, 71u8, 178u8, 177u8, 250u8, 161u8, 106u8,
				246u8, 14u8, 105u8, 58u8, 104u8, 55u8, 37u8, 103u8, 195u8, 184u8, 66u8, 89u8,
				103u8, 12u8, 63u8, 90u8, 134u8, 99u8, 4u8, 4u8,
			]
	}
	pub mod omni_account {
		use super::root_mod;
		use super::runtime_types;
		#[doc = "The `Error` enum of this pallet."]
		pub type Error = runtime_types::pallet_omni_account::pallet::Error;
		#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
		pub type Call = runtime_types::pallet_omni_account::pallet::Call;
		pub mod calls {
			use super::root_mod;
			use super::runtime_types;
			type DispatchError = runtime_types::sp_runtime::DispatchError;
			pub mod types {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct DispatchAsOmniAccount {
					pub member_account_hash: dispatch_as_omni_account::MemberAccountHash,
					pub call:
						::subxt::ext::subxt_core::alloc::boxed::Box<dispatch_as_omni_account::Call>,
					pub auth_type: dispatch_as_omni_account::AuthType,
				}
				pub mod dispatch_as_omni_account {
					use super::runtime_types;
					pub type MemberAccountHash = ::subxt::ext::subxt_core::utils::H256;
					pub type Call = runtime_types::paseo_parachain_runtime::RuntimeCall;
					pub type AuthType =
						runtime_types::core_primitives::omni_account::OmniAccountAuthType;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for DispatchAsOmniAccount {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "dispatch_as_omni_account";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct DispatchAsSigned {
					pub member_account_hash: dispatch_as_signed::MemberAccountHash,
					pub call: ::subxt::ext::subxt_core::alloc::boxed::Box<dispatch_as_signed::Call>,
					pub auth_type: dispatch_as_signed::AuthType,
				}
				pub mod dispatch_as_signed {
					use super::runtime_types;
					pub type MemberAccountHash = ::subxt::ext::subxt_core::utils::H256;
					pub type Call = runtime_types::paseo_parachain_runtime::RuntimeCall;
					pub type AuthType =
						runtime_types::core_primitives::omni_account::OmniAccountAuthType;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for DispatchAsSigned {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "dispatch_as_signed";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CreateAccountStore {
					pub identity: create_account_store::Identity,
				}
				pub mod create_account_store {
					use super::runtime_types;
					pub type Identity = runtime_types::core_primitives::identity::Identity;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for CreateAccountStore {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "create_account_store";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AddAccount {
					pub member_account: add_account::MemberAccount,
					pub permissions: add_account::Permissions,
				}
				pub mod add_account {
					use super::runtime_types;
					pub type MemberAccount =
						runtime_types::core_primitives::omni_account::MemberAccount;
					pub type Permissions = ::core::option::Option<
						::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::paseo_parachain_runtime::OmniAccountPermission,
						>,
					>;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for AddAccount {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "add_account";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct RemoveAccounts {
					pub member_account_hashes: remove_accounts::MemberAccountHashes,
				}
				pub mod remove_accounts {
					use super::runtime_types;
					pub type MemberAccountHashes = ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::utils::H256,
					>;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for RemoveAccounts {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "remove_accounts";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "make a member account public in the AccountStore"]
				#[doc = "we force `Identity` type to avoid misuse and additional check"]
				pub struct PublicizeAccount {
					pub member_account: publicize_account::MemberAccount,
				}
				pub mod publicize_account {
					use super::runtime_types;
					pub type MemberAccount = runtime_types::core_primitives::identity::Identity;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for PublicizeAccount {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "publicize_account";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct RequestIntent {
					pub intent: request_intent::Intent,
				}
				pub mod request_intent {
					use super::runtime_types;
					pub type Intent = runtime_types::core_primitives::intent::Intent;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for RequestIntent {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "request_intent";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "temporary extrinsic to upload the existing IDGraph from the worker onto chain"]
				pub struct UpdateAccountStoreByOne {
					pub who: update_account_store_by_one::Who,
					pub member_account: update_account_store_by_one::MemberAccount,
				}
				pub mod update_account_store_by_one {
					use super::runtime_types;
					pub type Who = runtime_types::core_primitives::identity::Identity;
					pub type MemberAccount =
						runtime_types::core_primitives::omni_account::MemberAccount;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for UpdateAccountStoreByOne {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "update_account_store_by_one";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct IntentExecuted {
					pub who: intent_executed::Who,
					pub intent: intent_executed::Intent,
					pub result: intent_executed::Result,
				}
				pub mod intent_executed {
					use super::runtime_types;
					pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
					pub type Intent = runtime_types::core_primitives::intent::Intent;
					pub type Result =
						runtime_types::pallet_omni_account::pallet::IntentExecutionResult;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for IntentExecuted {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "intent_executed";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct SetPermissions {
					pub member_account_hash: set_permissions::MemberAccountHash,
					pub permissions: set_permissions::Permissions,
				}
				pub mod set_permissions {
					use super::runtime_types;
					pub type MemberAccountHash = ::subxt::ext::subxt_core::utils::H256;
					pub type Permissions = ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::paseo_parachain_runtime::OmniAccountPermission,
					>;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for SetPermissions {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "set_permissions";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AuthTokenRequested {
					pub who: auth_token_requested::Who,
					pub expires_at: auth_token_requested::ExpiresAt,
				}
				pub mod auth_token_requested {
					use super::runtime_types;
					pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
					pub type ExpiresAt = ::core::primitive::u32;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for AuthTokenRequested {
					const PALLET: &'static str = "OmniAccount";
					const CALL: &'static str = "auth_token_requested";
				}
			}
			pub struct TransactionApi;
			impl TransactionApi {
				pub fn dispatch_as_omni_account(
					&self,
					member_account_hash: types::dispatch_as_omni_account::MemberAccountHash,
					call: types::dispatch_as_omni_account::Call,
					auth_type: types::dispatch_as_omni_account::AuthType,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::DispatchAsOmniAccount,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"dispatch_as_omni_account",
						types::DispatchAsOmniAccount {
							member_account_hash,
							call: ::subxt::ext::subxt_core::alloc::boxed::Box::new(call),
							auth_type,
						},
						[
							168u8, 40u8, 73u8, 240u8, 237u8, 200u8, 52u8, 79u8, 85u8, 185u8, 156u8,
							111u8, 31u8, 193u8, 95u8, 55u8, 219u8, 105u8, 95u8, 128u8, 105u8,
							243u8, 243u8, 116u8, 248u8, 90u8, 189u8, 161u8, 107u8, 233u8, 245u8,
							26u8,
						],
					)
				}
				pub fn dispatch_as_signed(
					&self,
					member_account_hash: types::dispatch_as_signed::MemberAccountHash,
					call: types::dispatch_as_signed::Call,
					auth_type: types::dispatch_as_signed::AuthType,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::DispatchAsSigned>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"dispatch_as_signed",
						types::DispatchAsSigned {
							member_account_hash,
							call: ::subxt::ext::subxt_core::alloc::boxed::Box::new(call),
							auth_type,
						},
						[
							126u8, 50u8, 166u8, 234u8, 160u8, 222u8, 75u8, 229u8, 48u8, 29u8,
							159u8, 3u8, 160u8, 197u8, 24u8, 25u8, 190u8, 231u8, 96u8, 99u8, 29u8,
							145u8, 229u8, 233u8, 225u8, 114u8, 97u8, 197u8, 48u8, 15u8, 90u8,
							150u8,
						],
					)
				}
				pub fn create_account_store(
					&self,
					identity: types::create_account_store::Identity,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::CreateAccountStore>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"create_account_store",
						types::CreateAccountStore { identity },
						[
							189u8, 220u8, 0u8, 169u8, 163u8, 244u8, 11u8, 79u8, 111u8, 11u8, 131u8,
							133u8, 39u8, 142u8, 173u8, 114u8, 90u8, 216u8, 38u8, 232u8, 40u8,
							199u8, 91u8, 90u8, 241u8, 238u8, 120u8, 190u8, 230u8, 49u8, 117u8,
							121u8,
						],
					)
				}
				pub fn add_account(
					&self,
					member_account: types::add_account::MemberAccount,
					permissions: types::add_account::Permissions,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::AddAccount> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"add_account",
						types::AddAccount { member_account, permissions },
						[
							144u8, 99u8, 144u8, 107u8, 29u8, 237u8, 161u8, 141u8, 0u8, 130u8,
							115u8, 4u8, 17u8, 227u8, 235u8, 154u8, 202u8, 33u8, 215u8, 124u8, 95u8,
							170u8, 229u8, 197u8, 180u8, 188u8, 209u8, 251u8, 175u8, 87u8, 16u8,
							165u8,
						],
					)
				}
				pub fn remove_accounts(
					&self,
					member_account_hashes: types::remove_accounts::MemberAccountHashes,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::RemoveAccounts>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"remove_accounts",
						types::RemoveAccounts { member_account_hashes },
						[
							147u8, 0u8, 0u8, 106u8, 168u8, 20u8, 59u8, 243u8, 124u8, 15u8, 253u8,
							155u8, 215u8, 122u8, 229u8, 18u8, 188u8, 148u8, 165u8, 74u8, 52u8,
							86u8, 251u8, 227u8, 144u8, 202u8, 214u8, 8u8, 176u8, 18u8, 148u8,
							231u8,
						],
					)
				}
				#[doc = "make a member account public in the AccountStore"]
				#[doc = "we force `Identity` type to avoid misuse and additional check"]
				pub fn publicize_account(
					&self,
					member_account: types::publicize_account::MemberAccount,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::PublicizeAccount>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"publicize_account",
						types::PublicizeAccount { member_account },
						[
							120u8, 63u8, 98u8, 113u8, 82u8, 230u8, 213u8, 41u8, 123u8, 73u8, 226u8,
							54u8, 90u8, 171u8, 8u8, 137u8, 11u8, 12u8, 175u8, 233u8, 56u8, 148u8,
							249u8, 244u8, 95u8, 235u8, 152u8, 56u8, 33u8, 66u8, 229u8, 125u8,
						],
					)
				}
				pub fn request_intent(
					&self,
					intent: types::request_intent::Intent,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::RequestIntent>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"request_intent",
						types::RequestIntent { intent },
						[
							162u8, 208u8, 121u8, 60u8, 38u8, 9u8, 209u8, 37u8, 37u8, 74u8, 16u8,
							84u8, 26u8, 62u8, 46u8, 229u8, 241u8, 54u8, 78u8, 183u8, 154u8, 192u8,
							118u8, 134u8, 131u8, 82u8, 225u8, 191u8, 206u8, 80u8, 53u8, 89u8,
						],
					)
				}
				#[doc = "temporary extrinsic to upload the existing IDGraph from the worker onto chain"]
				pub fn update_account_store_by_one(
					&self,
					who: types::update_account_store_by_one::Who,
					member_account: types::update_account_store_by_one::MemberAccount,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::UpdateAccountStoreByOne,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"update_account_store_by_one",
						types::UpdateAccountStoreByOne { who, member_account },
						[
							79u8, 87u8, 191u8, 146u8, 228u8, 87u8, 95u8, 147u8, 252u8, 128u8,
							190u8, 12u8, 35u8, 13u8, 12u8, 160u8, 199u8, 239u8, 43u8, 126u8, 157u8,
							41u8, 90u8, 225u8, 151u8, 207u8, 18u8, 59u8, 231u8, 35u8, 38u8, 63u8,
						],
					)
				}
				pub fn intent_executed(
					&self,
					who: types::intent_executed::Who,
					intent: types::intent_executed::Intent,
					result: types::intent_executed::Result,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::IntentExecuted>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"intent_executed",
						types::IntentExecuted { who, intent, result },
						[
							16u8, 225u8, 18u8, 95u8, 61u8, 185u8, 77u8, 181u8, 189u8, 48u8, 136u8,
							83u8, 29u8, 223u8, 147u8, 210u8, 119u8, 151u8, 105u8, 238u8, 214u8,
							243u8, 203u8, 237u8, 33u8, 156u8, 198u8, 100u8, 6u8, 204u8, 89u8,
							214u8,
						],
					)
				}
				pub fn set_permissions(
					&self,
					member_account_hash: types::set_permissions::MemberAccountHash,
					permissions: types::set_permissions::Permissions,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::SetPermissions>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"set_permissions",
						types::SetPermissions { member_account_hash, permissions },
						[
							250u8, 24u8, 165u8, 101u8, 77u8, 223u8, 96u8, 176u8, 103u8, 22u8, 51u8,
							151u8, 204u8, 164u8, 101u8, 60u8, 226u8, 161u8, 35u8, 119u8, 250u8,
							66u8, 68u8, 164u8, 188u8, 8u8, 213u8, 216u8, 0u8, 128u8, 130u8, 5u8,
						],
					)
				}
				pub fn auth_token_requested(
					&self,
					who: types::auth_token_requested::Who,
					expires_at: types::auth_token_requested::ExpiresAt,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::AuthTokenRequested>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"OmniAccount",
						"auth_token_requested",
						types::AuthTokenRequested { who, expires_at },
						[
							86u8, 8u8, 155u8, 175u8, 185u8, 11u8, 23u8, 58u8, 47u8, 215u8, 177u8,
							22u8, 223u8, 15u8, 107u8, 49u8, 159u8, 167u8, 221u8, 170u8, 224u8,
							189u8, 107u8, 220u8, 216u8, 29u8, 128u8, 177u8, 169u8, 178u8, 14u8,
							85u8,
						],
					)
				}
			}
		}
		#[doc = "The `Event` enum of this pallet"]
		pub type Event = runtime_types::pallet_omni_account::pallet::Event;
		pub mod events {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "An account store is created"]
			pub struct AccountStoreCreated {
				pub who: account_store_created::Who,
			}
			pub mod account_store_created {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AccountStoreCreated {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "AccountStoreCreated";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Some member account is added"]
			pub struct AccountAdded {
				pub who: account_added::Who,
				pub member_account_hash: account_added::MemberAccountHash,
			}
			pub mod account_added {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type MemberAccountHash = ::subxt::ext::subxt_core::utils::H256;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AccountAdded {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "AccountAdded";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Some member accounts are removed"]
			pub struct AccountRemoved {
				pub who: account_removed::Who,
				pub member_account_hashes: account_removed::MemberAccountHashes,
			}
			pub mod account_removed {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type MemberAccountHashes = ::subxt::ext::subxt_core::alloc::vec::Vec<
					::subxt::ext::subxt_core::utils::H256,
				>;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AccountRemoved {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "AccountRemoved";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Some member account is made public"]
			pub struct AccountMadePublic {
				pub who: account_made_public::Who,
				pub member_account_hash: account_made_public::MemberAccountHash,
			}
			pub mod account_made_public {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type MemberAccountHash = ::subxt::ext::subxt_core::utils::H256;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AccountMadePublic {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "AccountMadePublic";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "An account store is updated"]
			pub struct AccountStoreUpdated {
				pub who: account_store_updated::Who,
				pub account_store: account_store_updated::AccountStore,
			}
			pub mod account_store_updated {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type AccountStore = runtime_types::bounded_collections::bounded_vec::BoundedVec<
					runtime_types::core_primitives::omni_account::MemberAccount,
				>;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AccountStoreUpdated {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "AccountStoreUpdated";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Some call is dispatched as omni-account origin"]
			pub struct DispatchedAsOmniAccount {
				pub who: dispatched_as_omni_account::Who,
				pub auth_type: dispatched_as_omni_account::AuthType,
				pub result: dispatched_as_omni_account::Result,
			}
			pub mod dispatched_as_omni_account {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type AuthType =
					runtime_types::core_primitives::omni_account::OmniAccountAuthType;
				pub type Result =
					::core::result::Result<(), runtime_types::sp_runtime::DispatchError>;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for DispatchedAsOmniAccount {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "DispatchedAsOmniAccount";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Some call is dispatched as signed origin"]
			pub struct DispatchedAsSigned {
				pub who: dispatched_as_signed::Who,
				pub auth_type: dispatched_as_signed::AuthType,
				pub result: dispatched_as_signed::Result,
			}
			pub mod dispatched_as_signed {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type AuthType =
					runtime_types::core_primitives::omni_account::OmniAccountAuthType;
				pub type Result =
					::core::result::Result<(), runtime_types::sp_runtime::DispatchError>;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for DispatchedAsSigned {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "DispatchedAsSigned";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Intent is requested"]
			pub struct IntentRequested {
				pub who: intent_requested::Who,
				pub intent: intent_requested::Intent,
			}
			pub mod intent_requested {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type Intent = runtime_types::core_primitives::intent::Intent;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for IntentRequested {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "IntentRequested";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Intent is executed"]
			pub struct IntentExecuted {
				pub who: intent_executed::Who,
				pub intent: intent_executed::Intent,
				pub result: intent_executed::Result,
			}
			pub mod intent_executed {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type Intent = runtime_types::core_primitives::intent::Intent;
				pub type Result = runtime_types::pallet_omni_account::pallet::IntentExecutionResult;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for IntentExecuted {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "IntentExecuted";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "Member permission set"]
			pub struct AccountPermissionsSet {
				pub who: account_permissions_set::Who,
				pub member_account_hash: account_permissions_set::MemberAccountHash,
			}
			pub mod account_permissions_set {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type MemberAccountHash = ::subxt::ext::subxt_core::utils::H256;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AccountPermissionsSet {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "AccountPermissionsSet";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			#[doc = "An auth token is requested"]
			pub struct AuthTokenRequested {
				pub who: auth_token_requested::Who,
				pub expires_at: auth_token_requested::ExpiresAt,
			}
			pub mod auth_token_requested {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type ExpiresAt = ::core::primitive::u32;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AuthTokenRequested {
				const PALLET: &'static str = "OmniAccount";
				const EVENT: &'static str = "AuthTokenRequested";
			}
		}
		pub mod storage {
			use super::runtime_types;
			pub mod types {
				use super::runtime_types;
				pub mod account_store {
					use super::runtime_types;
					pub type AccountStore =
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::omni_account::MemberAccount,
						>;
					pub type Param0 = ::subxt::ext::subxt_core::utils::AccountId32;
				}
				pub mod member_account_hash {
					use super::runtime_types;
					pub type MemberAccountHash = ::subxt::ext::subxt_core::utils::AccountId32;
					pub type Param0 = ::subxt::ext::subxt_core::utils::H256;
				}
				pub mod member_account_permissions {
					use super::runtime_types;
					pub type MemberAccountPermissions =
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::paseo_parachain_runtime::OmniAccountPermission,
						>;
					pub type Param0 = ::subxt::ext::subxt_core::utils::H256;
				}
			}
			pub struct StorageApi;
			impl StorageApi {
				#[doc = " A map between OmniAccount and its MemberAccounts (a bounded vector of MemberAccount)"]
				pub fn account_store_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::account_store::AccountStore,
					(),
					(),
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"OmniAccount",
						"AccountStore",
						(),
						[
							223u8, 230u8, 236u8, 179u8, 45u8, 3u8, 198u8, 106u8, 181u8, 138u8,
							21u8, 141u8, 154u8, 118u8, 134u8, 130u8, 241u8, 197u8, 211u8, 9u8,
							152u8, 186u8, 209u8, 72u8, 161u8, 78u8, 166u8, 195u8, 59u8, 142u8,
							16u8, 125u8,
						],
					)
				}
				#[doc = " A map between OmniAccount and its MemberAccounts (a bounded vector of MemberAccount)"]
				pub fn account_store(
					&self,
					_0: impl ::core::borrow::Borrow<types::account_store::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::account_store::Param0,
					>,
					types::account_store::AccountStore,
					::subxt::ext::subxt_core::utils::Yes,
					(),
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"OmniAccount",
						"AccountStore",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							223u8, 230u8, 236u8, 179u8, 45u8, 3u8, 198u8, 106u8, 181u8, 138u8,
							21u8, 141u8, 154u8, 118u8, 134u8, 130u8, 241u8, 197u8, 211u8, 9u8,
							152u8, 186u8, 209u8, 72u8, 161u8, 78u8, 166u8, 195u8, 59u8, 142u8,
							16u8, 125u8,
						],
					)
				}
				#[doc = " A map between hash of MemberAccount and its belonging OmniAccount"]
				pub fn member_account_hash_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::member_account_hash::MemberAccountHash,
					(),
					(),
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"OmniAccount",
						"MemberAccountHash",
						(),
						[
							54u8, 90u8, 36u8, 240u8, 22u8, 77u8, 84u8, 27u8, 136u8, 13u8, 232u8,
							182u8, 187u8, 93u8, 127u8, 29u8, 148u8, 190u8, 91u8, 137u8, 211u8,
							106u8, 173u8, 194u8, 22u8, 75u8, 246u8, 3u8, 91u8, 200u8, 37u8, 135u8,
						],
					)
				}
				#[doc = " A map between hash of MemberAccount and its belonging OmniAccount"]
				pub fn member_account_hash(
					&self,
					_0: impl ::core::borrow::Borrow<types::member_account_hash::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::member_account_hash::Param0,
					>,
					types::member_account_hash::MemberAccountHash,
					::subxt::ext::subxt_core::utils::Yes,
					(),
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"OmniAccount",
						"MemberAccountHash",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							54u8, 90u8, 36u8, 240u8, 22u8, 77u8, 84u8, 27u8, 136u8, 13u8, 232u8,
							182u8, 187u8, 93u8, 127u8, 29u8, 148u8, 190u8, 91u8, 137u8, 211u8,
							106u8, 173u8, 194u8, 22u8, 75u8, 246u8, 3u8, 91u8, 200u8, 37u8, 135u8,
						],
					)
				}
				#[doc = " A map between hash of MemberAccount and its permissions"]
				pub fn member_account_permissions_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::member_account_permissions::MemberAccountPermissions,
					(),
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"OmniAccount",
						"MemberAccountPermissions",
						(),
						[
							119u8, 233u8, 243u8, 176u8, 52u8, 185u8, 224u8, 189u8, 195u8, 17u8,
							38u8, 215u8, 42u8, 28u8, 12u8, 138u8, 4u8, 60u8, 11u8, 241u8, 199u8,
							246u8, 170u8, 227u8, 151u8, 37u8, 166u8, 111u8, 240u8, 19u8, 156u8,
							50u8,
						],
					)
				}
				#[doc = " A map between hash of MemberAccount and its permissions"]
				pub fn member_account_permissions(
					&self,
					_0: impl ::core::borrow::Borrow<types::member_account_permissions::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::member_account_permissions::Param0,
					>,
					types::member_account_permissions::MemberAccountPermissions,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"OmniAccount",
						"MemberAccountPermissions",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							119u8, 233u8, 243u8, 176u8, 52u8, 185u8, 224u8, 189u8, 195u8, 17u8,
							38u8, 215u8, 42u8, 28u8, 12u8, 138u8, 4u8, 60u8, 11u8, 241u8, 199u8,
							246u8, 170u8, 227u8, 151u8, 37u8, 166u8, 111u8, 240u8, 19u8, 156u8,
							50u8,
						],
					)
				}
			}
		}
		pub mod constants {
			use super::runtime_types;
			pub struct ConstantsApi;
			impl ConstantsApi {
				#[doc = " The maximum number of accounts that an AccountGraph can have"]
				pub fn max_account_store_length(
					&self,
				) -> ::subxt::ext::subxt_core::constants::address::StaticAddress<
					::core::primitive::u32,
				> {
					::subxt::ext::subxt_core::constants::address::StaticAddress::new_static(
						"OmniAccount",
						"MaxAccountStoreLength",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " The maximum number of permissions that a member account can have"]
				pub fn max_permissions(
					&self,
				) -> ::subxt::ext::subxt_core::constants::address::StaticAddress<
					::core::primitive::u32,
				> {
					::subxt::ext::subxt_core::constants::address::StaticAddress::new_static(
						"OmniAccount",
						"MaxPermissions",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
			}
		}
	}
	pub mod teebag {
		use super::root_mod;
		use super::runtime_types;
		#[doc = "The `Error` enum of this pallet."]
		pub type Error = runtime_types::pallet_teebag::pallet::Error;
		#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
		pub type Call = runtime_types::pallet_teebag::pallet::Call;
		pub mod calls {
			use super::root_mod;
			use super::runtime_types;
			type DispatchError = runtime_types::sp_runtime::DispatchError;
			pub mod types {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Set the admin account"]
				#[doc = ""]
				#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
				pub struct SetAdmin {
					pub new_admin: set_admin::NewAdmin,
				}
				pub mod set_admin {
					use super::runtime_types;
					pub type NewAdmin = ::subxt::ext::subxt_core::utils::AccountId32;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for SetAdmin {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "set_admin";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Set the mode"]
				#[doc = ""]
				#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
				pub struct SetMode {
					pub new_mode: set_mode::NewMode,
				}
				pub mod set_mode {
					use super::runtime_types;
					pub type NewMode =
						runtime_types::core_primitives::teebag::types::OperationalMode;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for SetMode {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "set_mode";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ForceAddEnclave {
					pub who: force_add_enclave::Who,
					pub enclave: force_add_enclave::Enclave,
				}
				pub mod force_add_enclave {
					use super::runtime_types;
					pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
					pub type Enclave = runtime_types::core_primitives::teebag::types::Enclave;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for ForceAddEnclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "force_add_enclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ForceRemoveEnclave {
					pub who: force_remove_enclave::Who,
				}
				pub mod force_remove_enclave {
					use super::runtime_types;
					pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for ForceRemoveEnclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "force_remove_enclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ForceRemoveEnclaveByMrenclave {
					pub mrenclave: force_remove_enclave_by_mrenclave::Mrenclave,
				}
				pub mod force_remove_enclave_by_mrenclave {
					use super::runtime_types;
					pub type Mrenclave = [::core::primitive::u8; 32usize];
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for ForceRemoveEnclaveByMrenclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "force_remove_enclave_by_mrenclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ForceRemoveEnclaveByWorkerType {
					pub worker_type: force_remove_enclave_by_worker_type::WorkerType,
				}
				pub mod force_remove_enclave_by_worker_type {
					use super::runtime_types;
					pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for ForceRemoveEnclaveByWorkerType {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "force_remove_enclave_by_worker_type";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ForceAddAuthorizedEnclave {
					pub worker_type: force_add_authorized_enclave::WorkerType,
					pub mrenclave: force_add_authorized_enclave::Mrenclave,
				}
				pub mod force_add_authorized_enclave {
					use super::runtime_types;
					pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
					pub type Mrenclave = [::core::primitive::u8; 32usize];
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for ForceAddAuthorizedEnclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "force_add_authorized_enclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ForceRemoveAuthorizedEnclave {
					pub worker_type: force_remove_authorized_enclave::WorkerType,
					pub mrenclave: force_remove_authorized_enclave::Mrenclave,
				}
				pub mod force_remove_authorized_enclave {
					use super::runtime_types;
					pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
					pub type Mrenclave = [::core::primitive::u8; 32usize];
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for ForceRemoveAuthorizedEnclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "force_remove_authorized_enclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct RegisterEnclave {
					pub worker_type: register_enclave::WorkerType,
					pub worker_mode: register_enclave::WorkerMode,
					pub attestation: register_enclave::Attestation,
					pub worker_url: register_enclave::WorkerUrl,
					pub shielding_pubkey: register_enclave::ShieldingPubkey,
					pub vc_pubkey: register_enclave::VcPubkey,
					pub attestation_type: register_enclave::AttestationType,
				}
				pub mod register_enclave {
					use super::runtime_types;
					pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
					pub type WorkerMode = runtime_types::core_primitives::teebag::types::WorkerMode;
					pub type Attestation =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type WorkerUrl =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type ShieldingPubkey = ::core::option::Option<
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					>;
					pub type VcPubkey = ::core::option::Option<[::core::primitive::u8; 32usize]>;
					pub type AttestationType =
						runtime_types::core_primitives::teebag::types::AttestationType;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for RegisterEnclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "register_enclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct UnregisterEnclave;
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for UnregisterEnclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "unregister_enclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct RegisterQuotingEnclave {
					pub enclave_identity: register_quoting_enclave::EnclaveIdentity,
					pub signature: register_quoting_enclave::Signature,
					pub certificate_chain: register_quoting_enclave::CertificateChain,
				}
				pub mod register_quoting_enclave {
					use super::runtime_types;
					pub type EnclaveIdentity =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type Signature =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type CertificateChain =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for RegisterQuotingEnclave {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "register_quoting_enclave";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct RegisterTcbInfo {
					pub tcb_info: register_tcb_info::TcbInfo,
					pub signature: register_tcb_info::Signature,
					pub certificate_chain: register_tcb_info::CertificateChain,
				}
				pub mod register_tcb_info {
					use super::runtime_types;
					pub type TcbInfo =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type Signature =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
					pub type CertificateChain =
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for RegisterTcbInfo {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "register_tcb_info";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct PostOpaqueTask {
					pub request: post_opaque_task::Request,
				}
				pub mod post_opaque_task {
					use super::runtime_types;
					pub type Request = runtime_types::core_primitives::teebag::types::RsaRequest;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for PostOpaqueTask {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "post_opaque_task";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ParentchainBlockProcessed {
					pub block_hash: parentchain_block_processed::BlockHash,
					pub block_number: parentchain_block_processed::BlockNumber,
					pub task_merkle_root: parentchain_block_processed::TaskMerkleRoot,
				}
				pub mod parentchain_block_processed {
					use super::runtime_types;
					pub type BlockHash = ::subxt::ext::subxt_core::utils::H256;
					pub type BlockNumber = ::core::primitive::u32;
					pub type TaskMerkleRoot = ::subxt::ext::subxt_core::utils::H256;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for ParentchainBlockProcessed {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "parentchain_block_processed";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct SidechainBlockImported {
					pub shard: sidechain_block_imported::Shard,
					pub block_number: sidechain_block_imported::BlockNumber,
					pub next_finalization_candidate_block_number:
						sidechain_block_imported::NextFinalizationCandidateBlockNumber,
					pub block_header_hash: sidechain_block_imported::BlockHeaderHash,
				}
				pub mod sidechain_block_imported {
					use super::runtime_types;
					pub type Shard = ::subxt::ext::subxt_core::utils::H256;
					pub type BlockNumber = ::core::primitive::u64;
					pub type NextFinalizationCandidateBlockNumber = ::core::primitive::u64;
					pub type BlockHeaderHash = ::subxt::ext::subxt_core::utils::H256;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for SidechainBlockImported {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "sidechain_block_imported";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Batch {
					pub calls: batch::Calls,
				}
				pub mod batch {
					use super::runtime_types;
					pub type Calls = ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::paseo_parachain_runtime::RuntimeCall,
					>;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for Batch {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "batch";
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AddEnclaveIdentifier {
					pub worker_type: add_enclave_identifier::WorkerType,
					pub who: add_enclave_identifier::Who,
				}
				pub mod add_enclave_identifier {
					use super::runtime_types;
					pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
					pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				}
				impl ::subxt::ext::subxt_core::blocks::StaticExtrinsic for AddEnclaveIdentifier {
					const PALLET: &'static str = "Teebag";
					const CALL: &'static str = "add_enclave_identifier";
				}
			}
			pub struct TransactionApi;
			impl TransactionApi {
				#[doc = "Set the admin account"]
				#[doc = ""]
				#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
				pub fn set_admin(
					&self,
					new_admin: types::set_admin::NewAdmin,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::SetAdmin> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"set_admin",
						types::SetAdmin { new_admin },
						[
							132u8, 61u8, 150u8, 130u8, 228u8, 215u8, 89u8, 57u8, 48u8, 29u8, 240u8,
							171u8, 210u8, 98u8, 128u8, 76u8, 29u8, 48u8, 231u8, 95u8, 30u8, 199u8,
							52u8, 98u8, 126u8, 41u8, 166u8, 31u8, 98u8, 98u8, 165u8, 197u8,
						],
					)
				}
				#[doc = "Set the mode"]
				#[doc = ""]
				#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
				pub fn set_mode(
					&self,
					new_mode: types::set_mode::NewMode,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::SetMode> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"set_mode",
						types::SetMode { new_mode },
						[
							100u8, 203u8, 155u8, 40u8, 142u8, 58u8, 219u8, 228u8, 14u8, 26u8,
							235u8, 86u8, 95u8, 114u8, 54u8, 253u8, 142u8, 194u8, 167u8, 234u8,
							29u8, 105u8, 120u8, 132u8, 48u8, 95u8, 36u8, 73u8, 73u8, 14u8, 199u8,
							143u8,
						],
					)
				}
				pub fn force_add_enclave(
					&self,
					who: types::force_add_enclave::Who,
					enclave: types::force_add_enclave::Enclave,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::ForceAddEnclave>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"force_add_enclave",
						types::ForceAddEnclave { who, enclave },
						[
							138u8, 192u8, 193u8, 192u8, 215u8, 129u8, 201u8, 90u8, 149u8, 38u8,
							97u8, 19u8, 13u8, 233u8, 230u8, 228u8, 34u8, 24u8, 240u8, 232u8, 217u8,
							74u8, 223u8, 39u8, 41u8, 80u8, 234u8, 212u8, 15u8, 146u8, 230u8, 224u8,
						],
					)
				}
				pub fn force_remove_enclave(
					&self,
					who: types::force_remove_enclave::Who,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::ForceRemoveEnclave>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"force_remove_enclave",
						types::ForceRemoveEnclave { who },
						[
							205u8, 236u8, 214u8, 215u8, 71u8, 85u8, 108u8, 231u8, 123u8, 136u8,
							230u8, 195u8, 205u8, 214u8, 83u8, 58u8, 43u8, 184u8, 3u8, 72u8, 240u8,
							202u8, 197u8, 24u8, 64u8, 34u8, 178u8, 241u8, 5u8, 23u8, 13u8, 215u8,
						],
					)
				}
				pub fn force_remove_enclave_by_mrenclave(
					&self,
					mrenclave: types::force_remove_enclave_by_mrenclave::Mrenclave,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::ForceRemoveEnclaveByMrenclave,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"force_remove_enclave_by_mrenclave",
						types::ForceRemoveEnclaveByMrenclave { mrenclave },
						[
							212u8, 75u8, 231u8, 159u8, 178u8, 75u8, 195u8, 130u8, 104u8, 202u8,
							176u8, 191u8, 210u8, 115u8, 162u8, 197u8, 248u8, 14u8, 202u8, 141u8,
							188u8, 44u8, 209u8, 237u8, 11u8, 229u8, 147u8, 198u8, 201u8, 1u8,
							123u8, 229u8,
						],
					)
				}
				pub fn force_remove_enclave_by_worker_type(
					&self,
					worker_type: types::force_remove_enclave_by_worker_type::WorkerType,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::ForceRemoveEnclaveByWorkerType,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"force_remove_enclave_by_worker_type",
						types::ForceRemoveEnclaveByWorkerType { worker_type },
						[
							36u8, 231u8, 28u8, 185u8, 104u8, 134u8, 103u8, 52u8, 98u8, 222u8, 16u8,
							53u8, 233u8, 181u8, 251u8, 216u8, 167u8, 140u8, 59u8, 99u8, 138u8,
							167u8, 132u8, 108u8, 222u8, 227u8, 198u8, 52u8, 174u8, 172u8, 91u8,
							242u8,
						],
					)
				}
				pub fn force_add_authorized_enclave(
					&self,
					worker_type: types::force_add_authorized_enclave::WorkerType,
					mrenclave: types::force_add_authorized_enclave::Mrenclave,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::ForceAddAuthorizedEnclave,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"force_add_authorized_enclave",
						types::ForceAddAuthorizedEnclave { worker_type, mrenclave },
						[
							153u8, 35u8, 182u8, 98u8, 198u8, 198u8, 245u8, 12u8, 227u8, 84u8,
							167u8, 163u8, 144u8, 121u8, 70u8, 82u8, 240u8, 89u8, 244u8, 165u8,
							200u8, 123u8, 48u8, 59u8, 232u8, 228u8, 121u8, 59u8, 169u8, 114u8,
							192u8, 121u8,
						],
					)
				}
				pub fn force_remove_authorized_enclave(
					&self,
					worker_type: types::force_remove_authorized_enclave::WorkerType,
					mrenclave: types::force_remove_authorized_enclave::Mrenclave,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::ForceRemoveAuthorizedEnclave,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"force_remove_authorized_enclave",
						types::ForceRemoveAuthorizedEnclave { worker_type, mrenclave },
						[
							176u8, 14u8, 39u8, 190u8, 50u8, 34u8, 223u8, 127u8, 112u8, 73u8, 75u8,
							139u8, 109u8, 135u8, 6u8, 21u8, 53u8, 15u8, 241u8, 250u8, 157u8, 240u8,
							116u8, 125u8, 15u8, 37u8, 252u8, 105u8, 91u8, 138u8, 173u8, 13u8,
						],
					)
				}
				pub fn register_enclave(
					&self,
					worker_type: types::register_enclave::WorkerType,
					worker_mode: types::register_enclave::WorkerMode,
					attestation: types::register_enclave::Attestation,
					worker_url: types::register_enclave::WorkerUrl,
					shielding_pubkey: types::register_enclave::ShieldingPubkey,
					vc_pubkey: types::register_enclave::VcPubkey,
					attestation_type: types::register_enclave::AttestationType,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::RegisterEnclave>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"register_enclave",
						types::RegisterEnclave {
							worker_type,
							worker_mode,
							attestation,
							worker_url,
							shielding_pubkey,
							vc_pubkey,
							attestation_type,
						},
						[
							239u8, 194u8, 112u8, 43u8, 208u8, 184u8, 174u8, 11u8, 163u8, 251u8,
							79u8, 42u8, 92u8, 72u8, 40u8, 53u8, 180u8, 125u8, 91u8, 190u8, 176u8,
							142u8, 12u8, 47u8, 237u8, 159u8, 12u8, 137u8, 23u8, 99u8, 132u8, 251u8,
						],
					)
				}
				pub fn unregister_enclave(
					&self,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::UnregisterEnclave>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"unregister_enclave",
						types::UnregisterEnclave {},
						[
							144u8, 61u8, 144u8, 63u8, 118u8, 88u8, 169u8, 208u8, 166u8, 104u8,
							172u8, 200u8, 111u8, 24u8, 65u8, 252u8, 214u8, 227u8, 141u8, 43u8,
							222u8, 84u8, 212u8, 127u8, 238u8, 238u8, 130u8, 35u8, 84u8, 92u8, 35u8,
							109u8,
						],
					)
				}
				pub fn register_quoting_enclave(
					&self,
					enclave_identity: types::register_quoting_enclave::EnclaveIdentity,
					signature: types::register_quoting_enclave::Signature,
					certificate_chain: types::register_quoting_enclave::CertificateChain,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::RegisterQuotingEnclave,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"register_quoting_enclave",
						types::RegisterQuotingEnclave {
							enclave_identity,
							signature,
							certificate_chain,
						},
						[
							206u8, 211u8, 9u8, 42u8, 59u8, 202u8, 82u8, 59u8, 122u8, 158u8, 214u8,
							40u8, 136u8, 215u8, 2u8, 254u8, 202u8, 2u8, 138u8, 52u8, 29u8, 240u8,
							194u8, 231u8, 229u8, 239u8, 131u8, 96u8, 45u8, 188u8, 68u8, 210u8,
						],
					)
				}
				pub fn register_tcb_info(
					&self,
					tcb_info: types::register_tcb_info::TcbInfo,
					signature: types::register_tcb_info::Signature,
					certificate_chain: types::register_tcb_info::CertificateChain,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::RegisterTcbInfo>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"register_tcb_info",
						types::RegisterTcbInfo { tcb_info, signature, certificate_chain },
						[
							0u8, 93u8, 208u8, 241u8, 197u8, 169u8, 222u8, 191u8, 181u8, 83u8, 87u8,
							27u8, 197u8, 39u8, 135u8, 3u8, 24u8, 177u8, 58u8, 174u8, 15u8, 90u8,
							82u8, 76u8, 241u8, 214u8, 133u8, 3u8, 94u8, 247u8, 121u8, 4u8,
						],
					)
				}
				pub fn post_opaque_task(
					&self,
					request: types::post_opaque_task::Request,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::PostOpaqueTask>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"post_opaque_task",
						types::PostOpaqueTask { request },
						[
							138u8, 80u8, 6u8, 171u8, 205u8, 110u8, 165u8, 168u8, 68u8, 209u8,
							243u8, 168u8, 251u8, 152u8, 175u8, 142u8, 123u8, 89u8, 31u8, 106u8,
							34u8, 167u8, 162u8, 194u8, 48u8, 82u8, 158u8, 99u8, 147u8, 244u8,
							201u8, 153u8,
						],
					)
				}
				pub fn parentchain_block_processed(
					&self,
					block_hash: types::parentchain_block_processed::BlockHash,
					block_number: types::parentchain_block_processed::BlockNumber,
					task_merkle_root: types::parentchain_block_processed::TaskMerkleRoot,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::ParentchainBlockProcessed,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"parentchain_block_processed",
						types::ParentchainBlockProcessed {
							block_hash,
							block_number,
							task_merkle_root,
						},
						[
							59u8, 168u8, 96u8, 127u8, 108u8, 103u8, 182u8, 145u8, 111u8, 11u8,
							43u8, 20u8, 161u8, 77u8, 177u8, 214u8, 79u8, 245u8, 163u8, 40u8, 116u8,
							132u8, 41u8, 169u8, 91u8, 120u8, 52u8, 164u8, 141u8, 126u8, 18u8, 44u8,
						],
					)
				}
				pub fn sidechain_block_imported(
					&self,
					shard: types::sidechain_block_imported::Shard,
					block_number: types::sidechain_block_imported::BlockNumber,
					next_finalization_candidate_block_number : types :: sidechain_block_imported :: NextFinalizationCandidateBlockNumber,
					block_header_hash: types::sidechain_block_imported::BlockHeaderHash,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<
					types::SidechainBlockImported,
				> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"sidechain_block_imported",
						types::SidechainBlockImported {
							shard,
							block_number,
							next_finalization_candidate_block_number,
							block_header_hash,
						},
						[
							243u8, 211u8, 156u8, 220u8, 92u8, 96u8, 72u8, 241u8, 228u8, 51u8,
							178u8, 107u8, 224u8, 120u8, 225u8, 222u8, 223u8, 161u8, 225u8, 32u8,
							49u8, 0u8, 241u8, 241u8, 24u8, 60u8, 220u8, 125u8, 69u8, 59u8, 26u8,
							179u8,
						],
					)
				}
				pub fn batch(
					&self,
					calls: types::batch::Calls,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::Batch> {
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"batch",
						types::Batch { calls },
						[
							36u8, 189u8, 248u8, 135u8, 209u8, 177u8, 207u8, 46u8, 66u8, 249u8,
							183u8, 192u8, 155u8, 103u8, 210u8, 85u8, 183u8, 54u8, 240u8, 46u8,
							88u8, 140u8, 236u8, 168u8, 32u8, 145u8, 46u8, 229u8, 225u8, 6u8, 196u8,
							17u8,
						],
					)
				}
				pub fn add_enclave_identifier(
					&self,
					worker_type: types::add_enclave_identifier::WorkerType,
					who: types::add_enclave_identifier::Who,
				) -> ::subxt::ext::subxt_core::tx::payload::StaticPayload<types::AddEnclaveIdentifier>
				{
					::subxt::ext::subxt_core::tx::payload::StaticPayload::new_static(
						"Teebag",
						"add_enclave_identifier",
						types::AddEnclaveIdentifier { worker_type, who },
						[
							24u8, 196u8, 81u8, 213u8, 132u8, 39u8, 105u8, 125u8, 177u8, 178u8,
							109u8, 38u8, 238u8, 175u8, 45u8, 201u8, 196u8, 9u8, 210u8, 72u8, 210u8,
							255u8, 27u8, 98u8, 37u8, 67u8, 197u8, 142u8, 119u8, 98u8, 222u8, 125u8,
						],
					)
				}
			}
		}
		#[doc = "The `Event` enum of this pallet"]
		pub type Event = runtime_types::pallet_teebag::pallet::Event;
		pub mod events {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct ModeSet {
				pub new_mode: mode_set::NewMode,
			}
			pub mod mode_set {
				use super::runtime_types;
				pub type NewMode = runtime_types::core_primitives::teebag::types::OperationalMode;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for ModeSet {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "ModeSet";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct AdminSet {
				pub new_admin: admin_set::NewAdmin,
			}
			pub mod admin_set {
				use super::runtime_types;
				pub type NewAdmin =
					::core::option::Option<::subxt::ext::subxt_core::utils::AccountId32>;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for AdminSet {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "AdminSet";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct EnclaveAdded {
				pub who: enclave_added::Who,
				pub worker_type: enclave_added::WorkerType,
				pub url: enclave_added::Url,
			}
			pub mod enclave_added {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
				pub type Url = ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for EnclaveAdded {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "EnclaveAdded";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct EnclaveRemoved {
				pub who: enclave_removed::Who,
			}
			pub mod enclave_removed {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for EnclaveRemoved {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "EnclaveRemoved";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct EnclaveAuthorized {
				pub worker_type: enclave_authorized::WorkerType,
				pub mrenclave: enclave_authorized::Mrenclave,
			}
			pub mod enclave_authorized {
				use super::runtime_types;
				pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
				pub type Mrenclave = [::core::primitive::u8; 32usize];
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for EnclaveAuthorized {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "EnclaveAuthorized";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct EnclaveUnauthorized {
				pub worker_type: enclave_unauthorized::WorkerType,
				pub mrenclave: enclave_unauthorized::Mrenclave,
			}
			pub mod enclave_unauthorized {
				use super::runtime_types;
				pub type WorkerType = runtime_types::core_primitives::teebag::types::WorkerType;
				pub type Mrenclave = [::core::primitive::u8; 32usize];
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for EnclaveUnauthorized {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "EnclaveUnauthorized";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct OpaqueTaskPosted {
				pub request: opaque_task_posted::Request,
			}
			pub mod opaque_task_posted {
				use super::runtime_types;
				pub type Request = runtime_types::core_primitives::teebag::types::RsaRequest;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for OpaqueTaskPosted {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "OpaqueTaskPosted";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct ParentchainBlockProcessed {
				pub who: parentchain_block_processed::Who,
				pub block_number: parentchain_block_processed::BlockNumber,
				pub block_hash: parentchain_block_processed::BlockHash,
				pub task_merkle_root: parentchain_block_processed::TaskMerkleRoot,
			}
			pub mod parentchain_block_processed {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type BlockNumber = ::core::primitive::u32;
				pub type BlockHash = ::subxt::ext::subxt_core::utils::H256;
				pub type TaskMerkleRoot = ::subxt::ext::subxt_core::utils::H256;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for ParentchainBlockProcessed {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "ParentchainBlockProcessed";
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct SidechainBlockFinalized {
				pub who: sidechain_block_finalized::Who,
				pub sidechain_block_number: sidechain_block_finalized::SidechainBlockNumber,
			}
			pub mod sidechain_block_finalized {
				use super::runtime_types;
				pub type Who = ::subxt::ext::subxt_core::utils::AccountId32;
				pub type SidechainBlockNumber = ::core::primitive::u64;
			}
			impl ::subxt::ext::subxt_core::events::StaticEvent for SidechainBlockFinalized {
				const PALLET: &'static str = "Teebag";
				const EVENT: &'static str = "SidechainBlockFinalized";
			}
		}
		pub mod storage {
			use super::runtime_types;
			pub mod types {
				use super::runtime_types;
				pub mod admin {
					use super::runtime_types;
					pub type Admin = ::subxt::ext::subxt_core::utils::AccountId32;
				}
				pub mod mode {
					use super::runtime_types;
					pub type Mode = runtime_types::core_primitives::teebag::types::OperationalMode;
				}
				pub mod enclave_identifier {
					use super::runtime_types;
					pub type EnclaveIdentifier =
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>;
					pub type Param0 = runtime_types::core_primitives::teebag::types::WorkerType;
				}
				pub mod enclave_registry {
					use super::runtime_types;
					pub type EnclaveRegistry =
						runtime_types::core_primitives::teebag::types::Enclave;
					pub type Param0 = ::subxt::ext::subxt_core::utils::AccountId32;
				}
				pub mod allow_sgx_debug_mode {
					use super::runtime_types;
					pub type AllowSgxDebugMode = ::core::primitive::bool;
				}
				pub mod quoting_enclave_registry {
					use super::runtime_types;
					pub type QuotingEnclaveRegistry =
						runtime_types::core_primitives::teebag::quoting_enclave::QuotingEnclave;
				}
				pub mod tcb_info {
					use super::runtime_types;
					pub type TcbInfo = runtime_types::core_primitives::teebag::tcb::TcbInfoOnChain;
					pub type Param0 = [::core::primitive::u8; 6usize];
				}
				pub mod scheduled_enclave {
					use super::runtime_types;
					pub type ScheduledEnclave = [::core::primitive::u8; 32usize];
					pub type Param0 = runtime_types::core_primitives::teebag::types::WorkerType;
					pub type Param1 = ::core::primitive::u64;
				}
				pub mod authorized_enclave {
					use super::runtime_types;
					pub type AuthorizedEnclave =
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							[::core::primitive::u8; 32usize],
						>;
					pub type Param0 = runtime_types::core_primitives::teebag::types::WorkerType;
				}
				pub mod latest_sidechain_block_confirmation {
					use super::runtime_types;
					pub type LatestSidechainBlockConfirmation =
						runtime_types::core_primitives::teebag::types::SidechainBlockConfirmation;
					pub type Param0 = ::subxt::ext::subxt_core::utils::H256;
				}
				pub mod sidechain_block_finalization_candidate {
					use super::runtime_types;
					pub type SidechainBlockFinalizationCandidate = ::core::primitive::u64;
					pub type Param0 = ::subxt::ext::subxt_core::utils::H256;
				}
			}
			pub struct StorageApi;
			impl StorageApi {
				pub fn admin(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::admin::Admin,
					::subxt::ext::subxt_core::utils::Yes,
					(),
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"Admin",
						(),
						[
							39u8, 255u8, 131u8, 224u8, 101u8, 170u8, 98u8, 95u8, 222u8, 73u8, 76u8,
							217u8, 56u8, 31u8, 149u8, 77u8, 36u8, 174u8, 44u8, 90u8, 176u8, 230u8,
							208u8, 30u8, 78u8, 178u8, 232u8, 35u8, 226u8, 193u8, 229u8, 68u8,
						],
					)
				}
				pub fn mode(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::mode::Mode,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"Mode",
						(),
						[
							89u8, 62u8, 47u8, 6u8, 238u8, 184u8, 212u8, 126u8, 105u8, 142u8, 212u8,
							216u8, 238u8, 31u8, 33u8, 225u8, 188u8, 111u8, 106u8, 55u8, 248u8,
							57u8, 62u8, 241u8, 51u8, 211u8, 29u8, 176u8, 58u8, 3u8, 201u8, 215u8,
						],
					)
				}
				pub fn enclave_identifier_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::enclave_identifier::EnclaveIdentifier,
					(),
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"EnclaveIdentifier",
						(),
						[
							166u8, 73u8, 228u8, 71u8, 10u8, 245u8, 222u8, 93u8, 97u8, 254u8, 88u8,
							13u8, 14u8, 70u8, 145u8, 61u8, 175u8, 2u8, 150u8, 153u8, 123u8, 43u8,
							138u8, 63u8, 186u8, 161u8, 244u8, 48u8, 239u8, 161u8, 207u8, 56u8,
						],
					)
				}
				pub fn enclave_identifier(
					&self,
					_0: impl ::core::borrow::Borrow<types::enclave_identifier::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::enclave_identifier::Param0,
					>,
					types::enclave_identifier::EnclaveIdentifier,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"EnclaveIdentifier",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							166u8, 73u8, 228u8, 71u8, 10u8, 245u8, 222u8, 93u8, 97u8, 254u8, 88u8,
							13u8, 14u8, 70u8, 145u8, 61u8, 175u8, 2u8, 150u8, 153u8, 123u8, 43u8,
							138u8, 63u8, 186u8, 161u8, 244u8, 48u8, 239u8, 161u8, 207u8, 56u8,
						],
					)
				}
				pub fn enclave_registry_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::enclave_registry::EnclaveRegistry,
					(),
					(),
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"EnclaveRegistry",
						(),
						[
							47u8, 107u8, 125u8, 90u8, 209u8, 213u8, 41u8, 160u8, 113u8, 19u8,
							228u8, 10u8, 216u8, 183u8, 188u8, 179u8, 118u8, 30u8, 97u8, 173u8,
							74u8, 183u8, 232u8, 144u8, 129u8, 28u8, 165u8, 25u8, 72u8, 70u8, 38u8,
							93u8,
						],
					)
				}
				pub fn enclave_registry(
					&self,
					_0: impl ::core::borrow::Borrow<types::enclave_registry::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::enclave_registry::Param0,
					>,
					types::enclave_registry::EnclaveRegistry,
					::subxt::ext::subxt_core::utils::Yes,
					(),
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"EnclaveRegistry",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							47u8, 107u8, 125u8, 90u8, 209u8, 213u8, 41u8, 160u8, 113u8, 19u8,
							228u8, 10u8, 216u8, 183u8, 188u8, 179u8, 118u8, 30u8, 97u8, 173u8,
							74u8, 183u8, 232u8, 144u8, 129u8, 28u8, 165u8, 25u8, 72u8, 70u8, 38u8,
							93u8,
						],
					)
				}
				pub fn allow_sgx_debug_mode(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::allow_sgx_debug_mode::AllowSgxDebugMode,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"AllowSGXDebugMode",
						(),
						[
							49u8, 0u8, 218u8, 100u8, 163u8, 45u8, 3u8, 47u8, 85u8, 18u8, 49u8,
							84u8, 58u8, 216u8, 171u8, 117u8, 15u8, 225u8, 53u8, 39u8, 115u8, 82u8,
							178u8, 254u8, 33u8, 116u8, 117u8, 69u8, 173u8, 89u8, 115u8, 135u8,
						],
					)
				}
				pub fn quoting_enclave_registry(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::quoting_enclave_registry::QuotingEnclaveRegistry,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"QuotingEnclaveRegistry",
						(),
						[
							107u8, 148u8, 150u8, 73u8, 97u8, 188u8, 5u8, 118u8, 239u8, 97u8, 84u8,
							155u8, 27u8, 61u8, 133u8, 222u8, 82u8, 224u8, 87u8, 47u8, 207u8, 111u8,
							97u8, 202u8, 157u8, 166u8, 199u8, 242u8, 143u8, 190u8, 215u8, 222u8,
						],
					)
				}
				pub fn tcb_info_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::tcb_info::TcbInfo,
					(),
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"TcbInfo",
						(),
						[
							160u8, 198u8, 254u8, 216u8, 54u8, 201u8, 6u8, 127u8, 209u8, 253u8,
							152u8, 89u8, 10u8, 114u8, 99u8, 184u8, 44u8, 63u8, 46u8, 215u8, 84u8,
							173u8, 96u8, 39u8, 105u8, 124u8, 101u8, 207u8, 61u8, 103u8, 117u8,
							205u8,
						],
					)
				}
				pub fn tcb_info(
					&self,
					_0: impl ::core::borrow::Borrow<types::tcb_info::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::tcb_info::Param0,
					>,
					types::tcb_info::TcbInfo,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"TcbInfo",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							160u8, 198u8, 254u8, 216u8, 54u8, 201u8, 6u8, 127u8, 209u8, 253u8,
							152u8, 89u8, 10u8, 114u8, 99u8, 184u8, 44u8, 63u8, 46u8, 215u8, 84u8,
							173u8, 96u8, 39u8, 105u8, 124u8, 101u8, 207u8, 61u8, 103u8, 117u8,
							205u8,
						],
					)
				}
				pub fn scheduled_enclave_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::scheduled_enclave::ScheduledEnclave,
					(),
					(),
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"ScheduledEnclave",
						(),
						[
							71u8, 92u8, 229u8, 240u8, 77u8, 217u8, 78u8, 26u8, 247u8, 195u8, 246u8,
							100u8, 20u8, 248u8, 101u8, 37u8, 118u8, 18u8, 184u8, 10u8, 168u8,
							151u8, 38u8, 156u8, 223u8, 8u8, 186u8, 178u8, 142u8, 130u8, 179u8,
							35u8,
						],
					)
				}
				pub fn scheduled_enclave_iter1(
					&self,
					_0: impl ::core::borrow::Borrow<types::scheduled_enclave::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::scheduled_enclave::Param0,
					>,
					types::scheduled_enclave::ScheduledEnclave,
					(),
					(),
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"ScheduledEnclave",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							71u8, 92u8, 229u8, 240u8, 77u8, 217u8, 78u8, 26u8, 247u8, 195u8, 246u8,
							100u8, 20u8, 248u8, 101u8, 37u8, 118u8, 18u8, 184u8, 10u8, 168u8,
							151u8, 38u8, 156u8, 223u8, 8u8, 186u8, 178u8, 142u8, 130u8, 179u8,
							35u8,
						],
					)
				}
				pub fn scheduled_enclave(
					&self,
					_0: impl ::core::borrow::Borrow<types::scheduled_enclave::Param0>,
					_1: impl ::core::borrow::Borrow<types::scheduled_enclave::Param1>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(
						::subxt::ext::subxt_core::storage::address::StaticStorageKey<
							types::scheduled_enclave::Param0,
						>,
						::subxt::ext::subxt_core::storage::address::StaticStorageKey<
							types::scheduled_enclave::Param1,
						>,
					),
					types::scheduled_enclave::ScheduledEnclave,
					::subxt::ext::subxt_core::utils::Yes,
					(),
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"ScheduledEnclave",
						(
							::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
								_0.borrow(),
							),
							::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
								_1.borrow(),
							),
						),
						[
							71u8, 92u8, 229u8, 240u8, 77u8, 217u8, 78u8, 26u8, 247u8, 195u8, 246u8,
							100u8, 20u8, 248u8, 101u8, 37u8, 118u8, 18u8, 184u8, 10u8, 168u8,
							151u8, 38u8, 156u8, 223u8, 8u8, 186u8, 178u8, 142u8, 130u8, 179u8,
							35u8,
						],
					)
				}
				pub fn authorized_enclave_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::authorized_enclave::AuthorizedEnclave,
					(),
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"AuthorizedEnclave",
						(),
						[
							7u8, 207u8, 183u8, 15u8, 91u8, 77u8, 62u8, 126u8, 183u8, 118u8, 234u8,
							115u8, 81u8, 140u8, 199u8, 54u8, 68u8, 61u8, 11u8, 98u8, 93u8, 122u8,
							182u8, 112u8, 183u8, 129u8, 137u8, 152u8, 160u8, 138u8, 50u8, 245u8,
						],
					)
				}
				pub fn authorized_enclave(
					&self,
					_0: impl ::core::borrow::Borrow<types::authorized_enclave::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::authorized_enclave::Param0,
					>,
					types::authorized_enclave::AuthorizedEnclave,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"AuthorizedEnclave",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							7u8, 207u8, 183u8, 15u8, 91u8, 77u8, 62u8, 126u8, 183u8, 118u8, 234u8,
							115u8, 81u8, 140u8, 199u8, 54u8, 68u8, 61u8, 11u8, 98u8, 93u8, 122u8,
							182u8, 112u8, 183u8, 129u8, 137u8, 152u8, 160u8, 138u8, 50u8, 245u8,
						],
					)
				}
				pub fn latest_sidechain_block_confirmation_iter(
					&self,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					(),
					types::latest_sidechain_block_confirmation::LatestSidechainBlockConfirmation,
					(),
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"LatestSidechainBlockConfirmation",
						(),
						[
							117u8, 247u8, 10u8, 140u8, 249u8, 154u8, 144u8, 177u8, 67u8, 162u8,
							208u8, 198u8, 117u8, 91u8, 38u8, 80u8, 241u8, 228u8, 1u8, 52u8, 59u8,
							216u8, 87u8, 180u8, 5u8, 91u8, 225u8, 205u8, 159u8, 248u8, 148u8,
							179u8,
						],
					)
				}
				pub fn latest_sidechain_block_confirmation(
					&self,
					_0: impl ::core::borrow::Borrow<types::latest_sidechain_block_confirmation::Param0>,
				) -> ::subxt::ext::subxt_core::storage::address::StaticAddress<
					::subxt::ext::subxt_core::storage::address::StaticStorageKey<
						types::latest_sidechain_block_confirmation::Param0,
					>,
					types::latest_sidechain_block_confirmation::LatestSidechainBlockConfirmation,
					::subxt::ext::subxt_core::utils::Yes,
					::subxt::ext::subxt_core::utils::Yes,
					(),
				> {
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"LatestSidechainBlockConfirmation",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							117u8, 247u8, 10u8, 140u8, 249u8, 154u8, 144u8, 177u8, 67u8, 162u8,
							208u8, 198u8, 117u8, 91u8, 38u8, 80u8, 241u8, 228u8, 1u8, 52u8, 59u8,
							216u8, 87u8, 180u8, 5u8, 91u8, 225u8, 205u8, 159u8, 248u8, 148u8,
							179u8,
						],
					)
				}				pub fn sidechain_block_finalization_candidate_iter (& self ,) -> :: subxt :: ext :: subxt_core :: storage :: address :: StaticAddress :: < () , types :: sidechain_block_finalization_candidate :: SidechainBlockFinalizationCandidate , () , :: subxt :: ext :: subxt_core :: utils :: Yes , :: subxt :: ext :: subxt_core :: utils :: Yes >{
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"SidechainBlockFinalizationCandidate",
						(),
						[
							164u8, 88u8, 223u8, 231u8, 237u8, 68u8, 149u8, 249u8, 32u8, 89u8,
							210u8, 214u8, 177u8, 60u8, 138u8, 103u8, 42u8, 52u8, 68u8, 155u8, 52u8,
							161u8, 198u8, 141u8, 159u8, 235u8, 16u8, 249u8, 31u8, 206u8, 103u8,
							160u8,
						],
					)
				}				pub fn sidechain_block_finalization_candidate (& self , _0 : impl :: core :: borrow :: Borrow < types :: sidechain_block_finalization_candidate :: Param0 > ,) -> :: subxt :: ext :: subxt_core :: storage :: address :: StaticAddress :: < :: subxt :: ext :: subxt_core :: storage :: address :: StaticStorageKey < types :: sidechain_block_finalization_candidate :: Param0 > , types :: sidechain_block_finalization_candidate :: SidechainBlockFinalizationCandidate , :: subxt :: ext :: subxt_core :: utils :: Yes , :: subxt :: ext :: subxt_core :: utils :: Yes , () >{
					::subxt::ext::subxt_core::storage::address::StaticAddress::new_static(
						"Teebag",
						"SidechainBlockFinalizationCandidate",
						::subxt::ext::subxt_core::storage::address::StaticStorageKey::new(
							_0.borrow(),
						),
						[
							164u8, 88u8, 223u8, 231u8, 237u8, 68u8, 149u8, 249u8, 32u8, 89u8,
							210u8, 214u8, 177u8, 60u8, 138u8, 103u8, 42u8, 52u8, 68u8, 155u8, 52u8,
							161u8, 198u8, 141u8, 159u8, 235u8, 16u8, 249u8, 31u8, 206u8, 103u8,
							160u8,
						],
					)
				}
			}
		}
		pub mod constants {
			use super::runtime_types;
			pub struct ConstantsApi;
			impl ConstantsApi {
				#[doc = " Maximum number of enclave identifiers allowed to be registered for a given `worker_type`"]
				pub fn max_enclave_identifier(
					&self,
				) -> ::subxt::ext::subxt_core::constants::address::StaticAddress<
					::core::primitive::u32,
				> {
					::subxt::ext::subxt_core::constants::address::StaticAddress::new_static(
						"Teebag",
						"MaxEnclaveIdentifier",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
				#[doc = " Maximum number of authorized enclave for a given `worker_type`"]
				pub fn max_authorized_enclave(
					&self,
				) -> ::subxt::ext::subxt_core::constants::address::StaticAddress<
					::core::primitive::u32,
				> {
					::subxt::ext::subxt_core::constants::address::StaticAddress::new_static(
						"Teebag",
						"MaxAuthorizedEnclave",
						[
							98u8, 252u8, 116u8, 72u8, 26u8, 180u8, 225u8, 83u8, 200u8, 157u8,
							125u8, 151u8, 53u8, 76u8, 168u8, 26u8, 10u8, 9u8, 98u8, 68u8, 9u8,
							178u8, 197u8, 113u8, 31u8, 79u8, 200u8, 90u8, 203u8, 100u8, 41u8,
							145u8,
						],
					)
				}
			}
		}
	}
	pub mod runtime_types {
		use super::runtime_types;
		pub mod bounded_collections {
			use super::runtime_types;
			pub mod bounded_vec {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct BoundedVec<_0>(pub ::subxt::ext::subxt_core::alloc::vec::Vec<_0>);
			}
			pub mod weak_bounded_vec {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct WeakBoundedVec<_0>(pub ::subxt::ext::subxt_core::alloc::vec::Vec<_0>);
			}
		}
		pub mod core_primitives {
			use super::runtime_types;
			pub mod assertion {
				use super::runtime_types;
				pub mod achainable {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableAmount {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub amount:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableAmountHolding {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub amount:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub date: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub token: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableAmountToken {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub amount:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub token: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableAmounts {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub amount1:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub amount2:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableBasic {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableBetweenPercents {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub greater_than_or_equal_to:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub less_than_or_equal_to:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableClassOfYear {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableDate {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub date: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableDateInterval {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub start_date:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub end_date:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableDatePercent {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub token: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub date: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub percent:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableMirror {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub post_quantity: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AchainableParams {
						# [codec (index = 0)] AmountHolding (runtime_types :: core_primitives :: assertion :: achainable :: AchainableAmountHolding ,) , # [codec (index = 1)] AmountToken (runtime_types :: core_primitives :: assertion :: achainable :: AchainableAmountToken ,) , # [codec (index = 2)] Amount (runtime_types :: core_primitives :: assertion :: achainable :: AchainableAmount ,) , # [codec (index = 3)] Amounts (runtime_types :: core_primitives :: assertion :: achainable :: AchainableAmounts ,) , # [codec (index = 4)] Basic (runtime_types :: core_primitives :: assertion :: achainable :: AchainableBasic ,) , # [codec (index = 5)] BetweenPercents (runtime_types :: core_primitives :: assertion :: achainable :: AchainableBetweenPercents ,) , # [codec (index = 6)] ClassOfYear (runtime_types :: core_primitives :: assertion :: achainable :: AchainableClassOfYear ,) , # [codec (index = 7)] DateInterval (runtime_types :: core_primitives :: assertion :: achainable :: AchainableDateInterval ,) , # [codec (index = 8)] DatePercent (runtime_types :: core_primitives :: assertion :: achainable :: AchainableDatePercent ,) , # [codec (index = 9)] Date (runtime_types :: core_primitives :: assertion :: achainable :: AchainableDate ,) , # [codec (index = 10)] Token (runtime_types :: core_primitives :: assertion :: achainable :: AchainableToken ,) , # [codec (index = 11)] Mirror (runtime_types :: core_primitives :: assertion :: achainable :: AchainableMirror ,) , }
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AchainableToken {
						pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub chain: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::assertion::network::Web3Network,
						>,
						pub token: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
				}
				pub mod bnb_domain {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum BnbDigitDomainType {
						#[codec(index = 0)]
						Bnb999ClubMember,
						#[codec(index = 1)]
						Bnb10kClubMember,
					}
				}
				pub mod contest {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum ContestType {
						#[codec(index = 0)]
						Legend,
						#[codec(index = 1)]
						Popularity,
						#[codec(index = 2)]
						Participant,
					}
				}
				pub mod dynamic {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct DynamicParams {
						pub smart_contract_id: ::subxt::ext::subxt_core::utils::H160,
						pub smart_contract_params: ::core::option::Option<
							runtime_types::bounded_collections::bounded_vec::BoundedVec<
								::core::primitive::u8,
							>,
						>,
						pub return_log: ::core::primitive::bool,
					}
				}
				pub mod evm_amount_holding {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum EVMTokenType {
						#[codec(index = 0)]
						Ton,
						#[codec(index = 1)]
						Trx,
					}
				}
				pub mod generic_discord_role {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum GenericDiscordRoleType {
						#[codec(index = 0)]
						Contest(runtime_types::core_primitives::assertion::contest::ContestType),
						#[codec(index = 1)]
						SoraQuiz(runtime_types::core_primitives::assertion::soraquiz::SoraQuizType),
					}
				}
				pub mod network {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Web3Network {
						#[codec(index = 0)]
						Polkadot,
						#[codec(index = 1)]
						Kusama,
						#[codec(index = 2)]
						Litentry,
						#[codec(index = 3)]
						Litmus,
						#[codec(index = 4)]
						LitentryRococo,
						#[codec(index = 5)]
						Khala,
						#[codec(index = 6)]
						SubstrateTestnet,
						#[codec(index = 7)]
						Ethereum,
						#[codec(index = 8)]
						Bsc,
						#[codec(index = 9)]
						BitcoinP2tr,
						#[codec(index = 10)]
						BitcoinP2pkh,
						#[codec(index = 11)]
						BitcoinP2sh,
						#[codec(index = 12)]
						BitcoinP2wpkh,
						#[codec(index = 13)]
						BitcoinP2wsh,
						#[codec(index = 14)]
						Polygon,
						#[codec(index = 15)]
						Arbitrum,
						#[codec(index = 16)]
						Solana,
						#[codec(index = 17)]
						Combo,
					}
				}
				pub mod oneblock {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum OneBlockCourseType {
						#[codec(index = 0)]
						CourseCompletion,
						#[codec(index = 1)]
						CourseOutstanding,
						#[codec(index = 2)]
						CourseParticipation,
					}
				}
				pub mod platform_user {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum PlatformUserType {
						#[codec(index = 0)]
						KaratDao,
						#[codec(index = 1)]
						MagicCraftStaking,
						#[codec(index = 2)]
						DarenMarket,
					}
				}
				pub mod soraquiz {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum SoraQuizType {
						#[codec(index = 0)]
						Attendee,
						#[codec(index = 1)]
						Master,
					}
				}
				pub mod vip3 {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum VIP3MembershipCardLevel {
						#[codec(index = 0)]
						Gold,
						#[codec(index = 1)]
						Silver,
					}
				}
				pub mod web3_nft {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Web3NftType {
						#[codec(index = 0)]
						WeirdoGhostGang,
						#[codec(index = 1)]
						Club3Sbt,
						#[codec(index = 2)]
						MFan,
						#[codec(index = 3)]
						Mvp,
					}
				}
				pub mod web3_token {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Web3TokenType {
						#[codec(index = 0)]
						Bnb,
						#[codec(index = 1)]
						Eth,
						#[codec(index = 2)]
						SpaceId,
						#[codec(index = 3)]
						Lit,
						#[codec(index = 4)]
						Wbtc,
						#[codec(index = 5)]
						Usdc,
						#[codec(index = 6)]
						Usdt,
						#[codec(index = 7)]
						Crv,
						#[codec(index = 8)]
						Matic,
						#[codec(index = 9)]
						Dydx,
						#[codec(index = 10)]
						Amp,
						#[codec(index = 11)]
						Cvx,
						#[codec(index = 12)]
						Tusd,
						#[codec(index = 13)]
						Usdd,
						#[codec(index = 14)]
						Gusd,
						#[codec(index = 15)]
						Link,
						#[codec(index = 16)]
						Grt,
						#[codec(index = 17)]
						Comp,
						#[codec(index = 18)]
						People,
						#[codec(index = 19)]
						Gtc,
						#[codec(index = 20)]
						Ton,
						#[codec(index = 21)]
						Trx,
						#[codec(index = 22)]
						Nfp,
						#[codec(index = 23)]
						Sol,
						#[codec(index = 24)]
						Mcrt,
						#[codec(index = 25)]
						Btc,
						#[codec(index = 26)]
						Ada,
						#[codec(index = 27)]
						Doge,
						#[codec(index = 28)]
						Shib,
						#[codec(index = 29)]
						Uni,
						#[codec(index = 30)]
						Bch,
						#[codec(index = 31)]
						Etc,
						#[codec(index = 32)]
						Atom,
						#[codec(index = 33)]
						Dai,
						#[codec(index = 34)]
						Leo,
						#[codec(index = 35)]
						Fil,
						#[codec(index = 36)]
						Imx,
						#[codec(index = 37)]
						Cro,
						#[codec(index = 38)]
						Inj,
						#[codec(index = 39)]
						Bean,
						#[codec(index = 40)]
						An,
						#[codec(index = 41)]
						Tuna,
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Assertion {
					# [codec (index = 0)] A1 , # [codec (index = 1)] A2 (:: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > ,) , # [codec (index = 2)] A3 (:: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > , :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > , :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > ,) , # [codec (index = 3)] A4 (:: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > ,) , # [codec (index = 4)] A6 , # [codec (index = 5)] A7 (:: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > ,) , # [codec (index = 6)] A8 (runtime_types :: bounded_collections :: bounded_vec :: BoundedVec < runtime_types :: core_primitives :: assertion :: network :: Web3Network > ,) , # [codec (index = 7)] A10 (:: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > ,) , # [codec (index = 8)] A11 (:: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > ,) , # [codec (index = 9)] A13 (:: subxt :: ext :: subxt_core :: utils :: AccountId32 ,) , # [codec (index = 10)] A14 , # [codec (index = 11)] Achainable (runtime_types :: core_primitives :: assertion :: achainable :: AchainableParams ,) , # [codec (index = 12)] A20 , # [codec (index = 13)] OneBlock (runtime_types :: core_primitives :: assertion :: oneblock :: OneBlockCourseType ,) , # [codec (index = 14)] GenericDiscordRole (runtime_types :: core_primitives :: assertion :: generic_discord_role :: GenericDiscordRoleType ,) , # [codec (index = 16)] BnbDomainHolding , # [codec (index = 17)] BnbDigitDomainClub (runtime_types :: core_primitives :: assertion :: bnb_domain :: BnbDigitDomainType ,) , # [codec (index = 18)] VIP3MembershipCard (runtime_types :: core_primitives :: assertion :: vip3 :: VIP3MembershipCardLevel ,) , # [codec (index = 19)] WeirdoGhostGangHolder , # [codec (index = 20)] LITStaking , # [codec (index = 21)] EVMAmountHolding (runtime_types :: core_primitives :: assertion :: evm_amount_holding :: EVMTokenType ,) , # [codec (index = 22)] BRC20AmountHolder , # [codec (index = 23)] CryptoSummary , # [codec (index = 24)] TokenHoldingAmount (runtime_types :: core_primitives :: assertion :: web3_token :: Web3TokenType ,) , # [codec (index = 25)] PlatformUser (runtime_types :: core_primitives :: assertion :: platform_user :: PlatformUserType ,) , # [codec (index = 26)] NftHolder (runtime_types :: core_primitives :: assertion :: web3_nft :: Web3NftType ,) , # [codec (index = 27)] Dynamic (runtime_types :: core_primitives :: assertion :: dynamic :: DynamicParams ,) , # [codec (index = 28)] LinkedIdentities , }
			}
			pub mod error {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ErrorDetail {
					#[codec(index = 0)]
					ImportError,
					#[codec(index = 1)]
					UnauthorizedSigner,
					#[codec(index = 2)]
					StfError(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 3)]
					SendStfRequestFailed,
					#[codec(index = 4)]
					ParseError,
					#[codec(index = 5)]
					DataProviderError(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 6)]
					InvalidIdentity,
					#[codec(index = 7)]
					WrongWeb2Handle,
					#[codec(index = 8)]
					UnexpectedMessage,
					#[codec(index = 10)]
					VerifyWeb3SignatureFailed,
					#[codec(index = 11)]
					NoEligibleIdentity,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum IMPError {
					#[codec(index = 0)]
					LinkIdentityFailed(runtime_types::core_primitives::error::ErrorDetail),
					#[codec(index = 1)]
					DeactivateIdentityFailed(runtime_types::core_primitives::error::ErrorDetail),
					#[codec(index = 2)]
					ActivateIdentityFailed(runtime_types::core_primitives::error::ErrorDetail),
					#[codec(index = 3)]
					UnclassifiedError(runtime_types::core_primitives::error::ErrorDetail),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum VCMPError {
					#[codec(index = 0)]
					RequestVCFailed(
						runtime_types::core_primitives::assertion::Assertion,
						runtime_types::core_primitives::error::ErrorDetail,
					),
					#[codec(index = 1)]
					UnclassifiedError(runtime_types::core_primitives::error::ErrorDetail),
				}
			}
			pub mod identity {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Address20(pub [::core::primitive::u8; 20usize]);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Address32(pub [::core::primitive::u8; 32usize]);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Address33(pub [::core::primitive::u8; 33usize]);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Identity {
					#[codec(index = 0)]
					Twitter(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					#[codec(index = 1)]
					Discord(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					#[codec(index = 2)]
					Github(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					#[codec(index = 3)]
					Substrate(runtime_types::core_primitives::identity::Address32),
					#[codec(index = 4)]
					Evm(runtime_types::core_primitives::identity::Address20),
					#[codec(index = 5)]
					Bitcoin(runtime_types::core_primitives::identity::Address33),
					#[codec(index = 6)]
					Solana(runtime_types::core_primitives::identity::Address32),
					#[codec(index = 7)]
					Email(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					#[codec(index = 8)]
					Google(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					#[codec(index = 9)]
					Pumpx(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
				}
			}
			pub mod intent {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct CallEthereum {
					pub address: ::subxt::ext::subxt_core::utils::H160,
					pub input: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Intent {
					#[codec(index = 0)]
					TransferEthereum(runtime_types::core_primitives::intent::TransferEthereum),
					#[codec(index = 1)]
					CallEthereum(runtime_types::core_primitives::intent::CallEthereum),
					#[codec(index = 2)]
					SystemRemark(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 3)]
					TransferNative(runtime_types::core_primitives::intent::TransferNative),
					#[codec(index = 4)]
					TransferSolana(runtime_types::core_primitives::intent::TransferSolana),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TransferEthereum {
					pub to: ::subxt::ext::subxt_core::utils::H160,
					pub value: [::core::primitive::u8; 32usize],
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TransferNative {
					pub to: ::subxt::ext::subxt_core::utils::AccountId32,
					pub value: ::core::primitive::u128,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TransferSolana {
					pub to: [::core::primitive::u8; 32usize],
					pub value: ::core::primitive::u64,
				}
			}
			pub mod omni_account {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum MemberAccount {
					#[codec(index = 0)]
					Public(runtime_types::core_primitives::identity::Identity),
					#[codec(index = 1)]
					Private(
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						::subxt::ext::subxt_core::utils::H256,
					),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum OmniAccountAuthType {
					#[codec(index = 0)]
					Web3,
					#[codec(index = 1)]
					Email,
					#[codec(index = 2)]
					OAuth2,
					#[codec(index = 3)]
					AuthToken,
				}
			}
			pub mod teebag {
				use super::runtime_types;
				pub mod quoting_enclave {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct QuotingEnclave {
						pub issue_date: ::core::primitive::u64,
						pub next_update: ::core::primitive::u64,
						pub miscselect: [::core::primitive::u8; 4usize],
						pub miscselect_mask: [::core::primitive::u8; 4usize],
						pub attributes: [::core::primitive::u8; 16usize],
						pub attributes_mask: [::core::primitive::u8; 16usize],
						pub mrsigner: [::core::primitive::u8; 32usize],
						pub isvprodid: ::core::primitive::u16,
						pub tcb: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::core_primitives::teebag::tcb::QeTcb,
						>,
					}
				}
				pub mod tcb {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct QeTcb {
						pub isvsvn: ::core::primitive::u16,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct TcbInfoOnChain {
						pub issue_date: ::core::primitive::u64,
						pub next_update: ::core::primitive::u64,
						pub tcb_levels: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::core_primitives::teebag::tcb::TcbVersionStatus,
						>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct TcbVersionStatus {
						pub cpusvn: [::core::primitive::u8; 16usize],
						pub pcesvn: ::core::primitive::u16,
					}
				}
				pub mod types {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AttestationType {
						#[codec(index = 0)]
						Ignore,
						#[codec(index = 1)]
						Ias,
						#[codec(index = 2)]
						Dcap(runtime_types::core_primitives::teebag::types::DcapProvider),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum DcapProvider {
						#[codec(index = 0)]
						Intel,
						#[codec(index = 1)]
						MAA,
						#[codec(index = 2)]
						Local,
						#[codec(index = 3)]
						Integritee,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Enclave {
						pub worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						pub worker_mode: runtime_types::core_primitives::teebag::types::WorkerMode,
						pub mrenclave: [::core::primitive::u8; 32usize],
						pub last_seen_timestamp: ::core::primitive::u64,
						pub url: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						pub shielding_pubkey: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
						pub vc_pubkey: ::core::option::Option<[::core::primitive::u8; 32usize]>,
						pub sgx_build_mode:
							runtime_types::core_primitives::teebag::types::SgxBuildMode,
						pub attestation_type:
							runtime_types::core_primitives::teebag::types::AttestationType,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum OperationalMode {
						#[codec(index = 0)]
						Production,
						#[codec(index = 1)]
						Development,
						#[codec(index = 2)]
						Maintenance,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct RsaRequest {
						pub shard: ::subxt::ext::subxt_core::utils::H256,
						pub payload:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum SgxBuildMode {
						#[codec(index = 0)]
						Production,
						#[codec(index = 1)]
						Debug,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct SidechainBlockConfirmation {
						pub block_number: ::core::primitive::u64,
						pub block_header_hash: ::subxt::ext::subxt_core::utils::H256,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WorkerMode {
						#[codec(index = 0)]
						OffChainWorker,
						#[codec(index = 1)]
						Sidechain,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WorkerType {
						#[codec(index = 0)]
						Identity,
						#[codec(index = 1)]
						BitAcross,
						#[codec(index = 2)]
						OmniExecutor,
					}
				}
			}
		}
		pub mod cumulus_pallet_parachain_system {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					# [codec (index = 0)] # [doc = "Set the current validation data."] # [doc = ""] # [doc = "This should be invoked exactly once per block. It will panic at the finalization"] # [doc = "phase if the call was not invoked."] # [doc = ""] # [doc = "The dispatch origin for this call must be `Inherent`"] # [doc = ""] # [doc = "As a side effect, this function upgrades the current validation function"] # [doc = "if the appropriate time has come."] set_validation_data { data : runtime_types :: cumulus_primitives_parachain_inherent :: ParachainInherentData , } , # [codec (index = 1)] sudo_send_upward_message { message : :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > , } , # [codec (index = 2)] # [doc = "Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied"] # [doc = "later."] # [doc = ""] # [doc = "The `check_version` parameter sets a boolean flag for whether or not the runtime's spec"] # [doc = "version and name should be verified on upgrade. Since the authorization only has a hash,"] # [doc = "it cannot actually perform the verification."] # [doc = ""] # [doc = "This call requires Root origin."] authorize_upgrade { code_hash : :: subxt :: ext :: subxt_core :: utils :: H256 , check_version : :: core :: primitive :: bool , } , # [codec (index = 3)] # [doc = "Provide the preimage (runtime binary) `code` for an upgrade that has been authorized."] # [doc = ""] # [doc = "If the authorization required a version check, this call will ensure the spec name"] # [doc = "remains unchanged and that the spec version has increased."] # [doc = ""] # [doc = "Note that this function will not apply the new `code`, but only attempt to schedule the"] # [doc = "upgrade with the Relay Chain."] # [doc = ""] # [doc = "All origins are allowed."] enact_authorized_upgrade { code : :: subxt :: ext :: subxt_core :: alloc :: vec :: Vec < :: core :: primitive :: u8 > , } , }
			}
		}
		pub mod cumulus_pallet_xcm {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Origin {
					#[codec(index = 0)]
					Relay,
					#[codec(index = 1)]
					SiblingParachain(runtime_types::polkadot_parachain_primitives::primitives::Id),
				}
			}
		}
		pub mod cumulus_pallet_xcmp_queue {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 1)]
					#[doc = "Suspends all XCM executions for the XCMP queue, regardless of the sender's origin."]
					#[doc = ""]
					#[doc = "- `origin`: Must pass `ControllerOrigin`."]
					suspend_xcm_execution,
					#[codec(index = 2)]
					#[doc = "Resumes all XCM executions for the XCMP queue."]
					#[doc = ""]
					#[doc = "Note that this function doesn't change the status of the in/out bound channels."]
					#[doc = ""]
					#[doc = "- `origin`: Must pass `ControllerOrigin`."]
					resume_xcm_execution,
					#[codec(index = 3)]
					#[doc = "Overwrites the number of pages which must be in the queue for the other side to be"]
					#[doc = "told to suspend their sending."]
					#[doc = ""]
					#[doc = "- `origin`: Must pass `Root`."]
					#[doc = "- `new`: Desired value for `QueueConfigData.suspend_value`"]
					update_suspend_threshold { new: ::core::primitive::u32 },
					#[codec(index = 4)]
					#[doc = "Overwrites the number of pages which must be in the queue after which we drop any"]
					#[doc = "further messages from the channel."]
					#[doc = ""]
					#[doc = "- `origin`: Must pass `Root`."]
					#[doc = "- `new`: Desired value for `QueueConfigData.drop_threshold`"]
					update_drop_threshold { new: ::core::primitive::u32 },
					#[codec(index = 5)]
					#[doc = "Overwrites the number of pages which the queue must be reduced to before it signals"]
					#[doc = "that message sending may recommence after it has been suspended."]
					#[doc = ""]
					#[doc = "- `origin`: Must pass `Root`."]
					#[doc = "- `new`: Desired value for `QueueConfigData.resume_threshold`"]
					update_resume_threshold { new: ::core::primitive::u32 },
				}
			}
		}
		pub mod cumulus_primitives_core {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum AggregateMessageOrigin {
				#[codec(index = 0)]
				Here,
				#[codec(index = 1)]
				Parent,
				#[codec(index = 2)]
				Sibling(runtime_types::polkadot_parachain_primitives::primitives::Id),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct CollationInfo {
				pub upward_messages: ::subxt::ext::subxt_core::alloc::vec::Vec<
					::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				>,
				pub horizontal_messages: ::subxt::ext::subxt_core::alloc::vec::Vec<
					runtime_types::polkadot_core_primitives::OutboundHrmpMessage<
						runtime_types::polkadot_parachain_primitives::primitives::Id,
					>,
				>,
				pub new_validation_code: ::core::option::Option<
					runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
				>,
				pub processed_downward_messages: ::core::primitive::u32,
				pub hrmp_watermark: ::core::primitive::u32,
				pub head_data: runtime_types::polkadot_parachain_primitives::primitives::HeadData,
			}
		}
		pub mod cumulus_primitives_parachain_inherent {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct ParachainInherentData {
				pub validation_data:
					runtime_types::polkadot_primitives::v7::PersistedValidationData<
						::subxt::ext::subxt_core::utils::H256,
						::core::primitive::u32,
					>,
				pub relay_chain_state: runtime_types::sp_trie::storage_proof::StorageProof,
				pub downward_messages: ::subxt::ext::subxt_core::alloc::vec::Vec<
					runtime_types::polkadot_core_primitives::InboundDownwardMessage<
						::core::primitive::u32,
					>,
				>,
				pub horizontal_messages: ::subxt::ext::subxt_core::utils::KeyedVec<
					runtime_types::polkadot_parachain_primitives::primitives::Id,
					::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_core_primitives::InboundHrmpMessage<
							::core::primitive::u32,
						>,
					>,
				>,
			}
		}
		pub mod ethbloom {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct Bloom(pub [::core::primitive::u8; 256usize]);
		}
		pub mod ethereum {
			use super::runtime_types;
			pub mod block {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Block<_0> {
					pub header: runtime_types::ethereum::header::Header,
					pub transactions: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
					pub ommers: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::ethereum::header::Header,
					>,
				}
			}
			pub mod header {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Header {
					pub parent_hash: ::subxt::ext::subxt_core::utils::H256,
					pub ommers_hash: ::subxt::ext::subxt_core::utils::H256,
					pub beneficiary: ::subxt::ext::subxt_core::utils::H160,
					pub state_root: ::subxt::ext::subxt_core::utils::H256,
					pub transactions_root: ::subxt::ext::subxt_core::utils::H256,
					pub receipts_root: ::subxt::ext::subxt_core::utils::H256,
					pub logs_bloom: runtime_types::ethbloom::Bloom,
					pub difficulty: runtime_types::primitive_types::U256,
					pub number: runtime_types::primitive_types::U256,
					pub gas_limit: runtime_types::primitive_types::U256,
					pub gas_used: runtime_types::primitive_types::U256,
					pub timestamp: ::core::primitive::u64,
					pub extra_data:
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					pub mix_hash: ::subxt::ext::subxt_core::utils::H256,
					pub nonce: runtime_types::ethereum_types::hash::H64,
				}
			}
			pub mod log {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Log {
					pub address: ::subxt::ext::subxt_core::utils::H160,
					pub topics: ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::utils::H256,
					>,
					pub data: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				}
			}
			pub mod receipt {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct EIP658ReceiptData {
					pub status_code: ::core::primitive::u8,
					pub used_gas: runtime_types::primitive_types::U256,
					pub logs_bloom: runtime_types::ethbloom::Bloom,
					pub logs: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::ethereum::log::Log,
					>,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ReceiptV3 {
					#[codec(index = 0)]
					Legacy(runtime_types::ethereum::receipt::EIP658ReceiptData),
					#[codec(index = 1)]
					EIP2930(runtime_types::ethereum::receipt::EIP658ReceiptData),
					#[codec(index = 2)]
					EIP1559(runtime_types::ethereum::receipt::EIP658ReceiptData),
				}
			}
			pub mod transaction {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AccessListItem {
					pub address: ::subxt::ext::subxt_core::utils::H160,
					pub storage_keys: ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::utils::H256,
					>,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct EIP1559Transaction {
					pub chain_id: ::core::primitive::u64,
					pub nonce: runtime_types::primitive_types::U256,
					pub max_priority_fee_per_gas: runtime_types::primitive_types::U256,
					pub max_fee_per_gas: runtime_types::primitive_types::U256,
					pub gas_limit: runtime_types::primitive_types::U256,
					pub action: runtime_types::ethereum::transaction::TransactionAction,
					pub value: runtime_types::primitive_types::U256,
					pub input: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					pub access_list: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::ethereum::transaction::AccessListItem,
					>,
					pub odd_y_parity: ::core::primitive::bool,
					pub r: ::subxt::ext::subxt_core::utils::H256,
					pub s: ::subxt::ext::subxt_core::utils::H256,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct EIP2930Transaction {
					pub chain_id: ::core::primitive::u64,
					pub nonce: runtime_types::primitive_types::U256,
					pub gas_price: runtime_types::primitive_types::U256,
					pub gas_limit: runtime_types::primitive_types::U256,
					pub action: runtime_types::ethereum::transaction::TransactionAction,
					pub value: runtime_types::primitive_types::U256,
					pub input: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					pub access_list: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::ethereum::transaction::AccessListItem,
					>,
					pub odd_y_parity: ::core::primitive::bool,
					pub r: ::subxt::ext::subxt_core::utils::H256,
					pub s: ::subxt::ext::subxt_core::utils::H256,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct LegacyTransaction {
					pub nonce: runtime_types::primitive_types::U256,
					pub gas_price: runtime_types::primitive_types::U256,
					pub gas_limit: runtime_types::primitive_types::U256,
					pub action: runtime_types::ethereum::transaction::TransactionAction,
					pub value: runtime_types::primitive_types::U256,
					pub input: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					pub signature: runtime_types::ethereum::transaction::TransactionSignature,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum TransactionAction {
					#[codec(index = 0)]
					Call(::subxt::ext::subxt_core::utils::H160),
					#[codec(index = 1)]
					Create,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TransactionRecoveryId(pub ::core::primitive::u64);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct TransactionSignature {
					pub v: runtime_types::ethereum::transaction::TransactionRecoveryId,
					pub r: ::subxt::ext::subxt_core::utils::H256,
					pub s: ::subxt::ext::subxt_core::utils::H256,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum TransactionV2 {
					#[codec(index = 0)]
					Legacy(runtime_types::ethereum::transaction::LegacyTransaction),
					#[codec(index = 1)]
					EIP2930(runtime_types::ethereum::transaction::EIP2930Transaction),
					#[codec(index = 2)]
					EIP1559(runtime_types::ethereum::transaction::EIP1559Transaction),
				}
			}
		}
		pub mod ethereum_types {
			use super::runtime_types;
			pub mod hash {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct H64(pub [::core::primitive::u8; 8usize]);
			}
		}
		pub mod evm {
			use super::runtime_types;
			pub mod backend {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Basic {
					pub balance: runtime_types::primitive_types::U256,
					pub nonce: runtime_types::primitive_types::U256,
				}
			}
		}
		pub mod evm_core {
			use super::runtime_types;
			pub mod error {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ExitError {
					#[codec(index = 0)]
					StackUnderflow,
					#[codec(index = 1)]
					StackOverflow,
					#[codec(index = 2)]
					InvalidJump,
					#[codec(index = 3)]
					InvalidRange,
					#[codec(index = 4)]
					DesignatedInvalid,
					#[codec(index = 5)]
					CallTooDeep,
					#[codec(index = 6)]
					CreateCollision,
					#[codec(index = 7)]
					CreateContractLimit,
					#[codec(index = 15)]
					InvalidCode(runtime_types::evm_core::opcode::Opcode),
					#[codec(index = 8)]
					OutOfOffset,
					#[codec(index = 9)]
					OutOfGas,
					#[codec(index = 10)]
					OutOfFund,
					#[codec(index = 11)]
					PCUnderflow,
					#[codec(index = 12)]
					CreateEmpty,
					#[codec(index = 13)]
					Other(::subxt::ext::subxt_core::alloc::string::String),
					#[codec(index = 14)]
					MaxNonce,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ExitFatal {
					#[codec(index = 0)]
					NotSupported,
					#[codec(index = 1)]
					UnhandledInterrupt,
					#[codec(index = 2)]
					CallErrorAsFatal(runtime_types::evm_core::error::ExitError),
					#[codec(index = 3)]
					Other(::subxt::ext::subxt_core::alloc::string::String),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ExitReason {
					#[codec(index = 0)]
					Succeed(runtime_types::evm_core::error::ExitSucceed),
					#[codec(index = 1)]
					Error(runtime_types::evm_core::error::ExitError),
					#[codec(index = 2)]
					Revert(runtime_types::evm_core::error::ExitRevert),
					#[codec(index = 3)]
					Fatal(runtime_types::evm_core::error::ExitFatal),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ExitRevert {
					#[codec(index = 0)]
					Reverted,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ExitSucceed {
					#[codec(index = 0)]
					Stopped,
					#[codec(index = 1)]
					Returned,
					#[codec(index = 2)]
					Suicided,
				}
			}
			pub mod opcode {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Opcode(pub ::core::primitive::u8);
			}
		}
		pub mod fp_evm {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct ExecutionInfoV2<_0> {
				pub exit_reason: runtime_types::evm_core::error::ExitReason,
				pub value: _0,
				pub used_gas: runtime_types::fp_evm::UsedGas,
				pub weight_info: ::core::option::Option<runtime_types::fp_evm::WeightInfo>,
				pub logs:
					::subxt::ext::subxt_core::alloc::vec::Vec<runtime_types::ethereum::log::Log>,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct UsedGas {
				pub standard: runtime_types::primitive_types::U256,
				pub effective: runtime_types::primitive_types::U256,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct WeightInfo {
				pub ref_time_limit: ::core::option::Option<::core::primitive::u64>,
				pub proof_size_limit: ::core::option::Option<::core::primitive::u64>,
				pub ref_time_usage: ::core::option::Option<::core::primitive::u64>,
				pub proof_size_usage: ::core::option::Option<::core::primitive::u64>,
			}
		}
		pub mod fp_rpc {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct TransactionStatus {
				pub transaction_hash: ::subxt::ext::subxt_core::utils::H256,
				pub transaction_index: ::core::primitive::u32,
				pub from: ::subxt::ext::subxt_core::utils::H160,
				pub to: ::core::option::Option<::subxt::ext::subxt_core::utils::H160>,
				pub contract_address: ::core::option::Option<::subxt::ext::subxt_core::utils::H160>,
				pub logs:
					::subxt::ext::subxt_core::alloc::vec::Vec<runtime_types::ethereum::log::Log>,
				pub logs_bloom: runtime_types::ethbloom::Bloom,
			}
		}
		pub mod fp_self_contained {
			use super::runtime_types;
			pub mod unchecked_extrinsic {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct UncheckedExtrinsic<_0, _1, _2, _3>(
					pub ::subxt::ext::subxt_core::utils::UncheckedExtrinsic<_0, _1, _2, _3>,
				);
			}
		}
		pub mod frame_support {
			use super::runtime_types;
			pub mod dispatch {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum DispatchClass {
					#[codec(index = 0)]
					Normal,
					#[codec(index = 1)]
					Operational,
					#[codec(index = 2)]
					Mandatory,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum RawOrigin<_0> {
					#[codec(index = 0)]
					Root,
					#[codec(index = 1)]
					Signed(_0),
					#[codec(index = 2)]
					None,
				}
			}
			pub mod traits {
				use super::runtime_types;
				pub mod preimages {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Bounded<_0, _1> {
						#[codec(index = 0)]
						Legacy {
							hash: ::subxt::ext::subxt_core::utils::H256,
						},
						#[codec(index = 1)]
						Inline(
							runtime_types::bounded_collections::bounded_vec::BoundedVec<
								::core::primitive::u8,
							>,
						),
						#[codec(index = 2)]
						Lookup {
							hash: ::subxt::ext::subxt_core::utils::H256,
							len: ::core::primitive::u32,
						},
						__Ignore(::core::marker::PhantomData<(_0, _1)>),
					}
				}
				pub mod schedule {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum DispatchTime<_0> {
						#[codec(index = 0)]
						At(_0),
						#[codec(index = 1)]
						After(_0),
					}
				}
				pub mod tokens {
					use super::runtime_types;
					pub mod fungible {
						use super::runtime_types;
						pub mod union_of {
							use super::runtime_types;
							#[derive(
								:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
								:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
								:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
								:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
								Debug,
							)]
							# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
							#[codec(dumb_trait_bound)]
							#[decode_as_type(
								crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
							)]
							#[encode_as_type(
								crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
							)]
							pub enum NativeOrWithId<_0> {
								#[codec(index = 0)]
								Native,
								#[codec(index = 1)]
								WithId(_0),
							}
						}
					}
				}
			}
		}
		pub mod frame_system {
			use super::runtime_types;
			pub mod extensions {
				use super::runtime_types;
				pub mod check_genesis {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct CheckGenesis;
				}
				pub mod check_mortality {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct CheckMortality(pub runtime_types::sp_runtime::generic::era::Era);
				}
				pub mod check_non_zero_sender {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct CheckNonZeroSender;
				}
				pub mod check_nonce {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct CheckNonce(#[codec(compact)] pub ::core::primitive::u32);
				}
				pub mod check_spec_version {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct CheckSpecVersion;
				}
				pub mod check_tx_version {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct CheckTxVersion;
				}
				pub mod check_weight {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct CheckWeight;
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Make some on-chain remark."]
					#[doc = ""]
					#[doc = "Can be executed by every `origin`."]
					remark {
						remark: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 1)]
					#[doc = "Set the number of pages in the WebAssembly environment's heap."]
					set_heap_pages { pages: ::core::primitive::u64 },
					#[codec(index = 2)]
					#[doc = "Set the new runtime code."]
					set_code {
						code: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 3)]
					#[doc = "Set the new runtime code without doing any checks of the given `code`."]
					#[doc = ""]
					#[doc = "Note that runtime upgrades will not run if this is called with a not-increasing spec"]
					#[doc = "version!"]
					set_code_without_checks {
						code: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 4)]
					#[doc = "Set some items of storage."]
					set_storage {
						items: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						)>,
					},
					#[codec(index = 5)]
					#[doc = "Kill some items from storage."]
					kill_storage {
						keys: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					},
					#[codec(index = 6)]
					#[doc = "Kill all storage items with a key that starts with the given prefix."]
					#[doc = ""]
					#[doc = "**NOTE:** We rely on the Root origin to provide us the number of subkeys under"]
					#[doc = "the prefix we are removing to accurately calculate the weight of this function."]
					kill_prefix {
						prefix: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						subkeys: ::core::primitive::u32,
					},
					#[codec(index = 7)]
					#[doc = "Make some on-chain remark and emit event."]
					remark_with_event {
						remark: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 9)]
					#[doc = "Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied"]
					#[doc = "later."]
					#[doc = ""]
					#[doc = "This call requires Root origin."]
					authorize_upgrade { code_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 10)]
					#[doc = "Authorize an upgrade to a given `code_hash` for the runtime. The runtime can be supplied"]
					#[doc = "later."]
					#[doc = ""]
					#[doc = "WARNING: This authorizes an upgrade that will take place without any safety checks, for"]
					#[doc = "example that the spec name remains the same and that the version number increases. Not"]
					#[doc = "recommended for normal use. Use `authorize_upgrade` instead."]
					#[doc = ""]
					#[doc = "This call requires Root origin."]
					authorize_upgrade_without_checks {
						code_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 11)]
					#[doc = "Provide the preimage (runtime binary) `code` for an upgrade that has been authorized."]
					#[doc = ""]
					#[doc = "If the authorization required a version check, this call will ensure the spec name"]
					#[doc = "remains unchanged and that the spec version has increased."]
					#[doc = ""]
					#[doc = "Depending on the runtime's `OnSetCode` configuration, this function may directly apply"]
					#[doc = "the new `code` in the same block or attempt to schedule the upgrade."]
					#[doc = ""]
					#[doc = "All origins are allowed."]
					apply_authorized_upgrade {
						code: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
				}
			}
		}
		pub mod moonbeam_rpc_primitives_txpool {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct TxPoolResponse {
				pub ready: ::subxt::ext::subxt_core::alloc::vec::Vec<
					runtime_types::ethereum::transaction::TransactionV2,
				>,
				pub future: ::subxt::ext::subxt_core::alloc::vec::Vec<
					runtime_types::ethereum::transaction::TransactionV2,
				>,
			}
		}
		pub mod pallet_account_fix {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					inc_consumers {
						who: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
					},
					#[codec(index = 1)]
					#[doc = "add some balance of an existing account"]
					add_balance {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						add_free: ::core::primitive::u128,
						#[codec(compact)]
						add_reserved: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					burn {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						amount: ::core::primitive::u128,
					},
				}
			}
		}
		pub mod pallet_aiusd_convertor {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					mint_aiusd {
						source_asset_id: ::core::primitive::u128,
						aiusd_amount: ::core::primitive::u128,
					},
					#[codec(index = 1)]
					burn_aiusd {
						source_asset_id: ::core::primitive::u128,
						aiusd_amount: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					#[doc = "Enable a specific type of token available for switching"]
					enable_token {
						source_asset_id: ::core::primitive::u128,
						decimal_ratio: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					#[doc = "disable a specific type of token available for switching"]
					disable_token { source_asset_id: ::core::primitive::u128 },
				}
			}
		}
		pub mod pallet_asset_manager {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct AssetMetadata<_0> {
					pub name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					pub symbol: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					pub decimals: ::core::primitive::u8,
					pub minimal_balance: _0,
					pub is_frozen: ::core::primitive::bool,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Register new asset with the asset manager"]
					#[doc = "TODO::Reserve native token multilocation through GenesisBuild/RuntimeUpgrade"]
					#[doc = "TODO::Add Multilocation filter for register"]
					register_foreign_asset_type {
						asset_type:
							runtime_types::paseo_parachain_runtime::asset_config::ForeignAssetType,
						metadata: runtime_types::pallet_asset_manager::pallet::AssetMetadata<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 1)]
					update_foreign_asset_metadata {
						asset_id: ::core::primitive::u128,
						metadata: runtime_types::pallet_asset_manager::pallet::AssetMetadata<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 2)]
					#[doc = "Change the amount of units we are charging per execution second"]
					#[doc = "for a given ForeignAssetType"]
					#[doc = "0 means not support"]
					set_asset_units_per_second {
						asset_id: ::core::primitive::u128,
						units_per_second: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					#[doc = "Add the xcm type mapping for a existing assetId, other assetType still exists if any."]
					#[doc = "TODO: Change add_asset_type with internal function wrapper"]
					add_asset_type {
						asset_id: ::core::primitive::u128,
						new_asset_type:
							runtime_types::paseo_parachain_runtime::asset_config::ForeignAssetType,
					},
					#[codec(index = 4)]
					#[doc = "We do not allow the destroy of asset id so far; So at least one AssetTpye should be"]
					#[doc = "assigned to existing AssetId Both asset_type and potential new_default_asset_type must"]
					#[doc = "be an existing relation with asset_id"]
					#[doc = "TODO: Change remove_asset_type with internal function wrapper"]
					remove_asset_type {
						asset_type:
							runtime_types::paseo_parachain_runtime::asset_config::ForeignAssetType,
						new_default_asset_type: ::core::option::Option<
							runtime_types::paseo_parachain_runtime::asset_config::ForeignAssetType,
						>,
					},
				}
			}
		}
		pub mod pallet_assets {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Issue a new class of fungible assets from a public origin."]
					#[doc = ""]
					#[doc = "This new asset class has no assets initially and its owner is the origin."]
					#[doc = ""]
					#[doc = "The origin must conform to the configured `CreateOrigin` and have sufficient funds free."]
					#[doc = ""]
					#[doc = "Funds of sender are reserved by `AssetDeposit`."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `id`: The identifier of the new asset. This must not be currently in use to identify"]
					#[doc = "an existing asset. If [`NextAssetId`] is set, then this must be equal to it."]
					#[doc = "- `admin`: The admin of this class of assets. The admin is the initial address of each"]
					#[doc = "member of the asset class's admin team."]
					#[doc = "- `min_balance`: The minimum balance of this new asset that any single account must"]
					#[doc = "have. If an account's balance is reduced below this, then it collapses to zero."]
					#[doc = ""]
					#[doc = "Emits `Created` event when successful."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					create {
						#[codec(compact)]
						id: ::core::primitive::u128,
						admin: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						min_balance: ::core::primitive::u128,
					},
					#[codec(index = 1)]
					#[doc = "Issue a new class of fungible assets from a privileged origin."]
					#[doc = ""]
					#[doc = "This new asset class has no assets initially."]
					#[doc = ""]
					#[doc = "The origin must conform to `ForceOrigin`."]
					#[doc = ""]
					#[doc = "Unlike `create`, no funds are reserved."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the new asset. This must not be currently in use to identify"]
					#[doc = "an existing asset. If [`NextAssetId`] is set, then this must be equal to it."]
					#[doc = "- `owner`: The owner of this class of assets. The owner has full superuser permissions"]
					#[doc = "over this asset, but may later change and configure the permissions using"]
					#[doc = "`transfer_ownership` and `set_team`."]
					#[doc = "- `min_balance`: The minimum balance of this new asset that any single account must"]
					#[doc = "have. If an account's balance is reduced below this, then it collapses to zero."]
					#[doc = ""]
					#[doc = "Emits `ForceCreated` event when successful."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					force_create {
						#[codec(compact)]
						id: ::core::primitive::u128,
						owner: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						is_sufficient: ::core::primitive::bool,
						#[codec(compact)]
						min_balance: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					#[doc = "Start the process of destroying a fungible asset class."]
					#[doc = ""]
					#[doc = "`start_destroy` is the first in a series of extrinsics that should be called, to allow"]
					#[doc = "destruction of an asset class."]
					#[doc = ""]
					#[doc = "The origin must conform to `ForceOrigin` or must be `Signed` by the asset's `owner`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be destroyed. This must identify an existing"]
					#[doc = "  asset."]
					#[doc = ""]
					#[doc = "The asset class must be frozen before calling `start_destroy`."]
					start_destroy {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					#[doc = "Destroy all accounts associated with a given asset."]
					#[doc = ""]
					#[doc = "`destroy_accounts` should only be called after `start_destroy` has been called, and the"]
					#[doc = "asset is in a `Destroying` state."]
					#[doc = ""]
					#[doc = "Due to weight restrictions, this function may need to be called multiple times to fully"]
					#[doc = "destroy all accounts. It will destroy `RemoveItemsLimit` accounts at a time."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be destroyed. This must identify an existing"]
					#[doc = "  asset."]
					#[doc = ""]
					#[doc = "Each call emits the `Event::DestroyedAccounts` event."]
					destroy_accounts {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					#[doc = "Destroy all approvals associated with a given asset up to the max (T::RemoveItemsLimit)."]
					#[doc = ""]
					#[doc = "`destroy_approvals` should only be called after `start_destroy` has been called, and the"]
					#[doc = "asset is in a `Destroying` state."]
					#[doc = ""]
					#[doc = "Due to weight restrictions, this function may need to be called multiple times to fully"]
					#[doc = "destroy all approvals. It will destroy `RemoveItemsLimit` approvals at a time."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be destroyed. This must identify an existing"]
					#[doc = "  asset."]
					#[doc = ""]
					#[doc = "Each call emits the `Event::DestroyedApprovals` event."]
					destroy_approvals {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 5)]
					#[doc = "Complete destroying asset and unreserve currency."]
					#[doc = ""]
					#[doc = "`finish_destroy` should only be called after `start_destroy` has been called, and the"]
					#[doc = "asset is in a `Destroying` state. All accounts or approvals should be destroyed before"]
					#[doc = "hand."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be destroyed. This must identify an existing"]
					#[doc = "  asset."]
					#[doc = ""]
					#[doc = "Each successful call emits the `Event::Destroyed` event."]
					finish_destroy {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 6)]
					#[doc = "Mint assets of a particular class."]
					#[doc = ""]
					#[doc = "The origin must be Signed and the sender must be the Issuer of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to have some amount minted."]
					#[doc = "- `beneficiary`: The account to be credited with the minted assets."]
					#[doc = "- `amount`: The amount of the asset to be minted."]
					#[doc = ""]
					#[doc = "Emits `Issued` event when successful."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					#[doc = "Modes: Pre-existing balance of `beneficiary`; Account pre-existence of `beneficiary`."]
					mint {
						#[codec(compact)]
						id: ::core::primitive::u128,
						beneficiary: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
					},
					#[codec(index = 7)]
					#[doc = "Reduce the balance of `who` by as much as possible up to `amount` assets of `id`."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Manager of the asset `id`."]
					#[doc = ""]
					#[doc = "Bails with `NoAccount` if the `who` is already dead."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to have some amount burned."]
					#[doc = "- `who`: The account to be debited from."]
					#[doc = "- `amount`: The maximum amount by which `who`'s balance should be reduced."]
					#[doc = ""]
					#[doc = "Emits `Burned` with the actual amount burned. If this takes the balance to below the"]
					#[doc = "minimum for the asset, then the amount burned is increased to take it to zero."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					#[doc = "Modes: Post-existence of `who`; Pre & post Zombie-status of `who`."]
					burn {
						#[codec(compact)]
						id: ::core::primitive::u128,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
					},
					#[codec(index = 8)]
					#[doc = "Move some assets from the sender account to another."]
					#[doc = ""]
					#[doc = "Origin must be Signed."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to have some amount transferred."]
					#[doc = "- `target`: The account to be credited."]
					#[doc = "- `amount`: The amount by which the sender's balance of assets should be reduced and"]
					#[doc = "`target`'s balance increased. The amount actually transferred may be slightly greater in"]
					#[doc = "the case that the transfer would otherwise take the sender balance above zero but below"]
					#[doc = "the minimum balance. Must be greater than zero."]
					#[doc = ""]
					#[doc = "Emits `Transferred` with the actual amount transferred. If this takes the source balance"]
					#[doc = "to below the minimum for the asset, then the amount transferred is increased to take it"]
					#[doc = "to zero."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					#[doc = "Modes: Pre-existence of `target`; Post-existence of sender; Account pre-existence of"]
					#[doc = "`target`."]
					transfer {
						#[codec(compact)]
						id: ::core::primitive::u128,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
					},
					#[codec(index = 9)]
					#[doc = "Move some assets from the sender account to another, keeping the sender account alive."]
					#[doc = ""]
					#[doc = "Origin must be Signed."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to have some amount transferred."]
					#[doc = "- `target`: The account to be credited."]
					#[doc = "- `amount`: The amount by which the sender's balance of assets should be reduced and"]
					#[doc = "`target`'s balance increased. The amount actually transferred may be slightly greater in"]
					#[doc = "the case that the transfer would otherwise take the sender balance above zero but below"]
					#[doc = "the minimum balance. Must be greater than zero."]
					#[doc = ""]
					#[doc = "Emits `Transferred` with the actual amount transferred. If this takes the source balance"]
					#[doc = "to below the minimum for the asset, then the amount transferred is increased to take it"]
					#[doc = "to zero."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					#[doc = "Modes: Pre-existence of `target`; Post-existence of sender; Account pre-existence of"]
					#[doc = "`target`."]
					transfer_keep_alive {
						#[codec(compact)]
						id: ::core::primitive::u128,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
					},
					#[codec(index = 10)]
					#[doc = "Move some assets from one account to another."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Admin of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to have some amount transferred."]
					#[doc = "- `source`: The account to be debited."]
					#[doc = "- `dest`: The account to be credited."]
					#[doc = "- `amount`: The amount by which the `source`'s balance of assets should be reduced and"]
					#[doc = "`dest`'s balance increased. The amount actually transferred may be slightly greater in"]
					#[doc = "the case that the transfer would otherwise take the `source` balance above zero but"]
					#[doc = "below the minimum balance. Must be greater than zero."]
					#[doc = ""]
					#[doc = "Emits `Transferred` with the actual amount transferred. If this takes the source balance"]
					#[doc = "to below the minimum for the asset, then the amount transferred is increased to take it"]
					#[doc = "to zero."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					#[doc = "Modes: Pre-existence of `dest`; Post-existence of `source`; Account pre-existence of"]
					#[doc = "`dest`."]
					force_transfer {
						#[codec(compact)]
						id: ::core::primitive::u128,
						source: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
					},
					#[codec(index = 11)]
					#[doc = "Disallow further unprivileged transfers of an asset `id` from an account `who`. `who`"]
					#[doc = "must already exist as an entry in `Account`s of the asset. If you want to freeze an"]
					#[doc = "account that does not have an entry, use `touch_other` first."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Freezer of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be frozen."]
					#[doc = "- `who`: The account to be frozen."]
					#[doc = ""]
					#[doc = "Emits `Frozen`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					freeze {
						#[codec(compact)]
						id: ::core::primitive::u128,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 12)]
					#[doc = "Allow unprivileged transfers to and from an account again."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Admin of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be frozen."]
					#[doc = "- `who`: The account to be unfrozen."]
					#[doc = ""]
					#[doc = "Emits `Thawed`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					thaw {
						#[codec(compact)]
						id: ::core::primitive::u128,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 13)]
					#[doc = "Disallow further unprivileged transfers for the asset class."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Freezer of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be frozen."]
					#[doc = ""]
					#[doc = "Emits `Frozen`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					freeze_asset {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 14)]
					#[doc = "Allow unprivileged transfers for the asset again."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Admin of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be thawed."]
					#[doc = ""]
					#[doc = "Emits `Thawed`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					thaw_asset {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 15)]
					#[doc = "Change the Owner of an asset."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Owner of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset."]
					#[doc = "- `owner`: The new Owner of this asset."]
					#[doc = ""]
					#[doc = "Emits `OwnerChanged`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					transfer_ownership {
						#[codec(compact)]
						id: ::core::primitive::u128,
						owner: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 16)]
					#[doc = "Change the Issuer, Admin and Freezer of an asset."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Owner of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to be frozen."]
					#[doc = "- `issuer`: The new Issuer of this asset."]
					#[doc = "- `admin`: The new Admin of this asset."]
					#[doc = "- `freezer`: The new Freezer of this asset."]
					#[doc = ""]
					#[doc = "Emits `TeamChanged`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					set_team {
						#[codec(compact)]
						id: ::core::primitive::u128,
						issuer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						admin: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						freezer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 17)]
					#[doc = "Set the metadata for an asset."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Owner of the asset `id`."]
					#[doc = ""]
					#[doc = "Funds of sender are reserved according to the formula:"]
					#[doc = "`MetadataDepositBase + MetadataDepositPerByte * (name.len + symbol.len)` taking into"]
					#[doc = "account any already reserved funds."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to update."]
					#[doc = "- `name`: The user friendly name of this asset. Limited in length by `StringLimit`."]
					#[doc = "- `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`."]
					#[doc = "- `decimals`: The number of decimals this asset uses to represent one unit."]
					#[doc = ""]
					#[doc = "Emits `MetadataSet`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					set_metadata {
						#[codec(compact)]
						id: ::core::primitive::u128,
						name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						symbol: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						decimals: ::core::primitive::u8,
					},
					#[codec(index = 18)]
					#[doc = "Clear the metadata for an asset."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Owner of the asset `id`."]
					#[doc = ""]
					#[doc = "Any deposit is freed for the asset owner."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to clear."]
					#[doc = ""]
					#[doc = "Emits `MetadataCleared`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					clear_metadata {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 19)]
					#[doc = "Force the metadata for an asset to some value."]
					#[doc = ""]
					#[doc = "Origin must be ForceOrigin."]
					#[doc = ""]
					#[doc = "Any deposit is left alone."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to update."]
					#[doc = "- `name`: The user friendly name of this asset. Limited in length by `StringLimit`."]
					#[doc = "- `symbol`: The exchange symbol for this asset. Limited in length by `StringLimit`."]
					#[doc = "- `decimals`: The number of decimals this asset uses to represent one unit."]
					#[doc = ""]
					#[doc = "Emits `MetadataSet`."]
					#[doc = ""]
					#[doc = "Weight: `O(N + S)` where N and S are the length of the name and symbol respectively."]
					force_set_metadata {
						#[codec(compact)]
						id: ::core::primitive::u128,
						name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						symbol: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						decimals: ::core::primitive::u8,
						is_frozen: ::core::primitive::bool,
					},
					#[codec(index = 20)]
					#[doc = "Clear the metadata for an asset."]
					#[doc = ""]
					#[doc = "Origin must be ForceOrigin."]
					#[doc = ""]
					#[doc = "Any deposit is returned."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset to clear."]
					#[doc = ""]
					#[doc = "Emits `MetadataCleared`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					force_clear_metadata {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 21)]
					#[doc = "Alter the attributes of a given asset."]
					#[doc = ""]
					#[doc = "Origin must be `ForceOrigin`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset."]
					#[doc = "- `owner`: The new Owner of this asset."]
					#[doc = "- `issuer`: The new Issuer of this asset."]
					#[doc = "- `admin`: The new Admin of this asset."]
					#[doc = "- `freezer`: The new Freezer of this asset."]
					#[doc = "- `min_balance`: The minimum balance of this new asset that any single account must"]
					#[doc = "have. If an account's balance is reduced below this, then it collapses to zero."]
					#[doc = "- `is_sufficient`: Whether a non-zero balance of this asset is deposit of sufficient"]
					#[doc = "value to account for the state bloat associated with its balance storage. If set to"]
					#[doc = "`true`, then non-zero balances may be stored without a `consumer` reference (and thus"]
					#[doc = "an ED in the Balances pallet or whatever else is used to control user-account state"]
					#[doc = "growth)."]
					#[doc = "- `is_frozen`: Whether this asset class is frozen except for permissioned/admin"]
					#[doc = "instructions."]
					#[doc = ""]
					#[doc = "Emits `AssetStatusChanged` with the identity of the asset."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					force_asset_status {
						#[codec(compact)]
						id: ::core::primitive::u128,
						owner: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						issuer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						admin: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						freezer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						min_balance: ::core::primitive::u128,
						is_sufficient: ::core::primitive::bool,
						is_frozen: ::core::primitive::bool,
					},
					#[codec(index = 22)]
					#[doc = "Approve an amount of asset for transfer by a delegated third-party account."]
					#[doc = ""]
					#[doc = "Origin must be Signed."]
					#[doc = ""]
					#[doc = "Ensures that `ApprovalDeposit` worth of `Currency` is reserved from signing account"]
					#[doc = "for the purpose of holding the approval. If some non-zero amount of assets is already"]
					#[doc = "approved from signing account to `delegate`, then it is topped up or unreserved to"]
					#[doc = "meet the right value."]
					#[doc = ""]
					#[doc = "NOTE: The signing account does not need to own `amount` of assets at the point of"]
					#[doc = "making this call."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset."]
					#[doc = "- `delegate`: The account to delegate permission to transfer asset."]
					#[doc = "- `amount`: The amount of asset that may be transferred by `delegate`. If there is"]
					#[doc = "already an approval in place, then this acts additively."]
					#[doc = ""]
					#[doc = "Emits `ApprovedTransfer` on success."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					approve_transfer {
						#[codec(compact)]
						id: ::core::primitive::u128,
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
					},
					#[codec(index = 23)]
					#[doc = "Cancel all of some asset approved for delegated transfer by a third-party account."]
					#[doc = ""]
					#[doc = "Origin must be Signed and there must be an approval in place between signer and"]
					#[doc = "`delegate`."]
					#[doc = ""]
					#[doc = "Unreserves any deposit previously reserved by `approve_transfer` for the approval."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset."]
					#[doc = "- `delegate`: The account delegated permission to transfer asset."]
					#[doc = ""]
					#[doc = "Emits `ApprovalCancelled` on success."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					cancel_approval {
						#[codec(compact)]
						id: ::core::primitive::u128,
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 24)]
					#[doc = "Cancel all of some asset approved for delegated transfer by a third-party account."]
					#[doc = ""]
					#[doc = "Origin must be either ForceOrigin or Signed origin with the signer being the Admin"]
					#[doc = "account of the asset `id`."]
					#[doc = ""]
					#[doc = "Unreserves any deposit previously reserved by `approve_transfer` for the approval."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset."]
					#[doc = "- `delegate`: The account delegated permission to transfer asset."]
					#[doc = ""]
					#[doc = "Emits `ApprovalCancelled` on success."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					force_cancel_approval {
						#[codec(compact)]
						id: ::core::primitive::u128,
						owner: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 25)]
					#[doc = "Transfer some asset balance from a previously delegated account to some third-party"]
					#[doc = "account."]
					#[doc = ""]
					#[doc = "Origin must be Signed and there must be an approval in place by the `owner` to the"]
					#[doc = "signer."]
					#[doc = ""]
					#[doc = "If the entire amount approved for transfer is transferred, then any deposit previously"]
					#[doc = "reserved by `approve_transfer` is unreserved."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset."]
					#[doc = "- `owner`: The account which previously approved for a transfer of at least `amount` and"]
					#[doc = "from which the asset balance will be withdrawn."]
					#[doc = "- `destination`: The account to which the asset balance of `amount` will be transferred."]
					#[doc = "- `amount`: The amount of assets to transfer."]
					#[doc = ""]
					#[doc = "Emits `TransferredApproved` on success."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					transfer_approved {
						#[codec(compact)]
						id: ::core::primitive::u128,
						owner: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						destination: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
					},
					#[codec(index = 26)]
					#[doc = "Create an asset account for non-provider assets."]
					#[doc = ""]
					#[doc = "A deposit will be taken from the signer account."]
					#[doc = ""]
					#[doc = "- `origin`: Must be Signed; the signer account must have sufficient funds for a deposit"]
					#[doc = "  to be taken."]
					#[doc = "- `id`: The identifier of the asset for the account to be created."]
					#[doc = ""]
					#[doc = "Emits `Touched` event when successful."]
					touch {
						#[codec(compact)]
						id: ::core::primitive::u128,
					},
					#[codec(index = 27)]
					#[doc = "Return the deposit (if any) of an asset account or a consumer reference (if any) of an"]
					#[doc = "account."]
					#[doc = ""]
					#[doc = "The origin must be Signed."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset for which the caller would like the deposit"]
					#[doc = "  refunded."]
					#[doc = "- `allow_burn`: If `true` then assets may be destroyed in order to complete the refund."]
					#[doc = ""]
					#[doc = "Emits `Refunded` event when successful."]
					refund {
						#[codec(compact)]
						id: ::core::primitive::u128,
						allow_burn: ::core::primitive::bool,
					},
					#[codec(index = 28)]
					#[doc = "Sets the minimum balance of an asset."]
					#[doc = ""]
					#[doc = "Only works if there aren't any accounts that are holding the asset or if"]
					#[doc = "the new value of `min_balance` is less than the old one."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender has to be the Owner of the"]
					#[doc = "asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset."]
					#[doc = "- `min_balance`: The new value of `min_balance`."]
					#[doc = ""]
					#[doc = "Emits `AssetMinBalanceChanged` event when successful."]
					set_min_balance {
						#[codec(compact)]
						id: ::core::primitive::u128,
						min_balance: ::core::primitive::u128,
					},
					#[codec(index = 29)]
					#[doc = "Create an asset account for `who`."]
					#[doc = ""]
					#[doc = "A deposit will be taken from the signer account."]
					#[doc = ""]
					#[doc = "- `origin`: Must be Signed by `Freezer` or `Admin` of the asset `id`; the signer account"]
					#[doc = "  must have sufficient funds for a deposit to be taken."]
					#[doc = "- `id`: The identifier of the asset for the account to be created."]
					#[doc = "- `who`: The account to be created."]
					#[doc = ""]
					#[doc = "Emits `Touched` event when successful."]
					touch_other {
						#[codec(compact)]
						id: ::core::primitive::u128,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 30)]
					#[doc = "Return the deposit (if any) of a target asset account. Useful if you are the depositor."]
					#[doc = ""]
					#[doc = "The origin must be Signed and either the account owner, depositor, or asset `Admin`. In"]
					#[doc = "order to burn a non-zero balance of the asset, the caller must be the account and should"]
					#[doc = "use `refund`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the asset for the account holding a deposit."]
					#[doc = "- `who`: The account to refund."]
					#[doc = ""]
					#[doc = "Emits `Refunded` event when successful."]
					refund_other {
						#[codec(compact)]
						id: ::core::primitive::u128,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 31)]
					#[doc = "Disallow further unprivileged transfers of an asset `id` to and from an account `who`."]
					#[doc = ""]
					#[doc = "Origin must be Signed and the sender should be the Freezer of the asset `id`."]
					#[doc = ""]
					#[doc = "- `id`: The identifier of the account's asset."]
					#[doc = "- `who`: The account to be unblocked."]
					#[doc = ""]
					#[doc = "Emits `Blocked`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					block {
						#[codec(compact)]
						id: ::core::primitive::u128,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
				}
			}
		}
		pub mod pallet_assets_handler {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Stores an asset id on chain under an associated resource ID."]
					set_resource {
						resource_id: [::core::primitive::u8; 32usize],
						asset: runtime_types::pallet_bridge_common::AssetInfo<
							::core::primitive::u128,
							::core::primitive::u128,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Removes a resource ID from the resource mapping."]
					#[doc = ""]
					#[doc = "After this call, bridge transfers with the associated resource ID will"]
					#[doc = "be rejected."]
					remove_resource { resource_id: [::core::primitive::u8; 32usize] },
					#[codec(index = 2)]
					set_maximum_issuance { maximum_issuance: ::core::primitive::u128 },
					#[codec(index = 3)]
					set_external_balances { external_balances: ::core::primitive::u128 },
				}
			}
		}
		pub mod pallet_balances {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Transfer some liquid free balance to another account."]
					#[doc = ""]
					#[doc = "`transfer_allow_death` will set the `FreeBalance` of the sender and receiver."]
					#[doc = "If the sender's account is below the existential deposit as a result"]
					#[doc = "of the transfer, the account will be reaped."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be `Signed` by the transactor."]
					transfer_allow_death {
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						value: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					#[doc = "Exactly as `transfer_allow_death`, except the origin must be root and the source account"]
					#[doc = "may be specified."]
					force_transfer {
						source: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						value: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					#[doc = "Same as the [`transfer_allow_death`] call, but with a check that the transfer will not"]
					#[doc = "kill the origin account."]
					#[doc = ""]
					#[doc = "99% of the time you want [`transfer_allow_death`] instead."]
					#[doc = ""]
					#[doc = "[`transfer_allow_death`]: struct.Pallet.html#method.transfer"]
					transfer_keep_alive {
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						value: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					#[doc = "Transfer the entire transferable balance from the caller account."]
					#[doc = ""]
					#[doc = "NOTE: This function only attempts to transfer _transferable_ balances. This means that"]
					#[doc = "any locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be"]
					#[doc = "transferred by this function. To ensure that this function results in a killed account,"]
					#[doc = "you might need to prepare the account by removing any reference counters, storage"]
					#[doc = "deposits, etc..."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be Signed."]
					#[doc = ""]
					#[doc = "- `dest`: The recipient of the transfer."]
					#[doc = "- `keep_alive`: A boolean to determine if the `transfer_all` operation should send all"]
					#[doc = "  of the funds the account has, causing the sender account to be killed (false), or"]
					#[doc = "  transfer everything except at least the existential deposit, which will guarantee to"]
					#[doc = "  keep the sender account alive (true)."]
					transfer_all {
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						keep_alive: ::core::primitive::bool,
					},
					#[codec(index = 5)]
					#[doc = "Unreserve some balance from a user by force."]
					#[doc = ""]
					#[doc = "Can only be called by ROOT."]
					force_unreserve {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 6)]
					#[doc = "Upgrade a specified account."]
					#[doc = ""]
					#[doc = "- `origin`: Must be `Signed`."]
					#[doc = "- `who`: The account to be upgraded."]
					#[doc = ""]
					#[doc = "This will waive the transaction fee if at least all but 10% of the accounts needed to"]
					#[doc = "be upgraded. (We let some not have to be upgraded just in order to allow for the"]
					#[doc = "possibility of churn)."]
					upgrade_accounts {
						who: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
					},
					#[codec(index = 8)]
					#[doc = "Set the regular balance of a given account."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call is `root`."]
					force_set_balance {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						new_free: ::core::primitive::u128,
					},
					#[codec(index = 9)]
					#[doc = "Adjust the total issuance in a saturating way."]
					#[doc = ""]
					#[doc = "Can only be called by root and always needs a positive `delta`."]
					#[doc = ""]
					#[doc = "# Example"]
					force_adjust_total_issuance {
						direction: runtime_types::pallet_balances::types::AdjustmentDirection,
						#[codec(compact)]
						delta: ::core::primitive::u128,
					},
					#[codec(index = 10)]
					#[doc = "Burn the specified liquid free balance from the origin account."]
					#[doc = ""]
					#[doc = "If the origin's account ends up below the existential deposit as a result"]
					#[doc = "of the burn and `keep_alive` is false, the account will be reaped."]
					#[doc = ""]
					#[doc = "Unlike sending funds to a _burn_ address, which merely makes the funds inaccessible,"]
					#[doc = "this `burn` operation will reduce total issuance by the amount _burned_."]
					burn {
						#[codec(compact)]
						value: ::core::primitive::u128,
						keep_alive: ::core::primitive::bool,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum AdjustmentDirection {
					#[codec(index = 0)]
					Increase,
					#[codec(index = 1)]
					Decrease,
				}
			}
		}
		pub mod pallet_bitacross {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Set the admin account"]
					#[doc = ""]
					#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
					set_admin { new_admin: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 1)]
					add_relayer { account: runtime_types::core_primitives::identity::Identity },
					#[codec(index = 2)]
					remove_relayer { account: runtime_types::core_primitives::identity::Identity },
					#[codec(index = 3)]
					remove_vault { account: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 30)]
					#[doc = "---------------------------------------------------"]
					#[doc = "The following extrinsics are supposed to be called by TEE only"]
					#[doc = "---------------------------------------------------"]
					btc_wallet_generated { pub_key: [::core::primitive::u8; 33usize] },
					#[codec(index = 31)]
					eth_wallet_generated { pub_key: [::core::primitive::u8; 33usize] },
					#[codec(index = 32)]
					ton_wallet_generated { pub_key: [::core::primitive::u8; 32usize] },
					#[codec(index = 33)]
					task_complete,
				}
			}
		}
		pub mod pallet_bounties {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Propose a new bounty."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Payment: `TipReportDepositBase` will be reserved from the origin account, as well as"]
					#[doc = "`DataDepositPerByte` for each byte in `reason`. It will be unreserved upon approval,"]
					#[doc = "or slashed when rejected."]
					#[doc = ""]
					#[doc = "- `curator`: The curator account whom will manage this bounty."]
					#[doc = "- `fee`: The curator fee."]
					#[doc = "- `value`: The total payment amount of this bounty, curator fee included."]
					#[doc = "- `description`: The description of this bounty."]
					propose_bounty {
						#[codec(compact)]
						value: ::core::primitive::u128,
						description:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 1)]
					#[doc = "Approve a bounty proposal. At a later time, the bounty will be funded and become active"]
					#[doc = "and the original deposit will be returned."]
					#[doc = ""]
					#[doc = "May only be called from `T::SpendOrigin`."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					approve_bounty {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					#[doc = "Propose a curator to a funded bounty."]
					#[doc = ""]
					#[doc = "May only be called from `T::SpendOrigin`."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					propose_curator {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
						curator: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						fee: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					#[doc = "Unassign curator from a bounty."]
					#[doc = ""]
					#[doc = "This function can only be called by the `RejectOrigin` a signed origin."]
					#[doc = ""]
					#[doc = "If this function is called by the `RejectOrigin`, we assume that the curator is"]
					#[doc = "malicious or inactive. As a result, we will slash the curator when possible."]
					#[doc = ""]
					#[doc = "If the origin is the curator, we take this as a sign they are unable to do their job and"]
					#[doc = "they willingly give up. We could slash them, but for now we allow them to recover their"]
					#[doc = "deposit and exit without issue. (We may want to change this if it is abused.)"]
					#[doc = ""]
					#[doc = "Finally, the origin can be anyone if and only if the curator is \"inactive\". This allows"]
					#[doc = "anyone in the community to call out that a curator is not doing their due diligence, and"]
					#[doc = "we should pick a new curator. In this case the curator should also be slashed."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					unassign_curator {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
					},
					#[codec(index = 4)]
					#[doc = "Accept the curator role for a bounty."]
					#[doc = "A deposit will be reserved from curator and refund upon successful payout."]
					#[doc = ""]
					#[doc = "May only be called from the curator."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					accept_curator {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					#[doc = "Award bounty to a beneficiary account. The beneficiary will be able to claim the funds"]
					#[doc = "after a delay."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be the curator of this bounty."]
					#[doc = ""]
					#[doc = "- `bounty_id`: Bounty ID to award."]
					#[doc = "- `beneficiary`: The beneficiary account whom will receive the payout."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					award_bounty {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
						beneficiary: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 6)]
					#[doc = "Claim the payout from an awarded bounty after payout delay."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be the beneficiary of this bounty."]
					#[doc = ""]
					#[doc = "- `bounty_id`: Bounty ID to claim."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					claim_bounty {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
					},
					#[codec(index = 7)]
					#[doc = "Cancel a proposed or active bounty. All the funds will be sent to treasury and"]
					#[doc = "the curator deposit will be unreserved if possible."]
					#[doc = ""]
					#[doc = "Only `T::RejectOrigin` is able to cancel a bounty."]
					#[doc = ""]
					#[doc = "- `bounty_id`: Bounty ID to cancel."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					close_bounty {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					#[doc = "Extend the expiry time of an active bounty."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be the curator of this bounty."]
					#[doc = ""]
					#[doc = "- `bounty_id`: Bounty ID to extend."]
					#[doc = "- `remark`: additional information."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					extend_bounty_expiry {
						#[codec(compact)]
						bounty_id: ::core::primitive::u32,
						remark: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
				}
			}
		}
		pub mod pallet_bridge_common {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct AssetInfo<_0, _1> {
				pub fee: _1,
				pub asset: ::core::option::Option<_0>,
			}
		}
		pub mod pallet_bridge_transfer {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Transfers some amount of non-native token to some recipient on a (whitelisted)"]
					#[doc = "destination chain."]
					transfer_assets {
						amount: ::core::primitive::u128,
						recipient: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						dest_id: ::core::primitive::u8,
						resource_id: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 1)]
					#[doc = "Executes a simple currency transfer using the bridge account as the source"]
					#[doc = "Should only be called by bridge pallet"]
					transfer {
						to: ::subxt::ext::subxt_core::utils::AccountId32,
						amount: ::core::primitive::u128,
						rid: [::core::primitive::u8; 32usize],
					},
				}
			}
		}
		pub mod pallet_chain_bridge {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Sets the vote threshold for proposals."]
					#[doc = ""]
					#[doc = "This threshold is used to determine how many votes are required"]
					#[doc = "before a proposal is executed."]
					#[doc = ""]
					#[doc = "# <weight>"]
					#[doc = "- O(1) lookup and insert"]
					#[doc = "# </weight>"]
					set_threshold { threshold: ::core::primitive::u32 },
					#[codec(index = 1)]
					#[doc = "Enables a chain ID as a source or destination for a bridge transfer."]
					#[doc = ""]
					#[doc = "# <weight>"]
					#[doc = "- O(1) lookup and insert"]
					#[doc = "# </weight>"]
					whitelist_chain { id: ::core::primitive::u8 },
					#[codec(index = 2)]
					#[doc = "Adds a new relayer to the relayer set."]
					#[doc = ""]
					#[doc = "# <weight>"]
					#[doc = "- O(1) lookup and insert"]
					#[doc = "# </weight>"]
					add_relayer { v: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 3)]
					#[doc = "Removes an existing relayer from the set."]
					#[doc = ""]
					#[doc = "# <weight>"]
					#[doc = "- O(1) lookup and removal"]
					#[doc = "# </weight>"]
					remove_relayer { v: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 4)]
					#[doc = "Commits a vote in favour of the provided proposal."]
					#[doc = ""]
					#[doc = "If a proposal with the given nonce and source chain ID does not already exist, it will"]
					#[doc = "be created with an initial vote in favour from the caller."]
					#[doc = ""]
					#[doc = "# <weight>"]
					#[doc = "- weight of proposed call, regardless of whether execution is performed"]
					#[doc = "# </weight>"]
					acknowledge_proposal {
						nonce: ::core::primitive::u64,
						src_id: ::core::primitive::u8,
						r_id: [::core::primitive::u8; 32usize],
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 5)]
					#[doc = "Commits a vote against a provided proposal."]
					#[doc = ""]
					#[doc = "# <weight>"]
					#[doc = "- Fixed, since execution of proposal should not be included"]
					#[doc = "# </weight>"]
					reject_proposal {
						nonce: ::core::primitive::u64,
						src_id: ::core::primitive::u8,
						r_id: [::core::primitive::u8; 32usize],
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 6)]
					#[doc = "Evaluate the state of a proposal given the current vote threshold."]
					#[doc = ""]
					#[doc = "A proposal with enough votes will be either executed or cancelled, and the status"]
					#[doc = "will be updated accordingly."]
					#[doc = ""]
					#[doc = "# <weight>"]
					#[doc = "- weight of proposed call, regardless of whether execution is performed"]
					#[doc = "# </weight>"]
					eval_vote_state {
						nonce: ::core::primitive::u64,
						src_id: ::core::primitive::u8,
						prop: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
				}
			}
		}
		pub mod pallet_collab_ai_common {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum CandidateStatus {
				#[codec(index = 0)]
				Unverified,
				#[codec(index = 1)]
				Verified,
				#[codec(index = 2)]
				Banned,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum GuardianVote {
				#[codec(index = 0)]
				Neutral,
				#[codec(index = 1)]
				Aye,
				#[codec(index = 2)]
				Nay,
				#[codec(index = 3)]
				Specific(::core::primitive::u128),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct PoolSetting<_0, _1, _2> {
				pub start_time: _1,
				pub epoch: ::core::primitive::u128,
				pub epoch_range: _1,
				pub pool_cap: _2,
				pub admin: _0,
			}
		}
		pub mod pallet_collective {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Set the collective's membership."]
					#[doc = ""]
					#[doc = "- `new_members`: The new member list. Be nice to the chain and provide it sorted."]
					#[doc = "- `prime`: The prime member whose vote sets the default."]
					#[doc = "- `old_count`: The upper bound for the previous number of members in storage. Used for"]
					#[doc = "  weight estimation."]
					#[doc = ""]
					#[doc = "The dispatch of this call must be `SetMembersOrigin`."]
					#[doc = ""]
					#[doc = "NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but"]
					#[doc = "      the weight estimations rely on it to estimate dispatchable weight."]
					#[doc = ""]
					#[doc = "# WARNING:"]
					#[doc = ""]
					#[doc = "The `pallet-collective` can also be managed by logic outside of the pallet through the"]
					#[doc = "implementation of the trait [`ChangeMembers`]."]
					#[doc = "Any call to `set_members` must be careful that the member set doesn't get out of sync"]
					#[doc = "with other logic managing the member set."]
					#[doc = ""]
					#[doc = "## Complexity:"]
					#[doc = "- `O(MP + N)` where:"]
					#[doc = "  - `M` old-members-count (code- and governance-bounded)"]
					#[doc = "  - `N` new-members-count (code- and governance-bounded)"]
					#[doc = "  - `P` proposals-count (code-bounded)"]
					set_members {
						new_members: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
						prime: ::core::option::Option<::subxt::ext::subxt_core::utils::AccountId32>,
						old_count: ::core::primitive::u32,
					},
					#[codec(index = 1)]
					#[doc = "Dispatch a proposal from a member using the `Member` origin."]
					#[doc = ""]
					#[doc = "Origin must be a member of the collective."]
					#[doc = ""]
					#[doc = "## Complexity:"]
					#[doc = "- `O(B + M + P)` where:"]
					#[doc = "- `B` is `proposal` size in bytes (length-fee-bounded)"]
					#[doc = "- `M` members-count (code-bounded)"]
					#[doc = "- `P` complexity of dispatching `proposal`"]
					execute {
						proposal: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
						#[codec(compact)]
						length_bound: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					#[doc = "Add a new proposal to either be voted on or executed directly."]
					#[doc = ""]
					#[doc = "Requires the sender to be member."]
					#[doc = ""]
					#[doc = "`threshold` determines whether `proposal` is executed directly (`threshold < 2`)"]
					#[doc = "or put up for voting."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(B + M + P1)` or `O(B + M + P2)` where:"]
					#[doc = "  - `B` is `proposal` size in bytes (length-fee-bounded)"]
					#[doc = "  - `M` is members-count (code- and governance-bounded)"]
					#[doc = "  - branching is influenced by `threshold` where:"]
					#[doc = "    - `P1` is proposal execution complexity (`threshold < 2`)"]
					#[doc = "    - `P2` is proposals-count (code-bounded) (`threshold >= 2`)"]
					propose {
						#[codec(compact)]
						threshold: ::core::primitive::u32,
						proposal: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
						#[codec(compact)]
						length_bound: ::core::primitive::u32,
					},
					#[codec(index = 3)]
					#[doc = "Add an aye or nay vote for the sender to the given proposal."]
					#[doc = ""]
					#[doc = "Requires the sender to be a member."]
					#[doc = ""]
					#[doc = "Transaction fees will be waived if the member is voting on any particular proposal"]
					#[doc = "for the first time and the call is successful. Subsequent vote changes will charge a"]
					#[doc = "fee."]
					#[doc = "## Complexity"]
					#[doc = "- `O(M)` where `M` is members-count (code- and governance-bounded)"]
					vote {
						proposal: ::subxt::ext::subxt_core::utils::H256,
						#[codec(compact)]
						index: ::core::primitive::u32,
						approve: ::core::primitive::bool,
					},
					#[codec(index = 5)]
					#[doc = "Disapprove a proposal, close, and remove it from the system, regardless of its current"]
					#[doc = "state."]
					#[doc = ""]
					#[doc = "Must be called by the Root origin."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "* `proposal_hash`: The hash of the proposal that should be disapproved."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "O(P) where P is the number of max proposals"]
					disapprove_proposal { proposal_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 6)]
					#[doc = "Close a vote that is either approved, disapproved or whose voting period has ended."]
					#[doc = ""]
					#[doc = "May be called by any signed account in order to finish voting and close the proposal."]
					#[doc = ""]
					#[doc = "If called before the end of the voting period it will only close the vote if it is"]
					#[doc = "has enough votes to be approved or disapproved."]
					#[doc = ""]
					#[doc = "If called after the end of the voting period abstentions are counted as rejections"]
					#[doc = "unless there is a prime member set and the prime member cast an approval."]
					#[doc = ""]
					#[doc = "If the close operation completes successfully with disapproval, the transaction fee will"]
					#[doc = "be waived. Otherwise execution of the approved operation will be charged to the caller."]
					#[doc = ""]
					#[doc = "+ `proposal_weight_bound`: The maximum amount of weight consumed by executing the closed"]
					#[doc = "proposal."]
					#[doc = "+ `length_bound`: The upper bound for the length of the proposal in storage. Checked via"]
					#[doc = "`storage::read` so it is `size_of::<u32>() == 4` larger than the pure length."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(B + M + P1 + P2)` where:"]
					#[doc = "  - `B` is `proposal` size in bytes (length-fee-bounded)"]
					#[doc = "  - `M` is members-count (code- and governance-bounded)"]
					#[doc = "  - `P1` is the complexity of `proposal` preimage."]
					#[doc = "  - `P2` is proposal-count (code-bounded)"]
					close {
						proposal_hash: ::subxt::ext::subxt_core::utils::H256,
						#[codec(compact)]
						index: ::core::primitive::u32,
						proposal_weight_bound: runtime_types::sp_weights::weight_v2::Weight,
						#[codec(compact)]
						length_bound: ::core::primitive::u32,
					},
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum RawOrigin<_0> {
				#[codec(index = 0)]
				Members(::core::primitive::u32, ::core::primitive::u32),
				#[codec(index = 1)]
				Member(_0),
				#[codec(index = 2)]
				_Phantom,
			}
		}
		pub mod pallet_conviction_voting {
			use super::runtime_types;
			pub mod conviction {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Conviction {
					#[codec(index = 0)]
					None,
					#[codec(index = 1)]
					Locked1x,
					#[codec(index = 2)]
					Locked2x,
					#[codec(index = 3)]
					Locked3x,
					#[codec(index = 4)]
					Locked4x,
					#[codec(index = 5)]
					Locked5x,
					#[codec(index = 6)]
					Locked6x,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Vote in a poll. If `vote.is_aye()`, the vote is to enact the proposal;"]
					#[doc = "otherwise it is a vote to keep the status quo."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `poll_index`: The index of the poll to vote for."]
					#[doc = "- `vote`: The vote configuration."]
					#[doc = ""]
					#[doc = "Weight: `O(R)` where R is the number of polls the voter has voted on."]
					vote {
						#[codec(compact)]
						poll_index: ::core::primitive::u32,
						vote: runtime_types::pallet_conviction_voting::vote::AccountVote<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Delegate the voting power (with some given conviction) of the sending account for a"]
					#[doc = "particular class of polls."]
					#[doc = ""]
					#[doc = "The balance delegated is locked for as long as it's delegated, and thereafter for the"]
					#[doc = "time appropriate for the conviction's lock period."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_, and the signing account must either:"]
					#[doc = "  - be delegating already; or"]
					#[doc = "  - have no voting activity (if there is, then it will need to be removed through"]
					#[doc = "    `remove_vote`)."]
					#[doc = ""]
					#[doc = "- `to`: The account whose voting the `target` account's voting power will follow."]
					#[doc = "- `class`: The class of polls to delegate. To delegate multiple classes, multiple calls"]
					#[doc = "  to this function are required."]
					#[doc = "- `conviction`: The conviction that will be attached to the delegated votes. When the"]
					#[doc = "  account is undelegated, the funds will be locked for the corresponding period."]
					#[doc = "- `balance`: The amount of the account's balance to be used in delegating. This must not"]
					#[doc = "  be more than the account's current balance."]
					#[doc = ""]
					#[doc = "Emits `Delegated`."]
					#[doc = ""]
					#[doc = "Weight: `O(R)` where R is the number of polls the voter delegating to has"]
					#[doc = "  voted on. Weight is initially charged as if maximum votes, but is refunded later."]
					delegate {
						class: ::core::primitive::u16,
						to: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						conviction: runtime_types::pallet_conviction_voting::conviction::Conviction,
						balance: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					#[doc = "Undelegate the voting power of the sending account for a particular class of polls."]
					#[doc = ""]
					#[doc = "Tokens may be unlocked following once an amount of time consistent with the lock period"]
					#[doc = "of the conviction with which the delegation was issued has passed."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_ and the signing account must be"]
					#[doc = "currently delegating."]
					#[doc = ""]
					#[doc = "- `class`: The class of polls to remove the delegation from."]
					#[doc = ""]
					#[doc = "Emits `Undelegated`."]
					#[doc = ""]
					#[doc = "Weight: `O(R)` where R is the number of polls the voter delegating to has"]
					#[doc = "  voted on. Weight is initially charged as if maximum votes, but is refunded later."]
					undelegate { class: ::core::primitive::u16 },
					#[codec(index = 3)]
					#[doc = "Remove the lock caused by prior voting/delegating which has expired within a particular"]
					#[doc = "class."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `class`: The class of polls to unlock."]
					#[doc = "- `target`: The account to remove the lock on."]
					#[doc = ""]
					#[doc = "Weight: `O(R)` with R number of vote of target."]
					unlock {
						class: ::core::primitive::u16,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 4)]
					#[doc = "Remove a vote for a poll."]
					#[doc = ""]
					#[doc = "If:"]
					#[doc = "- the poll was cancelled, or"]
					#[doc = "- the poll is ongoing, or"]
					#[doc = "- the poll has ended such that"]
					#[doc = "  - the vote of the account was in opposition to the result; or"]
					#[doc = "  - there was no conviction to the account's vote; or"]
					#[doc = "  - the account made a split vote"]
					#[doc = "...then the vote is removed cleanly and a following call to `unlock` may result in more"]
					#[doc = "funds being available."]
					#[doc = ""]
					#[doc = "If, however, the poll has ended and:"]
					#[doc = "- it finished corresponding to the vote of the account, and"]
					#[doc = "- the account made a standard vote with conviction, and"]
					#[doc = "- the lock period of the conviction is not over"]
					#[doc = "...then the lock will be aggregated into the overall account's lock, which may involve"]
					#[doc = "*overlocking* (where the two locks are combined into a single lock that is the maximum"]
					#[doc = "of both the amount locked and the time is it locked for)."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_, and the signer must have a vote"]
					#[doc = "registered for poll `index`."]
					#[doc = ""]
					#[doc = "- `index`: The index of poll of the vote to be removed."]
					#[doc = "- `class`: Optional parameter, if given it indicates the class of the poll. For polls"]
					#[doc = "  which have finished or are cancelled, this must be `Some`."]
					#[doc = ""]
					#[doc = "Weight: `O(R + log R)` where R is the number of polls that `target` has voted on."]
					#[doc = "  Weight is calculated for the maximum number of vote."]
					remove_vote {
						class: ::core::option::Option<::core::primitive::u16>,
						index: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					#[doc = "Remove a vote for a poll."]
					#[doc = ""]
					#[doc = "If the `target` is equal to the signer, then this function is exactly equivalent to"]
					#[doc = "`remove_vote`. If not equal to the signer, then the vote must have expired,"]
					#[doc = "either because the poll was cancelled, because the voter lost the poll or"]
					#[doc = "because the conviction period is over."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `target`: The account of the vote to be removed; this account must have voted for poll"]
					#[doc = "  `index`."]
					#[doc = "- `index`: The index of poll of the vote to be removed."]
					#[doc = "- `class`: The class of the poll."]
					#[doc = ""]
					#[doc = "Weight: `O(R + log R)` where R is the number of polls that `target` has voted on."]
					#[doc = "  Weight is calculated for the maximum number of vote."]
					remove_other_vote {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						class: ::core::primitive::u16,
						index: ::core::primitive::u32,
					},
				}
			}
			pub mod vote {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum AccountVote<_0> {
					#[codec(index = 0)]
					Standard {
						vote: runtime_types::pallet_conviction_voting::vote::Vote,
						balance: _0,
					},
					#[codec(index = 1)]
					Split { aye: _0, nay: _0 },
					#[codec(index = 2)]
					SplitAbstain { aye: _0, nay: _0, abstain: _0 },
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Vote(pub ::core::primitive::u8);
			}
		}
		pub mod pallet_curator {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Registing a curator legal info"]
					regist_curator { info_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 1)]
					#[doc = "Updating a curator legal info"]
					update_curator { info_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					#[doc = "Clean a curator legal info"]
					clean_curator,
					#[codec(index = 3)]
					judge_curator_status {
						curator: ::subxt::ext::subxt_core::utils::AccountId32,
						status: runtime_types::pallet_collab_ai_common::CandidateStatus,
					},
				}
			}
		}
		pub mod pallet_democracy {
			use super::runtime_types;
			pub mod conviction {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Conviction {
					#[codec(index = 0)]
					None,
					#[codec(index = 1)]
					Locked1x,
					#[codec(index = 2)]
					Locked2x,
					#[codec(index = 3)]
					Locked3x,
					#[codec(index = 4)]
					Locked4x,
					#[codec(index = 5)]
					Locked5x,
					#[codec(index = 6)]
					Locked6x,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Propose a sensitive action to be taken."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_ and the sender must"]
					#[doc = "have funds to cover the deposit."]
					#[doc = ""]
					#[doc = "- `proposal_hash`: The hash of the proposal preimage."]
					#[doc = "- `value`: The amount of deposit (must be at least `MinimumDeposit`)."]
					#[doc = ""]
					#[doc = "Emits `Proposed`."]
					propose {
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
							runtime_types::sp_runtime::traits::BlakeTwo256,
						>,
						#[codec(compact)]
						value: ::core::primitive::u128,
					},
					#[codec(index = 1)]
					#[doc = "Signals agreement with a particular proposal."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_ and the sender"]
					#[doc = "must have funds to cover the deposit, equal to the original deposit."]
					#[doc = ""]
					#[doc = "- `proposal`: The index of the proposal to second."]
					second {
						#[codec(compact)]
						proposal: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					#[doc = "Vote in a referendum. If `vote.is_aye()`, the vote is to enact the proposal;"]
					#[doc = "otherwise it is a vote to keep the status quo."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `ref_index`: The index of the referendum to vote for."]
					#[doc = "- `vote`: The vote configuration."]
					vote {
						#[codec(compact)]
						ref_index: ::core::primitive::u32,
						vote: runtime_types::pallet_democracy::vote::AccountVote<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 3)]
					#[doc = "Schedule an emergency cancellation of a referendum. Cannot happen twice to the same"]
					#[doc = "referendum."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be `CancellationOrigin`."]
					#[doc = ""]
					#[doc = "-`ref_index`: The index of the referendum to cancel."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`."]
					emergency_cancel { ref_index: ::core::primitive::u32 },
					#[codec(index = 4)]
					#[doc = "Schedule a referendum to be tabled once it is legal to schedule an external"]
					#[doc = "referendum."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be `ExternalOrigin`."]
					#[doc = ""]
					#[doc = "- `proposal_hash`: The preimage hash of the proposal."]
					external_propose {
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
							runtime_types::sp_runtime::traits::BlakeTwo256,
						>,
					},
					#[codec(index = 5)]
					#[doc = "Schedule a majority-carries referendum to be tabled next once it is legal to schedule"]
					#[doc = "an external referendum."]
					#[doc = ""]
					#[doc = "The dispatch of this call must be `ExternalMajorityOrigin`."]
					#[doc = ""]
					#[doc = "- `proposal_hash`: The preimage hash of the proposal."]
					#[doc = ""]
					#[doc = "Unlike `external_propose`, blacklisting has no effect on this and it may replace a"]
					#[doc = "pre-scheduled `external_propose` call."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					external_propose_majority {
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
							runtime_types::sp_runtime::traits::BlakeTwo256,
						>,
					},
					#[codec(index = 6)]
					#[doc = "Schedule a negative-turnout-bias referendum to be tabled next once it is legal to"]
					#[doc = "schedule an external referendum."]
					#[doc = ""]
					#[doc = "The dispatch of this call must be `ExternalDefaultOrigin`."]
					#[doc = ""]
					#[doc = "- `proposal_hash`: The preimage hash of the proposal."]
					#[doc = ""]
					#[doc = "Unlike `external_propose`, blacklisting has no effect on this and it may replace a"]
					#[doc = "pre-scheduled `external_propose` call."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					external_propose_default {
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
							runtime_types::sp_runtime::traits::BlakeTwo256,
						>,
					},
					#[codec(index = 7)]
					#[doc = "Schedule the currently externally-proposed majority-carries referendum to be tabled"]
					#[doc = "immediately. If there is no externally-proposed referendum currently, or if there is one"]
					#[doc = "but it is not a majority-carries referendum then it fails."]
					#[doc = ""]
					#[doc = "The dispatch of this call must be `FastTrackOrigin`."]
					#[doc = ""]
					#[doc = "- `proposal_hash`: The hash of the current external proposal."]
					#[doc = "- `voting_period`: The period that is allowed for voting on this proposal. Increased to"]
					#[doc = "\tMust be always greater than zero."]
					#[doc = "\tFor `FastTrackOrigin` must be equal or greater than `FastTrackVotingPeriod`."]
					#[doc = "- `delay`: The number of block after voting has ended in approval and this should be"]
					#[doc = "  enacted. This doesn't have a minimum amount."]
					#[doc = ""]
					#[doc = "Emits `Started`."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`"]
					fast_track {
						proposal_hash: ::subxt::ext::subxt_core::utils::H256,
						voting_period: ::core::primitive::u32,
						delay: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					#[doc = "Veto and blacklist the external proposal hash."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be `VetoOrigin`."]
					#[doc = ""]
					#[doc = "- `proposal_hash`: The preimage hash of the proposal to veto and blacklist."]
					#[doc = ""]
					#[doc = "Emits `Vetoed`."]
					#[doc = ""]
					#[doc = "Weight: `O(V + log(V))` where V is number of `existing vetoers`"]
					veto_external { proposal_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 9)]
					#[doc = "Remove a referendum."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Root_."]
					#[doc = ""]
					#[doc = "- `ref_index`: The index of the referendum to cancel."]
					#[doc = ""]
					#[doc = "# Weight: `O(1)`."]
					cancel_referendum {
						#[codec(compact)]
						ref_index: ::core::primitive::u32,
					},
					#[codec(index = 10)]
					#[doc = "Delegate the voting power (with some given conviction) of the sending account."]
					#[doc = ""]
					#[doc = "The balance delegated is locked for as long as it's delegated, and thereafter for the"]
					#[doc = "time appropriate for the conviction's lock period."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_, and the signing account must either:"]
					#[doc = "  - be delegating already; or"]
					#[doc = "  - have no voting activity (if there is, then it will need to be removed/consolidated"]
					#[doc = "    through `reap_vote` or `unvote`)."]
					#[doc = ""]
					#[doc = "- `to`: The account whose voting the `target` account's voting power will follow."]
					#[doc = "- `conviction`: The conviction that will be attached to the delegated votes. When the"]
					#[doc = "  account is undelegated, the funds will be locked for the corresponding period."]
					#[doc = "- `balance`: The amount of the account's balance to be used in delegating. This must not"]
					#[doc = "  be more than the account's current balance."]
					#[doc = ""]
					#[doc = "Emits `Delegated`."]
					#[doc = ""]
					#[doc = "Weight: `O(R)` where R is the number of referendums the voter delegating to has"]
					#[doc = "  voted on. Weight is charged as if maximum votes."]
					delegate {
						to: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						conviction: runtime_types::pallet_democracy::conviction::Conviction,
						balance: ::core::primitive::u128,
					},
					#[codec(index = 11)]
					#[doc = "Undelegate the voting power of the sending account."]
					#[doc = ""]
					#[doc = "Tokens may be unlocked following once an amount of time consistent with the lock period"]
					#[doc = "of the conviction with which the delegation was issued."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_ and the signing account must be"]
					#[doc = "currently delegating."]
					#[doc = ""]
					#[doc = "Emits `Undelegated`."]
					#[doc = ""]
					#[doc = "Weight: `O(R)` where R is the number of referendums the voter delegating to has"]
					#[doc = "  voted on. Weight is charged as if maximum votes."]
					undelegate,
					#[codec(index = 12)]
					#[doc = "Clears all public proposals."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Root_."]
					#[doc = ""]
					#[doc = "Weight: `O(1)`."]
					clear_public_proposals,
					#[codec(index = 13)]
					#[doc = "Unlock tokens that have an expired lock."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `target`: The account to remove the lock on."]
					#[doc = ""]
					#[doc = "Weight: `O(R)` with R number of vote of target."]
					unlock {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 14)]
					#[doc = "Remove a vote for a referendum."]
					#[doc = ""]
					#[doc = "If:"]
					#[doc = "- the referendum was cancelled, or"]
					#[doc = "- the referendum is ongoing, or"]
					#[doc = "- the referendum has ended such that"]
					#[doc = "  - the vote of the account was in opposition to the result; or"]
					#[doc = "  - there was no conviction to the account's vote; or"]
					#[doc = "  - the account made a split vote"]
					#[doc = "...then the vote is removed cleanly and a following call to `unlock` may result in more"]
					#[doc = "funds being available."]
					#[doc = ""]
					#[doc = "If, however, the referendum has ended and:"]
					#[doc = "- it finished corresponding to the vote of the account, and"]
					#[doc = "- the account made a standard vote with conviction, and"]
					#[doc = "- the lock period of the conviction is not over"]
					#[doc = "...then the lock will be aggregated into the overall account's lock, which may involve"]
					#[doc = "*overlocking* (where the two locks are combined into a single lock that is the maximum"]
					#[doc = "of both the amount locked and the time is it locked for)."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_, and the signer must have a vote"]
					#[doc = "registered for referendum `index`."]
					#[doc = ""]
					#[doc = "- `index`: The index of referendum of the vote to be removed."]
					#[doc = ""]
					#[doc = "Weight: `O(R + log R)` where R is the number of referenda that `target` has voted on."]
					#[doc = "  Weight is calculated for the maximum number of vote."]
					remove_vote { index: ::core::primitive::u32 },
					#[codec(index = 15)]
					#[doc = "Remove a vote for a referendum."]
					#[doc = ""]
					#[doc = "If the `target` is equal to the signer, then this function is exactly equivalent to"]
					#[doc = "`remove_vote`. If not equal to the signer, then the vote must have expired,"]
					#[doc = "either because the referendum was cancelled, because the voter lost the referendum or"]
					#[doc = "because the conviction period is over."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `target`: The account of the vote to be removed; this account must have voted for"]
					#[doc = "  referendum `index`."]
					#[doc = "- `index`: The index of referendum of the vote to be removed."]
					#[doc = ""]
					#[doc = "Weight: `O(R + log R)` where R is the number of referenda that `target` has voted on."]
					#[doc = "  Weight is calculated for the maximum number of vote."]
					remove_other_vote {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						index: ::core::primitive::u32,
					},
					#[codec(index = 16)]
					#[doc = "Permanently place a proposal into the blacklist. This prevents it from ever being"]
					#[doc = "proposed again."]
					#[doc = ""]
					#[doc = "If called on a queued public or external proposal, then this will result in it being"]
					#[doc = "removed. If the `ref_index` supplied is an active referendum with the proposal hash,"]
					#[doc = "then it will be cancelled."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be `BlacklistOrigin`."]
					#[doc = ""]
					#[doc = "- `proposal_hash`: The proposal hash to blacklist permanently."]
					#[doc = "- `ref_index`: An ongoing referendum whose hash is `proposal_hash`, which will be"]
					#[doc = "cancelled."]
					#[doc = ""]
					#[doc = "Weight: `O(p)` (though as this is an high-privilege dispatch, we assume it has a"]
					#[doc = "  reasonable value)."]
					blacklist {
						proposal_hash: ::subxt::ext::subxt_core::utils::H256,
						maybe_ref_index: ::core::option::Option<::core::primitive::u32>,
					},
					#[codec(index = 17)]
					#[doc = "Remove a proposal."]
					#[doc = ""]
					#[doc = "The dispatch origin of this call must be `CancelProposalOrigin`."]
					#[doc = ""]
					#[doc = "- `prop_index`: The index of the proposal to cancel."]
					#[doc = ""]
					#[doc = "Weight: `O(p)` where `p = PublicProps::<T>::decode_len()`"]
					cancel_proposal {
						#[codec(compact)]
						prop_index: ::core::primitive::u32,
					},
					#[codec(index = 18)]
					#[doc = "Set or clear a metadata of a proposal or a referendum."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `origin`: Must correspond to the `MetadataOwner`."]
					#[doc = "    - `ExternalOrigin` for an external proposal with the `SuperMajorityApprove`"]
					#[doc = "      threshold."]
					#[doc = "    - `ExternalDefaultOrigin` for an external proposal with the `SuperMajorityAgainst`"]
					#[doc = "      threshold."]
					#[doc = "    - `ExternalMajorityOrigin` for an external proposal with the `SimpleMajority`"]
					#[doc = "      threshold."]
					#[doc = "    - `Signed` by a creator for a public proposal."]
					#[doc = "    - `Signed` to clear a metadata for a finished referendum."]
					#[doc = "    - `Root` to set a metadata for an ongoing referendum."]
					#[doc = "- `owner`: an identifier of a metadata owner."]
					#[doc = "- `maybe_hash`: The hash of an on-chain stored preimage. `None` to clear a metadata."]
					set_metadata {
						owner: runtime_types::pallet_democracy::types::MetadataOwner,
						maybe_hash: ::core::option::Option<::subxt::ext::subxt_core::utils::H256>,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum MetadataOwner {
					#[codec(index = 0)]
					External,
					#[codec(index = 1)]
					Proposal(::core::primitive::u32),
					#[codec(index = 2)]
					Referendum(::core::primitive::u32),
				}
			}
			pub mod vote {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum AccountVote<_0> {
					#[codec(index = 0)]
					Standard { vote: runtime_types::pallet_democracy::vote::Vote, balance: _0 },
					#[codec(index = 1)]
					Split { aye: _0, nay: _0 },
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Vote(pub ::core::primitive::u8);
			}
		}
		pub mod pallet_ethereum {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Transact an Ethereum transaction."]
					transact { transaction: runtime_types::ethereum::transaction::TransactionV2 },
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum RawOrigin {
				#[codec(index = 0)]
				EthereumTransaction(::subxt::ext::subxt_core::utils::H160),
			}
		}
		pub mod pallet_evm {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Withdraw balance from EVM into currency/balances pallet."]
					withdraw {
						address: ::subxt::ext::subxt_core::utils::H160,
						value: ::core::primitive::u128,
					},
					#[codec(index = 1)]
					#[doc = "Issue an EVM call operation. This is similar to a message call transaction in Ethereum."]
					call {
						source: ::subxt::ext::subxt_core::utils::H160,
						target: ::subxt::ext::subxt_core::utils::H160,
						input: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						value: runtime_types::primitive_types::U256,
						gas_limit: ::core::primitive::u64,
						max_fee_per_gas: runtime_types::primitive_types::U256,
						max_priority_fee_per_gas:
							::core::option::Option<runtime_types::primitive_types::U256>,
						nonce: ::core::option::Option<runtime_types::primitive_types::U256>,
						access_list: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::utils::H160,
							::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::H256,
							>,
						)>,
					},
					#[codec(index = 2)]
					#[doc = "Issue an EVM create operation. This is similar to a contract creation transaction in"]
					#[doc = "Ethereum."]
					create {
						source: ::subxt::ext::subxt_core::utils::H160,
						init: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						value: runtime_types::primitive_types::U256,
						gas_limit: ::core::primitive::u64,
						max_fee_per_gas: runtime_types::primitive_types::U256,
						max_priority_fee_per_gas:
							::core::option::Option<runtime_types::primitive_types::U256>,
						nonce: ::core::option::Option<runtime_types::primitive_types::U256>,
						access_list: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::utils::H160,
							::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::H256,
							>,
						)>,
					},
					#[codec(index = 3)]
					#[doc = "Issue an EVM create2 operation."]
					create2 {
						source: ::subxt::ext::subxt_core::utils::H160,
						init: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						salt: ::subxt::ext::subxt_core::utils::H256,
						value: runtime_types::primitive_types::U256,
						gas_limit: ::core::primitive::u64,
						max_fee_per_gas: runtime_types::primitive_types::U256,
						max_priority_fee_per_gas:
							::core::option::Option<runtime_types::primitive_types::U256>,
						nonce: ::core::option::Option<runtime_types::primitive_types::U256>,
						access_list: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::utils::H160,
							::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::H256,
							>,
						)>,
					},
				}
			}
		}
		pub mod pallet_evm_assertions {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					create_assertion {
						id: ::subxt::ext::subxt_core::utils::H160,
						byte_code: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						secrets: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Only called by the Identity-Worker"]
					store_assertion { id: ::subxt::ext::subxt_core::utils::H160 },
					#[codec(index = 2)]
					#[doc = "Only called by the Identity-Worker"]
					void_assertion { id: ::subxt::ext::subxt_core::utils::H160 },
				}
			}
		}
		pub mod pallet_extrinsic_filter {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Set the mode"]
					#[doc = ""]
					#[doc = "The storage of `BlockedExtrinsics` is unaffected."]
					#[doc = "The reason is we'd rather have this pallet behave conservatively:"]
					#[doc = "having extra blocked extrinsics is better than having unexpected whitelisted extrinsics."]
					#[doc = "See the test `set_mode_should_not_clear_blocked_extrinsics()`"]
					#[doc = ""]
					#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
					set_mode { mode: runtime_types::pallet_extrinsic_filter::OperationalMode },
					#[codec(index = 1)]
					#[doc = "block the given extrinsics"]
					#[doc = "(pallet_name_bytes, function_name_bytes) can uniquely identify an extrinsic"]
					#[doc = "if function_name_bytes is None, all extrinsics in `pallet_name_bytes` will be blocked"]
					block_extrinsics {
						pallet_name_bytes:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						function_name_bytes: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					},
					#[codec(index = 2)]
					#[doc = "unblock the given extrinsics"]
					#[doc = "(pallet_name_bytes, function_name_bytes) can uniquely identify an extrinsic"]
					#[doc = "if function_name_bytes is None, all extrinsics in `pallet_name_bytes` will be unblocked"]
					unblock_extrinsics {
						pallet_name_bytes:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						function_name_bytes: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					},
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum OperationalMode {
				#[codec(index = 0)]
				Normal,
				#[codec(index = 1)]
				Safe,
				#[codec(index = 2)]
				Test,
			}
		}
		pub mod pallet_group {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Adds a new group member"]
					add_group_member { v: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 1)]
					#[doc = "Batch adding of new group members"]
					batch_add_group_members {
						vs: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
					},
					#[codec(index = 2)]
					#[doc = "Removes an existing group members"]
					remove_group_member { v: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 3)]
					#[doc = "Batch Removing existing group members"]
					batch_remove_group_members {
						vs: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
					},
					#[codec(index = 4)]
					#[doc = "Swith GroupControlOn on"]
					switch_group_control_on,
					#[codec(index = 5)]
					#[doc = "Swith GroupControlOn off"]
					switch_group_control_off,
				}
			}
		}
		pub mod pallet_guardian {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Registing a guardian legal info"]
					regist_guardian { info_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 1)]
					#[doc = "Updating a guardian legal info"]
					update_guardian { info_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					#[doc = "Clean a guardian legal info"]
					clean_guardian,
					#[codec(index = 3)]
					judge_guardian_status {
						guardian: ::subxt::ext::subxt_core::utils::AccountId32,
						status: runtime_types::pallet_collab_ai_common::CandidateStatus,
					},
					#[codec(index = 4)]
					#[doc = "Anyone can vote for guardian"]
					#[doc = "However if voter is not participating the staking pool"]
					#[doc = "then its vote will never effecting guardian selection procedure"]
					vote {
						guardian: ::subxt::ext::subxt_core::utils::AccountId32,
						status: ::core::option::Option<
							runtime_types::pallet_collab_ai_common::GuardianVote,
						>,
					},
					#[codec(index = 5)]
					#[doc = "Remove vote to None"]
					remove_all_votes,
				}
			}
		}
		pub mod pallet_identity {
			use super::runtime_types;
			pub mod legacy {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct IdentityInfo {
					pub additional: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
						runtime_types::pallet_identity::types::Data,
						runtime_types::pallet_identity::types::Data,
					)>,
					pub display: runtime_types::pallet_identity::types::Data,
					pub legal: runtime_types::pallet_identity::types::Data,
					pub web: runtime_types::pallet_identity::types::Data,
					pub riot: runtime_types::pallet_identity::types::Data,
					pub email: runtime_types::pallet_identity::types::Data,
					pub pgp_fingerprint: ::core::option::Option<[::core::primitive::u8; 20usize]>,
					pub image: runtime_types::pallet_identity::types::Data,
					pub twitter: runtime_types::pallet_identity::types::Data,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Identity pallet declaration."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Add a registrar to the system."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be `T::RegistrarOrigin`."]
					#[doc = ""]
					#[doc = "- `account`: the account of the registrar."]
					#[doc = ""]
					#[doc = "Emits `RegistrarAdded` if successful."]
					add_registrar {
						account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 1)]
					#[doc = "Set an account's identity information and reserve the appropriate deposit."]
					#[doc = ""]
					#[doc = "If the account already has identity information, the deposit is taken as part payment"]
					#[doc = "for the new deposit."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `info`: The identity information."]
					#[doc = ""]
					#[doc = "Emits `IdentitySet` if successful."]
					set_identity {
						info: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::pallet_identity::legacy::IdentityInfo,
						>,
					},
					#[codec(index = 2)]
					#[doc = "Set the sub-accounts of the sender."]
					#[doc = ""]
					#[doc = "Payment: Any aggregate balance reserved by previous `set_subs` calls will be returned"]
					#[doc = "and an amount `SubAccountDeposit` will be reserved for each item in `subs`."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a registered"]
					#[doc = "identity."]
					#[doc = ""]
					#[doc = "- `subs`: The identity's (new) sub-accounts."]
					set_subs {
						subs: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::utils::AccountId32,
							runtime_types::pallet_identity::types::Data,
						)>,
					},
					#[codec(index = 3)]
					#[doc = "Clear an account's identity info and all sub-accounts and return all deposits."]
					#[doc = ""]
					#[doc = "Payment: All reserved balances on the account are returned."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a registered"]
					#[doc = "identity."]
					#[doc = ""]
					#[doc = "Emits `IdentityCleared` if successful."]
					clear_identity,
					#[codec(index = 4)]
					#[doc = "Request a judgement from a registrar."]
					#[doc = ""]
					#[doc = "Payment: At most `max_fee` will be reserved for payment to the registrar if judgement"]
					#[doc = "given."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a"]
					#[doc = "registered identity."]
					#[doc = ""]
					#[doc = "- `reg_index`: The index of the registrar whose judgement is requested."]
					#[doc = "- `max_fee`: The maximum fee that may be paid. This should just be auto-populated as:"]
					#[doc = ""]
					#[doc = "```nocompile"]
					#[doc = "Self::registrars().get(reg_index).unwrap().fee"]
					#[doc = "```"]
					#[doc = ""]
					#[doc = "Emits `JudgementRequested` if successful."]
					request_judgement {
						#[codec(compact)]
						reg_index: ::core::primitive::u32,
						#[codec(compact)]
						max_fee: ::core::primitive::u128,
					},
					#[codec(index = 5)]
					#[doc = "Cancel a previous request."]
					#[doc = ""]
					#[doc = "Payment: A previously reserved deposit is returned on success."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a"]
					#[doc = "registered identity."]
					#[doc = ""]
					#[doc = "- `reg_index`: The index of the registrar whose judgement is no longer requested."]
					#[doc = ""]
					#[doc = "Emits `JudgementUnrequested` if successful."]
					cancel_request { reg_index: ::core::primitive::u32 },
					#[codec(index = 6)]
					#[doc = "Set the fee required for a judgement to be requested from a registrar."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must be the account"]
					#[doc = "of the registrar whose index is `index`."]
					#[doc = ""]
					#[doc = "- `index`: the index of the registrar whose fee is to be set."]
					#[doc = "- `fee`: the new fee."]
					set_fee {
						#[codec(compact)]
						index: ::core::primitive::u32,
						#[codec(compact)]
						fee: ::core::primitive::u128,
					},
					#[codec(index = 7)]
					#[doc = "Change the account associated with a registrar."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must be the account"]
					#[doc = "of the registrar whose index is `index`."]
					#[doc = ""]
					#[doc = "- `index`: the index of the registrar whose fee is to be set."]
					#[doc = "- `new`: the new account ID."]
					set_account_id {
						#[codec(compact)]
						index: ::core::primitive::u32,
						new: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 8)]
					#[doc = "Set the field information for a registrar."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must be the account"]
					#[doc = "of the registrar whose index is `index`."]
					#[doc = ""]
					#[doc = "- `index`: the index of the registrar whose fee is to be set."]
					#[doc = "- `fields`: the fields that the registrar concerns themselves with."]
					set_fields {
						#[codec(compact)]
						index: ::core::primitive::u32,
						fields: ::core::primitive::u64,
					},
					#[codec(index = 9)]
					#[doc = "Provide a judgement for an account's identity."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must be the account"]
					#[doc = "of the registrar whose index is `reg_index`."]
					#[doc = ""]
					#[doc = "- `reg_index`: the index of the registrar whose judgement is being made."]
					#[doc = "- `target`: the account whose identity the judgement is upon. This must be an account"]
					#[doc = "  with a registered identity."]
					#[doc = "- `judgement`: the judgement of the registrar of index `reg_index` about `target`."]
					#[doc = "- `identity`: The hash of the [`IdentityInformationProvider`] for that the judgement is"]
					#[doc = "  provided."]
					#[doc = ""]
					#[doc = "Note: Judgements do not apply to a username."]
					#[doc = ""]
					#[doc = "Emits `JudgementGiven` if successful."]
					provide_judgement {
						#[codec(compact)]
						reg_index: ::core::primitive::u32,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						judgement: runtime_types::pallet_identity::types::Judgement<
							::core::primitive::u128,
						>,
						identity: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 10)]
					#[doc = "Remove an account's identity and sub-account information and slash the deposits."]
					#[doc = ""]
					#[doc = "Payment: Reserved balances from `set_subs` and `set_identity` are slashed and handled by"]
					#[doc = "`Slash`. Verification request deposits are not returned; they should be cancelled"]
					#[doc = "manually using `cancel_request`."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must match `T::ForceOrigin`."]
					#[doc = ""]
					#[doc = "- `target`: the account whose identity the judgement is upon. This must be an account"]
					#[doc = "  with a registered identity."]
					#[doc = ""]
					#[doc = "Emits `IdentityKilled` if successful."]
					kill_identity {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 11)]
					#[doc = "Add the given account to the sender's subs."]
					#[doc = ""]
					#[doc = "Payment: Balance reserved by a previous `set_subs` call for one sub will be repatriated"]
					#[doc = "to the sender."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a registered"]
					#[doc = "sub identity of `sub`."]
					add_sub {
						sub: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						data: runtime_types::pallet_identity::types::Data,
					},
					#[codec(index = 12)]
					#[doc = "Alter the associated name of the given sub-account."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a registered"]
					#[doc = "sub identity of `sub`."]
					rename_sub {
						sub: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						data: runtime_types::pallet_identity::types::Data,
					},
					#[codec(index = 13)]
					#[doc = "Remove the given account from the sender's subs."]
					#[doc = ""]
					#[doc = "Payment: Balance reserved by a previous `set_subs` call for one sub will be repatriated"]
					#[doc = "to the sender."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a registered"]
					#[doc = "sub identity of `sub`."]
					remove_sub {
						sub: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 14)]
					#[doc = "Remove the sender as a sub-account."]
					#[doc = ""]
					#[doc = "Payment: Balance reserved by a previous `set_subs` call for one sub will be repatriated"]
					#[doc = "to the sender (*not* the original depositor)."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have a registered"]
					#[doc = "super-identity."]
					#[doc = ""]
					#[doc = "NOTE: This should not normally be used, but is provided in the case that the non-"]
					#[doc = "controller of an account is maliciously registered as a sub-account."]
					quit_sub,
					#[codec(index = 15)]
					#[doc = "Add an `AccountId` with permission to grant usernames with a given `suffix` appended."]
					#[doc = ""]
					#[doc = "The authority can grant up to `allocation` usernames. To top up their allocation, they"]
					#[doc = "should just issue (or request via governance) a new `add_username_authority` call."]
					add_username_authority {
						authority: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						suffix: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						allocation: ::core::primitive::u32,
					},
					#[codec(index = 16)]
					#[doc = "Remove `authority` from the username authorities."]
					remove_username_authority {
						authority: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 17)]
					#[doc = "Set the username for `who`. Must be called by a username authority."]
					#[doc = ""]
					#[doc = "The authority must have an `allocation`. Users can either pre-sign their usernames or"]
					#[doc = "accept them later."]
					#[doc = ""]
					#[doc = "Usernames must:"]
					#[doc = "  - Only contain lowercase ASCII characters or digits."]
					#[doc = "  - When combined with the suffix of the issuing authority be _less than_ the"]
					#[doc = "    `MaxUsernameLength`."]
					set_username_for {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						username: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						signature:
							::core::option::Option<runtime_types::sp_runtime::MultiSignature>,
					},
					#[codec(index = 18)]
					#[doc = "Accept a given username that an `authority` granted. The call must include the full"]
					#[doc = "username, as in `username.suffix`."]
					accept_username {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 19)]
					#[doc = "Remove an expired username approval. The username was approved by an authority but never"]
					#[doc = "accepted by the user and must now be beyond its expiration. The call must include the"]
					#[doc = "full username, as in `username.suffix`."]
					remove_expired_approval {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 20)]
					#[doc = "Set a given username as the primary. The username should include the suffix."]
					set_primary_username {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 21)]
					#[doc = "Remove a username that corresponds to an account with no identity. Exists when a user"]
					#[doc = "gets a username but then calls `clear_identity`."]
					remove_dangling_username {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Data {
					#[codec(index = 0)]
					None,
					#[codec(index = 1)]
					Raw0([::core::primitive::u8; 0usize]),
					#[codec(index = 2)]
					Raw1([::core::primitive::u8; 1usize]),
					#[codec(index = 3)]
					Raw2([::core::primitive::u8; 2usize]),
					#[codec(index = 4)]
					Raw3([::core::primitive::u8; 3usize]),
					#[codec(index = 5)]
					Raw4([::core::primitive::u8; 4usize]),
					#[codec(index = 6)]
					Raw5([::core::primitive::u8; 5usize]),
					#[codec(index = 7)]
					Raw6([::core::primitive::u8; 6usize]),
					#[codec(index = 8)]
					Raw7([::core::primitive::u8; 7usize]),
					#[codec(index = 9)]
					Raw8([::core::primitive::u8; 8usize]),
					#[codec(index = 10)]
					Raw9([::core::primitive::u8; 9usize]),
					#[codec(index = 11)]
					Raw10([::core::primitive::u8; 10usize]),
					#[codec(index = 12)]
					Raw11([::core::primitive::u8; 11usize]),
					#[codec(index = 13)]
					Raw12([::core::primitive::u8; 12usize]),
					#[codec(index = 14)]
					Raw13([::core::primitive::u8; 13usize]),
					#[codec(index = 15)]
					Raw14([::core::primitive::u8; 14usize]),
					#[codec(index = 16)]
					Raw15([::core::primitive::u8; 15usize]),
					#[codec(index = 17)]
					Raw16([::core::primitive::u8; 16usize]),
					#[codec(index = 18)]
					Raw17([::core::primitive::u8; 17usize]),
					#[codec(index = 19)]
					Raw18([::core::primitive::u8; 18usize]),
					#[codec(index = 20)]
					Raw19([::core::primitive::u8; 19usize]),
					#[codec(index = 21)]
					Raw20([::core::primitive::u8; 20usize]),
					#[codec(index = 22)]
					Raw21([::core::primitive::u8; 21usize]),
					#[codec(index = 23)]
					Raw22([::core::primitive::u8; 22usize]),
					#[codec(index = 24)]
					Raw23([::core::primitive::u8; 23usize]),
					#[codec(index = 25)]
					Raw24([::core::primitive::u8; 24usize]),
					#[codec(index = 26)]
					Raw25([::core::primitive::u8; 25usize]),
					#[codec(index = 27)]
					Raw26([::core::primitive::u8; 26usize]),
					#[codec(index = 28)]
					Raw27([::core::primitive::u8; 27usize]),
					#[codec(index = 29)]
					Raw28([::core::primitive::u8; 28usize]),
					#[codec(index = 30)]
					Raw29([::core::primitive::u8; 29usize]),
					#[codec(index = 31)]
					Raw30([::core::primitive::u8; 30usize]),
					#[codec(index = 32)]
					Raw31([::core::primitive::u8; 31usize]),
					#[codec(index = 33)]
					Raw32([::core::primitive::u8; 32usize]),
					#[codec(index = 34)]
					BlakeTwo256([::core::primitive::u8; 32usize]),
					#[codec(index = 35)]
					Sha256([::core::primitive::u8; 32usize]),
					#[codec(index = 36)]
					Keccak256([::core::primitive::u8; 32usize]),
					#[codec(index = 37)]
					ShaThree256([::core::primitive::u8; 32usize]),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Judgement<_0> {
					#[codec(index = 0)]
					Unknown,
					#[codec(index = 1)]
					FeePaid(_0),
					#[codec(index = 2)]
					Reasonable,
					#[codec(index = 3)]
					KnownGood,
					#[codec(index = 4)]
					OutOfDate,
					#[codec(index = 5)]
					LowQuality,
					#[codec(index = 6)]
					Erroneous,
				}
			}
		}
		pub mod pallet_identity_management {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "add an account to the delegatees"]
					add_delegatee { account: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 1)]
					#[doc = "remove an account from the delegatees"]
					remove_delegatee { account: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 3)]
					#[doc = "Link an identity with given network types and validation data."]
					#[doc = "We do the origin check for this extrinsic, it has to be"]
					#[doc = "- either the caller themselves, i.e. ensure_signed(origin)? == who"]
					#[doc = "- or from a delegatee in the list"]
					#[doc = ""]
					#[doc = "`encrypted_web3networks`:"]
					#[doc = "an initial list of web3 networks on which the identity is used:"]
					#[doc = "Vec<Web3Network> encrypted with TEE's shielding key. In fact, it"]
					#[doc = "doesn't have to be encrypted as it's a finite set and you can sort"]
					#[doc = "it out by enumerating all possible combinations. But still, to keep"]
					#[doc = "it consistent with identities and validation data."]
					#[doc = ""]
					#[doc = "The networks must match the identity type, it means:"]
					#[doc = "- for web2 identity, Vec<Web3Network> must be empty;"]
					#[doc = "- for substrate identity, Vec<Web3Network> must have substrate networks only;"]
					#[doc = "- for evm identity, Vec<Web3Network> must have evm networks only."]
					#[doc = "Otherwise the linking will fail."]
					link_identity {
						shard: ::subxt::ext::subxt_core::utils::H256,
						user: ::subxt::ext::subxt_core::utils::AccountId32,
						encrypted_identity:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						encrypted_validation_data:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						encrypted_web3networks:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 4)]
					#[doc = "Deactivate an identity"]
					deactivate_identity {
						shard: ::subxt::ext::subxt_core::utils::H256,
						encrypted_identity:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 5)]
					#[doc = "Activate an identity"]
					activate_identity {
						shard: ::subxt::ext::subxt_core::utils::H256,
						encrypted_identity:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 6)]
					#[doc = "Register an OIDC client"]
					register_oidc_client {
						redirect_uris: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					},
					#[codec(index = 7)]
					#[doc = "Unregister an OIDC client"]
					unregister_oidc_client,
					#[codec(index = 31)]
					#[doc = "---------------------------------------------------"]
					#[doc = "The following extrinsics are supposed to be called by TEE only"]
					#[doc = "---------------------------------------------------"]
					identity_linked {
						prime_identity: runtime_types::core_primitives::identity::Identity,
						id_graph_hash: ::subxt::ext::subxt_core::utils::H256,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 32)]
					identity_deactivated {
						prime_identity: runtime_types::core_primitives::identity::Identity,
						id_graph_hash: ::subxt::ext::subxt_core::utils::H256,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 33)]
					identity_activated {
						prime_identity: runtime_types::core_primitives::identity::Identity,
						id_graph_hash: ::subxt::ext::subxt_core::utils::H256,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 34)]
					identity_networks_set {
						prime_identity: runtime_types::core_primitives::identity::Identity,
						id_graph_hash: ::subxt::ext::subxt_core::utils::H256,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 35)]
					some_error {
						prime_identity: ::core::option::Option<
							runtime_types::core_primitives::identity::Identity,
						>,
						error: runtime_types::core_primitives::error::IMPError,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
				}
			}
		}
		pub mod pallet_investing_pool {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Create a investing pool"]
					#[doc = "Admin should be guardian multisig"]
					create_investing_pool {
						pool_id: ::core::primitive::u128,
						setting: runtime_types::pallet_collab_ai_common::PoolSetting<
							::subxt::ext::subxt_core::utils::AccountId32,
							::core::primitive::u32,
							::core::primitive::u128,
						>,
						admin: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 1)]
					#[doc = "Update a reward for an investing pool of specific epoch"]
					#[doc = "Each epoch can be only updated once"]
					#[doc = "Pool admin will transfer its AIUSD into pool accordingly"]
					update_reward {
						pool_id: ::core::primitive::u128,
						epoch: ::core::primitive::u128,
						reward: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					claim { asset_id: ::core::primitive::u128, amount: ::core::primitive::u128 },
					#[codec(index = 3)]
					regist_aiusd { asset_id: ::core::primitive::u128 },
					#[codec(index = 4)]
					regist_can { asset_id: ::core::primitive::u128 },
				}
			}
		}
		pub mod pallet_membership {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Add a member `who` to the set."]
					#[doc = ""]
					#[doc = "May only be called from `T::AddOrigin`."]
					add_member {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 1)]
					#[doc = "Remove a member `who` from the set."]
					#[doc = ""]
					#[doc = "May only be called from `T::RemoveOrigin`."]
					remove_member {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 2)]
					#[doc = "Swap out one member `remove` for another `add`."]
					#[doc = ""]
					#[doc = "May only be called from `T::SwapOrigin`."]
					#[doc = ""]
					#[doc = "Prime membership is *not* passed from `remove` to `add`, if extant."]
					swap_member {
						remove: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						add: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 3)]
					#[doc = "Change the membership to a new set, disregarding the existing membership. Be nice and"]
					#[doc = "pass `members` pre-sorted."]
					#[doc = ""]
					#[doc = "May only be called from `T::ResetOrigin`."]
					reset_members {
						members: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
					},
					#[codec(index = 4)]
					#[doc = "Swap out the sending member for some other key `new`."]
					#[doc = ""]
					#[doc = "May only be called from `Signed` origin of a current member."]
					#[doc = ""]
					#[doc = "Prime membership is passed from the origin account to `new`, if extant."]
					change_key {
						new: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 5)]
					#[doc = "Set the prime member. Must be a current member."]
					#[doc = ""]
					#[doc = "May only be called from `T::PrimeOrigin`."]
					set_prime {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 6)]
					#[doc = "Remove the prime member if it exists."]
					#[doc = ""]
					#[doc = "May only be called from `T::PrimeOrigin`."]
					clear_prime,
				}
			}
		}
		pub mod pallet_message_queue {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Remove a page which has no more messages remaining to be processed or is stale."]
					reap_page {
						message_origin:
							runtime_types::cumulus_primitives_core::AggregateMessageOrigin,
						page_index: ::core::primitive::u32,
					},
					#[codec(index = 1)]
					#[doc = "Execute an overweight message."]
					#[doc = ""]
					#[doc = "Temporary processing errors will be propagated whereas permanent errors are treated"]
					#[doc = "as success condition."]
					#[doc = ""]
					#[doc = "- `origin`: Must be `Signed`."]
					#[doc = "- `message_origin`: The origin from which the message to be executed arrived."]
					#[doc = "- `page`: The page in the queue in which the message to be executed is sitting."]
					#[doc = "- `index`: The index into the queue of the message to be executed."]
					#[doc = "- `weight_limit`: The maximum amount of weight allowed to be consumed in the execution"]
					#[doc = "  of the message."]
					#[doc = ""]
					#[doc = "Benchmark complexity considerations: O(index + weight_limit)."]
					execute_overweight {
						message_origin:
							runtime_types::cumulus_primitives_core::AggregateMessageOrigin,
						page: ::core::primitive::u32,
						index: ::core::primitive::u32,
						weight_limit: runtime_types::sp_weights::weight_v2::Weight,
					},
				}
			}
		}
		pub mod pallet_multisig {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Immediately dispatch a multi-signature call using a single approval from the caller."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `other_signatories`: The accounts (other than the sender) who are part of the"]
					#[doc = "multi-signature, but do not participate in the approval process."]
					#[doc = "- `call`: The call to be executed."]
					#[doc = ""]
					#[doc = "Result is equivalent to the dispatched result."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "O(Z + C) where Z is the length of the call and C its execution weight."]
					as_multi_threshold_1 {
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Register approval for a dispatch to be made from a deterministic composite account if"]
					#[doc = "approved by a total of `threshold - 1` of `other_signatories`."]
					#[doc = ""]
					#[doc = "If there are enough, then dispatch the call."]
					#[doc = ""]
					#[doc = "Payment: `DepositBase` will be reserved if this is the first approval, plus"]
					#[doc = "`threshold` times `DepositFactor`. It is returned once this dispatch happens or"]
					#[doc = "is cancelled."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `threshold`: The total number of approvals for this dispatch before it is executed."]
					#[doc = "- `other_signatories`: The accounts (other than the sender) who can approve this"]
					#[doc = "dispatch. May not be empty."]
					#[doc = "- `maybe_timepoint`: If this is the first approval, then this must be `None`. If it is"]
					#[doc = "not the first approval, then it must be `Some`, with the timepoint (block number and"]
					#[doc = "transaction index) of the first approval transaction."]
					#[doc = "- `call`: The call to be executed."]
					#[doc = ""]
					#[doc = "NOTE: Unless this is the final approval, you will generally want to use"]
					#[doc = "`approve_as_multi` instead, since it only requires a hash of the call."]
					#[doc = ""]
					#[doc = "Result is equivalent to the dispatched result if `threshold` is exactly `1`. Otherwise"]
					#[doc = "on success, result is `Ok` and the result from the interior call, if it was executed,"]
					#[doc = "may be found in the deposited `MultisigExecuted` event."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(S + Z + Call)`."]
					#[doc = "- Up to one balance-reserve or unreserve operation."]
					#[doc = "- One passthrough operation, one insert, both `O(S)` where `S` is the number of"]
					#[doc = "  signatories. `S` is capped by `MaxSignatories`, with weight being proportional."]
					#[doc = "- One call encode & hash, both of complexity `O(Z)` where `Z` is tx-len."]
					#[doc = "- One encode & hash, both of complexity `O(S)`."]
					#[doc = "- Up to one binary search and insert (`O(logS + S)`)."]
					#[doc = "- I/O: 1 read `O(S)`, up to 1 mutate `O(S)`. Up to one remove."]
					#[doc = "- One event."]
					#[doc = "- The weight of the `call`."]
					#[doc = "- Storage: inserts one item, value size bounded by `MaxSignatories`, with a deposit"]
					#[doc = "  taken for its lifetime of `DepositBase + threshold * DepositFactor`."]
					as_multi {
						threshold: ::core::primitive::u16,
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
						maybe_timepoint: ::core::option::Option<
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
						max_weight: runtime_types::sp_weights::weight_v2::Weight,
					},
					#[codec(index = 2)]
					#[doc = "Register approval for a dispatch to be made from a deterministic composite account if"]
					#[doc = "approved by a total of `threshold - 1` of `other_signatories`."]
					#[doc = ""]
					#[doc = "Payment: `DepositBase` will be reserved if this is the first approval, plus"]
					#[doc = "`threshold` times `DepositFactor`. It is returned once this dispatch happens or"]
					#[doc = "is cancelled."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `threshold`: The total number of approvals for this dispatch before it is executed."]
					#[doc = "- `other_signatories`: The accounts (other than the sender) who can approve this"]
					#[doc = "dispatch. May not be empty."]
					#[doc = "- `maybe_timepoint`: If this is the first approval, then this must be `None`. If it is"]
					#[doc = "not the first approval, then it must be `Some`, with the timepoint (block number and"]
					#[doc = "transaction index) of the first approval transaction."]
					#[doc = "- `call_hash`: The hash of the call to be executed."]
					#[doc = ""]
					#[doc = "NOTE: If this is the final approval, you will want to use `as_multi` instead."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(S)`."]
					#[doc = "- Up to one balance-reserve or unreserve operation."]
					#[doc = "- One passthrough operation, one insert, both `O(S)` where `S` is the number of"]
					#[doc = "  signatories. `S` is capped by `MaxSignatories`, with weight being proportional."]
					#[doc = "- One encode & hash, both of complexity `O(S)`."]
					#[doc = "- Up to one binary search and insert (`O(logS + S)`)."]
					#[doc = "- I/O: 1 read `O(S)`, up to 1 mutate `O(S)`. Up to one remove."]
					#[doc = "- One event."]
					#[doc = "- Storage: inserts one item, value size bounded by `MaxSignatories`, with a deposit"]
					#[doc = "  taken for its lifetime of `DepositBase + threshold * DepositFactor`."]
					approve_as_multi {
						threshold: ::core::primitive::u16,
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
						maybe_timepoint: ::core::option::Option<
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						>,
						call_hash: [::core::primitive::u8; 32usize],
						max_weight: runtime_types::sp_weights::weight_v2::Weight,
					},
					#[codec(index = 3)]
					#[doc = "Cancel a pre-existing, on-going multisig transaction. Any deposit reserved previously"]
					#[doc = "for this operation will be unreserved on success."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `threshold`: The total number of approvals for this dispatch before it is executed."]
					#[doc = "- `other_signatories`: The accounts (other than the sender) who can approve this"]
					#[doc = "dispatch. May not be empty."]
					#[doc = "- `timepoint`: The timepoint (block number and transaction index) of the first approval"]
					#[doc = "transaction for this dispatch."]
					#[doc = "- `call_hash`: The hash of the call to be executed."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(S)`."]
					#[doc = "- Up to one balance-reserve or unreserve operation."]
					#[doc = "- One passthrough operation, one insert, both `O(S)` where `S` is the number of"]
					#[doc = "  signatories. `S` is capped by `MaxSignatories`, with weight being proportional."]
					#[doc = "- One encode & hash, both of complexity `O(S)`."]
					#[doc = "- One event."]
					#[doc = "- I/O: 1 read `O(S)`, one remove."]
					#[doc = "- Storage: removes one item."]
					cancel_as_multi {
						threshold: ::core::primitive::u16,
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
						timepoint:
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						call_hash: [::core::primitive::u8; 32usize],
					},
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct Timepoint<_0> {
				pub height: _0,
				pub index: ::core::primitive::u32,
			}
		}
		pub mod pallet_omni_account {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					dispatch_as_omni_account {
						member_account_hash: ::subxt::ext::subxt_core::utils::H256,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
						auth_type:
							runtime_types::core_primitives::omni_account::OmniAccountAuthType,
					},
					#[codec(index = 1)]
					dispatch_as_signed {
						member_account_hash: ::subxt::ext::subxt_core::utils::H256,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
						auth_type:
							runtime_types::core_primitives::omni_account::OmniAccountAuthType,
					},
					#[codec(index = 2)]
					create_account_store {
						identity: runtime_types::core_primitives::identity::Identity,
					},
					#[codec(index = 3)]
					add_account {
						member_account: runtime_types::core_primitives::omni_account::MemberAccount,
						permissions: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<
								runtime_types::paseo_parachain_runtime::OmniAccountPermission,
							>,
						>,
					},
					#[codec(index = 4)]
					remove_accounts {
						member_account_hashes: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::H256,
						>,
					},
					#[codec(index = 5)]
					#[doc = "make a member account public in the AccountStore"]
					#[doc = "we force `Identity` type to avoid misuse and additional check"]
					publicize_account {
						member_account: runtime_types::core_primitives::identity::Identity,
					},
					#[codec(index = 6)]
					request_intent { intent: runtime_types::core_primitives::intent::Intent },
					#[codec(index = 7)]
					#[doc = "temporary extrinsic to upload the existing IDGraph from the worker onto chain"]
					update_account_store_by_one {
						who: runtime_types::core_primitives::identity::Identity,
						member_account: runtime_types::core_primitives::omni_account::MemberAccount,
					},
					#[codec(index = 8)]
					intent_executed {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						intent: runtime_types::core_primitives::intent::Intent,
						result: runtime_types::pallet_omni_account::pallet::IntentExecutionResult,
					},
					#[codec(index = 9)]
					set_permissions {
						member_account_hash: ::subxt::ext::subxt_core::utils::H256,
						permissions: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::paseo_parachain_runtime::OmniAccountPermission,
						>,
					},
					#[codec(index = 10)]
					auth_token_requested {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						expires_at: ::core::primitive::u32,
					},
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "The `Error` enum of this pallet."]
				pub enum Error {
					#[codec(index = 0)]
					AccountAlreadyAdded,
					#[codec(index = 1)]
					AccountStoreLenLimitReached,
					#[codec(index = 2)]
					AccountNotFound,
					#[codec(index = 3)]
					InvalidAccount,
					#[codec(index = 4)]
					UnknownAccountStore,
					#[codec(index = 5)]
					EmptyAccount,
					#[codec(index = 6)]
					NoPermission,
					#[codec(index = 7)]
					PermissionsLenLimitReached,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "The `Event` enum of this pallet"]
				pub enum Event {
					#[codec(index = 0)]
					#[doc = "An account store is created"]
					AccountStoreCreated { who: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 1)]
					#[doc = "Some member account is added"]
					AccountAdded {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						member_account_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 2)]
					#[doc = "Some member accounts are removed"]
					AccountRemoved {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						member_account_hashes: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::H256,
						>,
					},
					#[codec(index = 3)]
					#[doc = "Some member account is made public"]
					AccountMadePublic {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						member_account_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 4)]
					#[doc = "An account store is updated"]
					AccountStoreUpdated {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						account_store: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::core_primitives::omni_account::MemberAccount,
						>,
					},
					#[codec(index = 5)]
					#[doc = "Some call is dispatched as omni-account origin"]
					DispatchedAsOmniAccount {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						auth_type:
							runtime_types::core_primitives::omni_account::OmniAccountAuthType,
						result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
					#[codec(index = 6)]
					#[doc = "Some call is dispatched as signed origin"]
					DispatchedAsSigned {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						auth_type:
							runtime_types::core_primitives::omni_account::OmniAccountAuthType,
						result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
					#[codec(index = 7)]
					#[doc = "Intent is requested"]
					IntentRequested {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						intent: runtime_types::core_primitives::intent::Intent,
					},
					#[codec(index = 8)]
					#[doc = "Intent is executed"]
					IntentExecuted {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						intent: runtime_types::core_primitives::intent::Intent,
						result: runtime_types::pallet_omni_account::pallet::IntentExecutionResult,
					},
					#[codec(index = 9)]
					#[doc = "Member permission set"]
					AccountPermissionsSet {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						member_account_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 10)]
					#[doc = "An auth token is requested"]
					AuthTokenRequested {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						expires_at: ::core::primitive::u32,
					},
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum IntentExecutionResult {
					#[codec(index = 0)]
					Success,
					#[codec(index = 1)]
					Failure,
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum RawOrigin<_0> {
				#[codec(index = 0)]
				OmniAccount(_0),
				#[codec(index = 1)]
				OmniAccountMembers(_0, ::core::primitive::u32, ::core::primitive::u32),
			}
		}
		pub mod pallet_omni_bridge {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					# [codec (index = 0)] set_admin { new_admin : :: subxt :: ext :: subxt_core :: utils :: AccountId32 , } , # [codec (index = 1)] pay_in { req : runtime_types :: pallet_omni_bridge :: PayInRequest < runtime_types :: frame_support :: traits :: tokens :: fungible :: union_of :: NativeOrWithId < :: core :: primitive :: u128 > , :: core :: primitive :: u128 > , } , # [codec (index = 2)] request_pay_out { req : runtime_types :: pallet_omni_bridge :: PayOutRequest < :: subxt :: ext :: subxt_core :: utils :: AccountId32 , :: core :: primitive :: u128 > , aye : :: core :: primitive :: bool , } , # [codec (index = 3)] add_relayer { who : :: subxt :: ext :: subxt_core :: utils :: AccountId32 , } , # [codec (index = 4)] remove_relayer { who : :: subxt :: ext :: subxt_core :: utils :: AccountId32 , } , # [codec (index = 5)] set_pay_in_fee { asset : runtime_types :: frame_support :: traits :: tokens :: fungible :: union_of :: NativeOrWithId < :: core :: primitive :: u128 > , dest_chain : runtime_types :: pallet_omni_bridge :: ChainType , fee : :: core :: primitive :: u128 , } , # [codec (index = 6)] set_resource_id { resource_id : [:: core :: primitive :: u8 ; 32usize] , chain_asset : runtime_types :: pallet_omni_bridge :: ChainAsset < runtime_types :: frame_support :: traits :: tokens :: fungible :: union_of :: NativeOrWithId < :: core :: primitive :: u128 > > , } , # [codec (index = 7)] remove_resource_id { resource_id : [:: core :: primitive :: u8 ; 32usize] , } , # [codec (index = 8)] set_relayer_threshold { threshold : :: core :: primitive :: u32 , } , # [codec (index = 9)] add_pay_in_pair { asset : runtime_types :: frame_support :: traits :: tokens :: fungible :: union_of :: NativeOrWithId < :: core :: primitive :: u128 > , dest_chain : runtime_types :: pallet_omni_bridge :: ChainType , } , # [codec (index = 10)] remove_pay_in_pair { asset : runtime_types :: frame_support :: traits :: tokens :: fungible :: union_of :: NativeOrWithId < :: core :: primitive :: u128 > , dest_chain : runtime_types :: pallet_omni_bridge :: ChainType , } , }
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct ChainAsset<_0> {
				pub chain: runtime_types::pallet_omni_bridge::ChainType,
				pub asset: _0,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum ChainType {
				#[codec(index = 0)]
				Heima,
				#[codec(index = 1)]
				Ethereum(::core::primitive::u32),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct PayInRequest<_0, _1> {
				pub asset: _0,
				pub dest_chain: runtime_types::pallet_omni_bridge::ChainType,
				pub dest_account: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				pub amount: _1,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct PayOutRequest<_0, _1> {
				pub source_chain: runtime_types::pallet_omni_bridge::ChainType,
				pub nonce: ::core::primitive::u64,
				pub resource_id: [::core::primitive::u8; 32usize],
				pub dest_account: _0,
				pub amount: _1,
			}
		}
		pub mod pallet_parachain_staking {
			use super::runtime_types;
			pub mod inflation {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Range<_0> {
					pub min: _0,
					pub ideal: _0,
					pub max: _0,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Set the expectations for total staked. These expectations determine the issuance for"]
					#[doc = "the round according to logic in `fn compute_issuance`"]
					set_staking_expectations {
						expectations: runtime_types::pallet_parachain_staking::inflation::Range<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Set the annual inflation rate to derive per-round inflation"]
					set_inflation {
						schedule: runtime_types::pallet_parachain_staking::inflation::Range<
							runtime_types::sp_arithmetic::per_things::Perbill,
						>,
					},
					#[codec(index = 2)]
					#[doc = "Set the account that will hold funds set aside for parachain bond"]
					set_parachain_bond_account { new: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 3)]
					#[doc = "Set the percent of inflation set aside for parachain bond"]
					set_parachain_bond_reserve_percent {
						new: runtime_types::sp_arithmetic::per_things::Percent,
					},
					#[codec(index = 4)]
					#[doc = "Set the total number of collator candidates selected per round"]
					#[doc = "- changes are not applied until the start of the next round"]
					set_total_selected { new: ::core::primitive::u32 },
					#[codec(index = 5)]
					#[doc = "Set the commission for all collators"]
					set_collator_commission {
						new: runtime_types::sp_arithmetic::per_things::Perbill,
					},
					#[codec(index = 6)]
					#[doc = "Set blocks per round"]
					#[doc = "- if called with `new` less than length of current round, will transition immediately"]
					#[doc = "in the next block"]
					#[doc = "- also updates per-round inflation config"]
					set_blocks_per_round { new: ::core::primitive::u32 },
					#[codec(index = 7)]
					#[doc = "add white list of candidates"]
					#[doc = "This function should be safe to delete after restriction removed"]
					add_candidates_whitelist {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 8)]
					#[doc = "remove white list of candidates"]
					#[doc = "This function should be safe to delete after restriction removed"]
					remove_candidates_whitelist {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 9)]
					#[doc = "Join the set of collator candidates"]
					join_candidates { bond: ::core::primitive::u128 },
					#[codec(index = 10)]
					#[doc = "Request to leave the set of candidates. If successful, the account is immediately"]
					#[doc = "removed from the candidate pool to prevent selection as a collator."]
					schedule_leave_candidates,
					#[codec(index = 11)]
					#[doc = "Execute leave candidates request"]
					execute_leave_candidates {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 12)]
					#[doc = "Cancel open request to leave candidates"]
					#[doc = "- only callable by collator account"]
					#[doc = "- result upon successful call is the candidate is active in the candidate pool"]
					cancel_leave_candidates,
					#[codec(index = 13)]
					#[doc = "Temporarily leave the set of collator candidates without unbonding"]
					go_offline,
					#[codec(index = 14)]
					#[doc = "Rejoin the set of collator candidates if previously had called `go_offline`"]
					go_online,
					#[codec(index = 15)]
					#[doc = "Increase collator candidate self bond by `more`"]
					candidate_bond_more { more: ::core::primitive::u128 },
					#[codec(index = 16)]
					#[doc = "Request by collator candidate to decrease self bond by `less`"]
					schedule_candidate_bond_less { less: ::core::primitive::u128 },
					#[codec(index = 17)]
					#[doc = "Execute pending request to adjust the collator candidate self bond"]
					execute_candidate_bond_less {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 18)]
					#[doc = "Cancel pending request to adjust the collator candidate self bond"]
					cancel_candidate_bond_less,
					#[codec(index = 19)]
					#[doc = "If caller is not a delegator and not a collator, then join the set of delegators"]
					#[doc = "If caller is a delegator, then makes delegation to change their delegation state"]
					delegate {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 20)]
					#[doc = "If caller is not a delegator and not a collator, then join the set of delegators"]
					#[doc = "If caller is a delegator, then makes delegation to change their delegation state"]
					#[doc = "Sets the auto-compound config for the delegation"]
					delegate_with_auto_compound {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
						amount: ::core::primitive::u128,
						auto_compound: runtime_types::sp_arithmetic::per_things::Percent,
					},
					#[codec(index = 21)]
					#[doc = "Request to leave the set of delegators. If successful, the caller is scheduled to be"]
					#[doc = "allowed to exit via a [DelegationAction::Revoke] towards all existing delegations."]
					#[doc = "Success forbids future delegation requests until the request is invoked or cancelled."]
					schedule_leave_delegators,
					#[codec(index = 22)]
					#[doc = "Execute the right to exit the set of delegators and revoke all ongoing delegations."]
					execute_leave_delegators {
						delegator: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 23)]
					#[doc = "Cancel a pending request to exit the set of delegators. Success clears the pending exit"]
					#[doc = "request (thereby resetting the delay upon another `leave_delegators` call)."]
					cancel_leave_delegators,
					#[codec(index = 24)]
					#[doc = "Request to revoke an existing delegation. If successful, the delegation is scheduled"]
					#[doc = "to be allowed to be revoked via the `execute_delegation_request` extrinsic."]
					schedule_revoke_delegation {
						collator: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 25)]
					#[doc = "Bond more for delegators wrt a specific collator candidate."]
					delegator_bond_more {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
						more: ::core::primitive::u128,
					},
					#[codec(index = 26)]
					#[doc = "Request bond less for delegators wrt a specific collator candidate."]
					schedule_delegator_bond_less {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
						less: ::core::primitive::u128,
					},
					#[codec(index = 27)]
					#[doc = "Execute pending request to change an existing delegation"]
					execute_delegation_request {
						delegator: ::subxt::ext::subxt_core::utils::AccountId32,
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 28)]
					#[doc = "Cancel request to change an existing delegation."]
					cancel_delegation_request {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 29)]
					#[doc = "Sets the auto-compounding reward percentage for a delegation."]
					set_auto_compound {
						candidate: ::subxt::ext::subxt_core::utils::AccountId32,
						value: runtime_types::sp_arithmetic::per_things::Percent,
					},
				}
			}
		}
		pub mod pallet_pool_proposal {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Curator propose a investing pool"]
					#[doc = ""]
					#[doc = "max_pool_size: At most this amount of raised money curator/investing pool willing to take"]
					#[doc = "proposal_last_time: How long does the proposal lasts for voting/preinvesting."]
					#[doc = "                    All ProposalStatusFlags must be satisfied after this period passed, which is also"]
					#[doc = "\t\t\t\t\tthe approximate date when pool begins."]
					#[doc = "pool_last_time: How long does the investing pool last if passed"]
					#[doc = "estimated_pool_reward: This number is only for displaying purpose without any techinical meaning"]
					#[doc = "pool_info_hash: Hash of pool info for including pool details"]
					propose_investing_pool {
						max_pool_size: ::core::primitive::u128,
						proposal_last_time: ::core::primitive::u32,
						pool_last_time: ::core::primitive::u32,
						estimated_pool_reward: ::core::primitive::u128,
						pool_info_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 1)]
					pre_stake_proposal {
						pool_proposal_index: ::core::primitive::u128,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					withdraw_pre_investing {
						pool_proposal_index: ::core::primitive::u128,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					public_vote_proposal {
						pool_proposal_index: ::core::primitive::u128,
						vote: ::core::primitive::bool,
					},
					#[codec(index = 4)]
					guardian_participate_proposal { pool_proposal_index: ::core::primitive::u128 },
					#[codec(index = 5)]
					bake_proposal,
					#[codec(index = 6)]
					dissolve_proposal,
				}
			}
		}
		pub mod pallet_preimage {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Register a preimage on-chain."]
					#[doc = ""]
					#[doc = "If the preimage was previously requested, no fees or deposits are taken for providing"]
					#[doc = "the preimage. Otherwise, a deposit is taken proportional to the size of the preimage."]
					note_preimage {
						bytes: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 1)]
					#[doc = "Clear an unrequested preimage from the runtime storage."]
					#[doc = ""]
					#[doc = "If `len` is provided, then it will be a much cheaper operation."]
					#[doc = ""]
					#[doc = "- `hash`: The hash of the preimage to be removed from the store."]
					#[doc = "- `len`: The length of the preimage of `hash`."]
					unnote_preimage { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					#[doc = "Request a preimage be uploaded to the chain without paying any fees or deposits."]
					#[doc = ""]
					#[doc = "If the preimage requests has already been provided on-chain, we unreserve any deposit"]
					#[doc = "a user may have paid, and take the control of the preimage out of their hands."]
					request_preimage { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 3)]
					#[doc = "Clear a previously made request for a preimage."]
					#[doc = ""]
					#[doc = "NOTE: THIS MUST NOT BE CALLED ON `hash` MORE TIMES THAN `request_preimage`."]
					unrequest_preimage { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 4)]
					#[doc = "Ensure that the a bulk of pre-images is upgraded."]
					#[doc = ""]
					#[doc = "The caller pays no fee if at least 90% of pre-images were successfully updated."]
					ensure_updated {
						hashes: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::H256,
						>,
					},
				}
			}
		}
		pub mod pallet_proxy {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Dispatch the given `call` from an account that the sender is authorised for through"]
					#[doc = "`add_proxy`."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `real`: The account that the proxy will make a call on behalf of."]
					#[doc = "- `force_proxy_type`: Specify the exact proxy type to be used and checked for this call."]
					#[doc = "- `call`: The call to be made by the `real` account."]
					proxy {
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						force_proxy_type: ::core::option::Option<
							runtime_types::paseo_parachain_runtime::ProxyType,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Register a proxy account for the sender that is able to make calls on its behalf."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `proxy`: The account that the `caller` would like to make a proxy."]
					#[doc = "- `proxy_type`: The permissions allowed for this proxy account."]
					#[doc = "- `delay`: The announcement period required of the initial proxy. Will generally be"]
					#[doc = "zero."]
					add_proxy {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						proxy_type: runtime_types::paseo_parachain_runtime::ProxyType,
						delay: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					#[doc = "Unregister a proxy account for the sender."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `proxy`: The account that the `caller` would like to remove as a proxy."]
					#[doc = "- `proxy_type`: The permissions currently enabled for the removed proxy account."]
					remove_proxy {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						proxy_type: runtime_types::paseo_parachain_runtime::ProxyType,
						delay: ::core::primitive::u32,
					},
					#[codec(index = 3)]
					#[doc = "Unregister all proxy accounts for the sender."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "WARNING: This may be called on accounts created by `pure`, however if done, then"]
					#[doc = "the unreserved fees will be inaccessible. **All access to this account will be lost.**"]
					remove_proxies,
					#[codec(index = 4)]
					#[doc = "Spawn a fresh new account that is guaranteed to be otherwise inaccessible, and"]
					#[doc = "initialize it with a proxy of `proxy_type` for `origin` sender."]
					#[doc = ""]
					#[doc = "Requires a `Signed` origin."]
					#[doc = ""]
					#[doc = "- `proxy_type`: The type of the proxy that the sender will be registered as over the"]
					#[doc = "new account. This will almost always be the most permissive `ProxyType` possible to"]
					#[doc = "allow for maximum flexibility."]
					#[doc = "- `index`: A disambiguation index, in case this is called multiple times in the same"]
					#[doc = "transaction (e.g. with `utility::batch`). Unless you're using `batch` you probably just"]
					#[doc = "want to use `0`."]
					#[doc = "- `delay`: The announcement period required of the initial proxy. Will generally be"]
					#[doc = "zero."]
					#[doc = ""]
					#[doc = "Fails with `Duplicate` if this has already been called in this transaction, from the"]
					#[doc = "same sender, with the same parameters."]
					#[doc = ""]
					#[doc = "Fails if there are insufficient funds to pay for deposit."]
					create_pure {
						proxy_type: runtime_types::paseo_parachain_runtime::ProxyType,
						delay: ::core::primitive::u32,
						index: ::core::primitive::u16,
					},
					#[codec(index = 5)]
					#[doc = "Removes a previously spawned pure proxy."]
					#[doc = ""]
					#[doc = "WARNING: **All access to this account will be lost.** Any funds held in it will be"]
					#[doc = "inaccessible."]
					#[doc = ""]
					#[doc = "Requires a `Signed` origin, and the sender account must have been created by a call to"]
					#[doc = "`pure` with corresponding parameters."]
					#[doc = ""]
					#[doc = "- `spawner`: The account that originally called `pure` to create this account."]
					#[doc = "- `index`: The disambiguation index originally passed to `pure`. Probably `0`."]
					#[doc = "- `proxy_type`: The proxy type originally passed to `pure`."]
					#[doc = "- `height`: The height of the chain when the call to `pure` was processed."]
					#[doc = "- `ext_index`: The extrinsic index in which the call to `pure` was processed."]
					#[doc = ""]
					#[doc = "Fails with `NoPermission` in case the caller is not a previously created pure"]
					#[doc = "account whose `pure` call has corresponding parameters."]
					kill_pure {
						spawner: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						proxy_type: runtime_types::paseo_parachain_runtime::ProxyType,
						index: ::core::primitive::u16,
						#[codec(compact)]
						height: ::core::primitive::u32,
						#[codec(compact)]
						ext_index: ::core::primitive::u32,
					},
					#[codec(index = 6)]
					#[doc = "Publish the hash of a proxy-call that will be made in the future."]
					#[doc = ""]
					#[doc = "This must be called some number of blocks before the corresponding `proxy` is attempted"]
					#[doc = "if the delay associated with the proxy relationship is greater than zero."]
					#[doc = ""]
					#[doc = "No more than `MaxPending` announcements may be made at any one time."]
					#[doc = ""]
					#[doc = "This will take a deposit of `AnnouncementDepositFactor` as well as"]
					#[doc = "`AnnouncementDepositBase` if there are no other pending announcements."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and a proxy of `real`."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `real`: The account that the proxy will make a call on behalf of."]
					#[doc = "- `call_hash`: The hash of the call to be made by the `real` account."]
					announce {
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						call_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 7)]
					#[doc = "Remove a given announcement."]
					#[doc = ""]
					#[doc = "May be called by a proxy account to remove a call they previously announced and return"]
					#[doc = "the deposit."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `real`: The account that the proxy will make a call on behalf of."]
					#[doc = "- `call_hash`: The hash of the call to be made by the `real` account."]
					remove_announcement {
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						call_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 8)]
					#[doc = "Remove the given announcement of a delegate."]
					#[doc = ""]
					#[doc = "May be called by a target (proxied) account to remove a call that one of their delegates"]
					#[doc = "(`delegate`) has announced they want to execute. The deposit is returned."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `delegate`: The account that previously announced the call."]
					#[doc = "- `call_hash`: The hash of the call to be made."]
					reject_announcement {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						call_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 9)]
					#[doc = "Dispatch the given `call` from an account that the sender is authorized for through"]
					#[doc = "`add_proxy`."]
					#[doc = ""]
					#[doc = "Removes any corresponding announcement(s)."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `real`: The account that the proxy will make a call on behalf of."]
					#[doc = "- `force_proxy_type`: Specify the exact proxy type to be used and checked for this call."]
					#[doc = "- `call`: The call to be made by the `real` account."]
					proxy_announced {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						force_proxy_type: ::core::option::Option<
							runtime_types::paseo_parachain_runtime::ProxyType,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
				}
			}
		}
		pub mod pallet_referenda {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Propose a referendum on a privileged action."]
					#[doc = ""]
					#[doc = "- `origin`: must be `SubmitOrigin` and the account must have `SubmissionDeposit` funds"]
					#[doc = "  available."]
					#[doc = "- `proposal_origin`: The origin from which the proposal should be executed."]
					#[doc = "- `proposal`: The proposal."]
					#[doc = "- `enactment_moment`: The moment that the proposal should be enacted."]
					#[doc = ""]
					#[doc = "Emits `Submitted`."]
					submit {
						proposal_origin: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::OriginCaller,
						>,
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
							runtime_types::sp_runtime::traits::BlakeTwo256,
						>,
						enactment_moment:
							runtime_types::frame_support::traits::schedule::DispatchTime<
								::core::primitive::u32,
							>,
					},
					#[codec(index = 1)]
					#[doc = "Post the Decision Deposit for a referendum."]
					#[doc = ""]
					#[doc = "- `origin`: must be `Signed` and the account must have funds available for the"]
					#[doc = "  referendum's track's Decision Deposit."]
					#[doc = "- `index`: The index of the submitted referendum whose Decision Deposit is yet to be"]
					#[doc = "  posted."]
					#[doc = ""]
					#[doc = "Emits `DecisionDepositPlaced`."]
					place_decision_deposit { index: ::core::primitive::u32 },
					#[codec(index = 2)]
					#[doc = "Refund the Decision Deposit for a closed referendum back to the depositor."]
					#[doc = ""]
					#[doc = "- `origin`: must be `Signed` or `Root`."]
					#[doc = "- `index`: The index of a closed referendum whose Decision Deposit has not yet been"]
					#[doc = "  refunded."]
					#[doc = ""]
					#[doc = "Emits `DecisionDepositRefunded`."]
					refund_decision_deposit { index: ::core::primitive::u32 },
					#[codec(index = 3)]
					#[doc = "Cancel an ongoing referendum."]
					#[doc = ""]
					#[doc = "- `origin`: must be the `CancelOrigin`."]
					#[doc = "- `index`: The index of the referendum to be cancelled."]
					#[doc = ""]
					#[doc = "Emits `Cancelled`."]
					cancel { index: ::core::primitive::u32 },
					#[codec(index = 4)]
					#[doc = "Cancel an ongoing referendum and slash the deposits."]
					#[doc = ""]
					#[doc = "- `origin`: must be the `KillOrigin`."]
					#[doc = "- `index`: The index of the referendum to be cancelled."]
					#[doc = ""]
					#[doc = "Emits `Killed` and `DepositSlashed`."]
					kill { index: ::core::primitive::u32 },
					#[codec(index = 5)]
					#[doc = "Advance a referendum onto its next logical state. Only used internally."]
					#[doc = ""]
					#[doc = "- `origin`: must be `Root`."]
					#[doc = "- `index`: the referendum to be advanced."]
					nudge_referendum { index: ::core::primitive::u32 },
					#[codec(index = 6)]
					#[doc = "Advance a track onto its next logical state. Only used internally."]
					#[doc = ""]
					#[doc = "- `origin`: must be `Root`."]
					#[doc = "- `track`: the track to be advanced."]
					#[doc = ""]
					#[doc = "Action item for when there is now one fewer referendum in the deciding phase and the"]
					#[doc = "`DecidingCount` is not yet updated. This means that we should either:"]
					#[doc = "- begin deciding another referendum (and leave `DecidingCount` alone); or"]
					#[doc = "- decrement `DecidingCount`."]
					one_fewer_deciding { track: ::core::primitive::u16 },
					#[codec(index = 7)]
					#[doc = "Refund the Submission Deposit for a closed referendum back to the depositor."]
					#[doc = ""]
					#[doc = "- `origin`: must be `Signed` or `Root`."]
					#[doc = "- `index`: The index of a closed referendum whose Submission Deposit has not yet been"]
					#[doc = "  refunded."]
					#[doc = ""]
					#[doc = "Emits `SubmissionDepositRefunded`."]
					refund_submission_deposit { index: ::core::primitive::u32 },
					#[codec(index = 8)]
					#[doc = "Set or clear metadata of a referendum."]
					#[doc = ""]
					#[doc = "Parameters:"]
					#[doc = "- `origin`: Must be `Signed` by a creator of a referendum or by anyone to clear a"]
					#[doc = "  metadata of a finished referendum."]
					#[doc = "- `index`:  The index of a referendum to set or clear metadata for."]
					#[doc = "- `maybe_hash`: The hash of an on-chain stored preimage. `None` to clear a metadata."]
					set_metadata {
						index: ::core::primitive::u32,
						maybe_hash: ::core::option::Option<::subxt::ext::subxt_core::utils::H256>,
					},
				}
			}
		}
		pub mod pallet_scheduler {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Anonymously schedule a task."]
					schedule {
						when: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Cancel an anonymously scheduled task."]
					cancel { when: ::core::primitive::u32, index: ::core::primitive::u32 },
					#[codec(index = 2)]
					#[doc = "Schedule a named task."]
					schedule_named {
						id: [::core::primitive::u8; 32usize],
						when: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 3)]
					#[doc = "Cancel a named scheduled task."]
					cancel_named { id: [::core::primitive::u8; 32usize] },
					#[codec(index = 4)]
					#[doc = "Anonymously schedule a task after a delay."]
					schedule_after {
						after: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 5)]
					#[doc = "Schedule a named task after a delay."]
					schedule_named_after {
						id: [::core::primitive::u8; 32usize],
						after: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 6)]
					#[doc = "Set a retry configuration for a task so that, in case its scheduled run fails, it will"]
					#[doc = "be retried after `period` blocks, for a total amount of `retries` retries or until it"]
					#[doc = "succeeds."]
					#[doc = ""]
					#[doc = "Tasks which need to be scheduled for a retry are still subject to weight metering and"]
					#[doc = "agenda space, same as a regular task. If a periodic task fails, it will be scheduled"]
					#[doc = "normally while the task is retrying."]
					#[doc = ""]
					#[doc = "Tasks scheduled as a result of a retry for a periodic task are unnamed, non-periodic"]
					#[doc = "clones of the original task. Their retry configuration will be derived from the"]
					#[doc = "original task's configuration, but will have a lower value for `remaining` than the"]
					#[doc = "original `total_retries`."]
					set_retry {
						task: (::core::primitive::u32, ::core::primitive::u32),
						retries: ::core::primitive::u8,
						period: ::core::primitive::u32,
					},
					#[codec(index = 7)]
					#[doc = "Set a retry configuration for a named task so that, in case its scheduled run fails, it"]
					#[doc = "will be retried after `period` blocks, for a total amount of `retries` retries or until"]
					#[doc = "it succeeds."]
					#[doc = ""]
					#[doc = "Tasks which need to be scheduled for a retry are still subject to weight metering and"]
					#[doc = "agenda space, same as a regular task. If a periodic task fails, it will be scheduled"]
					#[doc = "normally while the task is retrying."]
					#[doc = ""]
					#[doc = "Tasks scheduled as a result of a retry for a periodic task are unnamed, non-periodic"]
					#[doc = "clones of the original task. Their retry configuration will be derived from the"]
					#[doc = "original task's configuration, but will have a lower value for `remaining` than the"]
					#[doc = "original `total_retries`."]
					set_retry_named {
						id: [::core::primitive::u8; 32usize],
						retries: ::core::primitive::u8,
						period: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					#[doc = "Removes the retry configuration of a task."]
					cancel_retry { task: (::core::primitive::u32, ::core::primitive::u32) },
					#[codec(index = 9)]
					#[doc = "Cancel the retry configuration of a named task."]
					cancel_retry_named { id: [::core::primitive::u8; 32usize] },
				}
			}
		}
		pub mod pallet_score_staking {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					set_score_feeder {
						new_score_feeder: ::subxt::ext::subxt_core::utils::AccountId32,
					},
					#[codec(index = 1)]
					#[doc = "Start (or restart) a currently stopped pool"]
					#[doc = ""]
					#[doc = "It:"]
					#[doc = "- sets the RoundInfo.start_block to the current block number"]
					#[doc = "- advances the round index"]
					start_pool,
					#[codec(index = 2)]
					#[doc = "Stop a currently running pool, should be called as caution"]
					stop_pool,
					#[codec(index = 3)]
					set_round_config {
						config: runtime_types::pallet_score_staking::types::RoundSetting,
					},
					#[codec(index = 4)]
					update_score {
						user: runtime_types::core_primitives::identity::Identity,
						score: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					remove_score { user: runtime_types::core_primitives::identity::Identity },
					#[codec(index = 6)]
					clear_score,
					#[codec(index = 7)]
					claim { amount: ::core::primitive::u128 },
					#[codec(index = 8)]
					claim_all,
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct RoundSetting {
					pub interval: ::core::primitive::u32,
					pub stake_coef_n: ::core::primitive::u32,
					pub stake_coef_m: ::core::primitive::u32,
				}
			}
		}
		pub mod pallet_session {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Sets the session key(s) of the function caller to `keys`."]
					#[doc = "Allows an account to set its session key prior to becoming a validator."]
					#[doc = "This doesn't take effect until the next session."]
					#[doc = ""]
					#[doc = "The dispatch origin of this function must be signed."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)`. Actual cost depends on the number of length of `T::Keys::key_ids()` which is"]
					#[doc = "  fixed."]
					set_keys {
						keys: runtime_types::paseo_parachain_runtime::SessionKeys,
						proof: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 1)]
					#[doc = "Removes any session key(s) of the function caller."]
					#[doc = ""]
					#[doc = "This doesn't take effect until the next session."]
					#[doc = ""]
					#[doc = "The dispatch origin of this function must be Signed and the account must be either be"]
					#[doc = "convertible to a validator ID using the chain's typical addressing system (this usually"]
					#[doc = "means being a controller account) or directly convertible into a validator ID (which"]
					#[doc = "usually means being a stash account)."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)` in number of key types. Actual cost depends on the number of length of"]
					#[doc = "  `T::Keys::key_ids()` which is fixed."]
					purge_keys,
				}
			}
		}
		pub mod pallet_sudo {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Authenticates the sudo key and dispatches a function call with `Root` origin."]
					sudo {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Authenticates the sudo key and dispatches a function call with `Root` origin."]
					#[doc = "This function does not check the weight of the call, and instead allows the"]
					#[doc = "Sudo user to specify the weight of the call."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					sudo_unchecked_weight {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
						weight: runtime_types::sp_weights::weight_v2::Weight,
					},
					#[codec(index = 2)]
					#[doc = "Authenticates the current sudo key and sets the given AccountId (`new`) as the new sudo"]
					#[doc = "key."]
					set_key {
						new: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 3)]
					#[doc = "Authenticates the sudo key and dispatches a function call with `Signed` origin from"]
					#[doc = "a given account."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					sudo_as {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 4)]
					#[doc = "Permanently removes the sudo key."]
					#[doc = ""]
					#[doc = "**This cannot be un-done.**"]
					remove_key,
				}
			}
		}
		pub mod pallet_teebag {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Set the admin account"]
					#[doc = ""]
					#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
					set_admin { new_admin: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 1)]
					#[doc = "Set the mode"]
					#[doc = ""]
					#[doc = "Weights should be 2 DB writes: 1 for mode and 1 for event"]
					set_mode {
						new_mode: runtime_types::core_primitives::teebag::types::OperationalMode,
					},
					#[codec(index = 2)]
					force_add_enclave {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						enclave: runtime_types::core_primitives::teebag::types::Enclave,
					},
					#[codec(index = 3)]
					force_remove_enclave { who: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 4)]
					force_remove_enclave_by_mrenclave {
						mrenclave: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 5)]
					force_remove_enclave_by_worker_type {
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
					},
					#[codec(index = 6)]
					force_add_authorized_enclave {
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						mrenclave: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 7)]
					force_remove_authorized_enclave {
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						mrenclave: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 8)]
					register_enclave {
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						worker_mode: runtime_types::core_primitives::teebag::types::WorkerMode,
						attestation:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						worker_url:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						shielding_pubkey: ::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
						vc_pubkey: ::core::option::Option<[::core::primitive::u8; 32usize]>,
						attestation_type:
							runtime_types::core_primitives::teebag::types::AttestationType,
					},
					#[codec(index = 9)]
					unregister_enclave,
					#[codec(index = 10)]
					register_quoting_enclave {
						enclave_identity:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						signature: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						certificate_chain:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 11)]
					register_tcb_info {
						tcb_info: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						signature: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						certificate_chain:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 20)]
					post_opaque_task {
						request: runtime_types::core_primitives::teebag::types::RsaRequest,
					},
					#[codec(index = 21)]
					parentchain_block_processed {
						block_hash: ::subxt::ext::subxt_core::utils::H256,
						block_number: ::core::primitive::u32,
						task_merkle_root: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 22)]
					sidechain_block_imported {
						shard: ::subxt::ext::subxt_core::utils::H256,
						block_number: ::core::primitive::u64,
						next_finalization_candidate_block_number: ::core::primitive::u64,
						block_header_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 23)]
					batch {
						calls: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 24)]
					add_enclave_identifier {
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						who: ::subxt::ext::subxt_core::utils::AccountId32,
					},
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "The `Error` enum of this pallet."]
				pub enum Error {
					#[codec(index = 0)]
					#[doc = "This operation needs the admin permission"]
					RequireAdminOrRoot,
					#[codec(index = 1)]
					#[doc = "Failed to decode enclave signer."]
					EnclaveSignerDecodeError,
					#[codec(index = 2)]
					#[doc = "Sender does not match attested enclave in report."]
					SenderIsNotAttestedEnclave,
					#[codec(index = 3)]
					#[doc = "Verifying RA report failed."]
					RemoteAttestationVerificationFailed,
					#[codec(index = 4)]
					#[doc = "RA report is too old."]
					RemoteAttestationTooOld,
					#[codec(index = 5)]
					#[doc = "Invalid attestion type, e.g., an `Ignore` type under non-dev mode"]
					InvalidAttestationType,
					#[codec(index = 6)]
					#[doc = "The enclave cannot attest, because its building mode is not allowed."]
					InvalidSgxMode,
					#[codec(index = 7)]
					#[doc = "The specified dcap provider is not yet supported"]
					DcapProviderNotSupported,
					#[codec(index = 8)]
					#[doc = "The enclave doesn't exist."]
					EnclaveNotExist,
					#[codec(index = 9)]
					#[doc = "The enclave identifier doesn't exist."]
					EnclaveIdentifierNotExist,
					#[codec(index = 10)]
					#[doc = "The enclave identifier already exists."]
					EnclaveIdentifierAlreadyExist,
					#[codec(index = 11)]
					#[doc = "The worker type is unexpected, becuase e.g. when we try to re-register an"]
					#[doc = "existing enclave with a differnet worker type"]
					UnexpectedWorkerType,
					#[codec(index = 12)]
					#[doc = "The shard doesn't match the enclave."]
					WrongMrenclaveForShard,
					#[codec(index = 13)]
					#[doc = "The worker url is too long."]
					EnclaveUrlTooLong,
					#[codec(index = 14)]
					#[doc = "The raw attestation data is too long."]
					AttestationTooLong,
					#[codec(index = 15)]
					#[doc = "The worker mode is unexpected, because e.g. a non-sidechain worker calls"]
					#[doc = "sidechain specific extrinsic"]
					UnexpectedWorkerMode,
					#[codec(index = 16)]
					#[doc = "The authorized enclave doesn't exist."]
					AuthorizedEnclaveNotExist,
					#[codec(index = 17)]
					#[doc = "The authorized enclave already exists."]
					AuthorizedEnclaveAlreadyExist,
					#[codec(index = 18)]
					#[doc = "Enclave not authorized upon registration."]
					EnclaveNotAuthorized,
					#[codec(index = 19)]
					#[doc = "The provided collateral data is invalid."]
					CollateralInvalid,
					#[codec(index = 20)]
					#[doc = "MaxEnclaveIdentifier overflow."]
					MaxEnclaveIdentifierOverflow,
					#[codec(index = 21)]
					#[doc = "MaxAuthorizedEnclave overflow."]
					MaxAuthorizedEnclaveOverflow,
					#[codec(index = 22)]
					#[doc = "A proposed block is unexpected."]
					ReceivedUnexpectedSidechainBlock,
					#[codec(index = 23)]
					#[doc = "The value for the next finalization candidate is invalid."]
					InvalidNextFinalizationCandidateBlockNumber,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "The `Event` enum of this pallet"]
				pub enum Event {
					#[codec(index = 0)]
					ModeSet {
						new_mode: runtime_types::core_primitives::teebag::types::OperationalMode,
					},
					#[codec(index = 1)]
					AdminSet {
						new_admin:
							::core::option::Option<::subxt::ext::subxt_core::utils::AccountId32>,
					},
					#[codec(index = 2)]
					EnclaveAdded {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						url: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 3)]
					EnclaveRemoved { who: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 4)]
					EnclaveAuthorized {
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						mrenclave: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 5)]
					EnclaveUnauthorized {
						worker_type: runtime_types::core_primitives::teebag::types::WorkerType,
						mrenclave: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 6)]
					OpaqueTaskPosted {
						request: runtime_types::core_primitives::teebag::types::RsaRequest,
					},
					#[codec(index = 7)]
					ParentchainBlockProcessed {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						block_number: ::core::primitive::u32,
						block_hash: ::subxt::ext::subxt_core::utils::H256,
						task_merkle_root: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 8)]
					SidechainBlockFinalized {
						who: ::subxt::ext::subxt_core::utils::AccountId32,
						sidechain_block_number: ::core::primitive::u64,
					},
				}
			}
		}
		pub mod pallet_timestamp {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Set the current time."]
					#[doc = ""]
					#[doc = "This call should be invoked exactly once per block. It will panic at the finalization"]
					#[doc = "phase, if this call hasn't been invoked by that time."]
					#[doc = ""]
					#[doc = "The timestamp should be greater than the previous one by the amount specified by"]
					#[doc = "[`Config::MinimumPeriod`]."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _None_."]
					#[doc = ""]
					#[doc = "This dispatch class is _Mandatory_ to ensure it gets executed in the block. Be aware"]
					#[doc = "that changing the complexity of this call could result exhausting the resources in a"]
					#[doc = "block to execute any other calls."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)` (Note that implementations of `OnTimestampSet` must also be `O(1)`)"]
					#[doc = "- 1 storage read and 1 storage mutation (codec `O(1)` because of `DidUpdate::take` in"]
					#[doc = "  `on_finalize`)"]
					#[doc = "- 1 event handler `on_timestamp_set`. Must be `O(1)`."]
					set {
						#[codec(compact)]
						now: ::core::primitive::u64,
					},
				}
			}
		}
		pub mod pallet_tips {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Report something `reason` that deserves a tip and claim any eventual the finder's fee."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "Payment: `TipReportDepositBase` will be reserved from the origin account, as well as"]
					#[doc = "`DataDepositPerByte` for each byte in `reason`."]
					#[doc = ""]
					#[doc = "- `reason`: The reason for, or the thing that deserves, the tip; generally this will be"]
					#[doc = "  a UTF-8-encoded URL."]
					#[doc = "- `who`: The account which should be credited for the tip."]
					#[doc = ""]
					#[doc = "Emits `NewTip` if successful."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(R)` where `R` length of `reason`."]
					#[doc = "  - encoding and hashing of 'reason'"]
					report_awesome {
						reason: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 1)]
					#[doc = "Retract a prior tip-report from `report_awesome`, and cancel the process of tipping."]
					#[doc = ""]
					#[doc = "If successful, the original deposit will be unreserved."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the tip identified by `hash`"]
					#[doc = "must have been reported by the signing account through `report_awesome` (and not"]
					#[doc = "through `tip_new`)."]
					#[doc = ""]
					#[doc = "- `hash`: The identity of the open tip for which a tip value is declared. This is formed"]
					#[doc = "  as the hash of the tuple of the original tip `reason` and the beneficiary account ID."]
					#[doc = ""]
					#[doc = "Emits `TipRetracted` if successful."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)`"]
					#[doc = "  - Depends on the length of `T::Hash` which is fixed."]
					retract_tip { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					#[doc = "Give a tip for something new; no finder's fee will be taken."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the signing account must be a"]
					#[doc = "member of the `Tippers` set."]
					#[doc = ""]
					#[doc = "- `reason`: The reason for, or the thing that deserves, the tip; generally this will be"]
					#[doc = "  a UTF-8-encoded URL."]
					#[doc = "- `who`: The account which should be credited for the tip."]
					#[doc = "- `tip_value`: The amount of tip that the sender would like to give. The median tip"]
					#[doc = "  value of active tippers will be given to the `who`."]
					#[doc = ""]
					#[doc = "Emits `NewTip` if successful."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(R + T)` where `R` length of `reason`, `T` is the number of tippers."]
					#[doc = "  - `O(T)`: decoding `Tipper` vec of length `T`. `T` is charged as upper bound given by"]
					#[doc = "    `ContainsLengthBound`. The actual cost depends on the implementation of"]
					#[doc = "    `T::Tippers`."]
					#[doc = "  - `O(R)`: hashing and encoding of reason of length `R`"]
					tip_new {
						reason: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						#[codec(compact)]
						tip_value: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					#[doc = "Declare a tip value for an already-open tip."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the signing account must be a"]
					#[doc = "member of the `Tippers` set."]
					#[doc = ""]
					#[doc = "- `hash`: The identity of the open tip for which a tip value is declared. This is formed"]
					#[doc = "  as the hash of the tuple of the hash of the original tip `reason` and the beneficiary"]
					#[doc = "  account ID."]
					#[doc = "- `tip_value`: The amount of tip that the sender would like to give. The median tip"]
					#[doc = "  value of active tippers will be given to the `who`."]
					#[doc = ""]
					#[doc = "Emits `TipClosing` if the threshold of tippers has been reached and the countdown period"]
					#[doc = "has started."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(T)` where `T` is the number of tippers. decoding `Tipper` vec of length `T`, insert"]
					#[doc = "  tip and check closing, `T` is charged as upper bound given by `ContainsLengthBound`."]
					#[doc = "  The actual cost depends on the implementation of `T::Tippers`."]
					#[doc = ""]
					#[doc = "  Actually weight could be lower as it depends on how many tips are in `OpenTip` but it"]
					#[doc = "  is weighted as if almost full i.e of length `T-1`."]
					tip {
						hash: ::subxt::ext::subxt_core::utils::H256,
						#[codec(compact)]
						tip_value: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					#[doc = "Close and payout a tip."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "The tip identified by `hash` must have finished its countdown period."]
					#[doc = ""]
					#[doc = "- `hash`: The identity of the open tip for which a tip value is declared. This is formed"]
					#[doc = "  as the hash of the tuple of the original tip `reason` and the beneficiary account ID."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- : `O(T)` where `T` is the number of tippers. decoding `Tipper` vec of length `T`. `T`"]
					#[doc = "  is charged as upper bound given by `ContainsLengthBound`. The actual cost depends on"]
					#[doc = "  the implementation of `T::Tippers`."]
					close_tip { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 5)]
					#[doc = "Remove and slash an already-open tip."]
					#[doc = ""]
					#[doc = "May only be called from `T::RejectOrigin`."]
					#[doc = ""]
					#[doc = "As a result, the finder is slashed and the deposits are lost."]
					#[doc = ""]
					#[doc = "Emits `TipSlashed` if successful."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					slash_tip { hash: ::subxt::ext::subxt_core::utils::H256 },
				}
			}
		}
		pub mod pallet_transaction_payment {
			use super::runtime_types;
			pub mod types {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct FeeDetails<_0> {
					pub inclusion_fee: ::core::option::Option<
						runtime_types::pallet_transaction_payment::types::InclusionFee<_0>,
					>,
					pub tip: _0,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct InclusionFee<_0> {
					pub base_fee: _0,
					pub len_fee: _0,
					pub adjusted_weight_fee: _0,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct RuntimeDispatchInfo<_0, _1> {
					pub weight: _1,
					pub class: runtime_types::frame_support::dispatch::DispatchClass,
					pub partial_fee: _0,
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct ChargeTransactionPayment(#[codec(compact)] pub ::core::primitive::u128);
		}
		pub mod pallet_treasury {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 3)]
					#[doc = "Propose and approve a spend of treasury funds."]
					#[doc = ""]
					#[doc = "## Dispatch Origin"]
					#[doc = ""]
					#[doc = "Must be [`Config::SpendOrigin`] with the `Success` value being at least `amount`."]
					#[doc = ""]
					#[doc = "### Details"]
					#[doc = "NOTE: For record-keeping purposes, the proposer is deemed to be equivalent to the"]
					#[doc = "beneficiary."]
					#[doc = ""]
					#[doc = "### Parameters"]
					#[doc = "- `amount`: The amount to be transferred from the treasury to the `beneficiary`."]
					#[doc = "- `beneficiary`: The destination account for the transfer."]
					#[doc = ""]
					#[doc = "## Events"]
					#[doc = ""]
					#[doc = "Emits [`Event::SpendApproved`] if successful."]
					spend_local {
						#[codec(compact)]
						amount: ::core::primitive::u128,
						beneficiary: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 4)]
					#[doc = "Force a previously approved proposal to be removed from the approval queue."]
					#[doc = ""]
					#[doc = "## Dispatch Origin"]
					#[doc = ""]
					#[doc = "Must be [`Config::RejectOrigin`]."]
					#[doc = ""]
					#[doc = "## Details"]
					#[doc = ""]
					#[doc = "The original deposit will no longer be returned."]
					#[doc = ""]
					#[doc = "### Parameters"]
					#[doc = "- `proposal_id`: The index of a proposal"]
					#[doc = ""]
					#[doc = "### Complexity"]
					#[doc = "- O(A) where `A` is the number of approvals"]
					#[doc = ""]
					#[doc = "### Errors"]
					#[doc = "- [`Error::ProposalNotApproved`]: The `proposal_id` supplied was not found in the"]
					#[doc = "  approval queue, i.e., the proposal has not been approved. This could also mean the"]
					#[doc = "  proposal does not exist altogether, thus there is no way it would have been approved"]
					#[doc = "  in the first place."]
					remove_approval {
						#[codec(compact)]
						proposal_id: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					#[doc = "Propose and approve a spend of treasury funds."]
					#[doc = ""]
					#[doc = "## Dispatch Origin"]
					#[doc = ""]
					#[doc = "Must be [`Config::SpendOrigin`] with the `Success` value being at least"]
					#[doc = "`amount` of `asset_kind` in the native asset. The amount of `asset_kind` is converted"]
					#[doc = "for assertion using the [`Config::BalanceConverter`]."]
					#[doc = ""]
					#[doc = "## Details"]
					#[doc = ""]
					#[doc = "Create an approved spend for transferring a specific `amount` of `asset_kind` to a"]
					#[doc = "designated beneficiary. The spend must be claimed using the `payout` dispatchable within"]
					#[doc = "the [`Config::PayoutPeriod`]."]
					#[doc = ""]
					#[doc = "### Parameters"]
					#[doc = "- `asset_kind`: An indicator of the specific asset class to be spent."]
					#[doc = "- `amount`: The amount to be transferred from the treasury to the `beneficiary`."]
					#[doc = "- `beneficiary`: The beneficiary of the spend."]
					#[doc = "- `valid_from`: The block number from which the spend can be claimed. It can refer to"]
					#[doc = "  the past if the resulting spend has not yet expired according to the"]
					#[doc = "  [`Config::PayoutPeriod`]. If `None`, the spend can be claimed immediately after"]
					#[doc = "  approval."]
					#[doc = ""]
					#[doc = "## Events"]
					#[doc = ""]
					#[doc = "Emits [`Event::AssetSpendApproved`] if successful."]
					spend {
						asset_kind: ::subxt::ext::subxt_core::alloc::boxed::Box<()>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
						beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
							::subxt::ext::subxt_core::utils::AccountId32,
						>,
						valid_from: ::core::option::Option<::core::primitive::u32>,
					},
					#[codec(index = 6)]
					#[doc = "Claim a spend."]
					#[doc = ""]
					#[doc = "## Dispatch Origin"]
					#[doc = ""]
					#[doc = "Must be signed"]
					#[doc = ""]
					#[doc = "## Details"]
					#[doc = ""]
					#[doc = "Spends must be claimed within some temporal bounds. A spend may be claimed within one"]
					#[doc = "[`Config::PayoutPeriod`] from the `valid_from` block."]
					#[doc = "In case of a payout failure, the spend status must be updated with the `check_status`"]
					#[doc = "dispatchable before retrying with the current function."]
					#[doc = ""]
					#[doc = "### Parameters"]
					#[doc = "- `index`: The spend index."]
					#[doc = ""]
					#[doc = "## Events"]
					#[doc = ""]
					#[doc = "Emits [`Event::Paid`] if successful."]
					payout { index: ::core::primitive::u32 },
					#[codec(index = 7)]
					#[doc = "Check the status of the spend and remove it from the storage if processed."]
					#[doc = ""]
					#[doc = "## Dispatch Origin"]
					#[doc = ""]
					#[doc = "Must be signed."]
					#[doc = ""]
					#[doc = "## Details"]
					#[doc = ""]
					#[doc = "The status check is a prerequisite for retrying a failed payout."]
					#[doc = "If a spend has either succeeded or expired, it is removed from the storage by this"]
					#[doc = "function. In such instances, transaction fees are refunded."]
					#[doc = ""]
					#[doc = "### Parameters"]
					#[doc = "- `index`: The spend index."]
					#[doc = ""]
					#[doc = "## Events"]
					#[doc = ""]
					#[doc = "Emits [`Event::PaymentFailed`] if the spend payout has failed."]
					#[doc = "Emits [`Event::SpendProcessed`] if the spend payout has succeed."]
					check_status { index: ::core::primitive::u32 },
					#[codec(index = 8)]
					#[doc = "Void previously approved spend."]
					#[doc = ""]
					#[doc = "## Dispatch Origin"]
					#[doc = ""]
					#[doc = "Must be [`Config::RejectOrigin`]."]
					#[doc = ""]
					#[doc = "## Details"]
					#[doc = ""]
					#[doc = "A spend void is only possible if the payout has not been attempted yet."]
					#[doc = ""]
					#[doc = "### Parameters"]
					#[doc = "- `index`: The spend index."]
					#[doc = ""]
					#[doc = "## Events"]
					#[doc = ""]
					#[doc = "Emits [`Event::AssetSpendVoided`] if successful."]
					void_spend { index: ::core::primitive::u32 },
				}
			}
		}
		pub mod pallet_utility {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Send a batch of dispatch calls."]
					#[doc = ""]
					#[doc = "May be called from any origin except `None`."]
					#[doc = ""]
					#[doc = "- `calls`: The calls to be dispatched from the same origin. The number of call must not"]
					#[doc = "  exceed the constant: `batched_calls_limit` (available in constant metadata)."]
					#[doc = ""]
					#[doc = "If origin is root then the calls are dispatched without checking origin filter. (This"]
					#[doc = "includes bypassing `frame_system::Config::BaseCallFilter`)."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(C) where C is the number of calls to be batched."]
					#[doc = ""]
					#[doc = "This will return `Ok` in all circumstances. To determine the success of the batch, an"]
					#[doc = "event is deposited. If a call failed and the batch was interrupted, then the"]
					#[doc = "`BatchInterrupted` event is deposited, along with the number of successful calls made"]
					#[doc = "and the error of the failed call. If all were successful, then the `BatchCompleted`"]
					#[doc = "event is deposited."]
					batch {
						calls: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					#[doc = "Send a call through an indexed pseudonym of the sender."]
					#[doc = ""]
					#[doc = "Filter from origin are passed along. The call will be dispatched with an origin which"]
					#[doc = "use the same filter as the origin of this call."]
					#[doc = ""]
					#[doc = "NOTE: If you need to ensure that any account-based filtering is not honored (i.e."]
					#[doc = "because you expect `proxy` to have been used prior in the call stack and you do not want"]
					#[doc = "the call restrictions to apply to any sub-accounts), then use `as_multi_threshold_1`"]
					#[doc = "in the Multisig pallet instead."]
					#[doc = ""]
					#[doc = "NOTE: Prior to version *12, this was called `as_limited_sub`."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					as_derivative {
						index: ::core::primitive::u16,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 2)]
					#[doc = "Send a batch of dispatch calls and atomically execute them."]
					#[doc = "The whole transaction will rollback and fail if any of the calls failed."]
					#[doc = ""]
					#[doc = "May be called from any origin except `None`."]
					#[doc = ""]
					#[doc = "- `calls`: The calls to be dispatched from the same origin. The number of call must not"]
					#[doc = "  exceed the constant: `batched_calls_limit` (available in constant metadata)."]
					#[doc = ""]
					#[doc = "If origin is root then the calls are dispatched without checking origin filter. (This"]
					#[doc = "includes bypassing `frame_system::Config::BaseCallFilter`)."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(C) where C is the number of calls to be batched."]
					batch_all {
						calls: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 3)]
					#[doc = "Dispatches a function call with a provided origin."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Root_."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(1)."]
					dispatch_as {
						as_origin: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::OriginCaller,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 4)]
					#[doc = "Send a batch of dispatch calls."]
					#[doc = "Unlike `batch`, it allows errors and won't interrupt."]
					#[doc = ""]
					#[doc = "May be called from any origin except `None`."]
					#[doc = ""]
					#[doc = "- `calls`: The calls to be dispatched from the same origin. The number of call must not"]
					#[doc = "  exceed the constant: `batched_calls_limit` (available in constant metadata)."]
					#[doc = ""]
					#[doc = "If origin is root then the calls are dispatch without checking origin filter. (This"]
					#[doc = "includes bypassing `frame_system::Config::BaseCallFilter`)."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- O(C) where C is the number of calls to be batched."]
					force_batch {
						calls: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 5)]
					#[doc = "Dispatch a function call with a specified weight."]
					#[doc = ""]
					#[doc = "This function does not check the weight of the call, and instead allows the"]
					#[doc = "Root origin to specify the weight of the call."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Root_."]
					with_weight {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
						weight: runtime_types::sp_weights::weight_v2::Weight,
					},
				}
			}
		}
		pub mod pallet_vc_management {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "add an account to the delegatees"]
					add_delegatee { account: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 1)]
					#[doc = "remove an account from the delegatees"]
					remove_delegatee { account: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 2)]
					request_vc {
						shard: ::subxt::ext::subxt_core::utils::H256,
						assertion: runtime_types::core_primitives::assertion::Assertion,
					},
					#[codec(index = 5)]
					set_admin { new: ::subxt::ext::subxt_core::utils::AccountId32 },
					#[codec(index = 6)]
					add_schema {
						shard: ::subxt::ext::subxt_core::utils::H256,
						id: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						content: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 7)]
					disable_schema {
						shard: ::subxt::ext::subxt_core::utils::H256,
						index: ::core::primitive::u64,
					},
					#[codec(index = 8)]
					activate_schema {
						shard: ::subxt::ext::subxt_core::utils::H256,
						index: ::core::primitive::u64,
					},
					#[codec(index = 9)]
					revoke_schema {
						shard: ::subxt::ext::subxt_core::utils::H256,
						index: ::core::primitive::u64,
					},
					#[codec(index = 30)]
					#[doc = "---------------------------------------------------"]
					#[doc = "The following extrinsics are supposed to be called by TEE only"]
					#[doc = "---------------------------------------------------"]
					vc_issued {
						identity: runtime_types::core_primitives::identity::Identity,
						assertion: runtime_types::core_primitives::assertion::Assertion,
						id_graph_hash: ::subxt::ext::subxt_core::utils::H256,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 31)]
					some_error {
						identity: ::core::option::Option<
							runtime_types::core_primitives::identity::Identity,
						>,
						error: runtime_types::core_primitives::error::VCMPError,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 32)]
					on_vc_issued {
						identity: runtime_types::core_primitives::identity::Identity,
						assertion: runtime_types::core_primitives::assertion::Assertion,
						omni_account: ::subxt::ext::subxt_core::utils::AccountId32,
						req_ext_hash: ::subxt::ext::subxt_core::utils::H256,
					},
				}
			}
		}
		pub mod pallet_vesting {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					#[doc = "Unlock any vested funds of the sender account."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_ and the sender must have funds still"]
					#[doc = "locked under this pallet."]
					#[doc = ""]
					#[doc = "Emits either `VestingCompleted` or `VestingUpdated`."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)`."]
					vest,
					#[codec(index = 1)]
					#[doc = "Unlock any vested funds of a `target` account."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `target`: The account whose vested funds should be unlocked. Must have funds still"]
					#[doc = "locked under this pallet."]
					#[doc = ""]
					#[doc = "Emits either `VestingCompleted` or `VestingUpdated`."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)`."]
					vest_other {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
					},
					#[codec(index = 2)]
					#[doc = "Create a vested transfer."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `target`: The account receiving the vested funds."]
					#[doc = "- `schedule`: The vesting schedule attached to the transfer."]
					#[doc = ""]
					#[doc = "Emits `VestingCreated`."]
					#[doc = ""]
					#[doc = "NOTE: This will unlock all schedules through the current block."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)`."]
					vested_transfer {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						schedule: runtime_types::pallet_vesting::vesting_info::VestingInfo<
							::core::primitive::u128,
							::core::primitive::u32,
						>,
					},
					#[codec(index = 3)]
					#[doc = "Force a vested transfer."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Root_."]
					#[doc = ""]
					#[doc = "- `source`: The account whose funds should be transferred."]
					#[doc = "- `target`: The account that should be transferred the vested funds."]
					#[doc = "- `schedule`: The vesting schedule attached to the transfer."]
					#[doc = ""]
					#[doc = "Emits `VestingCreated`."]
					#[doc = ""]
					#[doc = "NOTE: This will unlock all schedules through the current block."]
					#[doc = ""]
					#[doc = "## Complexity"]
					#[doc = "- `O(1)`."]
					force_vested_transfer {
						source: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						schedule: runtime_types::pallet_vesting::vesting_info::VestingInfo<
							::core::primitive::u128,
							::core::primitive::u32,
						>,
					},
					#[codec(index = 4)]
					#[doc = "Merge two vesting schedules together, creating a new vesting schedule that unlocks over"]
					#[doc = "the highest possible start and end blocks. If both schedules have already started the"]
					#[doc = "current block will be used as the schedule start; with the caveat that if one schedule"]
					#[doc = "is finished by the current block, the other will be treated as the new merged schedule,"]
					#[doc = "unmodified."]
					#[doc = ""]
					#[doc = "NOTE: If `schedule1_index == schedule2_index` this is a no-op."]
					#[doc = "NOTE: This will unlock all schedules through the current block prior to merging."]
					#[doc = "NOTE: If both schedules have ended by the current block, no new schedule will be created"]
					#[doc = "and both will be removed."]
					#[doc = ""]
					#[doc = "Merged schedule attributes:"]
					#[doc = "- `starting_block`: `MAX(schedule1.starting_block, scheduled2.starting_block,"]
					#[doc = "  current_block)`."]
					#[doc = "- `ending_block`: `MAX(schedule1.ending_block, schedule2.ending_block)`."]
					#[doc = "- `locked`: `schedule1.locked_at(current_block) + schedule2.locked_at(current_block)`."]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Signed_."]
					#[doc = ""]
					#[doc = "- `schedule1_index`: index of the first schedule to merge."]
					#[doc = "- `schedule2_index`: index of the second schedule to merge."]
					merge_schedules {
						schedule1_index: ::core::primitive::u32,
						schedule2_index: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					#[doc = "Force remove a vesting schedule"]
					#[doc = ""]
					#[doc = "The dispatch origin for this call must be _Root_."]
					#[doc = ""]
					#[doc = "- `target`: An account that has a vesting schedule"]
					#[doc = "- `schedule_index`: The vesting schedule index that should be removed"]
					force_remove_vesting_schedule {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::subxt::ext::subxt_core::utils::AccountId32,
							(),
						>,
						schedule_index: ::core::primitive::u32,
					},
				}
			}
			pub mod vesting_info {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct VestingInfo<_0, _1> {
					pub locked: _0,
					pub per_block: _0,
					pub starting_block: _1,
				}
			}
		}
		pub mod pallet_whitelist {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					#[codec(index = 0)]
					whitelist_call { call_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 1)]
					remove_whitelisted_call { call_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					dispatch_whitelisted_call {
						call_hash: ::subxt::ext::subxt_core::utils::H256,
						call_encoded_len: ::core::primitive::u32,
						call_weight_witness: runtime_types::sp_weights::weight_v2::Weight,
					},
					#[codec(index = 3)]
					dispatch_whitelisted_call_with_preimage {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::paseo_parachain_runtime::RuntimeCall,
						>,
					},
				}
			}
		}
		pub mod pallet_xcm {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {
					# [codec (index = 0)] send { dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , message : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedXcm > , } , # [codec (index = 1)] # [doc = "Teleport some assets from the local chain to some destination chain."] # [doc = ""] # [doc = "**This function is deprecated: Use `limited_teleport_assets` instead.**"] # [doc = ""] # [doc = "Fee payment on the destination side is made from the asset in the `assets` vector of"] # [doc = "index `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited,"] # [doc = "with all fees taken as needed from the asset."] # [doc = ""] # [doc = "- `origin`: Must be capable of withdrawing the `assets` and executing XCM."] # [doc = "- `dest`: Destination context for the assets. Will typically be `[Parent,"] # [doc = "  Parachain(..)]` to send from parachain to parachain, or `[Parachain(..)]` to send from"] # [doc = "  relay to parachain."] # [doc = "- `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will"] # [doc = "  generally be an `AccountId32` value."] # [doc = "- `assets`: The assets to be withdrawn. This should include the assets used to pay the"] # [doc = "  fee on the `dest` chain."] # [doc = "- `fee_asset_item`: The index into `assets` of the item which should be used to pay"] # [doc = "  fees."] teleport_assets { dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , beneficiary : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , assets : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssets > , fee_asset_item : :: core :: primitive :: u32 , } , # [codec (index = 2)] # [doc = "Transfer some assets from the local chain to the destination chain through their local,"] # [doc = "destination or remote reserve."] # [doc = ""] # [doc = "`assets` must have same reserve location and may not be teleportable to `dest`."] # [doc = " - `assets` have local reserve: transfer assets to sovereign account of destination"] # [doc = "   chain and forward a notification XCM to `dest` to mint and deposit reserve-based"] # [doc = "   assets to `beneficiary`."] # [doc = " - `assets` have destination reserve: burn local assets and forward a notification to"] # [doc = "   `dest` chain to withdraw the reserve assets from this chain's sovereign account and"] # [doc = "   deposit them to `beneficiary`."] # [doc = " - `assets` have remote reserve: burn local assets, forward XCM to reserve chain to move"] # [doc = "   reserves from this chain's SA to `dest` chain's SA, and forward another XCM to `dest`"] # [doc = "   to mint and deposit reserve-based assets to `beneficiary`."] # [doc = ""] # [doc = "**This function is deprecated: Use `limited_reserve_transfer_assets` instead.**"] # [doc = ""] # [doc = "Fee payment on the destination side is made from the asset in the `assets` vector of"] # [doc = "index `fee_asset_item`. The weight limit for fees is not provided and thus is unlimited,"] # [doc = "with all fees taken as needed from the asset."] # [doc = ""] # [doc = "- `origin`: Must be capable of withdrawing the `assets` and executing XCM."] # [doc = "- `dest`: Destination context for the assets. Will typically be `[Parent,"] # [doc = "  Parachain(..)]` to send from parachain to parachain, or `[Parachain(..)]` to send from"] # [doc = "  relay to parachain."] # [doc = "- `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will"] # [doc = "  generally be an `AccountId32` value."] # [doc = "- `assets`: The assets to be withdrawn. This should include the assets used to pay the"] # [doc = "  fee on the `dest` (and possibly reserve) chains."] # [doc = "- `fee_asset_item`: The index into `assets` of the item which should be used to pay"] # [doc = "  fees."] reserve_transfer_assets { dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , beneficiary : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , assets : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssets > , fee_asset_item : :: core :: primitive :: u32 , } , # [codec (index = 3)] # [doc = "Execute an XCM message from a local, signed, origin."] # [doc = ""] # [doc = "An event is deposited indicating whether `msg` could be executed completely or only"] # [doc = "partially."] # [doc = ""] # [doc = "No more than `max_weight` will be used in its attempted execution. If this is less than"] # [doc = "the maximum amount of weight that the message could take to be executed, then no"] # [doc = "execution attempt will be made."] execute { message : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedXcm > , max_weight : runtime_types :: sp_weights :: weight_v2 :: Weight , } , # [codec (index = 4)] # [doc = "Extoll that a particular destination can be communicated with through a particular"] # [doc = "version of XCM."] # [doc = ""] # [doc = "- `origin`: Must be an origin specified by AdminOrigin."] # [doc = "- `location`: The destination that is being described."] # [doc = "- `xcm_version`: The latest version of XCM that `location` supports."] force_xcm_version { location : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: staging_xcm :: v4 :: location :: Location > , version : :: core :: primitive :: u32 , } , # [codec (index = 5)] # [doc = "Set a safe XCM version (the version that XCM should be encoded with if the most recent"] # [doc = "version a destination can accept is unknown)."] # [doc = ""] # [doc = "- `origin`: Must be an origin specified by AdminOrigin."] # [doc = "- `maybe_xcm_version`: The default XCM encoding version, or `None` to disable."] force_default_xcm_version { maybe_xcm_version : :: core :: option :: Option < :: core :: primitive :: u32 > , } , # [codec (index = 6)] # [doc = "Ask a location to notify us regarding their XCM version and any changes to it."] # [doc = ""] # [doc = "- `origin`: Must be an origin specified by AdminOrigin."] # [doc = "- `location`: The location to which we should subscribe for XCM version notifications."] force_subscribe_version_notify { location : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , } , # [codec (index = 7)] # [doc = "Require that a particular destination should no longer notify us regarding any XCM"] # [doc = "version changes."] # [doc = ""] # [doc = "- `origin`: Must be an origin specified by AdminOrigin."] # [doc = "- `location`: The location to which we are currently subscribed for XCM version"] # [doc = "  notifications which we no longer desire."] force_unsubscribe_version_notify { location : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , } , # [codec (index = 8)] # [doc = "Transfer some assets from the local chain to the destination chain through their local,"] # [doc = "destination or remote reserve."] # [doc = ""] # [doc = "`assets` must have same reserve location and may not be teleportable to `dest`."] # [doc = " - `assets` have local reserve: transfer assets to sovereign account of destination"] # [doc = "   chain and forward a notification XCM to `dest` to mint and deposit reserve-based"] # [doc = "   assets to `beneficiary`."] # [doc = " - `assets` have destination reserve: burn local assets and forward a notification to"] # [doc = "   `dest` chain to withdraw the reserve assets from this chain's sovereign account and"] # [doc = "   deposit them to `beneficiary`."] # [doc = " - `assets` have remote reserve: burn local assets, forward XCM to reserve chain to move"] # [doc = "   reserves from this chain's SA to `dest` chain's SA, and forward another XCM to `dest`"] # [doc = "   to mint and deposit reserve-based assets to `beneficiary`."] # [doc = ""] # [doc = "Fee payment on the destination side is made from the asset in the `assets` vector of"] # [doc = "index `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight"] # [doc = "is needed than `weight_limit`, then the operation will fail and the sent assets may be"] # [doc = "at risk."] # [doc = ""] # [doc = "- `origin`: Must be capable of withdrawing the `assets` and executing XCM."] # [doc = "- `dest`: Destination context for the assets. Will typically be `[Parent,"] # [doc = "  Parachain(..)]` to send from parachain to parachain, or `[Parachain(..)]` to send from"] # [doc = "  relay to parachain."] # [doc = "- `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will"] # [doc = "  generally be an `AccountId32` value."] # [doc = "- `assets`: The assets to be withdrawn. This should include the assets used to pay the"] # [doc = "  fee on the `dest` (and possibly reserve) chains."] # [doc = "- `fee_asset_item`: The index into `assets` of the item which should be used to pay"] # [doc = "  fees."] # [doc = "- `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase."] limited_reserve_transfer_assets { dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , beneficiary : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , assets : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssets > , fee_asset_item : :: core :: primitive :: u32 , weight_limit : runtime_types :: xcm :: v3 :: WeightLimit , } , # [codec (index = 9)] # [doc = "Teleport some assets from the local chain to some destination chain."] # [doc = ""] # [doc = "Fee payment on the destination side is made from the asset in the `assets` vector of"] # [doc = "index `fee_asset_item`, up to enough to pay for `weight_limit` of weight. If more weight"] # [doc = "is needed than `weight_limit`, then the operation will fail and the sent assets may be"] # [doc = "at risk."] # [doc = ""] # [doc = "- `origin`: Must be capable of withdrawing the `assets` and executing XCM."] # [doc = "- `dest`: Destination context for the assets. Will typically be `[Parent,"] # [doc = "  Parachain(..)]` to send from parachain to parachain, or `[Parachain(..)]` to send from"] # [doc = "  relay to parachain."] # [doc = "- `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will"] # [doc = "  generally be an `AccountId32` value."] # [doc = "- `assets`: The assets to be withdrawn. This should include the assets used to pay the"] # [doc = "  fee on the `dest` chain."] # [doc = "- `fee_asset_item`: The index into `assets` of the item which should be used to pay"] # [doc = "  fees."] # [doc = "- `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase."] limited_teleport_assets { dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , beneficiary : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , assets : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssets > , fee_asset_item : :: core :: primitive :: u32 , weight_limit : runtime_types :: xcm :: v3 :: WeightLimit , } , # [codec (index = 10)] # [doc = "Set or unset the global suspension state of the XCM executor."] # [doc = ""] # [doc = "- `origin`: Must be an origin specified by AdminOrigin."] # [doc = "- `suspended`: `true` to suspend, `false` to resume."] force_suspension { suspended : :: core :: primitive :: bool , } , # [codec (index = 11)] # [doc = "Transfer some assets from the local chain to the destination chain through their local,"] # [doc = "destination or remote reserve, or through teleports."] # [doc = ""] # [doc = "Fee payment on the destination side is made from the asset in the `assets` vector of"] # [doc = "index `fee_asset_item` (hence referred to as `fees`), up to enough to pay for"] # [doc = "`weight_limit` of weight. If more weight is needed than `weight_limit`, then the"] # [doc = "operation will fail and the sent assets may be at risk."] # [doc = ""] # [doc = "`assets` (excluding `fees`) must have same reserve location or otherwise be teleportable"] # [doc = "to `dest`, no limitations imposed on `fees`."] # [doc = " - for local reserve: transfer assets to sovereign account of destination chain and"] # [doc = "   forward a notification XCM to `dest` to mint and deposit reserve-based assets to"] # [doc = "   `beneficiary`."] # [doc = " - for destination reserve: burn local assets and forward a notification to `dest` chain"] # [doc = "   to withdraw the reserve assets from this chain's sovereign account and deposit them"] # [doc = "   to `beneficiary`."] # [doc = " - for remote reserve: burn local assets, forward XCM to reserve chain to move reserves"] # [doc = "   from this chain's SA to `dest` chain's SA, and forward another XCM to `dest` to mint"] # [doc = "   and deposit reserve-based assets to `beneficiary`."] # [doc = " - for teleports: burn local assets and forward XCM to `dest` chain to mint/teleport"] # [doc = "   assets and deposit them to `beneficiary`."] # [doc = ""] # [doc = "- `origin`: Must be capable of withdrawing the `assets` and executing XCM."] # [doc = "- `dest`: Destination context for the assets. Will typically be `X2(Parent,"] # [doc = "  Parachain(..))` to send from parachain to parachain, or `X1(Parachain(..))` to send"] # [doc = "  from relay to parachain."] # [doc = "- `beneficiary`: A beneficiary location for the assets in the context of `dest`. Will"] # [doc = "  generally be an `AccountId32` value."] # [doc = "- `assets`: The assets to be withdrawn. This should include the assets used to pay the"] # [doc = "  fee on the `dest` (and possibly reserve) chains."] # [doc = "- `fee_asset_item`: The index into `assets` of the item which should be used to pay"] # [doc = "  fees."] # [doc = "- `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase."] transfer_assets { dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , beneficiary : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , assets : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssets > , fee_asset_item : :: core :: primitive :: u32 , weight_limit : runtime_types :: xcm :: v3 :: WeightLimit , } , # [codec (index = 12)] # [doc = "Claims assets trapped on this pallet because of leftover assets during XCM execution."] # [doc = ""] # [doc = "- `origin`: Anyone can call this extrinsic."] # [doc = "- `assets`: The exact assets that were trapped. Use the version to specify what version"] # [doc = "was the latest when they were trapped."] # [doc = "- `beneficiary`: The location/account where the claimed assets will be deposited."] claim_assets { assets : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssets > , beneficiary : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , } , # [codec (index = 13)] # [doc = "Transfer assets from the local chain to the destination chain using explicit transfer"] # [doc = "types for assets and fees."] # [doc = ""] # [doc = "`assets` must have same reserve location or may be teleportable to `dest`. Caller must"] # [doc = "provide the `assets_transfer_type` to be used for `assets`:"] # [doc = " - `TransferType::LocalReserve`: transfer assets to sovereign account of destination"] # [doc = "   chain and forward a notification XCM to `dest` to mint and deposit reserve-based"] # [doc = "   assets to `beneficiary`."] # [doc = " - `TransferType::DestinationReserve`: burn local assets and forward a notification to"] # [doc = "   `dest` chain to withdraw the reserve assets from this chain's sovereign account and"] # [doc = "   deposit them to `beneficiary`."] # [doc = " - `TransferType::RemoteReserve(reserve)`: burn local assets, forward XCM to `reserve`"] # [doc = "   chain to move reserves from this chain's SA to `dest` chain's SA, and forward another"] # [doc = "   XCM to `dest` to mint and deposit reserve-based assets to `beneficiary`. Typically"] # [doc = "   the remote `reserve` is Asset Hub."] # [doc = " - `TransferType::Teleport`: burn local assets and forward XCM to `dest` chain to"] # [doc = "   mint/teleport assets and deposit them to `beneficiary`."] # [doc = ""] # [doc = "On the destination chain, as well as any intermediary hops, `BuyExecution` is used to"] # [doc = "buy execution using transferred `assets` identified by `remote_fees_id`."] # [doc = "Make sure enough of the specified `remote_fees_id` asset is included in the given list"] # [doc = "of `assets`. `remote_fees_id` should be enough to pay for `weight_limit`. If more weight"] # [doc = "is needed than `weight_limit`, then the operation will fail and the sent assets may be"] # [doc = "at risk."] # [doc = ""] # [doc = "`remote_fees_id` may use different transfer type than rest of `assets` and can be"] # [doc = "specified through `fees_transfer_type`."] # [doc = ""] # [doc = "The caller needs to specify what should happen to the transferred assets once they reach"] # [doc = "the `dest` chain. This is done through the `custom_xcm_on_dest` parameter, which"] # [doc = "contains the instructions to execute on `dest` as a final step."] # [doc = "  This is usually as simple as:"] # [doc = "  `Xcm(vec![DepositAsset { assets: Wild(AllCounted(assets.len())), beneficiary }])`,"] # [doc = "  but could be something more exotic like sending the `assets` even further."] # [doc = ""] # [doc = "- `origin`: Must be capable of withdrawing the `assets` and executing XCM."] # [doc = "- `dest`: Destination context for the assets. Will typically be `[Parent,"] # [doc = "  Parachain(..)]` to send from parachain to parachain, or `[Parachain(..)]` to send from"] # [doc = "  relay to parachain, or `(parents: 2, (GlobalConsensus(..), ..))` to send from"] # [doc = "  parachain across a bridge to another ecosystem destination."] # [doc = "- `assets`: The assets to be withdrawn. This should include the assets used to pay the"] # [doc = "  fee on the `dest` (and possibly reserve) chains."] # [doc = "- `assets_transfer_type`: The XCM `TransferType` used to transfer the `assets`."] # [doc = "- `remote_fees_id`: One of the included `assets` to be used to pay fees."] # [doc = "- `fees_transfer_type`: The XCM `TransferType` used to transfer the `fees` assets."] # [doc = "- `custom_xcm_on_dest`: The XCM to be executed on `dest` chain as the last step of the"] # [doc = "  transfer, which also determines what happens to the assets on the destination chain."] # [doc = "- `weight_limit`: The remote-side weight limit, if any, for the XCM fee purchase."] transfer_assets_using_type_and_then { dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedLocation > , assets : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssets > , assets_transfer_type : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: staging_xcm_executor :: traits :: asset_transfer :: TransferType > , remote_fees_id : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedAssetId > , fees_transfer_type : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: staging_xcm_executor :: traits :: asset_transfer :: TransferType > , custom_xcm_on_dest : :: subxt :: ext :: subxt_core :: alloc :: boxed :: Box < runtime_types :: xcm :: VersionedXcm > , weight_limit : runtime_types :: xcm :: v3 :: WeightLimit , } , }
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Origin {
					#[codec(index = 0)]
					Xcm(runtime_types::staging_xcm::v4::location::Location),
					#[codec(index = 1)]
					Response(runtime_types::staging_xcm::v4::location::Location),
				}
			}
		}
		pub mod paseo_parachain_runtime {
			use super::runtime_types;
			pub mod asset_config {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum ForeignAssetType {
					#[codec(index = 0)]
					Xcm(runtime_types::staging_xcm::v4::location::Location),
				}
			}
			pub mod governance_v2 {
				use super::runtime_types;
				pub mod origins {
					use super::runtime_types;
					pub mod pallet_custom_origins {
						use super::runtime_types;
						#[derive(
							:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
							:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
							:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
							:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
							Debug,
						)]
						# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
						#[codec(dumb_trait_bound)]
						#[decode_as_type(
							crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
						)]
						#[encode_as_type(
							crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
						)]
						pub enum Origin {
							#[codec(index = 0)]
							ReferendumCanceller,
							#[codec(index = 1)]
							ReferendumKiller,
							#[codec(index = 2)]
							WhitelistedCaller,
						}
					}
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum OmniAccountPermission {
				#[codec(index = 0)]
				All,
				#[codec(index = 1)]
				AccountManagement,
				#[codec(index = 2)]
				RequestNativeIntent,
				#[codec(index = 3)]
				RequestEthereumIntent,
				#[codec(index = 4)]
				RequestSolanaIntent,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum OriginCaller {
				# [codec (index = 0)] system (runtime_types :: frame_support :: dispatch :: RawOrigin < :: subxt :: ext :: subxt_core :: utils :: AccountId32 > ,) , # [codec (index = 22)] Council (runtime_types :: pallet_collective :: RawOrigin < :: subxt :: ext :: subxt_core :: utils :: AccountId32 > ,) , # [codec (index = 24)] TechnicalCommittee (runtime_types :: pallet_collective :: RawOrigin < :: subxt :: ext :: subxt_core :: utils :: AccountId32 > ,) , # [codec (index = 51)] PolkadotXcm (runtime_types :: pallet_xcm :: pallet :: Origin ,) , # [codec (index = 52)] CumulusXcm (runtime_types :: cumulus_pallet_xcm :: pallet :: Origin ,) , # [codec (index = 73)] DeveloperCommittee (runtime_types :: pallet_collective :: RawOrigin < :: subxt :: ext :: subxt_core :: utils :: AccountId32 > ,) , # [codec (index = 84)] OmniAccount (runtime_types :: pallet_omni_account :: RawOrigin < :: subxt :: ext :: subxt_core :: utils :: AccountId32 > ,) , # [codec (index = 121)] Ethereum (runtime_types :: pallet_ethereum :: RawOrigin ,) , # [codec (index = 172)] Origins (runtime_types :: paseo_parachain_runtime :: governance_v2 :: origins :: pallet_custom_origins :: Origin ,) , # [codec (index = 9)] Void (runtime_types :: sp_core :: Void ,) , }
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum ProxyType {
				#[codec(index = 0)]
				Any,
				#[codec(index = 1)]
				NonTransfer,
				#[codec(index = 2)]
				CancelProxy,
				#[codec(index = 3)]
				Collator,
				#[codec(index = 4)]
				Governance,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct Runtime;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum RuntimeCall {
				#[codec(index = 0)]
				System(runtime_types::frame_system::pallet::Call),
				#[codec(index = 1)]
				Timestamp(runtime_types::pallet_timestamp::pallet::Call),
				#[codec(index = 2)]
				Scheduler(runtime_types::pallet_scheduler::pallet::Call),
				#[codec(index = 3)]
				Utility(runtime_types::pallet_utility::pallet::Call),
				#[codec(index = 4)]
				Multisig(runtime_types::pallet_multisig::pallet::Call),
				#[codec(index = 5)]
				Proxy(runtime_types::pallet_proxy::pallet::Call),
				#[codec(index = 6)]
				Preimage(runtime_types::pallet_preimage::pallet::Call),
				#[codec(index = 10)]
				Balances(runtime_types::pallet_balances::pallet::Call),
				#[codec(index = 11)]
				Vesting(runtime_types::pallet_vesting::pallet::Call),
				#[codec(index = 13)]
				Treasury(runtime_types::pallet_treasury::pallet::Call),
				#[codec(index = 21)]
				Democracy(runtime_types::pallet_democracy::pallet::Call),
				#[codec(index = 22)]
				Council(runtime_types::pallet_collective::pallet::Call),
				#[codec(index = 23)]
				CouncilMembership(runtime_types::pallet_membership::pallet::Call),
				#[codec(index = 24)]
				TechnicalCommittee(runtime_types::pallet_collective::pallet::Call),
				#[codec(index = 25)]
				TechnicalCommitteeMembership(runtime_types::pallet_membership::pallet::Call),
				#[codec(index = 26)]
				Bounties(runtime_types::pallet_bounties::pallet::Call),
				#[codec(index = 27)]
				Tips(runtime_types::pallet_tips::pallet::Call),
				#[codec(index = 28)]
				ParachainIdentity(runtime_types::pallet_identity::pallet::Call),
				#[codec(index = 30)]
				ParachainSystem(runtime_types::cumulus_pallet_parachain_system::pallet::Call),
				#[codec(index = 31)]
				ParachainInfo(runtime_types::staging_parachain_info::pallet::Call),
				#[codec(index = 42)]
				Session(runtime_types::pallet_session::pallet::Call),
				#[codec(index = 45)]
				ParachainStaking(runtime_types::pallet_parachain_staking::pallet::Call),
				#[codec(index = 50)]
				XcmpQueue(runtime_types::cumulus_pallet_xcmp_queue::pallet::Call),
				#[codec(index = 51)]
				PolkadotXcm(runtime_types::pallet_xcm::pallet::Call),
				#[codec(index = 52)]
				CumulusXcm(runtime_types::cumulus_pallet_xcm::pallet::Call),
				#[codec(index = 56)]
				Assets(runtime_types::pallet_assets::pallet::Call),
				#[codec(index = 57)]
				MessageQueue(runtime_types::pallet_message_queue::pallet::Call),
				#[codec(index = 60)]
				ChainBridge(runtime_types::pallet_chain_bridge::pallet::Call),
				#[codec(index = 61)]
				BridgeTransfer(runtime_types::pallet_bridge_transfer::pallet::Call),
				#[codec(index = 63)]
				ExtrinsicFilter(runtime_types::pallet_extrinsic_filter::pallet::Call),
				#[codec(index = 64)]
				IdentityManagement(runtime_types::pallet_identity_management::pallet::Call),
				#[codec(index = 65)]
				AssetManager(runtime_types::pallet_asset_manager::pallet::Call),
				#[codec(index = 66)]
				VCManagement(runtime_types::pallet_vc_management::pallet::Call),
				#[codec(index = 67)]
				IMPExtrinsicWhitelist(runtime_types::pallet_group::pallet::Call),
				#[codec(index = 68)]
				VCMPExtrinsicWhitelist(runtime_types::pallet_group::pallet::Call),
				#[codec(index = 70)]
				Bitacross(runtime_types::pallet_bitacross::pallet::Call),
				#[codec(index = 71)]
				EvmAssertions(runtime_types::pallet_evm_assertions::pallet::Call),
				#[codec(index = 73)]
				DeveloperCommittee(runtime_types::pallet_collective::pallet::Call),
				#[codec(index = 74)]
				DeveloperCommitteeMembership(runtime_types::pallet_membership::pallet::Call),
				#[codec(index = 75)]
				ScoreStaking(runtime_types::pallet_score_staking::pallet::Call),
				#[codec(index = 76)]
				AssetsHandler(runtime_types::pallet_assets_handler::pallet::Call),
				#[codec(index = 84)]
				OmniAccount(runtime_types::pallet_omni_account::pallet::Call),
				#[codec(index = 85)]
				OmniBridge(runtime_types::pallet_omni_bridge::pallet::Call),
				#[codec(index = 93)]
				Teebag(runtime_types::pallet_teebag::pallet::Call),
				#[codec(index = 120)]
				EVM(runtime_types::pallet_evm::pallet::Call),
				#[codec(index = 121)]
				Ethereum(runtime_types::pallet_ethereum::pallet::Call),
				#[codec(index = 150)]
				Curator(runtime_types::pallet_curator::pallet::Call),
				#[codec(index = 151)]
				Guardian(runtime_types::pallet_guardian::pallet::Call),
				#[codec(index = 152)]
				PoolProposal(runtime_types::pallet_pool_proposal::pallet::Call),
				#[codec(index = 153)]
				InvestingPool(runtime_types::pallet_investing_pool::pallet::Call),
				#[codec(index = 154)]
				AIUSDConvertor(runtime_types::pallet_aiusd_convertor::pallet::Call),
				#[codec(index = 170)]
				ConvictionVoting(runtime_types::pallet_conviction_voting::pallet::Call),
				#[codec(index = 171)]
				Referenda(runtime_types::pallet_referenda::pallet::Call),
				#[codec(index = 173)]
				Whitelist(runtime_types::pallet_whitelist::pallet::Call),
				#[codec(index = 254)]
				AccountFix(runtime_types::pallet_account_fix::pallet::Call),
				#[codec(index = 255)]
				Sudo(runtime_types::pallet_sudo::pallet::Call),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum RuntimeError {
				#[codec(index = 84)]
				OmniAccount(runtime_types::pallet_omni_account::pallet::Error),
				#[codec(index = 93)]
				Teebag(runtime_types::pallet_teebag::pallet::Error),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum RuntimeEvent {
				#[codec(index = 84)]
				OmniAccount(runtime_types::pallet_omni_account::pallet::Event),
				#[codec(index = 93)]
				Teebag(runtime_types::pallet_teebag::pallet::Event),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct SessionKeys {
				pub aura: runtime_types::sp_consensus_aura::sr25519::app_sr25519::Public,
			}
		}
		pub mod polkadot_core_primitives {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct InboundDownwardMessage<_0> {
				pub sent_at: _0,
				pub msg: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct InboundHrmpMessage<_0> {
				pub sent_at: _0,
				pub data: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct OutboundHrmpMessage<_0> {
				pub recipient: _0,
				pub data: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			}
		}
		pub mod polkadot_parachain_primitives {
			use super::runtime_types;
			pub mod primitives {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct HeadData(
					pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Id(pub ::core::primitive::u32);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ValidationCode(
					pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				);
			}
		}
		pub mod polkadot_primitives {
			use super::runtime_types;
			pub mod v7 {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct PersistedValidationData<_0, _1> {
					pub parent_head:
						runtime_types::polkadot_parachain_primitives::primitives::HeadData,
					pub relay_parent_number: _1,
					pub relay_parent_storage_root: _0,
					pub max_pov_size: ::core::primitive::u32,
				}
			}
		}
		pub mod primitive_types {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct U256(pub [::core::primitive::u64; 4usize]);
		}
		pub mod sp_arithmetic {
			use super::runtime_types;
			pub mod per_things {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Perbill(pub ::core::primitive::u32);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Percent(pub ::core::primitive::u8);
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Permill(pub ::core::primitive::u32);
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum ArithmeticError {
				#[codec(index = 0)]
				Underflow,
				#[codec(index = 1)]
				Overflow,
				#[codec(index = 2)]
				DivisionByZero,
			}
		}
		pub mod sp_consensus_aura {
			use super::runtime_types;
			pub mod sr25519 {
				use super::runtime_types;
				pub mod app_sr25519 {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Public(pub [::core::primitive::u8; 32usize]);
				}
			}
		}
		pub mod sp_consensus_slots {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct Slot(pub ::core::primitive::u64);
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: CompactAs,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct SlotDuration(pub ::core::primitive::u64);
		}
		pub mod sp_core {
			use super::runtime_types;
			pub mod crypto {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct KeyTypeId(pub [::core::primitive::u8; 4usize]);
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct OpaqueMetadata(
				pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			);
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum Void {}
		}
		pub mod sp_inherents {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct CheckInherentsResult {
				pub okay: ::core::primitive::bool,
				pub fatal_error: ::core::primitive::bool,
				pub errors: runtime_types::sp_inherents::InherentData,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct InherentData {
				pub data: ::subxt::ext::subxt_core::utils::KeyedVec<
					[::core::primitive::u8; 8usize],
					::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				>,
			}
		}
		pub mod sp_runtime {
			use super::runtime_types;
			pub mod generic {
				use super::runtime_types;
				pub mod block {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Block<_0, _1> {
						pub header: _0,
						pub extrinsics: ::subxt::ext::subxt_core::alloc::vec::Vec<_1>,
					}
				}
				pub mod digest {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Digest {
						pub logs: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::sp_runtime::generic::digest::DigestItem,
						>,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum DigestItem {
						#[codec(index = 6)]
						PreRuntime(
							[::core::primitive::u8; 4usize],
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						),
						#[codec(index = 4)]
						Consensus(
							[::core::primitive::u8; 4usize],
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						),
						#[codec(index = 5)]
						Seal(
							[::core::primitive::u8; 4usize],
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						),
						#[codec(index = 0)]
						Other(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
						#[codec(index = 8)]
						RuntimeEnvironmentUpdated,
					}
				}
				pub mod era {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Era {
						#[codec(index = 0)]
						Immortal,
						#[codec(index = 1)]
						Mortal1(::core::primitive::u8),
						#[codec(index = 2)]
						Mortal2(::core::primitive::u8),
						#[codec(index = 3)]
						Mortal3(::core::primitive::u8),
						#[codec(index = 4)]
						Mortal4(::core::primitive::u8),
						#[codec(index = 5)]
						Mortal5(::core::primitive::u8),
						#[codec(index = 6)]
						Mortal6(::core::primitive::u8),
						#[codec(index = 7)]
						Mortal7(::core::primitive::u8),
						#[codec(index = 8)]
						Mortal8(::core::primitive::u8),
						#[codec(index = 9)]
						Mortal9(::core::primitive::u8),
						#[codec(index = 10)]
						Mortal10(::core::primitive::u8),
						#[codec(index = 11)]
						Mortal11(::core::primitive::u8),
						#[codec(index = 12)]
						Mortal12(::core::primitive::u8),
						#[codec(index = 13)]
						Mortal13(::core::primitive::u8),
						#[codec(index = 14)]
						Mortal14(::core::primitive::u8),
						#[codec(index = 15)]
						Mortal15(::core::primitive::u8),
						#[codec(index = 16)]
						Mortal16(::core::primitive::u8),
						#[codec(index = 17)]
						Mortal17(::core::primitive::u8),
						#[codec(index = 18)]
						Mortal18(::core::primitive::u8),
						#[codec(index = 19)]
						Mortal19(::core::primitive::u8),
						#[codec(index = 20)]
						Mortal20(::core::primitive::u8),
						#[codec(index = 21)]
						Mortal21(::core::primitive::u8),
						#[codec(index = 22)]
						Mortal22(::core::primitive::u8),
						#[codec(index = 23)]
						Mortal23(::core::primitive::u8),
						#[codec(index = 24)]
						Mortal24(::core::primitive::u8),
						#[codec(index = 25)]
						Mortal25(::core::primitive::u8),
						#[codec(index = 26)]
						Mortal26(::core::primitive::u8),
						#[codec(index = 27)]
						Mortal27(::core::primitive::u8),
						#[codec(index = 28)]
						Mortal28(::core::primitive::u8),
						#[codec(index = 29)]
						Mortal29(::core::primitive::u8),
						#[codec(index = 30)]
						Mortal30(::core::primitive::u8),
						#[codec(index = 31)]
						Mortal31(::core::primitive::u8),
						#[codec(index = 32)]
						Mortal32(::core::primitive::u8),
						#[codec(index = 33)]
						Mortal33(::core::primitive::u8),
						#[codec(index = 34)]
						Mortal34(::core::primitive::u8),
						#[codec(index = 35)]
						Mortal35(::core::primitive::u8),
						#[codec(index = 36)]
						Mortal36(::core::primitive::u8),
						#[codec(index = 37)]
						Mortal37(::core::primitive::u8),
						#[codec(index = 38)]
						Mortal38(::core::primitive::u8),
						#[codec(index = 39)]
						Mortal39(::core::primitive::u8),
						#[codec(index = 40)]
						Mortal40(::core::primitive::u8),
						#[codec(index = 41)]
						Mortal41(::core::primitive::u8),
						#[codec(index = 42)]
						Mortal42(::core::primitive::u8),
						#[codec(index = 43)]
						Mortal43(::core::primitive::u8),
						#[codec(index = 44)]
						Mortal44(::core::primitive::u8),
						#[codec(index = 45)]
						Mortal45(::core::primitive::u8),
						#[codec(index = 46)]
						Mortal46(::core::primitive::u8),
						#[codec(index = 47)]
						Mortal47(::core::primitive::u8),
						#[codec(index = 48)]
						Mortal48(::core::primitive::u8),
						#[codec(index = 49)]
						Mortal49(::core::primitive::u8),
						#[codec(index = 50)]
						Mortal50(::core::primitive::u8),
						#[codec(index = 51)]
						Mortal51(::core::primitive::u8),
						#[codec(index = 52)]
						Mortal52(::core::primitive::u8),
						#[codec(index = 53)]
						Mortal53(::core::primitive::u8),
						#[codec(index = 54)]
						Mortal54(::core::primitive::u8),
						#[codec(index = 55)]
						Mortal55(::core::primitive::u8),
						#[codec(index = 56)]
						Mortal56(::core::primitive::u8),
						#[codec(index = 57)]
						Mortal57(::core::primitive::u8),
						#[codec(index = 58)]
						Mortal58(::core::primitive::u8),
						#[codec(index = 59)]
						Mortal59(::core::primitive::u8),
						#[codec(index = 60)]
						Mortal60(::core::primitive::u8),
						#[codec(index = 61)]
						Mortal61(::core::primitive::u8),
						#[codec(index = 62)]
						Mortal62(::core::primitive::u8),
						#[codec(index = 63)]
						Mortal63(::core::primitive::u8),
						#[codec(index = 64)]
						Mortal64(::core::primitive::u8),
						#[codec(index = 65)]
						Mortal65(::core::primitive::u8),
						#[codec(index = 66)]
						Mortal66(::core::primitive::u8),
						#[codec(index = 67)]
						Mortal67(::core::primitive::u8),
						#[codec(index = 68)]
						Mortal68(::core::primitive::u8),
						#[codec(index = 69)]
						Mortal69(::core::primitive::u8),
						#[codec(index = 70)]
						Mortal70(::core::primitive::u8),
						#[codec(index = 71)]
						Mortal71(::core::primitive::u8),
						#[codec(index = 72)]
						Mortal72(::core::primitive::u8),
						#[codec(index = 73)]
						Mortal73(::core::primitive::u8),
						#[codec(index = 74)]
						Mortal74(::core::primitive::u8),
						#[codec(index = 75)]
						Mortal75(::core::primitive::u8),
						#[codec(index = 76)]
						Mortal76(::core::primitive::u8),
						#[codec(index = 77)]
						Mortal77(::core::primitive::u8),
						#[codec(index = 78)]
						Mortal78(::core::primitive::u8),
						#[codec(index = 79)]
						Mortal79(::core::primitive::u8),
						#[codec(index = 80)]
						Mortal80(::core::primitive::u8),
						#[codec(index = 81)]
						Mortal81(::core::primitive::u8),
						#[codec(index = 82)]
						Mortal82(::core::primitive::u8),
						#[codec(index = 83)]
						Mortal83(::core::primitive::u8),
						#[codec(index = 84)]
						Mortal84(::core::primitive::u8),
						#[codec(index = 85)]
						Mortal85(::core::primitive::u8),
						#[codec(index = 86)]
						Mortal86(::core::primitive::u8),
						#[codec(index = 87)]
						Mortal87(::core::primitive::u8),
						#[codec(index = 88)]
						Mortal88(::core::primitive::u8),
						#[codec(index = 89)]
						Mortal89(::core::primitive::u8),
						#[codec(index = 90)]
						Mortal90(::core::primitive::u8),
						#[codec(index = 91)]
						Mortal91(::core::primitive::u8),
						#[codec(index = 92)]
						Mortal92(::core::primitive::u8),
						#[codec(index = 93)]
						Mortal93(::core::primitive::u8),
						#[codec(index = 94)]
						Mortal94(::core::primitive::u8),
						#[codec(index = 95)]
						Mortal95(::core::primitive::u8),
						#[codec(index = 96)]
						Mortal96(::core::primitive::u8),
						#[codec(index = 97)]
						Mortal97(::core::primitive::u8),
						#[codec(index = 98)]
						Mortal98(::core::primitive::u8),
						#[codec(index = 99)]
						Mortal99(::core::primitive::u8),
						#[codec(index = 100)]
						Mortal100(::core::primitive::u8),
						#[codec(index = 101)]
						Mortal101(::core::primitive::u8),
						#[codec(index = 102)]
						Mortal102(::core::primitive::u8),
						#[codec(index = 103)]
						Mortal103(::core::primitive::u8),
						#[codec(index = 104)]
						Mortal104(::core::primitive::u8),
						#[codec(index = 105)]
						Mortal105(::core::primitive::u8),
						#[codec(index = 106)]
						Mortal106(::core::primitive::u8),
						#[codec(index = 107)]
						Mortal107(::core::primitive::u8),
						#[codec(index = 108)]
						Mortal108(::core::primitive::u8),
						#[codec(index = 109)]
						Mortal109(::core::primitive::u8),
						#[codec(index = 110)]
						Mortal110(::core::primitive::u8),
						#[codec(index = 111)]
						Mortal111(::core::primitive::u8),
						#[codec(index = 112)]
						Mortal112(::core::primitive::u8),
						#[codec(index = 113)]
						Mortal113(::core::primitive::u8),
						#[codec(index = 114)]
						Mortal114(::core::primitive::u8),
						#[codec(index = 115)]
						Mortal115(::core::primitive::u8),
						#[codec(index = 116)]
						Mortal116(::core::primitive::u8),
						#[codec(index = 117)]
						Mortal117(::core::primitive::u8),
						#[codec(index = 118)]
						Mortal118(::core::primitive::u8),
						#[codec(index = 119)]
						Mortal119(::core::primitive::u8),
						#[codec(index = 120)]
						Mortal120(::core::primitive::u8),
						#[codec(index = 121)]
						Mortal121(::core::primitive::u8),
						#[codec(index = 122)]
						Mortal122(::core::primitive::u8),
						#[codec(index = 123)]
						Mortal123(::core::primitive::u8),
						#[codec(index = 124)]
						Mortal124(::core::primitive::u8),
						#[codec(index = 125)]
						Mortal125(::core::primitive::u8),
						#[codec(index = 126)]
						Mortal126(::core::primitive::u8),
						#[codec(index = 127)]
						Mortal127(::core::primitive::u8),
						#[codec(index = 128)]
						Mortal128(::core::primitive::u8),
						#[codec(index = 129)]
						Mortal129(::core::primitive::u8),
						#[codec(index = 130)]
						Mortal130(::core::primitive::u8),
						#[codec(index = 131)]
						Mortal131(::core::primitive::u8),
						#[codec(index = 132)]
						Mortal132(::core::primitive::u8),
						#[codec(index = 133)]
						Mortal133(::core::primitive::u8),
						#[codec(index = 134)]
						Mortal134(::core::primitive::u8),
						#[codec(index = 135)]
						Mortal135(::core::primitive::u8),
						#[codec(index = 136)]
						Mortal136(::core::primitive::u8),
						#[codec(index = 137)]
						Mortal137(::core::primitive::u8),
						#[codec(index = 138)]
						Mortal138(::core::primitive::u8),
						#[codec(index = 139)]
						Mortal139(::core::primitive::u8),
						#[codec(index = 140)]
						Mortal140(::core::primitive::u8),
						#[codec(index = 141)]
						Mortal141(::core::primitive::u8),
						#[codec(index = 142)]
						Mortal142(::core::primitive::u8),
						#[codec(index = 143)]
						Mortal143(::core::primitive::u8),
						#[codec(index = 144)]
						Mortal144(::core::primitive::u8),
						#[codec(index = 145)]
						Mortal145(::core::primitive::u8),
						#[codec(index = 146)]
						Mortal146(::core::primitive::u8),
						#[codec(index = 147)]
						Mortal147(::core::primitive::u8),
						#[codec(index = 148)]
						Mortal148(::core::primitive::u8),
						#[codec(index = 149)]
						Mortal149(::core::primitive::u8),
						#[codec(index = 150)]
						Mortal150(::core::primitive::u8),
						#[codec(index = 151)]
						Mortal151(::core::primitive::u8),
						#[codec(index = 152)]
						Mortal152(::core::primitive::u8),
						#[codec(index = 153)]
						Mortal153(::core::primitive::u8),
						#[codec(index = 154)]
						Mortal154(::core::primitive::u8),
						#[codec(index = 155)]
						Mortal155(::core::primitive::u8),
						#[codec(index = 156)]
						Mortal156(::core::primitive::u8),
						#[codec(index = 157)]
						Mortal157(::core::primitive::u8),
						#[codec(index = 158)]
						Mortal158(::core::primitive::u8),
						#[codec(index = 159)]
						Mortal159(::core::primitive::u8),
						#[codec(index = 160)]
						Mortal160(::core::primitive::u8),
						#[codec(index = 161)]
						Mortal161(::core::primitive::u8),
						#[codec(index = 162)]
						Mortal162(::core::primitive::u8),
						#[codec(index = 163)]
						Mortal163(::core::primitive::u8),
						#[codec(index = 164)]
						Mortal164(::core::primitive::u8),
						#[codec(index = 165)]
						Mortal165(::core::primitive::u8),
						#[codec(index = 166)]
						Mortal166(::core::primitive::u8),
						#[codec(index = 167)]
						Mortal167(::core::primitive::u8),
						#[codec(index = 168)]
						Mortal168(::core::primitive::u8),
						#[codec(index = 169)]
						Mortal169(::core::primitive::u8),
						#[codec(index = 170)]
						Mortal170(::core::primitive::u8),
						#[codec(index = 171)]
						Mortal171(::core::primitive::u8),
						#[codec(index = 172)]
						Mortal172(::core::primitive::u8),
						#[codec(index = 173)]
						Mortal173(::core::primitive::u8),
						#[codec(index = 174)]
						Mortal174(::core::primitive::u8),
						#[codec(index = 175)]
						Mortal175(::core::primitive::u8),
						#[codec(index = 176)]
						Mortal176(::core::primitive::u8),
						#[codec(index = 177)]
						Mortal177(::core::primitive::u8),
						#[codec(index = 178)]
						Mortal178(::core::primitive::u8),
						#[codec(index = 179)]
						Mortal179(::core::primitive::u8),
						#[codec(index = 180)]
						Mortal180(::core::primitive::u8),
						#[codec(index = 181)]
						Mortal181(::core::primitive::u8),
						#[codec(index = 182)]
						Mortal182(::core::primitive::u8),
						#[codec(index = 183)]
						Mortal183(::core::primitive::u8),
						#[codec(index = 184)]
						Mortal184(::core::primitive::u8),
						#[codec(index = 185)]
						Mortal185(::core::primitive::u8),
						#[codec(index = 186)]
						Mortal186(::core::primitive::u8),
						#[codec(index = 187)]
						Mortal187(::core::primitive::u8),
						#[codec(index = 188)]
						Mortal188(::core::primitive::u8),
						#[codec(index = 189)]
						Mortal189(::core::primitive::u8),
						#[codec(index = 190)]
						Mortal190(::core::primitive::u8),
						#[codec(index = 191)]
						Mortal191(::core::primitive::u8),
						#[codec(index = 192)]
						Mortal192(::core::primitive::u8),
						#[codec(index = 193)]
						Mortal193(::core::primitive::u8),
						#[codec(index = 194)]
						Mortal194(::core::primitive::u8),
						#[codec(index = 195)]
						Mortal195(::core::primitive::u8),
						#[codec(index = 196)]
						Mortal196(::core::primitive::u8),
						#[codec(index = 197)]
						Mortal197(::core::primitive::u8),
						#[codec(index = 198)]
						Mortal198(::core::primitive::u8),
						#[codec(index = 199)]
						Mortal199(::core::primitive::u8),
						#[codec(index = 200)]
						Mortal200(::core::primitive::u8),
						#[codec(index = 201)]
						Mortal201(::core::primitive::u8),
						#[codec(index = 202)]
						Mortal202(::core::primitive::u8),
						#[codec(index = 203)]
						Mortal203(::core::primitive::u8),
						#[codec(index = 204)]
						Mortal204(::core::primitive::u8),
						#[codec(index = 205)]
						Mortal205(::core::primitive::u8),
						#[codec(index = 206)]
						Mortal206(::core::primitive::u8),
						#[codec(index = 207)]
						Mortal207(::core::primitive::u8),
						#[codec(index = 208)]
						Mortal208(::core::primitive::u8),
						#[codec(index = 209)]
						Mortal209(::core::primitive::u8),
						#[codec(index = 210)]
						Mortal210(::core::primitive::u8),
						#[codec(index = 211)]
						Mortal211(::core::primitive::u8),
						#[codec(index = 212)]
						Mortal212(::core::primitive::u8),
						#[codec(index = 213)]
						Mortal213(::core::primitive::u8),
						#[codec(index = 214)]
						Mortal214(::core::primitive::u8),
						#[codec(index = 215)]
						Mortal215(::core::primitive::u8),
						#[codec(index = 216)]
						Mortal216(::core::primitive::u8),
						#[codec(index = 217)]
						Mortal217(::core::primitive::u8),
						#[codec(index = 218)]
						Mortal218(::core::primitive::u8),
						#[codec(index = 219)]
						Mortal219(::core::primitive::u8),
						#[codec(index = 220)]
						Mortal220(::core::primitive::u8),
						#[codec(index = 221)]
						Mortal221(::core::primitive::u8),
						#[codec(index = 222)]
						Mortal222(::core::primitive::u8),
						#[codec(index = 223)]
						Mortal223(::core::primitive::u8),
						#[codec(index = 224)]
						Mortal224(::core::primitive::u8),
						#[codec(index = 225)]
						Mortal225(::core::primitive::u8),
						#[codec(index = 226)]
						Mortal226(::core::primitive::u8),
						#[codec(index = 227)]
						Mortal227(::core::primitive::u8),
						#[codec(index = 228)]
						Mortal228(::core::primitive::u8),
						#[codec(index = 229)]
						Mortal229(::core::primitive::u8),
						#[codec(index = 230)]
						Mortal230(::core::primitive::u8),
						#[codec(index = 231)]
						Mortal231(::core::primitive::u8),
						#[codec(index = 232)]
						Mortal232(::core::primitive::u8),
						#[codec(index = 233)]
						Mortal233(::core::primitive::u8),
						#[codec(index = 234)]
						Mortal234(::core::primitive::u8),
						#[codec(index = 235)]
						Mortal235(::core::primitive::u8),
						#[codec(index = 236)]
						Mortal236(::core::primitive::u8),
						#[codec(index = 237)]
						Mortal237(::core::primitive::u8),
						#[codec(index = 238)]
						Mortal238(::core::primitive::u8),
						#[codec(index = 239)]
						Mortal239(::core::primitive::u8),
						#[codec(index = 240)]
						Mortal240(::core::primitive::u8),
						#[codec(index = 241)]
						Mortal241(::core::primitive::u8),
						#[codec(index = 242)]
						Mortal242(::core::primitive::u8),
						#[codec(index = 243)]
						Mortal243(::core::primitive::u8),
						#[codec(index = 244)]
						Mortal244(::core::primitive::u8),
						#[codec(index = 245)]
						Mortal245(::core::primitive::u8),
						#[codec(index = 246)]
						Mortal246(::core::primitive::u8),
						#[codec(index = 247)]
						Mortal247(::core::primitive::u8),
						#[codec(index = 248)]
						Mortal248(::core::primitive::u8),
						#[codec(index = 249)]
						Mortal249(::core::primitive::u8),
						#[codec(index = 250)]
						Mortal250(::core::primitive::u8),
						#[codec(index = 251)]
						Mortal251(::core::primitive::u8),
						#[codec(index = 252)]
						Mortal252(::core::primitive::u8),
						#[codec(index = 253)]
						Mortal253(::core::primitive::u8),
						#[codec(index = 254)]
						Mortal254(::core::primitive::u8),
						#[codec(index = 255)]
						Mortal255(::core::primitive::u8),
					}
				}
				pub mod header {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Header<_0> {
						pub parent_hash: ::subxt::ext::subxt_core::utils::H256,
						#[codec(compact)]
						pub number: _0,
						pub state_root: ::subxt::ext::subxt_core::utils::H256,
						pub extrinsics_root: ::subxt::ext::subxt_core::utils::H256,
						pub digest: runtime_types::sp_runtime::generic::digest::Digest,
					}
				}
			}
			pub mod traits {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct BlakeTwo256;
			}
			pub mod transaction_validity {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum InvalidTransaction {
					#[codec(index = 0)]
					Call,
					#[codec(index = 1)]
					Payment,
					#[codec(index = 2)]
					Future,
					#[codec(index = 3)]
					Stale,
					#[codec(index = 4)]
					BadProof,
					#[codec(index = 5)]
					AncientBirthBlock,
					#[codec(index = 6)]
					ExhaustsResources,
					#[codec(index = 7)]
					Custom(::core::primitive::u8),
					#[codec(index = 8)]
					BadMandatory,
					#[codec(index = 9)]
					MandatoryValidation,
					#[codec(index = 10)]
					BadSigner,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum TransactionSource {
					#[codec(index = 0)]
					InBlock,
					#[codec(index = 1)]
					Local,
					#[codec(index = 2)]
					External,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum TransactionValidityError {
					#[codec(index = 0)]
					Invalid(runtime_types::sp_runtime::transaction_validity::InvalidTransaction),
					#[codec(index = 1)]
					Unknown(runtime_types::sp_runtime::transaction_validity::UnknownTransaction),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum UnknownTransaction {
					#[codec(index = 0)]
					CannotLookup,
					#[codec(index = 1)]
					NoUnsignedValidator,
					#[codec(index = 2)]
					Custom(::core::primitive::u8),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct ValidTransaction {
					pub priority: ::core::primitive::u64,
					pub requires: ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					>,
					pub provides: ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					>,
					pub longevity: ::core::primitive::u64,
					pub propagate: ::core::primitive::bool,
				}
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum DispatchError {
				#[codec(index = 0)]
				Other,
				#[codec(index = 1)]
				CannotLookup,
				#[codec(index = 2)]
				BadOrigin,
				#[codec(index = 3)]
				Module(runtime_types::sp_runtime::ModuleError),
				#[codec(index = 4)]
				ConsumerRemaining,
				#[codec(index = 5)]
				NoProviders,
				#[codec(index = 6)]
				TooManyConsumers,
				#[codec(index = 7)]
				Token(runtime_types::sp_runtime::TokenError),
				#[codec(index = 8)]
				Arithmetic(runtime_types::sp_arithmetic::ArithmeticError),
				#[codec(index = 9)]
				Transactional(runtime_types::sp_runtime::TransactionalError),
				#[codec(index = 10)]
				Exhausted,
				#[codec(index = 11)]
				Corruption,
				#[codec(index = 12)]
				Unavailable,
				#[codec(index = 13)]
				RootNotAllowed,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum ExtrinsicInclusionMode {
				#[codec(index = 0)]
				AllExtrinsics,
				#[codec(index = 1)]
				OnlyInherents,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct ModuleError {
				pub index: ::core::primitive::u8,
				pub error: [::core::primitive::u8; 4usize],
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum MultiSignature {
				#[codec(index = 0)]
				Ed25519([::core::primitive::u8; 64usize]),
				#[codec(index = 1)]
				Sr25519([::core::primitive::u8; 64usize]),
				#[codec(index = 2)]
				Ecdsa([::core::primitive::u8; 65usize]),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum TokenError {
				#[codec(index = 0)]
				FundsUnavailable,
				#[codec(index = 1)]
				OnlyProvider,
				#[codec(index = 2)]
				BelowMinimum,
				#[codec(index = 3)]
				CannotCreate,
				#[codec(index = 4)]
				UnknownAsset,
				#[codec(index = 5)]
				Frozen,
				#[codec(index = 6)]
				Unsupported,
				#[codec(index = 7)]
				CannotCreateHold,
				#[codec(index = 8)]
				NotExpendable,
				#[codec(index = 9)]
				Blocked,
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum TransactionalError {
				#[codec(index = 0)]
				LimitReached,
				#[codec(index = 1)]
				NoLayer,
			}
		}
		pub mod sp_trie {
			use super::runtime_types;
			pub mod storage_proof {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct StorageProof {
					pub trie_nodes: ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					>,
				}
			}
		}
		pub mod sp_version {
			use super::runtime_types;
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub struct RuntimeVersion {
				pub spec_name: ::subxt::ext::subxt_core::alloc::string::String,
				pub impl_name: ::subxt::ext::subxt_core::alloc::string::String,
				pub authoring_version: ::core::primitive::u32,
				pub spec_version: ::core::primitive::u32,
				pub impl_version: ::core::primitive::u32,
				pub apis: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					[::core::primitive::u8; 8usize],
					::core::primitive::u32,
				)>,
				pub transaction_version: ::core::primitive::u32,
				pub state_version: ::core::primitive::u8,
			}
		}
		pub mod sp_weights {
			use super::runtime_types;
			pub mod weight_v2 {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Weight {
					#[codec(compact)]
					pub ref_time: ::core::primitive::u64,
					#[codec(compact)]
					pub proof_size: ::core::primitive::u64,
				}
			}
		}
		pub mod staging_parachain_info {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				#[doc = "Contains a variant per dispatchable extrinsic that this pallet has."]
				pub enum Call {}
			}
		}
		pub mod staging_xcm {
			use super::runtime_types;
			pub mod v3 {
				use super::runtime_types;
				pub mod multilocation {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct MultiLocation {
						pub parents: ::core::primitive::u8,
						pub interior: runtime_types::xcm::v3::junctions::Junctions,
					}
				}
			}
			pub mod v4 {
				use super::runtime_types;
				pub mod asset {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Asset {
						pub id: runtime_types::staging_xcm::v4::asset::AssetId,
						pub fun: runtime_types::staging_xcm::v4::asset::Fungibility,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AssetFilter {
						#[codec(index = 0)]
						Definite(runtime_types::staging_xcm::v4::asset::Assets),
						#[codec(index = 1)]
						Wild(runtime_types::staging_xcm::v4::asset::WildAsset),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct AssetId(pub runtime_types::staging_xcm::v4::location::Location);
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AssetInstance {
						#[codec(index = 0)]
						Undefined,
						#[codec(index = 1)]
						Index(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 2)]
						Array4([::core::primitive::u8; 4usize]),
						#[codec(index = 3)]
						Array8([::core::primitive::u8; 8usize]),
						#[codec(index = 4)]
						Array16([::core::primitive::u8; 16usize]),
						#[codec(index = 5)]
						Array32([::core::primitive::u8; 32usize]),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Assets(
						pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::staging_xcm::v4::asset::Asset,
						>,
					);
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Fungibility {
						#[codec(index = 0)]
						Fungible(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 1)]
						NonFungible(runtime_types::staging_xcm::v4::asset::AssetInstance),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WildAsset {
						#[codec(index = 0)]
						All,
						#[codec(index = 1)]
						AllOf {
							id: runtime_types::staging_xcm::v4::asset::AssetId,
							fun: runtime_types::staging_xcm::v4::asset::WildFungibility,
						},
						#[codec(index = 2)]
						AllCounted(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 3)]
						AllOfCounted {
							id: runtime_types::staging_xcm::v4::asset::AssetId,
							fun: runtime_types::staging_xcm::v4::asset::WildFungibility,
							#[codec(compact)]
							count: ::core::primitive::u32,
						},
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WildFungibility {
						#[codec(index = 0)]
						Fungible,
						#[codec(index = 1)]
						NonFungible,
					}
				}
				pub mod junction {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Junction {
						#[codec(index = 0)]
						Parachain(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 1)]
						AccountId32 {
							network: ::core::option::Option<
								runtime_types::staging_xcm::v4::junction::NetworkId,
							>,
							id: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						AccountIndex64 {
							network: ::core::option::Option<
								runtime_types::staging_xcm::v4::junction::NetworkId,
							>,
							#[codec(compact)]
							index: ::core::primitive::u64,
						},
						#[codec(index = 3)]
						AccountKey20 {
							network: ::core::option::Option<
								runtime_types::staging_xcm::v4::junction::NetworkId,
							>,
							key: [::core::primitive::u8; 20usize],
						},
						#[codec(index = 4)]
						PalletInstance(::core::primitive::u8),
						#[codec(index = 5)]
						GeneralIndex(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 6)]
						GeneralKey {
							length: ::core::primitive::u8,
							data: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 7)]
						OnlyChild,
						#[codec(index = 8)]
						Plurality {
							id: runtime_types::xcm::v3::junction::BodyId,
							part: runtime_types::xcm::v3::junction::BodyPart,
						},
						#[codec(index = 9)]
						GlobalConsensus(runtime_types::staging_xcm::v4::junction::NetworkId),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum NetworkId {
						#[codec(index = 0)]
						ByGenesis([::core::primitive::u8; 32usize]),
						#[codec(index = 1)]
						ByFork {
							block_number: ::core::primitive::u64,
							block_hash: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						Polkadot,
						#[codec(index = 3)]
						Kusama,
						#[codec(index = 4)]
						Westend,
						#[codec(index = 5)]
						Rococo,
						#[codec(index = 6)]
						Wococo,
						#[codec(index = 7)]
						Ethereum {
							#[codec(compact)]
							chain_id: ::core::primitive::u64,
						},
						#[codec(index = 8)]
						BitcoinCore,
						#[codec(index = 9)]
						BitcoinCash,
						#[codec(index = 10)]
						PolkadotBulletin,
					}
				}
				pub mod junctions {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Junctions {
						#[codec(index = 0)]
						Here,
						#[codec(index = 1)]
						X1([runtime_types::staging_xcm::v4::junction::Junction; 1usize]),
						#[codec(index = 2)]
						X2([runtime_types::staging_xcm::v4::junction::Junction; 2usize]),
						#[codec(index = 3)]
						X3([runtime_types::staging_xcm::v4::junction::Junction; 3usize]),
						#[codec(index = 4)]
						X4([runtime_types::staging_xcm::v4::junction::Junction; 4usize]),
						#[codec(index = 5)]
						X5([runtime_types::staging_xcm::v4::junction::Junction; 5usize]),
						#[codec(index = 6)]
						X6([runtime_types::staging_xcm::v4::junction::Junction; 6usize]),
						#[codec(index = 7)]
						X7([runtime_types::staging_xcm::v4::junction::Junction; 7usize]),
						#[codec(index = 8)]
						X8([runtime_types::staging_xcm::v4::junction::Junction; 8usize]),
					}
				}
				pub mod location {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct Location {
						pub parents: ::core::primitive::u8,
						pub interior: runtime_types::staging_xcm::v4::junctions::Junctions,
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Instruction {
					#[codec(index = 0)]
					WithdrawAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 1)]
					ReserveAssetDeposited(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 2)]
					ReceiveTeleportedAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 3)]
					QueryResponse {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						response: runtime_types::staging_xcm::v4::Response,
						max_weight: runtime_types::sp_weights::weight_v2::Weight,
						querier: ::core::option::Option<
							runtime_types::staging_xcm::v4::location::Location,
						>,
					},
					#[codec(index = 4)]
					TransferAsset {
						assets: runtime_types::staging_xcm::v4::asset::Assets,
						beneficiary: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 5)]
					TransferReserveAsset {
						assets: runtime_types::staging_xcm::v4::asset::Assets,
						dest: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 6)]
					Transact {
						origin_kind: runtime_types::xcm::v3::OriginKind,
						require_weight_at_most: runtime_types::sp_weights::weight_v2::Weight,
						call: runtime_types::xcm::double_encoded::DoubleEncoded,
					},
					#[codec(index = 7)]
					HrmpNewChannelOpenRequest {
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						max_message_size: ::core::primitive::u32,
						#[codec(compact)]
						max_capacity: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					HrmpChannelAccepted {
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 9)]
					HrmpChannelClosing {
						#[codec(compact)]
						initiator: ::core::primitive::u32,
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 10)]
					ClearOrigin,
					#[codec(index = 11)]
					DescendOrigin(runtime_types::staging_xcm::v4::junctions::Junctions),
					#[codec(index = 12)]
					ReportError(runtime_types::staging_xcm::v4::QueryResponseInfo),
					#[codec(index = 13)]
					DepositAsset {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						beneficiary: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 14)]
					DepositReserveAsset {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						dest: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 15)]
					ExchangeAsset {
						give: runtime_types::staging_xcm::v4::asset::AssetFilter,
						want: runtime_types::staging_xcm::v4::asset::Assets,
						maximal: ::core::primitive::bool,
					},
					#[codec(index = 16)]
					InitiateReserveWithdraw {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						reserve: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 17)]
					InitiateTeleport {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						dest: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 18)]
					ReportHolding {
						response_info: runtime_types::staging_xcm::v4::QueryResponseInfo,
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
					},
					#[codec(index = 19)]
					BuyExecution {
						fees: runtime_types::staging_xcm::v4::asset::Asset,
						weight_limit: runtime_types::xcm::v3::WeightLimit,
					},
					#[codec(index = 20)]
					RefundSurplus,
					#[codec(index = 21)]
					SetErrorHandler(runtime_types::staging_xcm::v4::Xcm),
					#[codec(index = 22)]
					SetAppendix(runtime_types::staging_xcm::v4::Xcm),
					#[codec(index = 23)]
					ClearError,
					#[codec(index = 24)]
					ClaimAsset {
						assets: runtime_types::staging_xcm::v4::asset::Assets,
						ticket: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 25)]
					Trap(#[codec(compact)] ::core::primitive::u64),
					#[codec(index = 26)]
					SubscribeVersion {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						max_response_weight: runtime_types::sp_weights::weight_v2::Weight,
					},
					#[codec(index = 27)]
					UnsubscribeVersion,
					#[codec(index = 28)]
					BurnAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 29)]
					ExpectAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 30)]
					ExpectOrigin(
						::core::option::Option<runtime_types::staging_xcm::v4::location::Location>,
					),
					#[codec(index = 31)]
					ExpectError(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 32)]
					ExpectTransactStatus(runtime_types::xcm::v3::MaybeErrorCode),
					#[codec(index = 33)]
					QueryPallet {
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						response_info: runtime_types::staging_xcm::v4::QueryResponseInfo,
					},
					#[codec(index = 34)]
					ExpectPallet {
						#[codec(compact)]
						index: ::core::primitive::u32,
						name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						#[codec(compact)]
						crate_major: ::core::primitive::u32,
						#[codec(compact)]
						min_crate_minor: ::core::primitive::u32,
					},
					#[codec(index = 35)]
					ReportTransactStatus(runtime_types::staging_xcm::v4::QueryResponseInfo),
					#[codec(index = 36)]
					ClearTransactStatus,
					#[codec(index = 37)]
					UniversalOrigin(runtime_types::staging_xcm::v4::junction::Junction),
					#[codec(index = 38)]
					ExportMessage {
						network: runtime_types::staging_xcm::v4::junction::NetworkId,
						destination: runtime_types::staging_xcm::v4::junctions::Junctions,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 39)]
					LockAsset {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						unlocker: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 40)]
					UnlockAsset {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						target: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 41)]
					NoteUnlockable {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						owner: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 42)]
					RequestUnlock {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						locker: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 43)]
					SetFeesMode { jit_withdraw: ::core::primitive::bool },
					#[codec(index = 44)]
					SetTopic([::core::primitive::u8; 32usize]),
					#[codec(index = 45)]
					ClearTopic,
					#[codec(index = 46)]
					AliasOrigin(runtime_types::staging_xcm::v4::location::Location),
					#[codec(index = 47)]
					UnpaidExecution {
						weight_limit: runtime_types::xcm::v3::WeightLimit,
						check_origin: ::core::option::Option<
							runtime_types::staging_xcm::v4::location::Location,
						>,
					},
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct PalletInfo {
					#[codec(compact)]
					pub index: ::core::primitive::u32,
					pub name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					pub module_name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					#[codec(compact)]
					pub major: ::core::primitive::u32,
					#[codec(compact)]
					pub minor: ::core::primitive::u32,
					#[codec(compact)]
					pub patch: ::core::primitive::u32,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryResponseInfo {
					pub destination: runtime_types::staging_xcm::v4::location::Location,
					#[codec(compact)]
					pub query_id: ::core::primitive::u64,
					pub max_weight: runtime_types::sp_weights::weight_v2::Weight,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Response {
					#[codec(index = 0)]
					Null,
					#[codec(index = 1)]
					Assets(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 2)]
					ExecutionResult(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 3)]
					Version(::core::primitive::u32),
					#[codec(index = 4)]
					PalletsInfo(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::staging_xcm::v4::PalletInfo,
						>,
					),
					#[codec(index = 5)]
					DispatchResult(runtime_types::xcm::v3::MaybeErrorCode),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Xcm(
					pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::staging_xcm::v4::Instruction,
					>,
				);
			}
		}
		pub mod staging_xcm_executor {
			use super::runtime_types;
			pub mod traits {
				use super::runtime_types;
				pub mod asset_transfer {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum TransferType {
						#[codec(index = 0)]
						Teleport,
						#[codec(index = 1)]
						LocalReserve,
						#[codec(index = 2)]
						DestinationReserve,
						#[codec(index = 3)]
						RemoteReserve(runtime_types::xcm::VersionedLocation),
					}
				}
			}
		}
		pub mod xcm {
			use super::runtime_types;
			pub mod double_encoded {
				use super::runtime_types;
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct DoubleEncoded {
					pub encoded: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				}
			}
			pub mod v2 {
				use super::runtime_types;
				pub mod junction {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Junction {
						#[codec(index = 0)]
						Parachain(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 1)]
						AccountId32 {
							network: runtime_types::xcm::v2::NetworkId,
							id: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						AccountIndex64 {
							network: runtime_types::xcm::v2::NetworkId,
							#[codec(compact)]
							index: ::core::primitive::u64,
						},
						#[codec(index = 3)]
						AccountKey20 {
							network: runtime_types::xcm::v2::NetworkId,
							key: [::core::primitive::u8; 20usize],
						},
						#[codec(index = 4)]
						PalletInstance(::core::primitive::u8),
						#[codec(index = 5)]
						GeneralIndex(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 6)]
						GeneralKey(
							runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
								::core::primitive::u8,
							>,
						),
						#[codec(index = 7)]
						OnlyChild,
						#[codec(index = 8)]
						Plurality {
							id: runtime_types::xcm::v2::BodyId,
							part: runtime_types::xcm::v2::BodyPart,
						},
					}
				}
				pub mod multiasset {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AssetId {
						#[codec(index = 0)]
						Concrete(runtime_types::xcm::v2::multilocation::MultiLocation),
						#[codec(index = 1)]
						Abstract(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AssetInstance {
						#[codec(index = 0)]
						Undefined,
						#[codec(index = 1)]
						Index(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 2)]
						Array4([::core::primitive::u8; 4usize]),
						#[codec(index = 3)]
						Array8([::core::primitive::u8; 8usize]),
						#[codec(index = 4)]
						Array16([::core::primitive::u8; 16usize]),
						#[codec(index = 5)]
						Array32([::core::primitive::u8; 32usize]),
						#[codec(index = 6)]
						Blob(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Fungibility {
						#[codec(index = 0)]
						Fungible(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 1)]
						NonFungible(runtime_types::xcm::v2::multiasset::AssetInstance),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct MultiAsset {
						pub id: runtime_types::xcm::v2::multiasset::AssetId,
						pub fun: runtime_types::xcm::v2::multiasset::Fungibility,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum MultiAssetFilter {
						#[codec(index = 0)]
						Definite(runtime_types::xcm::v2::multiasset::MultiAssets),
						#[codec(index = 1)]
						Wild(runtime_types::xcm::v2::multiasset::WildMultiAsset),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct MultiAssets(
						pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::xcm::v2::multiasset::MultiAsset,
						>,
					);
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WildFungibility {
						#[codec(index = 0)]
						Fungible,
						#[codec(index = 1)]
						NonFungible,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WildMultiAsset {
						#[codec(index = 0)]
						All,
						#[codec(index = 1)]
						AllOf {
							id: runtime_types::xcm::v2::multiasset::AssetId,
							fun: runtime_types::xcm::v2::multiasset::WildFungibility,
						},
					}
				}
				pub mod multilocation {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Junctions {
						#[codec(index = 0)]
						Here,
						#[codec(index = 1)]
						X1(runtime_types::xcm::v2::junction::Junction),
						#[codec(index = 2)]
						X2(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 3)]
						X3(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 4)]
						X4(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 5)]
						X5(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 6)]
						X6(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 7)]
						X7(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 8)]
						X8(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct MultiLocation {
						pub parents: ::core::primitive::u8,
						pub interior: runtime_types::xcm::v2::multilocation::Junctions,
					}
				}
				pub mod traits {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Error {
						#[codec(index = 0)]
						Overflow,
						#[codec(index = 1)]
						Unimplemented,
						#[codec(index = 2)]
						UntrustedReserveLocation,
						#[codec(index = 3)]
						UntrustedTeleportLocation,
						#[codec(index = 4)]
						MultiLocationFull,
						#[codec(index = 5)]
						MultiLocationNotInvertible,
						#[codec(index = 6)]
						BadOrigin,
						#[codec(index = 7)]
						InvalidLocation,
						#[codec(index = 8)]
						AssetNotFound,
						#[codec(index = 9)]
						FailedToTransactAsset,
						#[codec(index = 10)]
						NotWithdrawable,
						#[codec(index = 11)]
						LocationCannotHold,
						#[codec(index = 12)]
						ExceedsMaxMessageSize,
						#[codec(index = 13)]
						DestinationUnsupported,
						#[codec(index = 14)]
						Transport,
						#[codec(index = 15)]
						Unroutable,
						#[codec(index = 16)]
						UnknownClaim,
						#[codec(index = 17)]
						FailedToDecode,
						#[codec(index = 18)]
						MaxWeightInvalid,
						#[codec(index = 19)]
						NotHoldingFees,
						#[codec(index = 20)]
						TooExpensive,
						#[codec(index = 21)]
						Trap(::core::primitive::u64),
						#[codec(index = 22)]
						UnhandledXcmVersion,
						#[codec(index = 23)]
						WeightLimitReached(::core::primitive::u64),
						#[codec(index = 24)]
						Barrier,
						#[codec(index = 25)]
						WeightNotComputable,
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum BodyId {
					#[codec(index = 0)]
					Unit,
					#[codec(index = 1)]
					Named(
						runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 2)]
					Index(#[codec(compact)] ::core::primitive::u32),
					#[codec(index = 3)]
					Executive,
					#[codec(index = 4)]
					Technical,
					#[codec(index = 5)]
					Legislative,
					#[codec(index = 6)]
					Judicial,
					#[codec(index = 7)]
					Defense,
					#[codec(index = 8)]
					Administration,
					#[codec(index = 9)]
					Treasury,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum BodyPart {
					#[codec(index = 0)]
					Voice,
					#[codec(index = 1)]
					Members {
						#[codec(compact)]
						count: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					Fraction {
						#[codec(compact)]
						nom: ::core::primitive::u32,
						#[codec(compact)]
						denom: ::core::primitive::u32,
					},
					#[codec(index = 3)]
					AtLeastProportion {
						#[codec(compact)]
						nom: ::core::primitive::u32,
						#[codec(compact)]
						denom: ::core::primitive::u32,
					},
					#[codec(index = 4)]
					MoreThanProportion {
						#[codec(compact)]
						nom: ::core::primitive::u32,
						#[codec(compact)]
						denom: ::core::primitive::u32,
					},
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Instruction {
					#[codec(index = 0)]
					WithdrawAsset(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 1)]
					ReserveAssetDeposited(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 2)]
					ReceiveTeleportedAsset(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 3)]
					QueryResponse {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						response: runtime_types::xcm::v2::Response,
						#[codec(compact)]
						max_weight: ::core::primitive::u64,
					},
					#[codec(index = 4)]
					TransferAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssets,
						beneficiary: runtime_types::xcm::v2::multilocation::MultiLocation,
					},
					#[codec(index = 5)]
					TransferReserveAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssets,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 6)]
					Transact {
						origin_type: runtime_types::xcm::v2::OriginKind,
						#[codec(compact)]
						require_weight_at_most: ::core::primitive::u64,
						call: runtime_types::xcm::double_encoded::DoubleEncoded,
					},
					#[codec(index = 7)]
					HrmpNewChannelOpenRequest {
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						max_message_size: ::core::primitive::u32,
						#[codec(compact)]
						max_capacity: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					HrmpChannelAccepted {
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 9)]
					HrmpChannelClosing {
						#[codec(compact)]
						initiator: ::core::primitive::u32,
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 10)]
					ClearOrigin,
					#[codec(index = 11)]
					DescendOrigin(runtime_types::xcm::v2::multilocation::Junctions),
					#[codec(index = 12)]
					ReportError {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						#[codec(compact)]
						max_response_weight: ::core::primitive::u64,
					},
					#[codec(index = 13)]
					DepositAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						#[codec(compact)]
						max_assets: ::core::primitive::u32,
						beneficiary: runtime_types::xcm::v2::multilocation::MultiLocation,
					},
					#[codec(index = 14)]
					DepositReserveAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						#[codec(compact)]
						max_assets: ::core::primitive::u32,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 15)]
					ExchangeAsset {
						give: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						receive: runtime_types::xcm::v2::multiasset::MultiAssets,
					},
					#[codec(index = 16)]
					InitiateReserveWithdraw {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						reserve: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 17)]
					InitiateTeleport {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 18)]
					QueryHolding {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						#[codec(compact)]
						max_response_weight: ::core::primitive::u64,
					},
					#[codec(index = 19)]
					BuyExecution {
						fees: runtime_types::xcm::v2::multiasset::MultiAsset,
						weight_limit: runtime_types::xcm::v2::WeightLimit,
					},
					#[codec(index = 20)]
					RefundSurplus,
					#[codec(index = 21)]
					SetErrorHandler(runtime_types::xcm::v2::Xcm),
					#[codec(index = 22)]
					SetAppendix(runtime_types::xcm::v2::Xcm),
					#[codec(index = 23)]
					ClearError,
					#[codec(index = 24)]
					ClaimAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssets,
						ticket: runtime_types::xcm::v2::multilocation::MultiLocation,
					},
					#[codec(index = 25)]
					Trap(#[codec(compact)] ::core::primitive::u64),
					#[codec(index = 26)]
					SubscribeVersion {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						#[codec(compact)]
						max_response_weight: ::core::primitive::u64,
					},
					#[codec(index = 27)]
					UnsubscribeVersion,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum NetworkId {
					#[codec(index = 0)]
					Any,
					#[codec(index = 1)]
					Named(
						runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 2)]
					Polkadot,
					#[codec(index = 3)]
					Kusama,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum OriginKind {
					#[codec(index = 0)]
					Native,
					#[codec(index = 1)]
					SovereignAccount,
					#[codec(index = 2)]
					Superuser,
					#[codec(index = 3)]
					Xcm,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Response {
					#[codec(index = 0)]
					Null,
					#[codec(index = 1)]
					Assets(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 2)]
					ExecutionResult(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v2::traits::Error,
						)>,
					),
					#[codec(index = 3)]
					Version(::core::primitive::u32),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum WeightLimit {
					#[codec(index = 0)]
					Unlimited,
					#[codec(index = 1)]
					Limited(#[codec(compact)] ::core::primitive::u64),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Xcm(
					pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::xcm::v2::Instruction,
					>,
				);
			}
			pub mod v3 {
				use super::runtime_types;
				pub mod junction {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum BodyId {
						#[codec(index = 0)]
						Unit,
						#[codec(index = 1)]
						Moniker([::core::primitive::u8; 4usize]),
						#[codec(index = 2)]
						Index(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 3)]
						Executive,
						#[codec(index = 4)]
						Technical,
						#[codec(index = 5)]
						Legislative,
						#[codec(index = 6)]
						Judicial,
						#[codec(index = 7)]
						Defense,
						#[codec(index = 8)]
						Administration,
						#[codec(index = 9)]
						Treasury,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum BodyPart {
						#[codec(index = 0)]
						Voice,
						#[codec(index = 1)]
						Members {
							#[codec(compact)]
							count: ::core::primitive::u32,
						},
						#[codec(index = 2)]
						Fraction {
							#[codec(compact)]
							nom: ::core::primitive::u32,
							#[codec(compact)]
							denom: ::core::primitive::u32,
						},
						#[codec(index = 3)]
						AtLeastProportion {
							#[codec(compact)]
							nom: ::core::primitive::u32,
							#[codec(compact)]
							denom: ::core::primitive::u32,
						},
						#[codec(index = 4)]
						MoreThanProportion {
							#[codec(compact)]
							nom: ::core::primitive::u32,
							#[codec(compact)]
							denom: ::core::primitive::u32,
						},
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Junction {
						#[codec(index = 0)]
						Parachain(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 1)]
						AccountId32 {
							network:
								::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
							id: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						AccountIndex64 {
							network:
								::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
							#[codec(compact)]
							index: ::core::primitive::u64,
						},
						#[codec(index = 3)]
						AccountKey20 {
							network:
								::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
							key: [::core::primitive::u8; 20usize],
						},
						#[codec(index = 4)]
						PalletInstance(::core::primitive::u8),
						#[codec(index = 5)]
						GeneralIndex(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 6)]
						GeneralKey {
							length: ::core::primitive::u8,
							data: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 7)]
						OnlyChild,
						#[codec(index = 8)]
						Plurality {
							id: runtime_types::xcm::v3::junction::BodyId,
							part: runtime_types::xcm::v3::junction::BodyPart,
						},
						#[codec(index = 9)]
						GlobalConsensus(runtime_types::xcm::v3::junction::NetworkId),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum NetworkId {
						#[codec(index = 0)]
						ByGenesis([::core::primitive::u8; 32usize]),
						#[codec(index = 1)]
						ByFork {
							block_number: ::core::primitive::u64,
							block_hash: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						Polkadot,
						#[codec(index = 3)]
						Kusama,
						#[codec(index = 4)]
						Westend,
						#[codec(index = 5)]
						Rococo,
						#[codec(index = 6)]
						Wococo,
						#[codec(index = 7)]
						Ethereum {
							#[codec(compact)]
							chain_id: ::core::primitive::u64,
						},
						#[codec(index = 8)]
						BitcoinCore,
						#[codec(index = 9)]
						BitcoinCash,
						#[codec(index = 10)]
						PolkadotBulletin,
					}
				}
				pub mod junctions {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Junctions {
						#[codec(index = 0)]
						Here,
						#[codec(index = 1)]
						X1(runtime_types::xcm::v3::junction::Junction),
						#[codec(index = 2)]
						X2(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 3)]
						X3(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 4)]
						X4(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 5)]
						X5(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 6)]
						X6(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 7)]
						X7(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 8)]
						X8(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
					}
				}
				pub mod multiasset {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AssetId {
						#[codec(index = 0)]
						Concrete(runtime_types::staging_xcm::v3::multilocation::MultiLocation),
						#[codec(index = 1)]
						Abstract([::core::primitive::u8; 32usize]),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum AssetInstance {
						#[codec(index = 0)]
						Undefined,
						#[codec(index = 1)]
						Index(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 2)]
						Array4([::core::primitive::u8; 4usize]),
						#[codec(index = 3)]
						Array8([::core::primitive::u8; 8usize]),
						#[codec(index = 4)]
						Array16([::core::primitive::u8; 16usize]),
						#[codec(index = 5)]
						Array32([::core::primitive::u8; 32usize]),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Fungibility {
						#[codec(index = 0)]
						Fungible(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 1)]
						NonFungible(runtime_types::xcm::v3::multiasset::AssetInstance),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct MultiAsset {
						pub id: runtime_types::xcm::v3::multiasset::AssetId,
						pub fun: runtime_types::xcm::v3::multiasset::Fungibility,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum MultiAssetFilter {
						#[codec(index = 0)]
						Definite(runtime_types::xcm::v3::multiasset::MultiAssets),
						#[codec(index = 1)]
						Wild(runtime_types::xcm::v3::multiasset::WildMultiAsset),
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub struct MultiAssets(
						pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::xcm::v3::multiasset::MultiAsset,
						>,
					);
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WildFungibility {
						#[codec(index = 0)]
						Fungible,
						#[codec(index = 1)]
						NonFungible,
					}
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum WildMultiAsset {
						#[codec(index = 0)]
						All,
						#[codec(index = 1)]
						AllOf {
							id: runtime_types::xcm::v3::multiasset::AssetId,
							fun: runtime_types::xcm::v3::multiasset::WildFungibility,
						},
						#[codec(index = 2)]
						AllCounted(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 3)]
						AllOfCounted {
							id: runtime_types::xcm::v3::multiasset::AssetId,
							fun: runtime_types::xcm::v3::multiasset::WildFungibility,
							#[codec(compact)]
							count: ::core::primitive::u32,
						},
					}
				}
				pub mod traits {
					use super::runtime_types;
					#[derive(
						:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
						:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
						:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
						:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
						Debug,
					)]
					# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
					#[codec(dumb_trait_bound)]
					#[decode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
					)]
					#[encode_as_type(
						crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
					)]
					pub enum Error {
						#[codec(index = 0)]
						Overflow,
						#[codec(index = 1)]
						Unimplemented,
						#[codec(index = 2)]
						UntrustedReserveLocation,
						#[codec(index = 3)]
						UntrustedTeleportLocation,
						#[codec(index = 4)]
						LocationFull,
						#[codec(index = 5)]
						LocationNotInvertible,
						#[codec(index = 6)]
						BadOrigin,
						#[codec(index = 7)]
						InvalidLocation,
						#[codec(index = 8)]
						AssetNotFound,
						#[codec(index = 9)]
						FailedToTransactAsset,
						#[codec(index = 10)]
						NotWithdrawable,
						#[codec(index = 11)]
						LocationCannotHold,
						#[codec(index = 12)]
						ExceedsMaxMessageSize,
						#[codec(index = 13)]
						DestinationUnsupported,
						#[codec(index = 14)]
						Transport,
						#[codec(index = 15)]
						Unroutable,
						#[codec(index = 16)]
						UnknownClaim,
						#[codec(index = 17)]
						FailedToDecode,
						#[codec(index = 18)]
						MaxWeightInvalid,
						#[codec(index = 19)]
						NotHoldingFees,
						#[codec(index = 20)]
						TooExpensive,
						#[codec(index = 21)]
						Trap(::core::primitive::u64),
						#[codec(index = 22)]
						ExpectationFalse,
						#[codec(index = 23)]
						PalletNotFound,
						#[codec(index = 24)]
						NameMismatch,
						#[codec(index = 25)]
						VersionIncompatible,
						#[codec(index = 26)]
						HoldingWouldOverflow,
						#[codec(index = 27)]
						ExportError,
						#[codec(index = 28)]
						ReanchorFailed,
						#[codec(index = 29)]
						NoDeal,
						#[codec(index = 30)]
						FeesNotMet,
						#[codec(index = 31)]
						LockError,
						#[codec(index = 32)]
						NoPermission,
						#[codec(index = 33)]
						Unanchored,
						#[codec(index = 34)]
						NotDepositable,
						#[codec(index = 35)]
						UnhandledXcmVersion,
						#[codec(index = 36)]
						WeightLimitReached(runtime_types::sp_weights::weight_v2::Weight),
						#[codec(index = 37)]
						Barrier,
						#[codec(index = 38)]
						WeightNotComputable,
						#[codec(index = 39)]
						ExceedsStackLimit,
					}
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Instruction {
					#[codec(index = 0)]
					WithdrawAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 1)]
					ReserveAssetDeposited(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 2)]
					ReceiveTeleportedAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 3)]
					QueryResponse {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						response: runtime_types::xcm::v3::Response,
						max_weight: runtime_types::sp_weights::weight_v2::Weight,
						querier: ::core::option::Option<
							runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						>,
					},
					#[codec(index = 4)]
					TransferAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssets,
						beneficiary: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 5)]
					TransferReserveAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssets,
						dest: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 6)]
					Transact {
						origin_kind: runtime_types::xcm::v3::OriginKind,
						require_weight_at_most: runtime_types::sp_weights::weight_v2::Weight,
						call: runtime_types::xcm::double_encoded::DoubleEncoded,
					},
					#[codec(index = 7)]
					HrmpNewChannelOpenRequest {
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						max_message_size: ::core::primitive::u32,
						#[codec(compact)]
						max_capacity: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					HrmpChannelAccepted {
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 9)]
					HrmpChannelClosing {
						#[codec(compact)]
						initiator: ::core::primitive::u32,
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 10)]
					ClearOrigin,
					#[codec(index = 11)]
					DescendOrigin(runtime_types::xcm::v3::junctions::Junctions),
					#[codec(index = 12)]
					ReportError(runtime_types::xcm::v3::QueryResponseInfo),
					#[codec(index = 13)]
					DepositAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						beneficiary: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 14)]
					DepositReserveAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						dest: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 15)]
					ExchangeAsset {
						give: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						want: runtime_types::xcm::v3::multiasset::MultiAssets,
						maximal: ::core::primitive::bool,
					},
					#[codec(index = 16)]
					InitiateReserveWithdraw {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						reserve: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 17)]
					InitiateTeleport {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						dest: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 18)]
					ReportHolding {
						response_info: runtime_types::xcm::v3::QueryResponseInfo,
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
					},
					#[codec(index = 19)]
					BuyExecution {
						fees: runtime_types::xcm::v3::multiasset::MultiAsset,
						weight_limit: runtime_types::xcm::v3::WeightLimit,
					},
					#[codec(index = 20)]
					RefundSurplus,
					#[codec(index = 21)]
					SetErrorHandler(runtime_types::xcm::v3::Xcm),
					#[codec(index = 22)]
					SetAppendix(runtime_types::xcm::v3::Xcm),
					#[codec(index = 23)]
					ClearError,
					#[codec(index = 24)]
					ClaimAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssets,
						ticket: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 25)]
					Trap(#[codec(compact)] ::core::primitive::u64),
					#[codec(index = 26)]
					SubscribeVersion {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						max_response_weight: runtime_types::sp_weights::weight_v2::Weight,
					},
					#[codec(index = 27)]
					UnsubscribeVersion,
					#[codec(index = 28)]
					BurnAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 29)]
					ExpectAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 30)]
					ExpectOrigin(
						::core::option::Option<
							runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						>,
					),
					#[codec(index = 31)]
					ExpectError(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 32)]
					ExpectTransactStatus(runtime_types::xcm::v3::MaybeErrorCode),
					#[codec(index = 33)]
					QueryPallet {
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						response_info: runtime_types::xcm::v3::QueryResponseInfo,
					},
					#[codec(index = 34)]
					ExpectPallet {
						#[codec(compact)]
						index: ::core::primitive::u32,
						name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						#[codec(compact)]
						crate_major: ::core::primitive::u32,
						#[codec(compact)]
						min_crate_minor: ::core::primitive::u32,
					},
					#[codec(index = 35)]
					ReportTransactStatus(runtime_types::xcm::v3::QueryResponseInfo),
					#[codec(index = 36)]
					ClearTransactStatus,
					#[codec(index = 37)]
					UniversalOrigin(runtime_types::xcm::v3::junction::Junction),
					#[codec(index = 38)]
					ExportMessage {
						network: runtime_types::xcm::v3::junction::NetworkId,
						destination: runtime_types::xcm::v3::junctions::Junctions,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 39)]
					LockAsset {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						unlocker: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 40)]
					UnlockAsset {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						target: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 41)]
					NoteUnlockable {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						owner: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 42)]
					RequestUnlock {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						locker: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 43)]
					SetFeesMode { jit_withdraw: ::core::primitive::bool },
					#[codec(index = 44)]
					SetTopic([::core::primitive::u8; 32usize]),
					#[codec(index = 45)]
					ClearTopic,
					#[codec(index = 46)]
					AliasOrigin(runtime_types::staging_xcm::v3::multilocation::MultiLocation),
					#[codec(index = 47)]
					UnpaidExecution {
						weight_limit: runtime_types::xcm::v3::WeightLimit,
						check_origin: ::core::option::Option<
							runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						>,
					},
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum MaybeErrorCode {
					#[codec(index = 0)]
					Success,
					#[codec(index = 1)]
					Error(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 2)]
					TruncatedError(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum OriginKind {
					#[codec(index = 0)]
					Native,
					#[codec(index = 1)]
					SovereignAccount,
					#[codec(index = 2)]
					Superuser,
					#[codec(index = 3)]
					Xcm,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct PalletInfo {
					#[codec(compact)]
					pub index: ::core::primitive::u32,
					pub name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					pub module_name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					#[codec(compact)]
					pub major: ::core::primitive::u32,
					#[codec(compact)]
					pub minor: ::core::primitive::u32,
					#[codec(compact)]
					pub patch: ::core::primitive::u32,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct QueryResponseInfo {
					pub destination: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					#[codec(compact)]
					pub query_id: ::core::primitive::u64,
					pub max_weight: runtime_types::sp_weights::weight_v2::Weight,
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum Response {
					#[codec(index = 0)]
					Null,
					#[codec(index = 1)]
					Assets(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 2)]
					ExecutionResult(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 3)]
					Version(::core::primitive::u32),
					#[codec(index = 4)]
					PalletsInfo(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::xcm::v3::PalletInfo,
						>,
					),
					#[codec(index = 5)]
					DispatchResult(runtime_types::xcm::v3::MaybeErrorCode),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub enum WeightLimit {
					#[codec(index = 0)]
					Unlimited,
					#[codec(index = 1)]
					Limited(runtime_types::sp_weights::weight_v2::Weight),
				}
				#[derive(
					:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
					:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
					:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
					:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
					Debug,
				)]
				# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
				#[codec(dumb_trait_bound)]
				#[decode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode"
				)]
				#[encode_as_type(
					crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode"
				)]
				pub struct Xcm(
					pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::xcm::v3::Instruction,
					>,
				);
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum VersionedAssetId {
				#[codec(index = 3)]
				V3(runtime_types::xcm::v3::multiasset::AssetId),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::asset::AssetId),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum VersionedAssets {
				#[codec(index = 1)]
				V2(runtime_types::xcm::v2::multiasset::MultiAssets),
				#[codec(index = 3)]
				V3(runtime_types::xcm::v3::multiasset::MultiAssets),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::asset::Assets),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum VersionedLocation {
				#[codec(index = 1)]
				V2(runtime_types::xcm::v2::multilocation::MultiLocation),
				#[codec(index = 3)]
				V3(runtime_types::staging_xcm::v3::multilocation::MultiLocation),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::location::Location),
			}
			#[derive(
				:: subxt :: ext :: subxt_core :: ext :: codec :: Decode,
				:: subxt :: ext :: subxt_core :: ext :: codec :: Encode,
				:: subxt :: ext :: subxt_core :: ext :: scale_decode :: DecodeAsType,
				:: subxt :: ext :: subxt_core :: ext :: scale_encode :: EncodeAsType,
				Debug,
			)]
			# [codec (crate = :: subxt :: ext :: subxt_core :: ext :: codec)]
			#[codec(dumb_trait_bound)]
			#[decode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_decode")]
			#[encode_as_type(crate_path = ":: subxt :: ext :: subxt_core :: ext :: scale_encode")]
			pub enum VersionedXcm {
				#[codec(index = 2)]
				V2(runtime_types::xcm::v2::Xcm),
				#[codec(index = 3)]
				V3(runtime_types::xcm::v3::Xcm),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::Xcm),
			}
		}
	}
}
