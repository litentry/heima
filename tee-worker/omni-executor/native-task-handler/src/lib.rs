mod aes256_key_store;
pub use aes256_key_store::Aes256KeyStore;

mod types;

use executor_core::native_call::NativeCall;
use executor_crypto::{
	aes256::{aes_encrypt_default, Aes256Key},
	jwt,
};
use executor_primitives::{intent::Intent, MemberAccount, OmniAccountAuthType, ValidationData};
use executor_storage::{MemberOmniAccountStorage, Storage, StorageDB};
use heima_authentication::auth_token::AuthTokenClaims;
use heima_identity_verification::{get_verification_message, web2, web3};
use parentchain_api_interface::runtime_types::{
	frame_system::pallet::Call as SystemCall, pallet_balances::pallet::Call as BalancesCall,
	pallet_omni_account::pallet::Call as OmniAccountCall, paseo_parachain_runtime::RuntimeCall,
};
use parentchain_rpc_client::{
	metadata::{Metadata, SubxtMetadataProvider},
	AccountId32, CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient,
	SubxtClientFactory, ToSubxtType, XtStatus,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TransactionSigner};
use parity_scale_codec::{Decode, Encode};
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
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
> {
	pub parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	pub storage_db: Arc<StorageDB>,
	pub jwt_secret: String,
	pub aes256_key: Aes256Key,
	pub transaction_signer: Arc<ParentchainTxSigner>,
	phantom_header: PhantomData<Header>,
	phantom_rpc_client: PhantomData<RpcClient>,
}

impl<
		Header,
		RpcClient: SubstrateRpcClient<Header>,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
	> TaskHandlerContext<Header, RpcClient, RpcClientFactory>
{
	pub fn new(
		parentchain_rpc_client_factory: Arc<RpcClientFactory>,
		transaction_signer: Arc<ParentchainTxSigner>,
		storage_db: Arc<StorageDB>,
		jwt_secret: String,
		aes256_key: Aes256Key,
	) -> Self {
		Self {
			parentchain_rpc_client_factory,
			transaction_signer,
			storage_db,
			jwt_secret,
			aes256_key,
			phantom_header: PhantomData,
			phantom_rpc_client: PhantomData,
		}
	}
}

pub async fn run_native_task_handler<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	buffer: usize,
	ctx: Arc<TaskHandlerContext<Header, RpcClient, RpcClientFactory>>,
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
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	ctx: Arc<TaskHandlerContext<Header, RpcClient, RpcClientFactory>>,
	task: NativeTask,
) {
	let Ok(mut rpc_client) = ctx.parentchain_rpc_client_factory.new_client().await else {
		log::error!("Failed to create rpc client");
		let response = NativeCallResponse::Err(NativeCallError::InternalError);
		if task.response_sender.send(response.encode()).is_err() {
			log::error!("Failed to send response");
		}
		return;
	};
	let (response_sender, tx) = match task.call {
		NativeCall::request_auth_token(sender_identity, auth_options) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
				let response = NativeCallResponse::Err(NativeCallError::UnauthorizedSender);
				if task.response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let claims = AuthTokenClaims::new(sender_identity.hash().to_string(), auth_options);
			let Ok(token) = jwt::create(&claims, ctx.jwt_secret.as_bytes()) else {
				let response = NativeCallResponse::Err(NativeCallError::AuthTokenCreationFailed);
				if task.response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let auth_token_requested_call = parentchain_api_interface::tx()
				.omni_account()
				.auth_token_requested(AccountId32(omni_account.into()), claims.exp);

			let signed_call = ctx.transaction_signer.sign(auth_token_requested_call).await;

			if rpc_client.submit_tx(&signed_call).await.is_err() {
				log::error!("Failed to submit tx");
				let response = NativeCallResponse::Err(NativeCallError::InternalError);
				if task.response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}

			let response = NativeCallResponse::Ok(NativeCallOk::AuthToken(token));

			if task.response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
		NativeCall::request_intent(sender_identity, intent) => {
			let tx = match intent {
				Intent::SystemRemark(remark) => {
					let remark_call = SystemCall::remark { remark: remark.to_vec() };
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
							sender_identity.hash().to_subxt_type(),
							RuntimeCall::System(remark_call),
							task.auth_type.to_subxt_type(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
				},
				Intent::TransferNative(transfer) => {
					let transfer_call = BalancesCall::transfer_allow_death {
						dest: transfer.to.to_subxt_type().into(),
						value: transfer.value,
					};
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
							sender_identity.hash().to_subxt_type(),
							RuntimeCall::Balances(transfer_call),
							task.auth_type.to_subxt_type(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
				},
				Intent::CallEthereum(_)
				| Intent::TransferEthereum(_)
				| Intent::TransferSolana(_) => {
					let request_intent_call =
						OmniAccountCall::request_intent { intent: intent.to_subxt_type() };
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
							sender_identity.hash().to_subxt_type(),
							RuntimeCall::OmniAccount(request_intent_call),
							task.auth_type.to_subxt_type(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
				},
			};
			(task.response_sender, tx)
		},
		NativeCall::create_account_store(sender_identity) => {
			let sender_identity_bytes = sender_identity.encode();
			let create_account_store_call = parentchain_api_interface::tx()
				.omni_account()
				.create_account_store(Decode::decode(&mut &sender_identity_bytes[..]).unwrap());
			let tx = ctx.transaction_signer.sign(create_account_store_call).await;
			(task.response_sender, tx)
		},
		NativeCall::add_account(
			sender_identity,
			identity,
			validation_data,
			public_account,
			permissions,
		) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender_identity.hash()) else {
				let response = NativeCallResponse::Err(NativeCallError::UnauthorizedSender);
				if task.response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Ok(nonce) = rpc_client.get_account_nonce(&omni_account).await else {
				log::error!("Failed to get account nonce");
				let response = NativeCallResponse::Err(NativeCallError::InternalError);
				if task.response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let verification_message = get_verification_message(&sender_identity, &identity, nonce);

			let validation_result = match validation_data {
				ValidationData::Web2(web2_validation_data) => {
					if !identity.is_web2() {
						Err(NativeCallError::InvalidMemberIdentity)
					} else {
						tokio::task::spawn_blocking({
							let identity = identity.clone();
							let storage_db = ctx.storage_db.clone();
							move || {
								web2::verify_identity(
									&identity,
									&verification_message,
									&web2_validation_data,
									storage_db,
								)
							}
						})
						.await
						.map_err(|e| {
							log::error!("Failed to verify identity: {:?}", e);
							NativeCallError::InternalError
						})
						.and_then(|result| {
							result.map_err(|_| NativeCallError::ValidationDataVerificationFailed)
						})
					}
				},
				ValidationData::Web3(web3_validation_data) => {
					if !identity.is_web3() {
						Err(NativeCallError::InvalidMemberIdentity)
					} else {
						tokio::task::spawn_blocking({
							let identity = identity.clone();
							move || {
								web3::verify_identity(
									&identity,
									&verification_message,
									&web3_validation_data,
								)
							}
						})
						.await
						.map_err(|e| {
							log::error!("Failed to verify identity: {:?}", e);
							NativeCallError::InternalError
						})
						.and_then(|result| {
							result.map_err(|_| NativeCallError::ValidationDataVerificationFailed)
						})
					}
				},
			};
			if let Err(e) = validation_result {
				let response = NativeCallResponse::Err(e);
				if task.response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}
			let member_account = match public_account {
				true => MemberAccount::Public(identity),
				false => MemberAccount::Private(
					aes_encrypt_default(&ctx.aes256_key, &identity.encode()).encode(),
					identity.hash(),
				),
			};
			let add_account_call = OmniAccountCall::add_account {
				member_account: member_account.to_subxt_type(),
				permissions: permissions.map(|p| p.to_subxt_type()),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(add_account_call),
					task.auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
			(task.response_sender, tx)
		},
		NativeCall::remove_accounts(sender_identity, identities) => {
			let remove_accounts = OmniAccountCall::remove_accounts {
				member_account_hashes: identities
					.iter()
					.map(|i| i.hash().to_subxt_type())
					.collect(),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(remove_accounts),
					task.auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
			(task.response_sender, tx)
		},
		NativeCall::publicize_account(sender_identity, identity) => {
			let publicize_account_call =
				OmniAccountCall::publicize_account { member_account: identity.to_subxt_type() };
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(publicize_account_call),
					task.auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
			(task.response_sender, tx)
		},
		NativeCall::set_permissions(sender_identity, identity, permissions) => {
			let set_permissions_call = OmniAccountCall::set_permissions {
				member_account_hash: identity.hash().to_subxt_type(),
				permissions: permissions.to_subxt_type(),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender_identity.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(set_permissions_call),
					task.auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
			(task.response_sender, tx)
		},
		_ => {
			let response = NativeCallResponse::Err(NativeCallError::UnexpectedCall(format!(
				"Unexpected call: {:?}",
				task.call
			)));
			if task.response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
	};
	let report = match rpc_client.submit_and_watch_tx_until(&tx, XtStatus::Finalized).await {
		Ok(report) => report,
		Err(e) => {
			log::error!("Failed to submit and watch tx: {:?}", e);
			let response = NativeCallResponse::Err(NativeCallError::InternalError);
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
	};
	let response = NativeCallResponse::Ok(NativeCallOk::ExtrinsicReport {
		extrinsic_hash: report.extrinsic_hash,
		block_hash: report.block_hash,
		status: report.status,
	});
	if response_sender.send(response.encode()).is_err() {
		log::error!("Failed to send response");
	}
}
