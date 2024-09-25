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

//! Defines all concrete types and global components of the enclave.
//!
//! This allows the crates themselves to stay as generic as possible
//! and ensures that the global instances are initialized once.
use crate::{
	initialization::parentchain::{
		integritee_parachain::IntegriteeParachainHandler,
		integritee_solochain::IntegriteeSolochainHandler,
		target_a_parachain::TargetAParachainHandler, target_a_solochain::TargetASolochainHandler,
		target_b_parachain::TargetBParachainHandler, target_b_solochain::TargetBSolochainHandler,
	},
	ocall::OcallApi,
	rpc::rpc_response_channel::RpcResponseChannel,
	tls_ra::seal_handler::SealHandler,
};
use bc_enclave_registry::EnclaveRegistry;
use bc_relayer_registry::RelayerRegistry;
use bc_signer_registry::SignerRegistry;
use ita_parentchain_interface::{integritee, target_a, target_b};
use ita_sgx_runtime::Runtime;
use ita_stf::{Getter, State as StfState, Stf, TrustedCallSigned};
use itc_direct_rpc_server::{
	rpc_connection_registry::ConnectionRegistry, rpc_responder::RpcResponder,
	rpc_watch_extractor::RpcWatchExtractor, rpc_ws_handler::RpcWsHandler,
};
use itc_parentchain::{
	block_import_dispatcher::{
		immediate_dispatcher::ImmediateDispatcher, triggered_dispatcher::TriggeredDispatcher,
		BlockImportDispatcher,
	},
	block_importer::ParentchainBlockImporter,
	indirect_calls_executor::{filter_metadata::EventCreator, IndirectCallsExecutor},
	light_client::{
		concurrent_access::ValidatorAccessor, io::LightClientStateSealSync,
		light_validation::LightValidation, light_validation_state::LightValidationState,
	},
};
use itc_tls_websocket_server::{
	config_provider::FromFileConfigProvider, ws_server::TungsteniteWsServer, ConnectionToken,
};
use itp_attestation_handler::IntelAttestationHandler;
use itp_component_container::ComponentContainer;
use itp_extrinsics_factory::ExtrinsicsFactory;
use itp_import_queue::ImportQueue;
use itp_node_api::{
	api_client::PairSignature,
	metadata::{provider::NodeMetadataRepository, NodeMetadata},
};
use itp_nonce_cache::NonceCache;
use itp_sgx_crypto::{
	ecdsa::{Pair as EcdsaPair, Seal as EcdsaSeal},
	key_repository::KeyRepository,
	schnorr::{Pair as SchnorrPair, Seal as SchnorrSeal},
	Aes, AesSeal, Ed25519Seal, Rsa3072Seal,
};
use itp_stf_executor::{
	enclave_signer::StfEnclaveSigner, executor::StfExecutor, getter_executor::GetterExecutor,
	state_getter::StfStateGetter,
};
use itp_stf_primitives::types::{Hash, TrustedOperation};
use itp_stf_state_handler::{
	file_io::sgx::SgxStateFileIo, state_initializer::StateInitializer,
	state_snapshot_repository::StateSnapshotRepository, StateHandler,
};
use itp_stf_state_observer::state_observer::StateObserver;
use itp_top_pool::basic_pool::BasicPool;
use itp_top_pool_author::{
	api::SidechainApi,
	author::{Author, AuthorTopFilter},
};
use itp_types::{Block as ParentchainBlock, SignedBlock as SignedParentchainBlock};
use lazy_static::lazy_static;
use sgx_crypto_helper::rsa3072::Rsa3072KeyPair;
use sgx_tstd::vec::Vec;
use sp_core::{ed25519, ed25519::Pair};
use std::sync::Arc;

pub type EnclaveParentchainSigner =
	itp_node_api::api_client::StaticExtrinsicSigner<Pair, PairSignature>;

pub type EnclaveGetter = Getter;
pub type EnclaveTrustedCallSigned = TrustedCallSigned;
pub type EnclaveStf = Stf<EnclaveTrustedCallSigned, EnclaveGetter, StfState, Runtime>;
pub type EnclaveStateKeyRepository = KeyRepository<Aes, AesSeal>;
pub type EnclaveShieldingKeyRepository = KeyRepository<Rsa3072KeyPair, Rsa3072Seal>;
pub type EnclaveSigningKeyRepository = KeyRepository<ed25519::Pair, Ed25519Seal>;
pub type EnclaveBitcoinKeyRepository = KeyRepository<SchnorrPair, SchnorrSeal>;
pub type EnclaveEthereumKeyRepository = KeyRepository<EcdsaPair, EcdsaSeal>;
pub type EnclaveTonKeyRepository = KeyRepository<ed25519::Pair, Ed25519Seal>;
pub type EnclaveStateFileIo = SgxStateFileIo<EnclaveStateKeyRepository, StfState>;
pub type EnclaveStateSnapshotRepository = StateSnapshotRepository<EnclaveStateFileIo>;
pub type EnclaveStateObserver = StateObserver<StfState>;
pub type EnclaveStateInitializer =
	StateInitializer<StfState, EnclaveStf, EnclaveShieldingKeyRepository>;
pub type EnclaveStateHandler =
	StateHandler<EnclaveStateSnapshotRepository, EnclaveStateObserver, EnclaveStateInitializer>;
pub type EnclaveGetterExecutor =
	GetterExecutor<EnclaveStateObserver, StfStateGetter<EnclaveStf>, Getter>;
pub type EnclaveOCallApi = OcallApi;
pub type EnclaveNodeMetadataRepository = NodeMetadataRepository<NodeMetadata>;
pub type EnclaveStfExecutor = StfExecutor<
	EnclaveOCallApi,
	EnclaveStateHandler,
	EnclaveNodeMetadataRepository,
	EnclaveStf,
	EnclaveTrustedCallSigned,
	EnclaveGetter,
>;
pub type EnclaveStfEnclaveSigner = StfEnclaveSigner<
	EnclaveOCallApi,
	EnclaveStateObserver,
	EnclaveShieldingKeyRepository,
	EnclaveStf,
	EnclaveTopPoolAuthor,
	EnclaveTrustedCallSigned,
	EnclaveGetter,
>;
pub type EnclaveAttestationHandler =
	IntelAttestationHandler<EnclaveOCallApi, EnclaveSigningKeyRepository>;

pub type EnclaveRpcConnectionRegistry = ConnectionRegistry<Hash, ConnectionToken>;
pub type EnclaveRpcWsHandler =
	RpcWsHandler<RpcWatchExtractor<Hash>, EnclaveRpcConnectionRegistry, Hash>;
pub type EnclaveWebSocketServer = TungsteniteWsServer<EnclaveRpcWsHandler, FromFileConfigProvider>;
pub type EnclaveRpcResponder = RpcResponder<EnclaveRpcConnectionRegistry, Hash, RpcResponseChannel>;
pub type EnclaveSidechainApi = SidechainApi<ParentchainBlock, EnclaveTrustedCallSigned>;

// Parentchain types relevant for all parentchains
pub type EnclaveLightClientSeal =
	LightClientStateSealSync<ParentchainBlock, LightValidationState<ParentchainBlock>>;
pub type EnclaveExtrinsicsFactory =
	ExtrinsicsFactory<EnclaveParentchainSigner, NonceCache, EnclaveNodeMetadataRepository>;

pub type EnclaveValidatorAccessor = ValidatorAccessor<
	LightValidation<ParentchainBlock, EnclaveOCallApi>,
	ParentchainBlock,
	EnclaveLightClientSeal,
>;

pub type IntegriteeParentchainBlockImportQueue = ImportQueue<SignedParentchainBlock>;
pub type TargetAParentchainBlockImportQueue = ImportQueue<SignedParentchainBlock>;
pub type TargetBParentchainBlockImportQueue = ImportQueue<SignedParentchainBlock>;

/// Import queue for the events
///
/// Note: `Vec<u8>` is correct. It should not be `Vec<Vec<u8>`
pub type IntegriteeParentchainEventImportQueue = ImportQueue<Vec<u8>>;
pub type TargetAParentchainEventImportQueue = ImportQueue<Vec<u8>>;
pub type TargetBParentchainEventImportQueue = ImportQueue<Vec<u8>>;

// Stuff for the integritee parentchain

pub type IntegriteeParentchainIndirectCallsExecutor = IndirectCallsExecutor<
	EnclaveShieldingKeyRepository,
	EnclaveStfEnclaveSigner,
	EnclaveTopPoolAuthor,
	EnclaveNodeMetadataRepository,
	EventCreator<integritee::FilterableEvents>,
	integritee::ParentchainEventHandler,
	EnclaveTrustedCallSigned,
	EnclaveGetter,
	RelayerRegistry,
	SignerRegistry,
	EnclaveRegistry,
>;

pub type IntegriteeParentchainBlockImporter = ParentchainBlockImporter<
	ParentchainBlock,
	EnclaveValidatorAccessor,
	EnclaveStfExecutor,
	EnclaveExtrinsicsFactory,
	IntegriteeParentchainIndirectCallsExecutor,
	EnclaveOCallApi,
>;

pub type IntegriteeParentchainTriggeredBlockImportDispatcher = TriggeredDispatcher<
	IntegriteeParentchainBlockImporter,
	IntegriteeParentchainBlockImportQueue,
	IntegriteeParentchainEventImportQueue,
>;

pub type IntegriteeParentchainImmediateBlockImportDispatcher =
	ImmediateDispatcher<IntegriteeParentchainBlockImporter>;

pub type IntegriteeParentchainBlockImportDispatcher = BlockImportDispatcher<
	IntegriteeParentchainTriggeredBlockImportDispatcher,
	IntegriteeParentchainImmediateBlockImportDispatcher,
>;

// Stuff for the Target A parentchain

/// IndirectCalls executor instance of the Target A parentchain.
///
/// **Note**: The filter here is purely used for demo purposes.
///
/// Also note that the extrinsic parser must be changed if the signed extra contains the
/// `AssetTxPayment`.
pub type TargetAParentchainIndirectCallsExecutor = IndirectCallsExecutor<
	EnclaveShieldingKeyRepository,
	EnclaveStfEnclaveSigner,
	EnclaveTopPoolAuthor,
	EnclaveNodeMetadataRepository,
	EventCreator<target_a::FilterableEvents>,
	target_a::ParentchainEventHandler,
	EnclaveTrustedCallSigned,
	EnclaveGetter,
	RelayerRegistry,
	SignerRegistry,
	EnclaveRegistry,
>;

pub type TargetAParentchainBlockImporter = ParentchainBlockImporter<
	ParentchainBlock,
	EnclaveValidatorAccessor,
	EnclaveStfExecutor,
	EnclaveExtrinsicsFactory,
	TargetAParentchainIndirectCallsExecutor,
	EnclaveOCallApi,
>;

pub type TargetAParentchainTriggeredBlockImportDispatcher = TriggeredDispatcher<
	TargetAParentchainBlockImporter,
	TargetAParentchainBlockImportQueue,
	TargetAParentchainEventImportQueue,
>;

pub type TargetAParentchainImmediateBlockImportDispatcher =
	ImmediateDispatcher<TargetAParentchainBlockImporter>;

pub type TargetAParentchainBlockImportDispatcher = BlockImportDispatcher<
	TargetAParentchainTriggeredBlockImportDispatcher,
	TargetAParentchainImmediateBlockImportDispatcher,
>;

// Stuff for the Target B parentchain

/// IndirectCalls executor instance of the Target B parentchain.
///
/// **Note**: The filter here is purely used for demo purposes.
///
/// Also note that the extrinsic parser must be changed if the signed extra contains the
/// `AssetTxPayment`.
pub type TargetBParentchainIndirectCallsExecutor = IndirectCallsExecutor<
	EnclaveShieldingKeyRepository,
	EnclaveStfEnclaveSigner,
	EnclaveTopPoolAuthor,
	EnclaveNodeMetadataRepository,
	EventCreator<target_b::FilterableEvents>,
	target_b::ParentchainEventHandler,
	EnclaveTrustedCallSigned,
	EnclaveGetter,
	RelayerRegistry,
	SignerRegistry,
	EnclaveRegistry,
>;

pub type TargetBParentchainBlockImporter = ParentchainBlockImporter<
	ParentchainBlock,
	EnclaveValidatorAccessor,
	EnclaveStfExecutor,
	EnclaveExtrinsicsFactory,
	TargetBParentchainIndirectCallsExecutor,
	EnclaveOCallApi,
>;

pub type TargetBParentchainTriggeredBlockImportDispatcher = TriggeredDispatcher<
	TargetBParentchainBlockImporter,
	TargetBParentchainBlockImportQueue,
	TargetBParentchainEventImportQueue,
>;

pub type TargetBParentchainImmediateBlockImportDispatcher =
	ImmediateDispatcher<TargetBParentchainBlockImporter>;

pub type TargetBParentchainBlockImportDispatcher = BlockImportDispatcher<
	TargetBParentchainTriggeredBlockImportDispatcher,
	TargetBParentchainImmediateBlockImportDispatcher,
>;

/// Sidechain types
pub type EnclaveTopPool = BasicPool<
	EnclaveSidechainApi,
	ParentchainBlock,
	EnclaveRpcResponder,
	TrustedOperation<EnclaveTrustedCallSigned, EnclaveGetter>,
>;

pub type EnclaveTopPoolAuthor = Author<
	EnclaveTopPool,
	AuthorTopFilter<EnclaveTrustedCallSigned, EnclaveGetter>,
	EnclaveStateHandler,
	EnclaveShieldingKeyRepository,
	EnclaveTrustedCallSigned,
	EnclaveGetter,
>;
pub type EnclaveSealHandler = SealHandler<
	EnclaveShieldingKeyRepository,
	EnclaveStateKeyRepository,
	EnclaveStateHandler,
	EnclaveLightClientSeal,
	SignerRegistry,
	EnclaveRegistry,
>;
pub type EnclaveOffchainWorkerExecutor = itc_offchain_worker_executor::executor::Executor<
	ParentchainBlock,
	EnclaveTopPoolAuthor,
	EnclaveStfExecutor,
	EnclaveStateHandler,
	EnclaveValidatorAccessor,
	EnclaveExtrinsicsFactory,
	EnclaveStf,
	EnclaveTrustedCallSigned,
	EnclaveGetter,
>;

// Base component instances
//-------------------------------------------------------------------------------------------------

/// State key repository
pub static GLOBAL_STATE_KEY_REPOSITORY_COMPONENT: ComponentContainer<EnclaveStateKeyRepository> =
	ComponentContainer::new("State key repository");

/// Shielding key repository
pub static GLOBAL_SHIELDING_KEY_REPOSITORY_COMPONENT: ComponentContainer<
	EnclaveShieldingKeyRepository,
> = ComponentContainer::new("Shielding key repository");

/// Signing key repository
pub static GLOBAL_SIGNING_KEY_REPOSITORY_COMPONENT: ComponentContainer<
	EnclaveSigningKeyRepository,
> = ComponentContainer::new("Signing key repository");

/// Bitcoin key repository
pub static GLOBAL_BITCOIN_KEY_REPOSITORY_COMPONENT: ComponentContainer<
	EnclaveBitcoinKeyRepository,
> = ComponentContainer::new("Bitcoin key repository");

/// Ethereum key repository
pub static GLOBAL_ETHEREUM_KEY_REPOSITORY_COMPONENT: ComponentContainer<
	EnclaveEthereumKeyRepository,
> = ComponentContainer::new("Ethereum key repository");

/// Ton key repository
pub static GLOBAL_TON_KEY_REPOSITORY_COMPONENT: ComponentContainer<EnclaveTonKeyRepository> =
	ComponentContainer::new("Ton key repository");

/// Light client db seal for the Integritee parentchain
pub static GLOBAL_INTEGRITEE_PARENTCHAIN_LIGHT_CLIENT_SEAL: ComponentContainer<
	EnclaveLightClientSeal,
> = ComponentContainer::new("Integritee Parentchain EnclaveLightClientSealSync");

/// Light client db seal for the Target A parentchain.
pub static GLOBAL_TARGET_A_PARENTCHAIN_LIGHT_CLIENT_SEAL: ComponentContainer<
	EnclaveLightClientSeal,
> = ComponentContainer::new("Target A EnclaveLightClientSealSync");

/// Light client db seal for the Target A parentchain.
pub static GLOBAL_TARGET_B_PARENTCHAIN_LIGHT_CLIENT_SEAL: ComponentContainer<
	EnclaveLightClientSeal,
> = ComponentContainer::new("Target B EnclaveLightClientSealSync");

/// O-Call API
pub static GLOBAL_OCALL_API_COMPONENT: ComponentContainer<EnclaveOCallApi> =
	ComponentContainer::new("O-call API");

/// Trusted Web-socket server
pub static GLOBAL_WEB_SOCKET_SERVER_COMPONENT: ComponentContainer<EnclaveWebSocketServer> =
	ComponentContainer::new("Web-socket server");

/// State handler.
pub static GLOBAL_STATE_HANDLER_COMPONENT: ComponentContainer<EnclaveStateHandler> =
	ComponentContainer::new("state handler");

/// State observer.
pub static GLOBAL_STATE_OBSERVER_COMPONENT: ComponentContainer<EnclaveStateObserver> =
	ComponentContainer::new("state observer");

/// TOP pool author.
pub static GLOBAL_TOP_POOL_AUTHOR_COMPONENT: ComponentContainer<EnclaveTopPoolAuthor> =
	ComponentContainer::new("top_pool_author");

/// attestation handler
pub static GLOBAL_ATTESTATION_HANDLER_COMPONENT: ComponentContainer<EnclaveAttestationHandler> =
	ComponentContainer::new("Attestation handler");

// Parentchain component instances
//-------------------------------------------------------------------------------------------------

lazy_static! {
	/// Global nonce cache for the Integritee Parentchain.
	pub static ref GLOBAL_INTEGRITEE_PARENTCHAIN_NONCE_CACHE: Arc<NonceCache> = Default::default();

	/// Global nonce cache for the Target A parentchain..
	pub static ref GLOBAL_TARGET_A_PARENTCHAIN_NONCE_CACHE: Arc<NonceCache> = Default::default();

	/// Global nonce cache for the Target B parentchain..
	pub static ref GLOBAL_TARGET_B_PARENTCHAIN_NONCE_CACHE: Arc<NonceCache> = Default::default();
}

/// Solochain Handler.
pub static GLOBAL_INTEGRITEE_SOLOCHAIN_HANDLER_COMPONENT: ComponentContainer<
	IntegriteeSolochainHandler,
> = ComponentContainer::new("integritee solochain handler");

pub static GLOBAL_INTEGRITEE_PARACHAIN_HANDLER_COMPONENT: ComponentContainer<
	IntegriteeParachainHandler,
> = ComponentContainer::new("integritee parachain handler");

pub static GLOBAL_TARGET_A_SOLOCHAIN_HANDLER_COMPONENT: ComponentContainer<
	TargetASolochainHandler,
> = ComponentContainer::new("target A solochain handler");

pub static GLOBAL_TARGET_A_PARACHAIN_HANDLER_COMPONENT: ComponentContainer<
	TargetAParachainHandler,
> = ComponentContainer::new("target A parachain handler");

pub static GLOBAL_TARGET_B_SOLOCHAIN_HANDLER_COMPONENT: ComponentContainer<
	TargetBSolochainHandler,
> = ComponentContainer::new("target B solochain handler");

pub static GLOBAL_TARGET_B_PARACHAIN_HANDLER_COMPONENT: ComponentContainer<
	TargetBParachainHandler,
> = ComponentContainer::new("target B parachain handler");

// Sidechain component instances
//-------------------------------------------------------------------------------------------------

/// Enclave RPC WS handler.
pub static GLOBAL_RPC_WS_HANDLER_COMPONENT: ComponentContainer<EnclaveRpcWsHandler> =
	ComponentContainer::new("rpc_ws_handler");

/// Relayer registry
pub static GLOBAL_RELAYER_REGISTRY: ComponentContainer<RelayerRegistry> =
	ComponentContainer::new("relayer_registry");

/// Signer registry
pub static GLOBAL_SIGNER_REGISTRY: ComponentContainer<SignerRegistry> =
	ComponentContainer::new("signer_registry");

/// Enclave registry
pub static GLOBAL_ENCLAVE_REGISTRY: ComponentContainer<EnclaveRegistry> =
	ComponentContainer::new("enclave_registry");