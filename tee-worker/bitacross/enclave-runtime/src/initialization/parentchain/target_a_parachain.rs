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

//! Naive implementation of adding a second parachain handler to the setup.
//!
//! Ideally, most of the redundant code can be abstracted away, but it turns out
//! that this is quite tedious, so for now this is a copy-past of the [IntegriteeParachainHandler]:
//! * https://github.com/integritee-network/worker/issues/1417

use crate::{
	error::Result,
	initialization::{
		global_components::{
			EnclaveExtrinsicsFactory, EnclaveNodeMetadataRepository, EnclaveOCallApi,
			EnclaveStfExecutor, EnclaveValidatorAccessor, TargetAParentchainBlockImportDispatcher,
			GLOBAL_OCALL_API_COMPONENT, GLOBAL_STATE_HANDLER_COMPONENT,
			GLOBAL_TARGET_A_PARENTCHAIN_LIGHT_CLIENT_SEAL, GLOBAL_TARGET_A_PARENTCHAIN_NONCE_CACHE,
		},
		parentchain::common::{
			create_extrinsics_factory, create_target_a_offchain_immediate_import_dispatcher,
			create_target_a_parentchain_block_importer,
		},
	},
};
use itc_parentchain::light_client::{concurrent_access::ValidatorAccess, LightClientState};
use itp_component_container::ComponentGetter;
use itp_stf_interface::ShardCreationInfo;
use itp_types::parentchain::ParentchainId;
pub use itp_types::parentchain::{ParachainBlock, ParachainHeader, ParachainParams};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct TargetAParachainHandler {
	pub genesis_header: ParachainHeader,
	pub node_metadata_repository: Arc<EnclaveNodeMetadataRepository>,
	pub stf_executor: Arc<EnclaveStfExecutor>,
	pub validator_accessor: Arc<EnclaveValidatorAccessor>,
	pub extrinsics_factory: Arc<EnclaveExtrinsicsFactory>,
	pub import_dispatcher: Arc<TargetAParentchainBlockImportDispatcher>,
}

impl TargetAParachainHandler {
	pub fn init(
		_base_path: PathBuf,
		params: ParachainParams,
		shard_creation_info: ShardCreationInfo,
	) -> Result<Self> {
		let ocall_api = GLOBAL_OCALL_API_COMPONENT.get()?;
		let state_handler = GLOBAL_STATE_HANDLER_COMPONENT.get()?;
		let node_metadata_repository = Arc::new(EnclaveNodeMetadataRepository::default());

		let genesis_header = params.genesis_header.clone();

		let light_client_seal = GLOBAL_TARGET_A_PARENTCHAIN_LIGHT_CLIENT_SEAL.get()?;
		let validator = itc_parentchain::light_client::io::read_or_init_parachain_validator::<
			ParachainBlock,
			EnclaveOCallApi,
			_,
		>(params, ocall_api.clone(), &*light_client_seal, ParentchainId::TargetA)?;
		let validator_accessor =
			Arc::new(EnclaveValidatorAccessor::new(validator, light_client_seal));

		let genesis_hash = validator_accessor.execute_on_validator(|v| v.genesis_hash())?;

		let extrinsics_factory = create_extrinsics_factory(
			genesis_hash,
			GLOBAL_TARGET_A_PARENTCHAIN_NONCE_CACHE.clone(),
			node_metadata_repository.clone(),
		)?;

		let stf_executor = Arc::new(EnclaveStfExecutor::new(
			ocall_api,
			state_handler,
			node_metadata_repository.clone(),
		));

		let block_importer = create_target_a_parentchain_block_importer(
			validator_accessor.clone(),
			stf_executor.clone(),
			extrinsics_factory.clone(),
			node_metadata_repository.clone(),
			shard_creation_info,
		)?;

		let import_dispatcher = create_target_a_offchain_immediate_import_dispatcher(
			stf_executor.clone(),
			block_importer,
			validator_accessor.clone(),
			extrinsics_factory.clone(),
		)?;

		let parachain_handler = Self {
			genesis_header,
			node_metadata_repository,
			stf_executor,
			validator_accessor,
			extrinsics_factory,
			import_dispatcher,
		};

		Ok(parachain_handler)
	}
}
