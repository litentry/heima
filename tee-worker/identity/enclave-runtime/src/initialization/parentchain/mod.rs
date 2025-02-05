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

use crate::{
	error::Result,
	initialization::{
		global_components::{
			GLOBAL_INTEGRITEE_PARACHAIN_HANDLER_COMPONENT,
			GLOBAL_INTEGRITEE_SOLOCHAIN_HANDLER_COMPONENT,
			GLOBAL_TARGET_A_PARACHAIN_HANDLER_COMPONENT,
			GLOBAL_TARGET_A_SOLOCHAIN_HANDLER_COMPONENT,
			GLOBAL_TARGET_B_PARACHAIN_HANDLER_COMPONENT,
			GLOBAL_TARGET_B_SOLOCHAIN_HANDLER_COMPONENT,
		},
		parentchain::{
			target_a_parachain::TargetAParachainHandler,
			target_a_solochain::TargetASolochainHandler,
			target_b_parachain::TargetBParachainHandler,
			target_b_solochain::TargetBSolochainHandler,
		},
	},
	shard_creation_info::get_shard_creation_info_internal,
};
use codec::{Decode, Encode};
use integritee_parachain::IntegriteeParachainHandler;
use integritee_solochain::IntegriteeSolochainHandler;
use itc_parentchain::light_client::{concurrent_access::ValidatorAccess, LightClientState};
use itp_component_container::ComponentInitializer;
use itp_settings::worker_mode::ProvideWorkerMode;
use itp_types::parentchain::{ParentchainId, ParentchainInitParams};
use log::*;
use std::{path::PathBuf, vec::Vec};

mod common;
pub mod integritee_parachain;
pub mod integritee_solochain;
pub mod target_a_parachain;
pub mod target_a_solochain;
pub mod target_b_parachain;
pub mod target_b_solochain;

pub(crate) fn init_parentchain_components<WorkerModeProvider: ProvideWorkerMode>(
	base_path: PathBuf,
	encoded_params: Vec<u8>,
) -> Result<Vec<u8>> {
	match ParentchainInitParams::decode(&mut encoded_params.as_slice())? {
		ParentchainInitParams::Parachain { id, shard, params } => {
			info!(
				"[{:?}] initializing parachain parentchain components for shard: {:?}",
				id, shard
			);
			let shard_creation_info = get_shard_creation_info_internal(shard)?;

			// todo: query timestamp of creation header to give a creation reference to target_a/b as well in order to fast-sync
			match id {
				ParentchainId::Litentry => {
					let handler = IntegriteeParachainHandler::init::<WorkerModeProvider>(
						base_path,
						params,
						shard_creation_info,
					)?;
					let header = handler
						.validator_accessor
						.execute_on_validator(|v| v.latest_finalized_header())?;
					GLOBAL_INTEGRITEE_PARACHAIN_HANDLER_COMPONENT.initialize(handler.into());
					Ok(header.encode())
				},
				ParentchainId::TargetA => {
					let handler = TargetAParachainHandler::init::<WorkerModeProvider>(
						base_path,
						params,
						shard_creation_info,
					)?;
					let header = handler
						.validator_accessor
						.execute_on_validator(|v| v.latest_finalized_header())?;
					GLOBAL_TARGET_A_PARACHAIN_HANDLER_COMPONENT.initialize(handler.into());
					Ok(header.encode())
				},
				ParentchainId::TargetB => {
					let handler = TargetBParachainHandler::init::<WorkerModeProvider>(
						base_path,
						params,
						shard_creation_info,
					)?;
					let header = handler
						.validator_accessor
						.execute_on_validator(|v| v.latest_finalized_header())?;
					GLOBAL_TARGET_B_PARACHAIN_HANDLER_COMPONENT.initialize(handler.into());
					Ok(header.encode())
				},
			}
		},
		ParentchainInitParams::Solochain { id, shard, params } => {
			info!(
				"[{:?}] initializing solochain parentchain components for shard: {:?}",
				id, shard
			);
			let shard_creation_info = get_shard_creation_info_internal(shard)?;
			// todo: query timestamp of creation header to give a creation reference to target_a/b as well in order to fast-sync
			match id {
				ParentchainId::Litentry => {
					let handler = IntegriteeSolochainHandler::init::<WorkerModeProvider>(
						base_path,
						params,
						shard_creation_info,
					)?;
					let header = handler
						.validator_accessor
						.execute_on_validator(|v| v.latest_finalized_header())?;
					GLOBAL_INTEGRITEE_SOLOCHAIN_HANDLER_COMPONENT.initialize(handler.into());
					Ok(header.encode())
				},
				ParentchainId::TargetA => {
					let handler = TargetASolochainHandler::init::<WorkerModeProvider>(
						base_path,
						params,
						shard_creation_info,
					)?;
					let header = handler
						.validator_accessor
						.execute_on_validator(|v| v.latest_finalized_header())?;
					GLOBAL_TARGET_A_SOLOCHAIN_HANDLER_COMPONENT.initialize(handler.into());
					Ok(header.encode())
				},
				ParentchainId::TargetB => {
					let handler = TargetBSolochainHandler::init::<WorkerModeProvider>(
						base_path,
						params,
						shard_creation_info,
					)?;
					let header = handler
						.validator_accessor
						.execute_on_validator(|v| v.latest_finalized_header())?;
					GLOBAL_TARGET_B_SOLOCHAIN_HANDLER_COMPONENT.initialize(handler.into());
					Ok(header.encode())
				},
			}
		},
	}
}
