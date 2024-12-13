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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use super::*;
use crate::{
	helpers::{ensure_enclave_signer_or_self, get_expected_raw_message, verify_web3_identity},
	trusted_call_result::{LinkIdentityResult, TrustedCallResult},
	Arc, Vec,
};
use codec::Encode;
use frame_support::{dispatch::UnfilteredDispatchable, ensure};
use ita_sgx_runtime::{RuntimeOrigin, System};
use itp_node_api::metadata::NodeMetadataTrait;
use itp_node_api_metadata::pallet_imp::IMPCallIndexes;
use itp_node_api_metadata_provider::AccessNodeMetadata;
use itp_stf_primitives::{
	error::{StfError, StfResult},
	types::{AccountId, ShardIdentifier},
};
use itp_types::{parentchain::ParentchainCall, OpaqueCall, H256};
use itp_utils::stringify::account_id_to_string;
use lc_stf_task_sender::{SendStfRequest, StfRequestSender};
use litentry_macros::if_development_or;
use litentry_primitives::{
	ErrorDetail, Identity, RequestAesKey, RequestType, ValidationData,
	Web2IdentityVerificationRequest, Web3Network,
};
use log::*;

impl TrustedCallSigned {
	#[allow(clippy::too_many_arguments)]
	pub fn link_identity_internal(
		shard: &ShardIdentifier,
		signer: AccountId,
		who: Identity,
		identity: Identity,
		validation_data: ValidationData,
		web3networks: Vec<Web3Network>,
		top_hash: H256,
		maybe_key: Option<RequestAesKey>,
		req_ext_hash: H256,
	) -> StfResult<bool> {
		ensure!(
			ensure_enclave_signer_or_self(&signer, who.to_native_account()),
			StfError::LinkIdentityFailed(ErrorDetail::UnauthorizedSigner)
		);

		// note it's the signer's nonce, not `who`
		// we intentionally use `System::account_nonce - 1` to make up for the increment at the
		// beginning of STF execution, otherwise it might be unexpected that we were hoping
		// (current nonce + 1) when verifying the validation data.
		let sidechain_nonce = System::account_nonce(&signer) - 1;

		let raw_msg = get_expected_raw_message(&who, &identity, sidechain_nonce);

		match validation_data {
			ValidationData::Web2(data) => {
				ensure!(
					identity.is_web2(),
					StfError::LinkIdentityFailed(ErrorDetail::InvalidIdentity)
				);
				let request: RequestType = Web2IdentityVerificationRequest {
					shard: *shard,
					who,
					identity,
					raw_msg,
					validation_data: data,
					web3networks,
					top_hash,
					maybe_key,
					req_ext_hash,
				}
				.into();
				StfRequestSender::new()
					.send_stf_request(request)
					.map_err(|_| StfError::LinkIdentityFailed(ErrorDetail::SendStfRequestFailed))?;
				Ok(false)
			},
			ValidationData::Web3(validation_data) => {
				ensure!(
					identity.is_web3(),
					StfError::LinkIdentityFailed(ErrorDetail::InvalidIdentity)
				);

				verify_web3_identity(&identity, &raw_msg, &validation_data)?;

				Ok(true)
			},
		}
	}

	pub fn deactivate_identity_internal(
		signer: AccountId,
		who: Identity,
		identity: Identity,
	) -> StfResult<()> {
		ensure!(
			ensure_enclave_signer_or_self(&signer, who.to_native_account()),
			StfError::DeactivateIdentityFailed(ErrorDetail::UnauthorizedSigner)
		);

		IMTCall::deactivate_identity { who, identity }
			.dispatch_bypass_filter(RuntimeOrigin::root())
			.map_err(|e| StfError::DeactivateIdentityFailed(e.into()))?;

		Ok(())
	}

	pub fn activate_identity_internal(
		signer: AccountId,
		who: Identity,
		identity: Identity,
	) -> StfResult<()> {
		ensure!(
			ensure_enclave_signer_or_self(&signer, who.to_native_account()),
			StfError::ActivateIdentityFailed(ErrorDetail::UnauthorizedSigner)
		);

		IMTCall::activate_identity { who, identity }
			.dispatch_bypass_filter(RuntimeOrigin::root())
			.map_err(|e| StfError::ActivateIdentityFailed(e.into()))?;

		Ok(())
	}

	pub fn link_identity_callback_internal(
		signer: AccountId,
		who: Identity,
		identity: Identity,
		web3networks: Vec<Web3Network>,
	) -> StfResult<()> {
		if_development_or!(
			{
				use crate::helpers::ensure_enclave_signer_or_alice;
				// In non-prod: we allow to use `Alice` as the dummy signer
				ensure!(
					ensure_enclave_signer_or_alice(&signer),
					StfError::LinkIdentityFailed(ErrorDetail::UnauthorizedSigner)
				);
			},
			{
				use crate::helpers::ensure_enclave_signer_account;
				// In prod: the signer has to be enclave_signer_account, as this TrustedCall can only be constructed internally
				ensure_enclave_signer_account(&signer)
					.map_err(|_| StfError::LinkIdentityFailed(ErrorDetail::UnauthorizedSigner))?;
			}
		);
		IMTCall::link_identity { who, identity, web3networks }
			.dispatch_bypass_filter(RuntimeOrigin::root())
			.map_err(|e| StfError::LinkIdentityFailed(e.into()))?;

		Ok(())
	}

	// common handler for both web2 and web3 identity verification
	#[allow(clippy::too_many_arguments)]
	pub fn handle_link_identity_callback<NodeMetadataRepository>(
		calls: &mut Vec<ParentchainCall>,
		node_metadata_repo: Arc<NodeMetadataRepository>,
		signer: Identity,
		who: Identity,
		identity: Identity,
		web3networks: Vec<Web3Network>,
		maybe_key: Option<RequestAesKey>,
		req_ext_hash: H256,
	) -> StfResult<TrustedCallResult>
	where
		NodeMetadataRepository: AccessNodeMetadata,
		NodeMetadataRepository::MetadataType: NodeMetadataTrait,
	{
		debug!("link_identity_callback, who: {}", account_id_to_string(&who));
		// the pallet extrinsic doesn't accept customised return type, so
		// we have to do the if-condition outside of extrinsic call
		let old_id_graph = IMT::id_graph(&who);

		Self::link_identity_callback_internal(
			signer.to_native_account().ok_or(StfError::InvalidAccount)?,
			who.clone(),
			identity,
			web3networks,
		)
		.map_err(|e| {
			debug!("pushing error event ... error: {}", e);
			push_call_imp_some_error(
				calls,
				node_metadata_repo.clone(),
				Some(who.clone()),
				e.to_imp_error(),
				req_ext_hash,
			);
			e
		})?;

		let id_graph_hash: H256 = IMT::id_graph_hash(&who).ok_or(StfError::EmptyIDGraph)?;

		debug!("pushing identity_linked event ...");
		// push `identity_linked` call
		let call_index =
			node_metadata_repo.get_from_metadata(|m| m.identity_linked_call_indexes())??;
		calls.push(ParentchainCall::Litentry(OpaqueCall::from_tuple(&(
			call_index,
			who.clone(),
			id_graph_hash,
			req_ext_hash,
		))));

		let mut mutated_id_graph = IMT::id_graph(&who);
		mutated_id_graph.retain(|i| !old_id_graph.contains(i));

		if let Some(key) = maybe_key {
			return Ok(TrustedCallResult::LinkIdentity(LinkIdentityResult {
				mutated_id_graph: aes_encrypt_default(&key, &mutated_id_graph.encode()),
				id_graph_hash,
			}))
		}

		Ok(TrustedCallResult::Empty)
	}
}
