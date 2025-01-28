mod types;

use executor_core::native_call::NativeCall;
use executor_crypto::jwt;
use executor_storage::{MemberOmniAccountStorage, Storage, StorageDB};
use heima_authentication::auth_token::AuthTokenClaims;
use parentchain_rpc_client::{
	metadata::{Metadata, SubxtMetadataProvider},
	AccountId32, CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient,
	SubxtClientFactory,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TransactionSigner};
use parity_scale_codec::Encode;
use primitives::{utils::hex::ToHexPrefixed, OmniAccountAuthType};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use types::{NativeCallError, NativeCallOk};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;

pub type NativeTaskSender = mpsc::Sender<NativeTask>;

type NativeCallResponse = Result<NativeCallOk, NativeCallError>;

pub type ParentchainTxSigner = TransactionSigner<
	SubstrateKeyStore,
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

pub struct NativeTask {
	pub call: NativeCall,
	pub auth_type: OmniAccountAuthType,
	pub response_sender: ResponseSender,
}

pub struct TaskHandlerContext<
	AccountId,
	Header,
	RpcClient: SubstrateRpcClient<AccountId, Header>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient>,
> {
	pub parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	pub storage_db: Arc<StorageDB>,
	pub jwt_secret: String,
	pub transaction_signer: Arc<ParentchainTxSigner>,
	phantom_account_id: PhantomData<AccountId>,
	phantom_header: PhantomData<Header>,
	phantom_rpc_client: PhantomData<RpcClient>,
}

impl<
		AccountId,
		Header,
		RpcClient: SubstrateRpcClient<AccountId, Header>,
		RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient>,
	> TaskHandlerContext<AccountId, Header, RpcClient, RpcClientFactory>
{
	pub fn new(
		parentchain_rpc_client_factory: Arc<RpcClientFactory>,
		transaction_signer: Arc<ParentchainTxSigner>,
		storage_db: Arc<StorageDB>,
		jwt_secret: String,
	) -> Self {
		Self {
			parentchain_rpc_client_factory,
			transaction_signer,
			storage_db,
			jwt_secret,
			phantom_account_id: PhantomData,
			phantom_header: PhantomData,
			phantom_rpc_client: PhantomData,
		}
	}
}

pub async fn run_native_task_handler<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient> + Send + Sync + 'static,
>(
	buffer: usize,
	ctx: Arc<TaskHandlerContext<AccountId, Header, RpcClient, RpcClientFactory>>,
) -> NativeTaskSender {
	let (sender, mut receiver) = mpsc::channel::<NativeTask>(buffer);

	tokio::spawn(async move {
		while let Some(task) = receiver.recv().await {
			handle_native_call(ctx.clone(), task.call, task.response_sender).await;
		}
	});

	sender
}

async fn handle_native_call<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient> + Send + Sync + 'static,
>(
	ctx: Arc<TaskHandlerContext<AccountId, Header, RpcClient, RpcClientFactory>>,
	call: NativeCall,
	response_sender: ResponseSender,
) {
	match call {
		NativeCall::request_auth_token(sender_identity, auth_options) => {
			let member_omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = member_omni_account_storage.get(&sender_identity.hash())
			else {
				let response = NativeCallResponse::Err(NativeCallError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let claims = AuthTokenClaims::new(omni_account.to_hex(), auth_options);
			let Ok(token) = jwt::create(&claims, ctx.jwt_secret.as_bytes()) else {
				let response = NativeCallResponse::Err(NativeCallError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let auth_token_requested_call = parentchain_api_interface::tx()
				.omni_account()
				.auth_token_requested(AccountId32(omni_account.into()), claims.exp);

			let Ok(mut client) = ctx.parentchain_rpc_client_factory.new_client().await else {
				let response = NativeCallResponse::Err(NativeCallError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};

			let signed_call = ctx.transaction_signer.sign(auth_token_requested_call).await;

			if client.submit_tx(&signed_call).await.is_err() {
				let response = NativeCallResponse::Err(NativeCallError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}

			let response = NativeCallResponse::Ok(NativeCallOk::AuthToken(token));

			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
		_ => {
			let response = NativeCallResponse::Err(NativeCallError::UnexpectedCall(format!(
				"Unexpected call: {:?}",
				call
			)));
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
	}
}
