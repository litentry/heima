use crate::{
	types::{NativeOperationError, QueryResponse},
	NativeOperationResponse, ResponseSender, TaskHandlerContext,
};
use executor_core::{intent_executor::IntentExecutor, native_operation::NativeQuery};
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
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<
		TaskHandlerContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
		>,
	>,
	query: NativeQuery,
	response_sender: ResponseSender,
) {
	match query {
		NativeQuery::get_account_store(sender_identity) => {
			let join_handle = tokio::task::spawn_blocking(move || -> NativeOperationResponse {
				let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
				let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
					return NativeOperationResponse::Err(
						NativeOperationError::InvalidMemberIdentity,
					);
				};
				let account_store_storage = AccountStoreStorage::new(ctx.storage_db.clone());
				let account_store_result = account_store_storage
					.get(&omni_account)
					.map(|acc_store| {
						acc_store
							.into_iter()
							.map(|member| match member {
								MemberAccount::Public(member_identity) => Ok(member_identity),
								MemberAccount::Private(encoded_aes_output, _) => {
									let mut encrypted_member =
										AesOutput::decode(&mut encoded_aes_output.as_slice())
											.map_err(|_| {
												"Failed to decode AesOutput".to_string()
											})?;
									let encoded_member =
										aes_decrypt(&ctx.aes256_key, &mut encrypted_member)
											.ok_or("Failed to decrypt AesOutput".to_string())?;
									let member_identity: Identity =
										Identity::decode(&mut encoded_member.as_slice())
											.map_err(|_| "Failed to decode Identity".to_string())?;
									Ok(member_identity)
								},
							})
							.collect::<Result<Vec<Identity>, String>>()
					})
					.transpose();
				let account_store = match account_store_result {
					Ok(account_store) => account_store.unwrap_or_default(),
					Err(e) => {
						log::error!("Failed to get account store: {}", e);
						let response =
							NativeOperationResponse::Err(NativeOperationError::InternalError);
						return response;
					},
				};
				QueryResponse::AccountStore(account_store).into()
			});

			match join_handle.await {
				Ok(response) => {
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
				},
				Err(e) => {
					log::error!("Failed to handle get_account_store: {:?}", e);
					let response =
						NativeOperationResponse::Err(NativeOperationError::InternalError);
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
				},
			}
		},
	}
}
