// Copyright 2020-2023 Litentry Technologies GmbH.
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
	error::Error, executor::Executor, indirect_calls_executor::hash_of, ExecutionStatus,
	IndirectCallsExecutor,
};
use codec::{Decode, Encode};
use ita_stf::{TrustedCall, TrustedOperation};
use itp_node_api::{
	api_client::ParentchainUncheckedExtrinsic,
	metadata::{
		pallet_imp::IMPCallIndexes, pallet_teerex::TeerexCallIndexes, pallet_vcmp::VCMPCallIndexes,
		provider::AccessNodeMetadata, Error as MetadataError,
	},
};
use itp_sgx_crypto::{key_repository::AccessKey, ShieldingCryptoDecrypt, ShieldingCryptoEncrypt};
use itp_stf_executor::traits::StfEnclaveSigning;
use itp_top_pool_author::traits::AuthorApi;
use itp_types::{VerifyIdentityFn, H256};
use litentry_primitives::{Identity, ParentchainBlockNumber, ValidationData};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};

pub(crate) struct VerifyIdentity {
	pub(crate) block_number: ParentchainBlockNumber,
}

impl<ShieldingKeyRepository, StfEnclaveSigner, TopPoolAuthor, NodeMetadataProvider>
	Executor<ShieldingKeyRepository, StfEnclaveSigner, TopPoolAuthor, NodeMetadataProvider>
	for VerifyIdentity
where
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoDecrypt<Error = itp_sgx_crypto::Error>
		+ ShieldingCryptoEncrypt<Error = itp_sgx_crypto::Error>,
	StfEnclaveSigner: StfEnclaveSigning,
	TopPoolAuthor: AuthorApi<H256, H256> + Send + Sync + 'static,
	NodeMetadataProvider: AccessNodeMetadata,
	NodeMetadataProvider::MetadataType: IMPCallIndexes + TeerexCallIndexes + VCMPCallIndexes,
{
	type Call = VerifyIdentityFn;

	fn call_index(&self, call: Self::Call) -> [u8; 2] {
		call.0
	}

	fn call_index_from_metadata(
		&self,
		metadata_type: &NodeMetadataProvider::MetadataType,
	) -> Result<[u8; 2], MetadataError> {
		metadata_type.verify_identity_call_indexes()
	}

	fn execute(
		&self,
		context: &IndirectCallsExecutor<
			ShieldingKeyRepository,
			StfEnclaveSigner,
			TopPoolAuthor,
			NodeMetadataProvider,
		>,
		extrinsic: ParentchainUncheckedExtrinsic<Self::Call>,
	) -> Result<ExecutionStatus<H256>, Error> {
		let (_, shard, encrypted_identity, encrypted_validation_data) = extrinsic.function.clone();
		let shielding_key = context.shielding_key_repo.retrieve_key()?;

		let identity: Identity =
			Identity::decode(&mut shielding_key.decrypt(&encrypted_identity).unwrap().as_slice())?;
		let validation_data = ValidationData::decode(
			&mut shielding_key.decrypt(&encrypted_validation_data).unwrap().as_slice(),
		)?;

		if let Some((multiaddress_account, _, _)) = extrinsic.signature.clone() {
			let account = AccountIdLookup::lookup(multiaddress_account)?;
			let enclave_account_id = context.stf_enclave_signer.get_enclave_account()?;
			let trusted_call = TrustedCall::verify_identity_preflight(
				enclave_account_id,
				account,
				identity,
				validation_data,
				self.block_number,
			);
			let signed_trusted_call =
				context.stf_enclave_signer.sign_call_with_self(&trusted_call, &shard)?;
			let trusted_operation = TrustedOperation::indirect_call(signed_trusted_call);

			let encrypted_trusted_call = shielding_key.encrypt(&trusted_operation.encode())?;
			context.submit_trusted_call(shard, encrypted_trusted_call);
		}
		Ok(ExecutionStatus::Success(hash_of(&extrinsic)))
	}
}
