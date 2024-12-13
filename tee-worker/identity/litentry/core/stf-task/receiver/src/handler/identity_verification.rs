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

use crate::{
	handler::TaskHandler, EnclaveMetricsOCallApi, EnclaveOnChainOCallApi, Getter, StfTaskContext,
	TrustedCall, TrustedCallSigned,
};
use ita_sgx_runtime::Hash;
use itp_sgx_crypto::{key_repository::AccessKey, ShieldingCryptoEncrypt};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_stf_executor::traits::StfEnclaveSigning;
use itp_stf_state_handler::handle_state::HandleState;
use itp_top_pool_author::traits::AuthorApi;
use itp_types::{ShardIdentifier, H256};
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use litentry_primitives::{IMPError, Web2IdentityVerificationRequest};
use log::*;
use sp_core::H160;
use std::sync::{mpsc::Sender, Arc};
pub(crate) struct IdentityVerificationHandler<
	ShieldingKeyRepository,
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter>,
	S: StfEnclaveSigning<TrustedCallSigned>,
	H: HandleState,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem>,
> where
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + 'static,
{
	pub(crate) req: Web2IdentityVerificationRequest,
	pub(crate) context: Arc<StfTaskContext<ShieldingKeyRepository, A, S, H, O, AR>>,
}

impl<ShieldingKeyRepository, A, S, H, O, AR> TaskHandler
	for IdentityVerificationHandler<ShieldingKeyRepository, A, S, H, O, AR>
where
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + 'static,
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter>,
	S: StfEnclaveSigning<TrustedCallSigned>,
	H: HandleState,
	H::StateT: SgxExternalitiesTrait,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem>,
{
	type Error = IMPError;
	type Result = ();

	fn on_process(&self) -> Result<Self::Result, Self::Error> {
		lc_identity_verification::verify(&self.req, &self.context.data_provider_config)
	}

	fn on_success(
		&self,
		_result: Self::Result,
		sender: Sender<(ShardIdentifier, H256, TrustedCall)>,
	) {
		debug!("verify identity OK");
		if let Ok(enclave_signer) = self.context.enclave_signer.get_enclave_account() {
			let c = TrustedCall::link_identity_callback(
				enclave_signer.into(),
				self.req.who.clone(),
				self.req.identity.clone(),
				self.req.web3networks.clone(),
				self.req.maybe_key,
				self.req.req_ext_hash,
			);
			if let Err(e) = sender.send((self.req.shard, self.req.top_hash, c)) {
				error!("Unable to send message to the trusted_call_receiver: {:?}", e);
			}
		} else {
			error!("can't get enclave signer");
		}
	}

	fn on_failure(&self, error: Self::Error, sender: Sender<(ShardIdentifier, H256, TrustedCall)>) {
		error!("verify identity failed:{:?}", error);
		if let Ok(enclave_signer) = self.context.enclave_signer.get_enclave_account() {
			let c = TrustedCall::handle_imp_error(
				enclave_signer.into(),
				Some(self.req.who.clone()),
				error,
				self.req.req_ext_hash,
			);
			if let Err(e) = sender.send((self.req.shard, self.req.top_hash, c)) {
				error!("Unable to send message to the trusted_call_receiver: {:?}", e);
			}
		} else {
			error!("can't get enclave signer");
		}
	}
}
