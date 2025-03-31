mod aes256_key_store;
mod types;

use executor_core::{
	intent_executor::IntentExecutor,
	native_task::{NativeTask, NativeTaskWrapper},
};
use executor_crypto::{
	aes256::{aes_encrypt_default, Aes256Key},
	jwt,
};
use executor_primitives::{Intent, MemberAccount, OmniAccountAuthType, ValidationData};
use executor_storage::{MemberOmniAccountStorage, Storage, StorageDB};
use heima_authentication::auth_token::*;
use heima_identity_verification::{get_verification_message, web2, web3};
use parentchain_api_interface::runtime_types::{
	frame_system::pallet::Call as SystemCall,
	pallet_balances::pallet::Call as BalancesCall,
	pallet_omni_account::pallet::{Call as OmniAccountCall, IntentExecutionResult},
	paseo_runtime::RuntimeCall,
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
use types::{NativeTaskError, NativeTaskOk};

pub use aes256_key_store::Aes256KeyStore;

pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeTaskChannelType = (NativeTaskWrapper<NativeTask>, ResponseSender);
pub type NativeTaskSender = mpsc::Sender<NativeTaskChannelType>;

type NativeTaskResponse = Result<NativeTaskOk, NativeTaskError>;

pub type ParentchainTxSigner = TransactionSigner<
	SubstrateKeyStore,
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

pub struct TaskHandlerContext<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
	EthereumIntentExecutor: IntentExecutor,
	SolanaIntentExecutor: IntentExecutor,
	CrossChainIntentExecutor: IntentExecutor,
> {
	pub parentchain_rpc_client_factory: Arc<RpcClientFactory>,
	pub storage_db: Arc<StorageDB>,
	pub jwt_rsa_private_key: Vec<u8>,
	pub aes256_key: Aes256Key,
	pub transaction_signer: Arc<ParentchainTxSigner>,
	pub ethereum_intent_executor: Arc<EthereumIntentExecutor>,
	pub solana_intent_executor: Arc<SolanaIntentExecutor>,
	pub cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
	phantom_header: PhantomData<Header>,
	phantom_rpc_client: PhantomData<RpcClient>,
}

impl<
		Header,
		RpcClient: SubstrateRpcClient<Header>,
		RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
		EthereumIntentExecutor: IntentExecutor,
		SolanaIntentExecutor: IntentExecutor,
		CrossChainIntentExecutor: IntentExecutor,
	>
	TaskHandlerContext<
		Header,
		RpcClient,
		RpcClientFactory,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		CrossChainIntentExecutor,
	>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		parentchain_rpc_client_factory: Arc<RpcClientFactory>,
		transaction_signer: Arc<ParentchainTxSigner>,
		storage_db: Arc<StorageDB>,
		jwt_rsa_private_key: Vec<u8>,
		aes256_key: Aes256Key,
		ethereum_intent_executor: Arc<EthereumIntentExecutor>,
		solana_intent_executor: Arc<SolanaIntentExecutor>,
		cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
	) -> Self {
		Self {
			parentchain_rpc_client_factory,
			transaction_signer,
			storage_db,
			jwt_rsa_private_key,
			aes256_key,
			ethereum_intent_executor,
			solana_intent_executor,
			cross_chain_intent_executor,
			phantom_header: PhantomData,
			phantom_rpc_client: PhantomData,
		}
	}
}

pub async fn run_native_task_handler<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	buffer: usize,
	ctx: Arc<
		TaskHandlerContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) -> NativeTaskSender {
	let (sender, mut receiver) = mpsc::channel::<NativeTaskChannelType>(buffer);

	tokio::spawn(async move {
		while let Some((wrapper, sender)) = receiver.recv().await {
			handle_native_task(ctx.clone(), wrapper, sender).await;
		}
	});

	sender
}

async fn handle_native_task<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<
		TaskHandlerContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
	wrapper: NativeTaskWrapper<NativeTask>,
	response_sender: ResponseSender,
) {
	let Ok(mut rpc_client) = ctx.parentchain_rpc_client_factory.new_client().await else {
		log::error!("Failed to create rpc client");
		let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
		if response_sender.send(response.encode()).is_err() {
			log::error!("Failed to send response");
		}
		return;
	};

	let auth_type: Option<OmniAccountAuthType> = wrapper.auth.into();

	let (response_sender, tx) = match wrapper.task {
		NativeTask::RequestAuthToken(sender) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender.hash()) else {
				let response = NativeTaskResponse::Err(NativeTaskError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Ok(current_block) = rpc_client.get_last_finalized_block_num().await else {
				log::error!("Failed to get last finalized block number");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let auth_options = AuthOptions { expires_at: current_block + AUTH_TOKEN_EXPIRATION };
			let claims = AuthTokenClaims::new(
				sender.hash().to_string(),
				AUTH_TOKEN_SESSION_TYPE.to_string(),
				auth_options,
			);
			let Ok(token) = jwt::create(&claims, &ctx.jwt_rsa_private_key) else {
				let response = NativeTaskResponse::Err(NativeTaskError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let auth_token_requested_call = parentchain_api_interface::tx()
				.omni_account()
				.auth_token_requested(AccountId32(omni_account.into()), claims.exp);

			// Without increase nonce, all requests after request_auth_token will failure with below error.
			// Could not submit tx: Rpc(ClientError(Call(ErrorObject { code: ServerError(1014), message: "Priority is too low: (2564 vs 2564)",
			// data: Some(RawValue("The transaction has too low priority to replace another transaction already in the pool.")) })))
			let signer_account_id = ctx.transaction_signer.get_signer_account_id();
			let nonce = match rpc_client.get_account_nonce(&signer_account_id).await {
				Ok(n) => n,
				Err(e) => {
					log::error!("Failed to get account nonce: {:?}", e);
					let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
					return;
				},
			};
			// Increment nonce for the next transaction
			let tx = ctx.transaction_signer.sign(auth_token_requested_call, Some(nonce + 1)).await;

			if rpc_client.submit_tx(&tx).await.is_err() {
				log::error!("Failed to submit tx");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}

			let response = NativeTaskResponse::Ok(NativeTaskOk::AuthToken(token));

			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
		NativeTask::RequestIntent(sender, intent) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender.hash()) else {
				let response = NativeTaskResponse::Err(NativeTaskError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};

			let call = OmniAccountCall::request_intent { intent: intent.to_subxt_type() };
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(call),
					auth_type.to_subxt_type(),
				);

			let signer_account_id = ctx.transaction_signer.get_signer_account_id();
			let mut nonce = match rpc_client.get_account_nonce(&signer_account_id).await {
				Ok(n) => n,
				Err(e) => {
					log::error!("Failed to get account nonce: {:?}", e);
					let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
					return;
				},
			};

			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, Some(nonce)).await;
			if rpc_client.submit_tx(&tx).await.is_err() {
				log::error!("Failed to submit RequestIntent tx");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			}
			// Increment nonce for the next transaction
			nonce += 1;

			let mut execution_result = IntentExecutionResult::Success;

			let tx = match intent {
				Intent::SystemRemark(remark) => {
					let remark_call = SystemCall::remark { remark: remark.to_vec() };
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_signed(
							sender.hash().to_subxt_type(),
							RuntimeCall::System(remark_call),
							auth_type.to_subxt_type(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call, Some(nonce)).await
				},
				Intent::TransferNative(transfer) => {
					let transfer_call = BalancesCall::transfer_allow_death {
						dest: transfer.to.to_subxt_type().into(),
						value: transfer.value,
					};
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_signed(
							sender.hash().to_subxt_type(),
							RuntimeCall::Balances(transfer_call),
							auth_type.to_subxt_type(),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call, Some(nonce)).await
				},
				Intent::CallEthereum(_) | Intent::TransferEthereum(_) => {
					if let Err(e) = ctx
						.ethereum_intent_executor
						.execute(omni_account.as_ref(), intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentExecutionResult::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_executed(
							omni_account.to_subxt_type(),
							intent.to_subxt_type(),
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
				Intent::TransferSolana(_) => {
					if let Err(e) = ctx
						.solana_intent_executor
						.execute(omni_account.as_ref(), intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentExecutionResult::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_executed(
							omni_account.to_subxt_type(),
							intent.to_subxt_type(),
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
				Intent::CrossChainSwap(_) => {
					if let Err(e) = ctx
						.cross_chain_intent_executor
						.execute(omni_account.as_ref(), intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentExecutionResult::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_executed(
							omni_account.to_subxt_type(),
							intent.to_subxt_type(),
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
			};

			(response_sender, tx)
		},
		NativeTask::CreateAccountStore(sender) => {
			let sender_bytes = sender.encode();
			let create_account_store_call = parentchain_api_interface::tx()
				.omni_account()
				.create_account_store(Decode::decode(&mut &sender_bytes[..]).unwrap());
			let tx = ctx.transaction_signer.sign(create_account_store_call, None).await;
			(response_sender, tx)
		},
		NativeTask::AddAccount(sender, identity, validation_data, public_account, permissions) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender.hash()) else {
				let response = NativeTaskResponse::Err(NativeTaskError::UnauthorizedSender);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Ok(nonce) = rpc_client.get_account_nonce(&omni_account).await else {
				log::error!("Failed to get account nonce");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let verification_message = get_verification_message(&sender, &identity, nonce);

			let validation_result = match validation_data {
				ValidationData::Web2(web2_validation_data) => {
					if !identity.is_web2() {
						Err(NativeTaskError::InvalidMemberIdentity)
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
							NativeTaskError::InternalError
						})
						.and_then(|result| {
							result.map_err(|_| NativeTaskError::ValidationDataVerificationFailed)
						})
					}
				},
				ValidationData::Web3(web3_validation_data) => {
					if !identity.is_web3() {
						Err(NativeTaskError::InvalidMemberIdentity)
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
							NativeTaskError::InternalError
						})
						.and_then(|result| {
							result.map_err(|_| NativeTaskError::ValidationDataVerificationFailed)
						})
					}
				},
			};
			if let Err(e) = validation_result {
				let response = NativeTaskResponse::Err(e);
				if response_sender.send(response.encode()).is_err() {
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
			let call = OmniAccountCall::add_account {
				member_account: member_account.to_subxt_type(),
				permissions: permissions.map(|p| p.to_subxt_type()),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(call),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeTask::RemoveAccounts(sender, identities) => {
			let call = OmniAccountCall::remove_accounts {
				member_account_hashes: identities
					.iter()
					.map(|i| i.hash().to_subxt_type())
					.collect(),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(call),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeTask::PublicizeAccount(sender, identity) => {
			let call =
				OmniAccountCall::publicize_account { member_account: identity.to_subxt_type() };
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(call),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeTask::SetPermissions(sender, identity, permissions) => {
			let call = OmniAccountCall::set_permissions {
				member_account_hash: identity.hash().to_subxt_type(),
				permissions: permissions.to_subxt_type(),
			};
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(call),
					auth_type.to_subxt_type(),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeTask::PumpxRequestJwt(sender) => {
			let Ok(current_block) = rpc_client.get_last_finalized_block_num().await else {
				log::error!("Failed to get last finalized block number");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let expires_at = current_block + AUTH_TOKEN_EXPIRATION;
			let auth_options = AuthOptions { expires_at };
			let session_claims = AuthTokenClaims::new(
				sender.hash().to_string(),
				AUTH_TOKEN_SESSION_TYPE.to_string(),
				auth_options.clone(),
			);
			let Ok(session_token) = jwt::create(&session_claims, &ctx.jwt_rsa_private_key) else {
				let response = NativeTaskResponse::Err(NativeTaskError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let trade_claims = AuthTokenClaims::new(
				sender.hash().to_string(),
				AUTH_TOKEN_TRADE_TYPE.to_string(),
				auth_options,
			);
			let Ok(trade_token) = jwt::create(&trade_claims, &ctx.jwt_rsa_private_key) else {
				let response = NativeTaskResponse::Err(NativeTaskError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};

			let response =
				NativeTaskResponse::Ok(NativeTaskOk::PumpxJwt { session_token, trade_token });

			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
	};
	let report = match rpc_client.submit_and_watch_tx_until(&tx, XtStatus::Finalized).await {
		Ok(report) => report,
		Err(e) => {
			log::error!("Failed to submit and watch tx: {:?}", e);
			let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
	};
	let response = NativeTaskResponse::Ok(NativeTaskOk::ExtrinsicReport {
		extrinsic_hash: report.extrinsic_hash,
		block_hash: report.block_hash,
		status: report.status,
	});

	if response_sender.send(response.encode()).is_err() {
		log::error!("Failed to send response");
	}
}
