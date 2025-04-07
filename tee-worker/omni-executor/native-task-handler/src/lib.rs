mod aes256_key_store;
mod types;

use chrono::{Days, Utc};
use executor_core::{
	intent_executor::IntentExecutor,
	native_task::{NativeTask, NativeTaskWrapper},
};
use executor_crypto::{
	aes256::{aes_decrypt, aes_encrypt_default, Aes256Key},
	jwt,
};
use executor_primitives::{
	utils::hex::ToHexPrefixed, Identity, Intent, MemberAccount, OmniAccountAuthType, ValidationData,
};
use executor_storage::{MemberOmniAccountStorage, PumpxJwtStorage, Storage, StorageDB};
use heima_authentication::auth_token::*;
use heima_identity_verification::{get_verification_message, web2, web3};
use intent_core::IntentIdStore;
use parentchain_api_interface::runtime_types::{
	frame_system::pallet::Call as SystemCall,
	pallet_balances::pallet::Call as BalancesCall,
	pallet_omni_account::pallet::{Call as OmniAccountCall, IntentCompletedDetail},
	paseo_runtime::RuntimeCall,
};
use parentchain_rpc_client::{
	metadata::{Metadata, SubxtMetadataProvider},
	AccountId32, CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient,
	SubxtClientFactory, ToSubxtType, XtStatus,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TransactionSigner};
use parity_scale_codec::{Decode, Encode};
use pumpx::{signer_client::SignerClient, PumpxApi};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::{mpsc, oneshot};

pub use aes256_key_store::Aes256KeyStore;
pub use types::{NativeTaskError, NativeTaskOk, PumpxApiError};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeTaskChannelType = (NativeTaskWrapper<NativeTask>, ResponseSender);
pub type NativeTaskSender = mpsc::Sender<NativeTaskChannelType>;

pub type NativeTaskResponse = Result<NativeTaskOk, NativeTaskError>;

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
	pub pumpx_api: Arc<PumpxApi>,
	pumpx_signer_client: Arc<SignerClient>,
	intent_id_store: Arc<Box<dyn IntentIdStore>>,
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
		pumpx_api: Arc<PumpxApi>,
		pumpx_signer_client: Arc<SignerClient>,
		intent_id_store: Arc<Box<dyn IntentIdStore>>,
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
			pumpx_api,
			pumpx_signer_client,
			intent_id_store,
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

	let auth_type: Option<OmniAccountAuthType> = wrapper.auth.map(|t| t.into());

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
			let expires_at = Utc::now()
				.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
				.expect("Failed to calculate expiration")
				.timestamp();
			let auth_options = AuthOptions { expires_at };
			let claims = match sender {
				Identity::Email(ref identity_string) => {
					let Ok(email) = std::str::from_utf8(identity_string.inner_ref()) else {
						let response =
							NativeTaskResponse::Err(NativeTaskError::InvalidMemberIdentity);
						if response_sender.send(response.encode()).is_err() {
							log::error!("Failed to send response");
						}
						return;
					};
					AuthTokenClaims::new(
						email.to_string(),
						AUTH_TOKEN_ID_TYPE.to_string(),
						auth_options,
					)
				},
				_ => AuthTokenClaims::new(
					sender.hash().to_string(),
					AUTH_TOKEN_ID_TYPE.to_string(),
					auth_options,
				),
			};
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
		NativeTask::RequestIntent(sender, intent_id, intent) => {
			if intent_id != ctx.intent_id_store.get(&sender.to_omni_account()).await.unwrap() + 1 {
				ctx.intent_id_store.update(sender.to_omni_account(), intent_id).await.unwrap()
			} else {
				let response = NativeTaskResponse::Err(NativeTaskError::IntentNonceMismatch);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Intent id different than expected");
				}
				return;
			}
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
					auth_type.clone().map(|t| t.to_subxt_type()),
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

			let mut execution_result = IntentCompletedDetail::Success;

			let tx = match intent {
				Intent::SystemRemark(remark) => {
					let remark_call = SystemCall::remark { remark: remark.to_vec() };
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_signed(
							sender.hash().to_subxt_type(),
							RuntimeCall::System(remark_call),
							auth_type.map(|t| t.to_subxt_type()),
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
							auth_type.map(|t| t.to_subxt_type()),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call, Some(nonce)).await
				},
				Intent::CallEthereum(_) | Intent::TransferEthereum(_) => {
					if let Err(e) = ctx
						.ethereum_intent_executor
						.execute(&omni_account, intent_id, intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentCompletedDetail::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_completed(
							omni_account.to_subxt_type(),
							0, // TODO
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
				Intent::TransferSolana(_) => {
					if let Err(e) = ctx
						.solana_intent_executor
						.execute(&omni_account, intent_id, intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentCompletedDetail::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_completed(
							omni_account.to_subxt_type(),
							0, // TODO
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call, Some(nonce)).await
				},
				Intent::Swap(..) => {
					if let Err(e) = ctx
						.cross_chain_intent_executor
						.execute(&omni_account, intent_id, intent.clone())
						.await
					{
						log::error!("Error executing intent: {:?}", e);
						execution_result = IntentCompletedDetail::Failure;
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_completed(
							omni_account.to_subxt_type(),
							0, // TODO
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
					auth_type.map(|t| t.to_subxt_type()),
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
					auth_type.map(|t| t.to_subxt_type()),
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
					auth_type.map(|t| t.to_subxt_type()),
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
					auth_type.map(|t| t.to_subxt_type()),
				);
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call, None).await;
			(response_sender, tx)
		},
		NativeTask::PumpxRequestJwt(sender, invite_code, maybe_google_code, language) => {
			let email = match sender {
				Identity::Email(ref identity_string) => {
					let Ok(email) = std::str::from_utf8(identity_string.inner_ref()) else {
						let response =
							NativeTaskResponse::Err(NativeTaskError::InvalidMemberIdentity);
						if response_sender.send(response.encode()).is_err() {
							log::error!("Failed to send response");
						}
						return;
					};
					email.to_string()
				},
				_ => {
					let response =
						NativeTaskResponse::Err(NativeTaskError::UnsupportedIdentityType);
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
					return;
				},
			};
			let expires_at = Utc::now()
				.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
				.expect("Failed to calculate expiration")
				.timestamp();
			let auth_options = AuthOptions { expires_at };

			let access_token_claims = AuthTokenClaims::new(
				sender.to_omni_account().to_hex(),
				AUTH_TOKEN_ACCESS_TYPE.to_string(),
				auth_options.clone(),
			);
			let Ok(access_token) = jwt::create(&access_token_claims, &ctx.jwt_rsa_private_key)
			else {
				let response = NativeTaskResponse::Err(NativeTaskError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};

			let storage = PumpxJwtStorage::new(ctx.storage_db.clone());
			if storage
				.insert((sender.to_omni_account(), AUTH_TOKEN_ACCESS_TYPE), access_token.clone())
				.is_err()
			{
				log::error!(
					"Failed to insert pumpx_{}_jwt_token into storage",
					AUTH_TOKEN_ACCESS_TYPE
				);
			};

			let Ok(user_connect_response) = ctx
				.pumpx_api
				.connect_user(
					&access_token,
					email.clone(),
					invite_code,
					maybe_google_code,
					language,
				)
				.await
			else {
				log::error!("Failed to connect user");
				let response = NativeTaskResponse::Err(NativeTaskError::PumpxApiError(
					PumpxApiError::UserConnectionFailed,
				));
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let id_token_claims = AuthTokenClaims::new(
				sender.to_omni_account().to_hex(),
				AUTH_TOKEN_ID_TYPE.to_string(),
				auth_options,
			);
			let Ok(id_token) = jwt::create(&id_token_claims, &ctx.jwt_rsa_private_key) else {
				let response = NativeTaskResponse::Err(NativeTaskError::AuthTokenCreationFailed);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};

			if storage
				.insert((sender.to_omni_account(), AUTH_TOKEN_ID_TYPE), id_token.clone())
				.is_err()
			{
				log::error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ID_TYPE);
			};

			let response = NativeTaskResponse::Ok(NativeTaskOk::PumpxJwt {
				access_token,
				id_token,
				user_connect_response,
			});

			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
		NativeTask::PumpxExportWallet(
			sender,
			maybe_google_code,
			pumpx_wallet_chain,
			pumpx_wallet_index,
			expected_wallet_address,
		) => {
			if let Some(ref google_code) = maybe_google_code {
				let storage = PumpxJwtStorage::new(ctx.storage_db.clone());
				let Some(access_token) =
					storage.get(&(sender.to_omni_account(), AUTH_TOKEN_ACCESS_TYPE))
				else {
					log::error!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE);
					let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
					return;
				};

				let verify_result = ctx
					.pumpx_api
					.verify_google_code(&access_token, google_code.to_string(), None)
					.await;
				let verify_success = match verify_result {
					Ok(response) => response.data.result,
					Err(_) => {
						log::error!("Google code verification request failed");
						false
					},
				};
				if !verify_success {
					let response = NativeTaskResponse::Err(NativeTaskError::PumpxApiError(
						PumpxApiError::GoogleCodeVerificationFailed,
					));
					if response_sender.send(response.encode()).is_err() {
						log::error!("Failed to send response");
					}
					return;
				}
			}

			let Ok(mut wallet) = ctx
				.pumpx_signer_client
				.export_wallet(
					pumpx_wallet_chain.into(),
					pumpx_wallet_index,
					sender.to_omni_account().into(),
					// TODO: theoretically we could pass the aes_key from initial RPC to signer, so that
					//       we don't have to do double encryption/decryption
					ctx.aes256_key.to_vec(),
					expected_wallet_address,
				)
				.await
			else {
				log::error!("Failed to export wallet from pumpx-signer");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Some(decrypted_wallet) = aes_decrypt(&ctx.aes256_key, &mut wallet) else {
				log::error!("No wallet after decryption");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let response =
				NativeTaskResponse::Ok(NativeTaskOk::PumpxExportWallet(decrypted_wallet));
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
