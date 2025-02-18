use crate::{
	types::{NativeOperationError, NativeOperationOk},
	NativeOperationResponse, ResponseSender, TaskHandlerContext,
};
use executor_core::native_operation::NativeQuery;
use executor_crypto::aes256::{aes_decrypt, AesOutput};
use executor_primitives::{Identity, MemberAccount};
use executor_storage::{AccountStoreStorage, MemberOmniAccountStorage, Storage};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;
use subxt::ext::subxt_core::alloc::vec::Vec;

pub async fn handle_native_query<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	ctx: Arc<TaskHandlerContext<Header, RpcClient, RpcClientFactory>>,
	query: NativeQuery,
	response_sender: ResponseSender,
) {
	match query {
		NativeQuery::get_account_store(sender_identity) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
				let response =
					NativeOperationResponse::Err(NativeOperationError::InvalidMemberIdentity);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let account_store_storage = AccountStoreStorage::new(ctx.storage_db.clone());
			let account_store_result = account_store_storage
				.get(&omni_account)
				.map(|acc_store| {
					acc_store
						.into_iter()
						.map(|member| match member {
							MemberAccount::Public(_) => Ok(member),
							MemberAccount::Private(encoded_aes_output, _) => {
								let mut encrypted_member =
									AesOutput::decode(&mut encoded_aes_output.as_slice())
										.map_err(|_| "Failed to decode AesOutput".to_string())?;
								let encoded_member =
									aes_decrypt(&ctx.aes256_key, &mut encrypted_member)
										.ok_or("Failed to decrypt AesOutput".to_string())?;
								let member_identity: Identity =
									Identity::decode(&mut encoded_member.as_slice())
										.map_err(|_| "Failed to decode Identity".to_string())?;
								Ok(MemberAccount::Public(member_identity))
							},
						})
						.collect::<Result<Vec<MemberAccount>, String>>()
						.map(BoundedVec)
				})
				.transpose();
			let account_store = match account_store_result {
				Ok(account_store) => account_store,
				Err(e) => {
					log::error!("Failed to get account store: {}", e);
					let response =
						NativeOperationResponse::Err(NativeOperationError::InternalError);
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
					return;
				},
			};
			let response =
				NativeOperationResponse::Ok(NativeOperationOk::QueryData(account_store.encode()));
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response to the sender");
			}
		},
	}
}
