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
use parentchain_signer::TxSigner;
use parity_scale_codec::{Decode, Encode};
use pumpx::{
	signer_client::{ChainType, SignerClient},
	PumpxApi,
};
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::{mpsc, oneshot};

pub use aes256_key_store::Aes256KeyStore;
pub use types::{NativeTaskError, NativeTaskOk, PumpxApiError, PumpxSignerError};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;
pub type NativeTaskChannelType = (NativeTaskWrapper<NativeTask>, ResponseSender);
pub type NativeTaskSender = mpsc::Sender<NativeTaskChannelType>;

pub type NativeTaskResponse = Result<NativeTaskOk, NativeTaskError>;

pub type ParentchainTxSigner = TxSigner<
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
		send_error(
			"Failed to create rpc client".to_string(),
			response_sender,
			NativeTaskError::InternalError,
		);
		return;
	};

	let auth_type: Option<OmniAccountAuthType> = wrapper.auth.map(|t| t.into());

	let (response_sender, tx) = match wrapper.task {
		NativeTask::RequestAuthToken(sender) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender.hash()) else {
				send_error(
					"No omni account found".to_string(),
					response_sender,
					NativeTaskError::UnauthorizedSender,
				);
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
						send_error(
							"Invalid email identity".to_string(),
							response_sender,
							NativeTaskError::InvalidMemberIdentity,
						);
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
				send_error(
					"Failed to create auth token".to_string(),
					response_sender,
					NativeTaskError::AuthTokenCreationFailed,
				);
				return;
			};
			let auth_token_requested_call = parentchain_api_interface::tx()
				.omni_account()
				.auth_token_requested(AccountId32(omni_account.into()), claims.exp);

			let tx = ctx.transaction_signer.sign(auth_token_requested_call).await;

			if rpc_client.submit_tx(&tx).await.is_err() {
				send_error(
					"Failed to submit tx".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				ctx.transaction_signer.update_nonce().await;
				return;
			}

			send_ok(response_sender, NativeTaskOk::AuthToken(token));
			return;
		},
		NativeTask::RequestIntent(sender, intent_id, intent) => {
			let Ok(stored_intent_id) = ctx.intent_id_store.get(&sender.to_omni_account()).await
			else {
				send_error(
					"Failed to read intent from store".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};
			if intent_id == stored_intent_id + 1 {
				if ctx.intent_id_store.update(sender.to_omni_account(), intent_id).await.is_err() {
					send_error(
						"Failed to save intent id".to_string(),
						response_sender,
						NativeTaskError::InternalError,
					);
					return;
				}
			} else {
				send_error(
					format!(
						"Intent id different than expected, expected: {:?}, got: {:?}",
						stored_intent_id + 1,
						intent_id
					),
					response_sender,
					NativeTaskError::IntentNonceMismatch,
				);
				return;
			}

			let call = OmniAccountCall::request_intent { intent: intent.to_subxt_type() };
			let dispatch_as_omni_account_call =
				parentchain_api_interface::tx().omni_account().dispatch_as_omni_account(
					sender.hash().to_subxt_type(),
					RuntimeCall::OmniAccount(call),
					auth_type.clone().map(|t| t.to_subxt_type()),
				);

			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
			if rpc_client.submit_tx(&tx).await.is_err() {
				send_error(
					"Failed to submit RequestIntent tx".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				ctx.transaction_signer.update_nonce().await;
				return;
			}

			let mut execution_result = IntentCompletedDetail::Success;
			let omni_account = sender.to_omni_account();

			let tx = match intent {
				Intent::SystemRemark(remark) => {
					let remark_call = SystemCall::remark { remark: remark.to_vec() };
					let dispatch_as_omni_account_call =
						parentchain_api_interface::tx().omni_account().dispatch_as_signed(
							sender.hash().to_subxt_type(),
							RuntimeCall::System(remark_call),
							auth_type.map(|t| t.to_subxt_type()),
						);
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
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
					ctx.transaction_signer.sign(dispatch_as_omni_account_call).await
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
							intent_id,
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call).await
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
							intent_id,
							execution_result,
						);
					ctx.transaction_signer.sign(intent_executed_call).await
				},
				Intent::Swap(..) => {
					let (execution_result, response) = match ctx
						.cross_chain_intent_executor
						.execute(&omni_account, intent_id, intent.clone())
						.await
					{
						Ok(response) => (IntentCompletedDetail::Success, response),
						Err(e) => {
							log::error!("Error executing intent: {:?}", e);
							(IntentCompletedDetail::Failure, None)
						},
					};
					if let Some(response) = response {
						let response =
							NativeTaskResponse::Ok(NativeTaskOk::IntentSwapResponse(response));
						if response_sender.send(response.encode()).is_err() {
							log::error!("Failed to send response");
						}
					}
					let intent_executed_call =
						parentchain_api_interface::tx().omni_account().intent_completed(
							omni_account.to_subxt_type(),
							intent_id,
							execution_result,
						);
					let tx = ctx.transaction_signer.sign(intent_executed_call).await;
					if rpc_client.submit_tx(&tx).await.is_err() {
						log::error!("Failed to submit RequestIntent tx");
						ctx.transaction_signer.update_nonce().await;
					}
					return;
				},
			};

			(response_sender, tx)
		},
		NativeTask::CreateAccountStore(sender) => {
			let sender_bytes = sender.encode();
			let create_account_store_call = parentchain_api_interface::tx()
				.omni_account()
				.create_account_store(Decode::decode(&mut &sender_bytes[..]).unwrap());
			let tx = ctx.transaction_signer.sign(create_account_store_call).await;
			(response_sender, tx)
		},
		NativeTask::AddAccount(sender, identity, validation_data, public_account, permissions) => {
			let omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
			let Some(omni_account) = omni_account_storage.get(&sender.hash()) else {
				send_error(
					"No omni account found".to_string(),
					response_sender,
					NativeTaskError::UnauthorizedSender,
				);
				return;
			};
			let Ok(nonce) = rpc_client.get_account_nonce(&omni_account).await else {
				send_error(
					"Failed to get account nonce".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
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
				send_error("Validation failed".to_string(), response_sender, e);
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
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
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
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
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
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
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
			let tx = ctx.transaction_signer.sign(dispatch_as_omni_account_call).await;
			(response_sender, tx)
		},
		NativeTask::PumpxRequestJwt(sender, invite_code, google_code, language) => {
			let email = match sender {
				Identity::Email(ref identity_string) => {
					let Ok(email) = std::str::from_utf8(identity_string.inner_ref()) else {
						send_error(
							"Invalid email identity".to_string(),
							response_sender,
							NativeTaskError::InvalidMemberIdentity,
						);
						return;
					};
					email.to_string()
				},
				_ => {
					send_error(
						"Unsupported identity type".to_string(),
						response_sender,
						NativeTaskError::UnsupportedIdentityType,
					);
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
				send_error(
					"Failed to create access token".to_string(),
					response_sender,
					NativeTaskError::AuthTokenCreationFailed,
				);
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

			let Ok(backend_response) = ctx
				.pumpx_api
				.user_connect(&access_token, email.clone(), invite_code, google_code, language)
				.await
			else {
				send_error(
					"Failed to connect user".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::UserConnectionFailed),
				);
				return;
			};
			let id_token_claims = AuthTokenClaims::new(
				sender.to_omni_account().to_hex(),
				AUTH_TOKEN_ID_TYPE.to_string(),
				auth_options,
			);
			let Ok(id_token) = jwt::create(&id_token_claims, &ctx.jwt_rsa_private_key) else {
				send_error(
					"Failed to create id token".to_string(),
					response_sender,
					NativeTaskError::AuthTokenCreationFailed,
				);
				return;
			};

			if storage
				.insert((sender.to_omni_account(), AUTH_TOKEN_ID_TYPE), id_token.clone())
				.is_err()
			{
				log::error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ID_TYPE);
			};

			send_ok(
				response_sender,
				NativeTaskOk::PumpxRequestJwt { access_token, id_token, backend_response },
			);
			return;
		},
		NativeTask::PumpxExportWallet(
			sender,
			google_code,
			pumpx_chain_id,
			pumpx_wallet_index,
			expected_wallet_address,
		) => {
			let storage = PumpxJwtStorage::new(ctx.storage_db.clone());
			let Some(access_token) =
				storage.get(&(sender.to_omni_account(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				send_error(
					format!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			let verify_result =
				ctx.pumpx_api.verify_google_code(&access_token, google_code, None).await;
			let verify_success = match verify_result {
				Ok(response) => response.data.result,
				Err(_) => {
					log::error!("Google code verification request failed");
					false
				},
			};
			if !verify_success {
				send_error(
					"Google code verification failed".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::GoogleCodeVerificationFailed),
				);
				return;
			}

			let Some(chain) = ChainType::from_pumpx_chain_id(pumpx_chain_id) else {
				send_error(
					format!("Failed to map pumpx chain_id {}", pumpx_chain_id),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			let Ok(mut wallet) = ctx
				.pumpx_signer_client
				.export_wallet(
					chain,
					pumpx_wallet_index,
					sender.to_omni_account().into(),
					// TODO: theoretically we could pass the aes_key from initial RPC to signer, so that
					//       we don't have to do double encryption/decryption
					ctx.aes256_key.to_vec(),
					expected_wallet_address,
				)
				.await
			else {
				send_error(
					"Failed to export wallet from pumpx-signer".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};
			let Some(decrypted_wallet) = aes_decrypt(&ctx.aes256_key, &mut wallet) else {
				send_error(
					"No wallet after decryption".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};
			send_ok(response_sender, NativeTaskOk::PumpxExportWallet(decrypted_wallet));
			return;
		},
		NativeTask::PumpxAddWallet(sender) => {
			let storage = PumpxJwtStorage::new(ctx.storage_db.clone());
			let Some(access_token) =
				storage.get(&(sender.to_omni_account(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				send_error(
					format!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			// Call Pumpx API to add wallet
			let Ok(backend_response) = ctx.pumpx_api.add_wallet(&access_token, None).await else {
				send_error(
					"Failed to add wallet through Pumpx API".to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::AddWalletFailed),
				);
				return;
			};

			send_ok(response_sender, NativeTaskOk::PumpxAddWallet(backend_response));
			return;
		},
		NativeTask::PumpxSignLimitOrder(sender, chain_id, wallet_index, unsigned_tx) => {
			let omni_account = sender.to_omni_account();
			let Some(chain) = ChainType::from_pumpx_chain_id(chain_id) else {
				log::error!("Failed to map pumpx chain_id {}", chain_id);
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let Ok(signed_txs) = ctx
				.pumpx_signer_client
				.request_signatures(chain, wallet_index, omni_account.into(), unsigned_tx)
				.await
			else {
				log::error!("Failed to request signatures from pumpx-signer");
				let response = NativeTaskResponse::Err(NativeTaskError::InternalError);
				if response_sender.send(response.encode()).is_err() {
					log::error!("Failed to send response");
				}
				return;
			};
			let response = NativeTaskResponse::Ok(NativeTaskOk::PumpxSignLimitOrder(signed_txs));
			if response_sender.send(response.encode()).is_err() {
				log::error!("Failed to send response");
			}
			return;
		},
		NativeTask::PumpxTransferWidthdraw(
			sender,
			chain_id,
			wallet_index,
			recipient_address,
			token_ca,
			amount,
			google_code,
			language,
		) => {
			// 1. Verify we have a valid Pumpx "access" token for the user
			let storage = PumpxJwtStorage::new(ctx.storage_db.clone());
			let Some(access_token) =
				storage.get(&(sender.to_omni_account(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				send_error(
					"Failed to get access_token within NativeTask::PumpxTransferWidthdraw"
						.to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			// 2. Verify google code in every case
			let verify_result = ctx
				.pumpx_api
				.verify_google_code(&access_token, google_code, language.clone())
				.await;

			let verify_success = match verify_result {
				Ok(res) => res.data.result,
				Err(_) => {
					log::error!("Google code verification request failed");
					false
				},
			};

			if !verify_success {
				send_error(
					"Failed to verify google code within NativeTask::PumpxTransferWidthdraw"
						.to_string(),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::GoogleCodeVerificationFailed),
				);
				return;
			}

			// 3. Create an unsigned tx with Pumpx backend
			let create_transfer_res = match ctx
				.pumpx_api
				.create_transfer_unsigned_tx(
					&access_token,
					chain_id,
					wallet_index,
					&recipient_address,
					&token_ca,
					&amount,
					language.clone(),
				)
				.await
			{
				Ok(res) => res,
				Err(e) => {
					send_error(
						format!("Failed to create_transfer_unsigned_tx: {:?}", e),
						response_sender,
						NativeTaskError::PumpxApiError(
							PumpxApiError::CreateTransferUnsignedTxFailed,
						),
					);
					return;
				},
			};

			let transfer_id = create_transfer_res.data.transfer_id;
			let tx_data = create_transfer_res.data.tx_data;
			let tx_data = match tx_data {
				Some(data) => data,
				None => {
					send_error(
						"No tx_data in create_transfer_unsigned_tx response".to_string(),
						response_sender,
						NativeTaskError::PumpxApiError(
							PumpxApiError::CreateTransferUnsignedTxFailed,
						),
					);
					return;
				},
			};

			let Some(chain_type) = ChainType::from_pumpx_chain_id(chain_id) else {
				send_error(
					format!("Failed to map pumpx chain_id {}", chain_id),
					response_sender,
					NativeTaskError::InternalError,
				);
				return;
			};

			// 4. Use the pumpx_signer_client to sign all tx_data entries at once
			let mut messages_to_sign = Vec::new();
			for tx in tx_data {
				let tx_cleaned = tx.strip_prefix("0x").unwrap_or(&tx);
				let tx_bytes = match hex::decode(tx_cleaned) {
					Ok(bytes) => bytes,
					Err(e) => {
						send_error(
							format!("Failed to decode tx_data (hex): {:?}", e),
							response_sender,
							NativeTaskError::InternalError,
						);
						return;
					},
				};
				messages_to_sign.push(tx_bytes);
			}

			let signatures = match ctx
				.pumpx_signer_client
				.request_signatures(
					chain_type,
					wallet_index,
					sender.to_omni_account().into(),
					messages_to_sign,
				)
				.await
			{
				Ok(sigs) => sigs,
				Err(e) => {
					send_error(
						format!("Failed to sign transfer tx: {:?}", e),
						response_sender,
						NativeTaskError::PumpxSignerError(PumpxSignerError::RequestSignatureFailed),
					);
					return;
				},
			};

			let signed_tx_data: Vec<String> =
				signatures.into_iter().map(|sig| sig.to_hex()).collect();

			// 5. Send the signed tx to the Pumpx backend
			match ctx
				.pumpx_api
				.send_transfer_tx(&access_token, transfer_id, chain_id, signed_tx_data, language)
				.await
			{
				Ok(res) => {
					send_ok(response_sender, NativeTaskOk::PumpxTransferWithdraw(res));
					return;
				},
				Err(e) => {
					send_error(
						format!("Failed to send_transfer_tx: {:?}", e),
						response_sender,
						NativeTaskError::PumpxApiError(PumpxApiError::SendTransferTxFailed),
					);
					return;
				},
			};
		},
		NativeTask::PumpxNotifyLimitOrderResult(sender, intent_id, result, message) => {
			if result != "ok" && result != "nok" {
				send_error(
					format!("Invalid result value: {}. Must be 'ok' or 'nok'", result),
					response_sender,
					NativeTaskError::PumpxApiError(PumpxApiError::InvalidInput),
				);
				return;
			}

			let execution_result = match result.as_str() {
				"ok" => IntentCompletedDetail::Success,
				"nok" => IntentCompletedDetail::Failure,
				_ => unreachable!(), // Already validated above
			};

			if let Some(msg) = message {
				log::info!("Limit order result message for intent_id {}: {}", intent_id, msg);
			}

			let intent_executed_call =
				parentchain_api_interface::tx().omni_account().intent_completed(
					sender.to_omni_account().to_subxt_type(),
					intent_id,
					execution_result,
				);

			let tx = ctx.transaction_signer.sign(intent_executed_call).await;

			if rpc_client.submit_tx(&tx).await.is_err() {
				send_error(
					"Failed to submit tx".to_string(),
					response_sender,
					NativeTaskError::InternalError,
				);
				ctx.transaction_signer.update_nonce().await;
				return;
			}

			send_ok(response_sender, NativeTaskOk::PumpxNotifyLimitOrderResult);
			return;
		},
	};

	match rpc_client.submit_and_watch_tx_until(&tx, XtStatus::Finalized).await {
		Ok(report) => {
			send_ok(
				response_sender,
				NativeTaskOk::ExtrinsicReport {
					extrinsic_hash: report.extrinsic_hash,
					block_hash: report.block_hash,
					status: report.status,
				},
			);
		},
		Err(e) => {
			send_error(
				format!("Failed to submit and watch tx: {:?}", e),
				response_sender,
				NativeTaskError::InternalError,
			);
			ctx.transaction_signer.update_nonce().await;
		},
	};
}

fn send_response(sender: ResponseSender, response: NativeTaskResponse) {
	if sender.send(response.encode()).is_err() {
		log::error!("Failed to send response");
	}
}

fn send_error(err_msg: String, sender: ResponseSender, error: NativeTaskError) {
	log::error!("{}", err_msg);
	send_response(sender, NativeTaskResponse::Err(error));
}

fn send_ok(sender: ResponseSender, ok_res: NativeTaskOk) {
	send_response(sender, NativeTaskResponse::Ok(ok_res));
}
