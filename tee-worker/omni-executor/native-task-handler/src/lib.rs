mod types;

use executor_core::native_call::NativeCall;
use executor_crypto::jwt;
use executor_primitives::{intent::Intent, OmniAccountAuthType};
use executor_storage::{MemberOmniAccountStorage, Storage, StorageDB};
use heima_authentication::auth_token::AuthTokenClaims;
use parentchain_api_interface::{
	omni_account::calls::types::create_account_store,
	runtime_types::{
		frame_system::pallet::Call as SystemCall, pallet_balances::pallet::Call as BalancesCall,
		pallet_omni_account::pallet::Call as OmniAccountCall, paseo_parachain_runtime::RuntimeCall,
	},
};
use parentchain_rpc_client::{
	metadata::{Metadata, SubxtMetadataProvider},
	AccountId32, CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient,
	SubxtClientFactory, XtStatus,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TransactionSigner};
use parity_scale_codec::{Codec, Decode, Encode};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use types::{NativeCallError, NativeCallOk};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;

pub type NativeTaskSender = mpsc::Sender<NativeTask>;

type NativeCallResponse<Hash> = Result<NativeCallOk<Hash>, NativeCallError>;

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
	Hash,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
> {
	pub parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	pub storage_db: Arc<StorageDB>,
	pub jwt_secret: String,
	pub transaction_signer: Arc<ParentchainTxSigner>,
	phantom_account_id: PhantomData<AccountId>,
	phantom_header: PhantomData<Header>,
	phantom_hash: PhantomData<Hash>,
	phantom_rpc_client: PhantomData<RpcClient>,
}

impl<
		AccountId,
		Header,
		Hash,
		RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
		RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
	> TaskHandlerContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>
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
			phantom_hash: PhantomData,
			phantom_rpc_client: PhantomData,
		}
	}
}

pub async fn run_native_task_handler<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	Hash: Send + Sync + Codec + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient> + Send + Sync + 'static,
>(
	buffer: usize,
	ctx: Arc<TaskHandlerContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>>,
) -> NativeTaskSender {
	let (sender, mut receiver) = mpsc::channel::<NativeTask>(buffer);

	tokio::spawn(async move {
		while let Some(task) = receiver.recv().await {
			handle_native_task(ctx.clone(), task).await;
		}
	});

	sender
}

async fn handle_native_task<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	Hash: Send + Sync + Codec + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient> + Send + Sync + 'static,
>(
	ctx: Arc<TaskHandlerContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>>,
	task: NativeTask,
) {
	let response_sender = task.response_sender;

	let Ok(mut rpc_client) = ctx.parentchain_rpc_client_factory.new_client().await else {
		let response = NativeCallResponse::<Hash>::Err(NativeCallError::InternalError);
		if response_sender.send(response.encode()).is_err() {
			log::error!("Failed to send response");
		}
		return;
	};

	match task.call {
		NativeCall::request_auth_token(sender_identity, auth_options) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
				let response = NativeCallResponse::<Hash>::Err(NativeCallError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let claims = AuthTokenClaims::new(sender_identity.hash().to_string(), auth_options);
			let Ok(token) = jwt::create(&claims, ctx.jwt_secret.as_bytes()) else {
				let response =
					NativeCallResponse::<Hash>::Err(NativeCallError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let auth_token_requested_call = parentchain_api_interface::tx()
				.omni_account()
				.auth_token_requested(AccountId32(omni_account.into()), claims.exp);

			let signed_call = ctx.transaction_signer.sign(auth_token_requested_call).await;

			if rpc_client.submit_tx(&signed_call).await.is_err() {
				let response = NativeCallResponse::<Hash>::Err(NativeCallError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}

			let response = NativeCallResponse::Ok(NativeCallOk::<Hash>::AuthToken(token));

			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
		NativeCall::request_intent(sender_identity, intent) => {
			let tx = match intent {
				Intent::SystemRemark(remark) => {
					let remark_call = SystemCall::remark { remark: remark.to_vec() };
					let auth_type_bytes = task.auth_type.encode();
					let member_hash_bytes = sender_identity.hash().encode();
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
							Decode::decode(&mut &member_hash_bytes[..]).unwrap(),
							RuntimeCall::System(remark_call),
							Decode::decode(&mut &auth_type_bytes[..]).unwrap(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
				},
				Intent::TransferNative(transfer) => {
					let to_bytes = transfer.to.encode();
					let to: AccountId32 = Decode::decode(&mut &to_bytes[..]).unwrap();
					let transfer_call = BalancesCall::transfer_allow_death {
						dest: to.into(),
						value: transfer.value,
					};
					let auth_type_bytes = task.auth_type.encode();
					let member_hash_bytes = sender_identity.hash().encode();
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
							Decode::decode(&mut &member_hash_bytes[..]).unwrap(),
							RuntimeCall::Balances(transfer_call),
							Decode::decode(&mut &auth_type_bytes[..]).unwrap(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
				},
				Intent::CallEthereum(_)
				| Intent::TransferEthereum(_)
				| Intent::TransferSolana(_) => {
					let intent_bytes = intent.encode();
					let request_intent_call = OmniAccountCall::request_intent {
						intent: Decode::decode(&mut &intent_bytes[..]).unwrap(),
					};
					let auth_type_bytes = task.auth_type.encode();
					let member_hash_bytes = sender_identity.hash().encode();
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
							Decode::decode(&mut &member_hash_bytes[..]).unwrap(),
							RuntimeCall::OmniAccount(request_intent_call),
							Decode::decode(&mut &auth_type_bytes[..]).unwrap(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
				},
			};
			let Ok(report) = rpc_client.submit_and_watch_tx_until(&tx, XtStatus::Finalized).await
			else {
				let response = NativeCallResponse::<Hash>::Err(NativeCallError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let response = NativeCallResponse::Ok(NativeCallOk::ExtrinsicReport {
				extrinsic_hash: report.extrinsic_hash,
				block_hash: report.block_hash,
				status: report.status,
			});
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
		NativeCall::create_account_store(sender_identity) => {
			let sender_identity_bytes = sender_identity.encode();
			let create_account_store_call = parentchain_api_interface::tx()
				.omni_account()
				.create_account_store(Decode::decode(&mut &sender_identity_bytes[..]).unwrap());
			let tx = ctx.transaction_signer.sign(create_account_store_call).await;
			let Ok(report) = rpc_client.submit_and_watch_tx_until(&tx, XtStatus::Finalized).await
			else {
				let response = NativeCallResponse::<Hash>::Err(NativeCallError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let response = NativeCallResponse::Ok(NativeCallOk::ExtrinsicReport {
				extrinsic_hash: report.extrinsic_hash,
				block_hash: report.block_hash,
				status: report.status,
			});
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
		_ => {
			let response = NativeCallResponse::<Hash>::Err(NativeCallError::UnexpectedCall(
				format!("Unexpected call: {:?}", task.call),
			));
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
		},
	}
}
