mod aes256_key_store;
pub mod types;

use aa_contracts_client::EntryPointClient;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use binance_api::BinancePaymasterApi;
use ethereum_rpc::AlloyRpcProvider;
use executor_core::types::SerializablePackedUserOperation;
use executor_core::{
	intent_executor::IntentExecutor,
	native_task::{NativeTask, NativeTaskWrapper},
};
use executor_crypto::aes256::Aes256Key;
use executor_primitives::utils::hex::decode_hex;
use executor_storage::StorageDB;
use pumpx::PumpxApi;
use signer_client::SignerClient;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::oneshot;

pub use aes256_key_store::Aes256KeyStore;
pub use types::{NativeTaskError, NativeTaskOk, PumpxApiError, PumpxSignerError};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;

/// Maximum allowed paymaster gas to prevent abuse
const MAX_PAYMASTER_GAS: u128 = 5_000_000;

pub struct TaskHandlerContext<
	EthereumIntentExecutor: IntentExecutor,
	SolanaIntentExecutor: IntentExecutor,
	CrossChainIntentExecutor: IntentExecutor,
> {
	pub storage_db: Arc<StorageDB>,
	pub jwt_rsa_private_key: Vec<u8>,
	pub aes256_key: Aes256Key,
	pub ethereum_intent_executor: Arc<EthereumIntentExecutor>,
	pub solana_intent_executor: Arc<SolanaIntentExecutor>,
	pub cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
	pub pumpx_api: Arc<Box<dyn PumpxApi>>,
	pumpx_signer_client: Arc<Box<dyn SignerClient>>,
	pub binance_api_client: Arc<dyn BinancePaymasterApi>,
	pub entry_point_clients: Arc<HashMap<u64, Arc<EntryPointClient<AlloyRpcProvider>>>>,
	pub whitelisted_paymaster: Arc<Vec<Address>>,
}

impl<
		EthereumIntentExecutor: IntentExecutor,
		SolanaIntentExecutor: IntentExecutor,
		CrossChainIntentExecutor: IntentExecutor,
	> TaskHandlerContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		storage_db: Arc<StorageDB>,
		jwt_rsa_private_key: Vec<u8>,
		aes256_key: Aes256Key,
		ethereum_intent_executor: Arc<EthereumIntentExecutor>,
		solana_intent_executor: Arc<SolanaIntentExecutor>,
		cross_chain_intent_executor: Arc<CrossChainIntentExecutor>,
		pumpx_api: Arc<Box<dyn PumpxApi>>,
		pumpx_signer_client: Arc<Box<dyn SignerClient>>,
		binance_api_client: Arc<dyn BinancePaymasterApi>,
		entry_point_clients: Arc<HashMap<u64, Arc<EntryPointClient<AlloyRpcProvider>>>>,
	) -> Self {
		Self {
			storage_db,
			jwt_rsa_private_key,
			aes256_key,
			ethereum_intent_executor,
			solana_intent_executor,
			cross_chain_intent_executor,
			pumpx_api,
			pumpx_signer_client,
			binance_api_client,
			entry_point_clients,
			whitelisted_paymaster: Arc::new(vec![]), // Empty for now
		}
	}

	/// Get EntryPoint client for a specific chain
	pub fn get_entry_point_client(
		&self,
		chain_id: u64,
	) -> Option<Arc<EntryPointClient<AlloyRpcProvider>>> {
		self.entry_point_clients.get(&chain_id).cloned()
	}
}

pub async fn handle_native_task<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	_ctx: Arc<
		TaskHandlerContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
	_task: NativeTaskWrapper<NativeTask>,
	_response_sender: Option<ResponseSender>,
) -> Result<Vec<u8>, NativeTaskError> {
	// For now, return an error since we've removed the parachain functionality
	Err(NativeTaskError::InternalError(Some(
		"Parachain functionality has been removed".to_string(),
	)))
}

/// Convert SerializablePackedUserOperation to aa_contracts_client::PackedUserOperation
pub fn convert_to_packed_user_op(
	user_op: SerializablePackedUserOperation,
) -> Result<aa_contracts_client::PackedUserOperation, String> {
	use std::str::FromStr;

	Ok(aa_contracts_client::PackedUserOperation {
		sender: Address::from_str(&user_op.sender)
			.map_err(|e| format!("Invalid sender address '{}': {}", user_op.sender, e))?,
		nonce: U256::from(user_op.nonce),
		initCode: Bytes::from(
			decode_hex(&user_op.init_code).map_err(|e| format!("Invalid init_code hex: {}", e))?,
		),
		callData: Bytes::from(
			decode_hex(&user_op.call_data).map_err(|e| format!("Invalid call_data hex: {}", e))?,
		),
		accountGasLimits: user_op.account_gas_limits.parse::<FixedBytes<32>>().map_err(|e| {
			format!("Invalid account_gas_limits '{}': {}", user_op.account_gas_limits, e)
		})?,
		preVerificationGas: U256::from(user_op.pre_verification_gas),
		gasFees: user_op
			.gas_fees
			.parse::<FixedBytes<32>>()
			.map_err(|e| format!("Invalid gas_fees '{}': {}", user_op.gas_fees, e))?,
		paymasterAndData: Bytes::from(
			decode_hex(&user_op.paymaster_and_data)
				.map_err(|e| format!("Invalid paymaster_and_data hex: {}", e))?,
		),
		signature: match user_op.signature {
			Some(sig) => {
				Bytes::from(decode_hex(&sig).map_err(|e| format!("Invalid signature hex: {}", e))?)
			},
			None => Bytes::new(),
		},
	})
}
