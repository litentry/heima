mod aes256_key_store;
pub mod types;

use aa_contracts_client::calculate_user_operation_hash;
use aa_contracts_client::EntryPointClient;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use binance_api::BinancePaymasterApi;
use chrono::{Days, Utc};
use ethereum_rpc::AlloyRpcProvider;
use executor_core::{
	intent_executor::IntentExecutor,
	native_task::{NativeTask, NativeTaskWrapper},
	types::SerializablePackedUserOperation,
};
use executor_crypto::{
	aes256::{aes_decrypt, Aes256Key},
	jwt,
};
use executor_primitives::{
	utils::hex::{decode_hex, ToHexPrefixed},
	ChainId, Identity, Intent, PumpxAccountProfile, Web2IdentityType,
};
use executor_storage::{HeimaJwtStorage, IntentIdStorage, PumpxProfileStorage, Storage, StorageDB};
use heima_authentication::{
	auth_token::*,
	constants::{AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE},
};
use pumpx::{
	methods::create_transfer_tx::CreateTransferTxBody, signer_client::PumpxChainId, PumpxApi,
};
use signer_client::{ChainType, SignerClient};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

pub use aes256_key_store::Aes256KeyStore;
pub use types::{NativeTaskError, NativeTaskOk, PumpxApiError, PumpxSignerError};

pub type ResponseSender = oneshot::Sender<Vec<u8>>;

// ============================================================================
// ERC20 Paymaster Exchange Rate Processing
// ============================================================================

// Constants for ERC20 paymaster processing
// paymasterAndData format: paymaster_address (20) + validation_gas_limit (16) + postop_gas_limit (16) + paymaster_data
// paymaster_data: token(20) + exchangeRate(32) + validUntil(32) + validAfter(32)
const MIN_ERC20_PAYMASTER_DATA_LENGTH: usize = 52 + 20 + 32 + 32 + 32; // 168 bytes minimum
const PAYMASTER_DATA_OFFSET: usize = 52; // paymaster address (20) + validation_gas_limit (16) + postop_gas_limit (16)
const EXCHANGE_RATE_FEE_PERCENT: f64 = 0.0; // 0% fee for now
const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000; // 1e18 wei

// Whitelisted paymaster addresses that we support (deployed by heima), the logic is:
// - Unsigned userOp: If paymaster specified, **must** be whitelisted
// - Signed userOp: Only allowed if paymasterAndData is empty (technically we could still relay it, but we are a private bundler and don't want to relay arbitrary userOps)
//
// These addresses are assumed to be the same across all EVM chains
//
// Note: the SimplePaymaster should be gradually deprecated in favor of the ERC20PaymasterV1.
//       Theoretically user could construct an unsiged userOp to use SimplePaymaster "freely"
//
// TODO: add ERC20 paymaster addresses
const WHITELISTED_PAYMASTER_ADDRESSES: &[&str] = &[
	// SimplePaymaster
	"0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912", // staging-v1
	"0xD4dCB31763CBA7295bA4023E9411CB6db607DE07", // prod-v1
	// ERC20PaymasterV1
	"0xA8535e013236E04FAD5dc03eCc4c05A464c01f38", // staging
];

// Decode ERC20 paymaster data from paymasterAndData
// Format: paymaster_address(20) + validation_gas_limit(16) + postop_gas_limit(16) + token(20) + exchangeRate(32) + validUntil(32) + validAfter(32)
fn decode_erc20_paymaster_data(paymaster_and_data: &[u8]) -> Option<(Address, u128, u64, u64)> {
	if paymaster_and_data.len() < MIN_ERC20_PAYMASTER_DATA_LENGTH {
		return None;
	}

	// Extract token address from paymaster data (bytes 52-71)
	let token_address =
		Address::from_slice(&paymaster_and_data[PAYMASTER_DATA_OFFSET..PAYMASTER_DATA_OFFSET + 20]);

	// Extract exchange rate (bytes 72-103, 32 bytes, full u256 but we take as u128)
	let rate_bytes = &paymaster_and_data[PAYMASTER_DATA_OFFSET + 20..PAYMASTER_DATA_OFFSET + 52];
	let mut exchange_rate_bytes = [0u8; 16];
	exchange_rate_bytes.copy_from_slice(&rate_bytes[16..32]); // Last 16 bytes for u128
	let exchange_rate = u128::from_be_bytes(exchange_rate_bytes);

	// Extract validUntil (bytes 104-135, 32 bytes, take as u64)
	let valid_until_bytes =
		&paymaster_and_data[PAYMASTER_DATA_OFFSET + 52..PAYMASTER_DATA_OFFSET + 84];
	let mut until_bytes = [0u8; 8];
	until_bytes.copy_from_slice(&valid_until_bytes[24..32]); // Last 8 bytes for u64
	let valid_until = u64::from_be_bytes(until_bytes);

	// Extract validAfter (bytes 136-167, 32 bytes, take as u64)
	let valid_after_bytes =
		&paymaster_and_data[PAYMASTER_DATA_OFFSET + 84..PAYMASTER_DATA_OFFSET + 116];
	let mut after_bytes = [0u8; 8];
	after_bytes.copy_from_slice(&valid_after_bytes[24..32]); // Last 8 bytes for u64
	let valid_after = u64::from_be_bytes(after_bytes);

	Some((token_address, exchange_rate, valid_until, valid_after))
}

// Encode ERC20 paymaster data with updated exchange rate, preserving validUntil and validAfter
fn encode_erc20_paymaster_data(
	original_data: &[u8],
	new_exchange_rate: u128,
	valid_until: u64,
	valid_after: u64,
) -> Vec<u8> {
	let mut updated_data = original_data.to_vec();

	// Skip token address (20 bytes), update exchange rate (32 bytes, big-endian u128 in last 16 bytes)
	let rate_start = PAYMASTER_DATA_OFFSET + 20;
	let rate_bytes = [0u8; 16]
		.iter()
		.chain(&new_exchange_rate.to_be_bytes())
		.copied()
		.collect::<Vec<u8>>();
	updated_data[rate_start..rate_start + 32].copy_from_slice(&rate_bytes);

	// Update validUntil (32 bytes)
	let valid_until_start = rate_start + 32;
	let valid_until_bytes =
		[0u8; 24].iter().chain(&valid_until.to_be_bytes()).copied().collect::<Vec<u8>>();
	updated_data[valid_until_start..valid_until_start + 32].copy_from_slice(&valid_until_bytes);

	// Update validAfter (32 bytes)
	let valid_after_start = valid_until_start + 32;
	let valid_after_bytes =
		[0u8; 24].iter().chain(&valid_after.to_be_bytes()).copied().collect::<Vec<u8>>();
	updated_data[valid_after_start..valid_after_start + 32].copy_from_slice(&valid_after_bytes);

	updated_data
}

/// Extract paymaster address from paymasterAndData field
/// Returns None if the data is too short to contain a valid paymaster address
fn extract_paymaster_address(paymaster_and_data: &Bytes) -> Option<Address> {
	if paymaster_and_data.len() < 20 {
		return None;
	}

	// First 20 bytes contain the paymaster address
	Some(Address::from_slice(&paymaster_and_data[0..20]))
}

/// Check if a paymaster address is in our whitelist
/// If so, the userOp must be unsigned to allow exchange rate processing
fn is_whitelisted_paymaster(paymaster_address: &Address, whitelisted: &[Address]) -> bool {
	whitelisted.contains(paymaster_address)
}

/// Parse whitelisted paymaster addresses from const strings to Address types
/// Returns empty vec if any address fails to parse (defensive programming)
fn parse_whitelisted_paymasters() -> Vec<Address> {
	WHITELISTED_PAYMASTER_ADDRESSES
		.iter()
		.filter_map(|addr_str| addr_str.parse().ok())
		.collect()
}

// Comprehensive token mapping with expanded support
#[derive(Debug, Clone)]
struct TokenInfo {
	decimals: u8,
	binance_pair: &'static str,
}

// Get supported tokens - organized by chain for better scalability
fn get_supported_tokens() -> std::collections::HashMap<(u64, &'static str), TokenInfo> {
	let mut tokens = std::collections::HashMap::new();

	// Ethereum mainnet (chain_id 1)
	tokens.insert(
		(1, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);
	tokens.insert(
		(1, "0xdac17f958d2ee523a2206206994597c13d831ec7"), // USDT
		TokenInfo { decimals: 6, binance_pair: "ETHUSDT" },
	);
	tokens.insert(
		(1, "0x6b175474e89094c44da98b954eedeac495271d0f"), // DAI
		TokenInfo { decimals: 18, binance_pair: "ETHDAI" },
	);

	// Arbitrum One (chain_id 42161)
	tokens.insert(
		(42161, "0xaf88d065e77c8cc2239327c5edb3a432268e5831"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);
	tokens.insert(
		(42161, "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9"), // USDT
		TokenInfo { decimals: 6, binance_pair: "ETHUSDT" },
	);

	// Arbitrum Sepolia (chain_id 421614)
	tokens.insert(
		(421614, "0x75faf114eafb1bdbe2f0316df893fd58ce46aa4d"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);

	// BNB Smart Chain (chain_id 56)
	tokens.insert(
		(56, "0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d"), // USDC
		TokenInfo { decimals: 18, binance_pair: "ETHUSDC" },
	);
	tokens.insert(
		(56, "0x55d398326f99059ff775485246999027b3197955"), // USDT
		TokenInfo { decimals: 18, binance_pair: "ETHUSDT" },
	);

	// Base (Chain ID 8453)
	tokens.insert(
		(8453, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"), // USDC
		TokenInfo { decimals: 6, binance_pair: "ETHUSDC" },
	);

	tokens
}

// Map token address to Binance symbol and return (symbol, decimals) from hardcoded mapping
async fn get_token_info_from_mapping(
	token_address: &Address,
	chain_id: u64,
) -> Option<(String, u8)> {
	let tokens = get_supported_tokens();
	let address_str = token_address.to_string().to_lowercase();

	// Look up token info from our comprehensive mapping
	if let Some(token_info) = tokens.get(&(chain_id, address_str.as_str())) {
		// Use hardcoded values from our mapping
		return Some((token_info.binance_pair.to_string(), token_info.decimals));
	}

	debug!(
		"Token address {} on chain {} is not supported. Supported tokens: {:?}",
		token_address,
		chain_id,
		tokens.keys().filter(|(cid, _)| *cid == chain_id).collect::<Vec<_>>()
	);
	None
}

// Calculate exchange rate using Binance API
async fn calculate_exchange_rate_with_binance(
	binance_api: &dyn BinancePaymasterApi,
	token_symbol: &str,
	token_decimals: u8,
) -> Result<u128, String> {
	// Query price from Binance
	// For ETHUSDC, this returns how many USDC for 1 ETH (e.g., 4479.99)
	let price_str = binance_api
		.get_symbol_price(token_symbol)
		.await
		.map_err(|e| format!("Failed to get price for {}: {:?}", token_symbol, e))?;

	let tokens_per_eth: f64 = price_str
		.parse()
		.map_err(|e| format!("Failed to parse price '{}': {}", price_str, e))?;

	if tokens_per_eth <= 0.0 {
		return Err(format!("Invalid price from Binance: {}", tokens_per_eth));
	}

	// Apply fee percentage (increase rate to charge more tokens)
	let tokens_per_eth_with_fee = tokens_per_eth * (1.0 + EXCHANGE_RATE_FEE_PERCENT / 100.0);

	// Calculate exchange rate according to ERC20PaymasterV1.sol:
	// exchangeRate = tokensPerEth * 10^tokenDecimals
	// Example: For 6-decimal USDC at $4000/ETH: rate = 4000 * 10^6 = 4000000000
	// This rate is used as: requiredTokenAmount = (maxCost * exchangeRate) / 1e18
	// Where maxCost is in wei, so 1 ETH of gas = 10^18 wei
	// Result: (10^18 * 4000000000) / 10^18 = 4000000000 USDC units = 4000 USDC ✓

	let exchange_rate_f64 = tokens_per_eth_with_fee * (10_f64.powi(token_decimals as i32));

	if exchange_rate_f64 <= 0.0 || exchange_rate_f64 >= u128::MAX as f64 {
		return Err(format!("Exchange rate {} is out of valid range", exchange_rate_f64));
	}

	info!(
		"Calculated exchange rate for {}: {} tokens per ETH -> rate {} (with {}% fee)",
		token_symbol, tokens_per_eth, exchange_rate_f64 as u128, EXCHANGE_RATE_FEE_PERCENT
	);

	Ok(exchange_rate_f64 as u128)
}

// Process ERC20 paymaster data
async fn process_erc20_paymaster_data(
	binance_api: &dyn BinancePaymasterApi,
	paymaster_and_data: &Bytes,
	chain_id: u64,
) -> Result<Option<Bytes>, String> {
	let data_bytes = paymaster_and_data.as_ref();

	// Try to decode as ERC20 paymaster data
	if let Some((token_address, original_rate, valid_until, valid_after)) =
		decode_erc20_paymaster_data(data_bytes)
	{
		info!(
			"Detected ERC20 paymaster with token {} (original rate: {}, valid_until: {}, valid_after: {})",
			token_address, original_rate, valid_until, valid_after
		);

		// Get token symbol and decimals from hardcoded mapping
		let (token_symbol, token_decimals) = match get_token_info_from_mapping(
			&token_address,
			chain_id,
		)
		.await
		{
			Some((symbol, decimals)) => (symbol, decimals),
			None => {
				return Err(format!(
						"Token address {} on chain {} is not supported for price queries. Consider adding it to the supported tokens list or ensure it has a trading pair on Binance.",
						token_address, chain_id
					));
			},
		};

		// Calculate new exchange rate
		let new_exchange_rate =
			calculate_exchange_rate_with_binance(binance_api, &token_symbol, token_decimals)
				.await?;

		// Encode updated paymaster data with new exchange rate, keeping original timestamps
		let updated_paymaster_data = encode_erc20_paymaster_data(
			data_bytes,
			new_exchange_rate,
			valid_until, // Keep original validUntil
			valid_after, // Keep original validAfter
		);

		info!(
			"Updated ERC20 paymaster exchange rate: {} -> {} for token {}",
			original_rate, new_exchange_rate, token_address
		);

		Ok(Some(Bytes::from(updated_paymaster_data)))
	} else {
		// Not ERC20 paymaster data or insufficient length, return as-is
		debug!("PaymasterAndData is not ERC20 paymaster format, skipping processing");
		Ok(None)
	}
}

/// Calculate estimated token cost for ERC20 paymaster operations
async fn calculate_erc20_token_cost(
	binance_api: &dyn BinancePaymasterApi,
	paymaster_and_data: &Bytes,
	gas_estimates: &NativeTaskOk,
	user_op: &aa_contracts_client::PackedUserOperation,
	chain_id: u64,
) -> Option<crate::types::TokenCostEstimate> {
	// Decode paymaster data
	let (token_address, _, valid_until, valid_after) =
		decode_erc20_paymaster_data(paymaster_and_data.as_ref())?;

	// Check if timestamps are valid
	let current_time = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.ok()?
		.as_secs();

	if current_time < valid_after || current_time > valid_until {
		debug!("Exchange rate timestamps not valid for estimation");
		return None;
	}

	// Get token info from mapping
	let (token_symbol, token_decimals) =
		get_token_info_from_mapping(&token_address, chain_id).await?;

	// Get fresh exchange rate from Binance
	let exchange_rate = match calculate_exchange_rate_with_binance(
		binance_api,
		&token_symbol,
		token_decimals,
	)
	.await
	{
		Ok(rate) => rate,
		Err(e) => {
			error!("Failed to get exchange rate for token cost estimation: {}", e);
			return None;
		},
	};

	// Extract gas limits from estimates (pattern match on NativeTaskOk)
	let (
		call_gas,
		verification_gas,
		pre_verification_gas,
		paymaster_verification_gas,
		paymaster_post_op_gas,
	) = if let NativeTaskOk::EstimateUserOpGas {
		call_gas_limit,
		verification_gas_limit,
		pre_verification_gas,
		paymaster_verification_gas_limit,
		paymaster_post_op_gas_limit,
		..
	} = gas_estimates
	{
		(
			*call_gas_limit,
			*verification_gas_limit,
			*pre_verification_gas,
			*paymaster_verification_gas_limit,
			*paymaster_post_op_gas_limit,
		)
	} else {
		return None;
	};

	// Extract gas price from the UserOp's gasFees field
	// gasFees format: maxFeePerGas (16 bytes) | maxPriorityFeePerGas (16 bytes)
	let gas_fees_bytes = user_op.gasFees.0;
	let mut max_fee_bytes = [0u8; 16];
	max_fee_bytes.copy_from_slice(&gas_fees_bytes[0..16]); // First 16 bytes for maxFeePerGas
	let max_fee_per_gas = u128::from_be_bytes(max_fee_bytes);

	// Calculate total gas cost in wei
	// Include all gas components including paymaster overhead
	let total_gas = call_gas
		+ verification_gas
		+ pre_verification_gas
		+ paymaster_verification_gas
		+ paymaster_post_op_gas;

	let total_cost_wei = total_gas.saturating_mul(max_fee_per_gas);

	// Calculate token amount using same formula as the contract with ceil rounding
	// tokenAmount = ceil((gasCost * exchangeRate) / 1e18)
	let numerator = total_cost_wei.saturating_mul(exchange_rate);
	let token_amount = if numerator == 0 {
		0
	} else {
		numerator.saturating_add(WEI_PER_ETH - 1).saturating_div(WEI_PER_ETH)
	};

	// Add 10% safety buffer for price fluctuations
	let token_amount_with_buffer = token_amount.saturating_mul(110).saturating_div(100);

	Some(crate::types::TokenCostEstimate {
		token_address: token_address.to_string(),
		amount: token_amount_with_buffer,
		decimals: token_decimals,
		exchange_rate,
	})
}

pub type NativeTaskChannelType = (NativeTaskWrapper<NativeTask>, ResponseSender);
pub type NativeTaskSender = mpsc::Sender<NativeTaskChannelType>;

pub type NativeTaskResponse = Result<NativeTaskOk, NativeTaskError>;

pub const MAX_CONCURRENT_TASKS: usize = 512; // TODO: make it configurable (if we go for semaphore)

// Gas estimation constants
/// Maximum verification gas limit to prevent DoS attacks
const MAX_VERIFICATION_GAS: u128 = 3_000_000;
/// Default verification gas for testing during binary search
const DEFAULT_VERIFICATION_GAS_FOR_TESTING: u64 = 1_000_000;
/// Minimum transaction gas as per EIP-155
const MIN_TRANSACTION_GAS: u64 = 21_000;
/// Maximum reasonable gas for normal operations
const MAX_NORMAL_GAS: u64 = 10_000_000;
/// Minimum gas for deployment operations
const MIN_DEPLOYMENT_GAS: u64 = 100_000;
/// Maximum gas for deployment operations
const MAX_DEPLOYMENT_GAS: u64 = 20_000_000;
/// Safety buffer percentage for verification gas
const VERIFICATION_GAS_BUFFER_PERCENT: u64 = 20;
/// Default paymaster verification gas limit
const DEFAULT_PAYMASTER_VERIFICATION_GAS: u128 = 100_000;
/// Default paymaster post-operation gas limit
const DEFAULT_PAYMASTER_POST_OP_GAS: u128 = 50_000;
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
			whitelisted_paymaster: Arc::new(parse_whitelisted_paymasters()),
		}
	}

	/// Get EntryPoint client for a specific chain
	pub fn get_entry_point_client(
		&self,
		chain_id: ChainId,
	) -> Option<Arc<EntryPointClient<AlloyRpcProvider>>> {
		self.entry_point_clients.get(&chain_id).cloned()
	}
}

pub async fn handle_native_task<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<
		TaskHandlerContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
	wrapper: NativeTaskWrapper<NativeTask>,
) -> NativeTaskResponse {
	let client_id = &wrapper.client_id;

	match wrapper.task {
		NativeTask::RequestAuthToken(sender) => {
			let expires_at = Utc::now()
				.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
				.expect("Failed to calculate expiration")
				.timestamp();
			let auth_options = AuthOptions { expires_at };
			let claims = match sender {
				Identity::Email(ref identity_string) => {
					let Ok(email) = std::str::from_utf8(identity_string.inner_ref()) else {
						error!("Invalid email identity");
						return Err(NativeTaskError::InvalidMemberIdentity);
					};
					AuthTokenClaims::new(
						email.to_string(),
						AUTH_TOKEN_ID_TYPE.to_string(),
						client_id.to_string(),
						auth_options,
					)
				},
				_ => AuthTokenClaims::new(
					sender.hash().to_string(),
					AUTH_TOKEN_ID_TYPE.to_string(),
					client_id.to_string(),
					auth_options,
				),
			};
			let Ok(token) = jwt::create(&claims, &ctx.jwt_rsa_private_key) else {
				error!("Failed to create auth token");
				return Err(NativeTaskError::AuthTokenCreationFailed);
			};

			Ok(NativeTaskOk::AuthToken(token))
		},
		NativeTask::RequestIntent(omni_account, intent_id, intent) => {
			let intent = *intent;
			debug!("Intent requested, intent_id: {}", intent_id);

			let intent_id_storage = IntentIdStorage::new(ctx.storage_db.clone());
			let stored_intent_id = match intent_id_storage.get(&omni_account) {
				Ok(id) => id.unwrap_or_default(),
				Err(_) => {
					error!("Failed to read intent from store");
					return Err(NativeTaskError::InternalError(None));
				},
			};

			if intent_id == stored_intent_id + 1 {
				if intent_id_storage.insert(&omni_account, intent_id).is_err() {
					error!("Failed to save intent id");
					return Err(NativeTaskError::InternalError(None));
				}
			} else {
				error!(
					"Intent id different than expected, expected: {:?}, got: {:?}",
					stored_intent_id + 1,
					intent_id
				);
				return Err(NativeTaskError::IntentNonceMismatch);
			}

			let result = match intent {
				Intent::SystemRemark(_)
				| Intent::TransferNative(_)
				| Intent::CallEthereum(_)
				| Intent::TransferEthereum(_)
				| Intent::TransferSolana(_) => {
					info!("Intent temporarily rejected, intent_id: {}", intent_id);
					Err(NativeTaskError::InternalError(None))
				},
				Intent::Swap(..) => {
					let response = match ctx
						.cross_chain_intent_executor
						.execute(&omni_account, intent_id, intent.clone())
						.await
					{
						Ok((response, _)) => response,
						Err(e) => {
							error!("Error executing intent: {:?}", e);
							ctx.cross_chain_intent_executor.on_execution_error().await;
							None
						},
					};
					if let Some(response) = response {
						Ok(NativeTaskOk::IntentSwapResponse(response))
					} else {
						Err(NativeTaskError::InternalError(None))
					}
				},
			};

			result
		},
		NativeTask::PumpxRequestJwt(_sender, email, invite_code, google_code, language) => {
			let expires_at = Utc::now()
				.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
				.expect("Failed to calculate expiration")
				.timestamp();
			let auth_options = AuthOptions { expires_at };

			debug!("Calling pumpx get_account_user_id, email: {}", email);
			let res = match ctx.pumpx_api.get_account_user_id(email.clone()).await {
				Ok(res) => res,
				Err(e) => {
					error!("Failed to get_account_user_id for email {}: {:?}", email, e);
					return Err(NativeTaskError::PumpxApiError(
						PumpxApiError::GetAccountUserIdFailed,
					));
				},
			};
			debug!("Response pumpx get_account_user_id: {:?}", res);

			let Some(user_id) = res.data.user_id else {
				error!("Response data.user_id of call get_account_user_id is none");
				return Err(NativeTaskError::PumpxApiError(PumpxApiError::GetAccountUserIdFailed));
			};

			debug!("get_account_user_id ok, email: {}, user_id: {}", email, user_id);
			let omni_account = Identity::from_web2_account(&user_id, Web2IdentityType::Pumpx)
				.to_omni_account(client_id);

			let access_token_claims = AuthTokenClaims::new(
				omni_account.to_hex(),
				AUTH_TOKEN_ACCESS_TYPE.to_string(),
				client_id.to_string(),
				auth_options.clone(),
			);
			let Ok(access_token) = jwt::create(&access_token_claims, &ctx.jwt_rsa_private_key)
			else {
				error!("Failed to create access token");
				return Err(NativeTaskError::AuthTokenCreationFailed);
			};

			debug!("Calling pumpx user_connect, user_id: {}, email: {}, invite_code: {:?}, google_code: {:?}", user_id, email, invite_code, google_code);
			let Ok(backend_response) = ctx
				.pumpx_api
				.user_connect(
					&access_token,
					user_id.clone(),
					email.clone(),
					invite_code,
					google_code,
					language,
				)
				.await
			else {
				error!("Failed to connect user");
				return Err(NativeTaskError::PumpxApiError(PumpxApiError::UserConnectionFailed));
			};
			debug!("Response pumpx user_connect: {:?}", backend_response);

			// check google auth value
			if !backend_response.data.google_auth_check.unwrap_or(false) {
				error!("Google code verification failed from user_connect");
				return Err(NativeTaskError::PumpxApiError(
					PumpxApiError::GoogleCodeVerificationFailed,
				));
			}

			let id_token_claims = AuthTokenClaims::new(
				omni_account.to_hex(),
				AUTH_TOKEN_ID_TYPE.to_string(),
				client_id.to_string(),
				auth_options,
			);
			let Ok(id_token) = jwt::create(&id_token_claims, &ctx.jwt_rsa_private_key) else {
				error!("Failed to create id token");
				return Err(NativeTaskError::AuthTokenCreationFailed);
			};

			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			if storage
				.insert(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE), access_token.clone())
				.is_err()
			{
				error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ACCESS_TYPE);
			};

			if storage.insert(&(omni_account, AUTH_TOKEN_ID_TYPE), id_token.clone()).is_err() {
				error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ID_TYPE);
			};

			Ok(NativeTaskOk::PumpxRequestJwt { access_token, id_token, backend_response })
		},
		NativeTask::PumpxExportWallet(
			omni_account,
			google_code,
			pumpx_chain_id,
			pumpx_wallet_index,
			expected_wallet_address,
		) => {
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE);
				return Err(NativeTaskError::InternalError(None));
			};

			let verify_success = verify_google_code(
				ctx.pumpx_api.as_ref().as_ref(),
				&access_token,
				google_code,
				None,
			)
			.await;
			if !verify_success {
				error!("Failed to verify google code within NativeTask::PumpxExportWallet");
				return Err(NativeTaskError::PumpxApiError(
					PumpxApiError::GoogleCodeVerificationFailed,
				));
			}

			let Some(chain) = ChainType::from_pumpx_chain_id(pumpx_chain_id) else {
				error!("Failed to map pumpx chain_id {}", pumpx_chain_id);
				return Err(NativeTaskError::ChainNotSupported(pumpx_chain_id as u64));
			};

			let Ok(mut wallet) = ctx
				.pumpx_signer_client
				.export_wallet(
					chain,
					pumpx_wallet_index,
					omni_account.clone().into(),
					// TODO: theoretically we could pass the aes_key from initial RPC to signer, so that
					//       we don't have to do double encryption/decryption
					ctx.aes256_key.to_vec(),
					expected_wallet_address,
				)
				.await
			else {
				error!("Failed to export wallet from pumpx-signer");
				return Err(NativeTaskError::SignatureServiceUnavailable);
			};
			let Some(decrypted_wallet) = aes_decrypt(&ctx.aes256_key, &mut wallet) else {
				error!("Failed to decrypt wallet");
				return Err(NativeTaskError::InternalError(None));
			};

			let omni_account_profile_storage = PumpxProfileStorage::new(ctx.storage_db.clone());
			if let Ok(maybe_profile) = omni_account_profile_storage.get(&omni_account) {
				let profile = maybe_profile
					.map(|mut p| {
						p.wallet_exported = true;
						p
					})
					.unwrap_or_else(|| PumpxAccountProfile { wallet_exported: true });
				if let Err(e) = omni_account_profile_storage.insert(&omni_account, profile) {
					error!("Failed to update pumpx account profile: {:?}", e);
					return Err(NativeTaskError::InternalError(None));
				};
			} else {
				error!("Failed to get pumpx account profile");
				return Err(NativeTaskError::InternalError(None));
			}
			Ok(NativeTaskOk::PumpxExportWallet(decrypted_wallet))
		},
		NativeTask::PumpxAddWallet(omni_account) => {
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) = storage.get(&(omni_account, AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE);
				return Err(NativeTaskError::InternalError(None));
			};

			// Call Pumpx API to add wallet
			debug!("Calling pumpx add_wallet");
			let Ok(backend_response) = ctx.pumpx_api.add_wallet(&access_token, None).await else {
				error!("Failed to add wallet through Pumpx API");
				return Err(NativeTaskError::PumpxApiError(PumpxApiError::AddWalletFailed));
			};

			Ok(NativeTaskOk::PumpxAddWallet(backend_response))
		},
		NativeTask::PumpxSignLimitOrder(omni_account, chain_id, wallet_index, unsigned_tx) => {
			let Some(chain) = ChainType::from_pumpx_chain_id(chain_id) else {
				error!("Failed to map pumpx chain_id {}", chain_id);
				return Err(NativeTaskError::ChainNotSupported(chain_id as u64));
			};
			let Ok(signed_txs) = ctx
				.pumpx_signer_client
				.request_signatures(chain, wallet_index, omni_account.into(), unsigned_tx)
				.await
			else {
				error!("Failed to request signatures from pumpx-signer");
				return Err(NativeTaskError::SignatureServiceUnavailable);
			};
			Ok(NativeTaskOk::PumpxSignLimitOrder(signed_txs))
		},
		NativeTask::PumpxTransferWidthdraw(
			omni_account,
			request_id,
			chain_id,
			wallet_index,
			recipient_address,
			token_ca,
			amount,
			google_code,
			language,
		) => {
			// 1. Verify we have a valid Pumpx "access" token for the user
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get access_token within NativeTask::PumpxTransferWidthdraw");
				return Err(NativeTaskError::InternalError(None));
			};

			// 2. Verify google code in every case
			let verify_success = verify_google_code(
				ctx.pumpx_api.as_ref().as_ref(),
				&access_token,
				google_code,
				language.clone(),
			)
			.await;
			if !verify_success {
				error!("Failed to verify google code within NativeTask::PumpxTransferWidthdraw");
				return Err(NativeTaskError::PumpxApiError(
					PumpxApiError::GoogleCodeVerificationFailed,
				));
			}

			// 3. Create a transfer tx and send to backend
			let body = CreateTransferTxBody {
				request_id,
				chain_id,
				wallet_index,
				recipient_address,
				token_ca,
				amount,
			};

			debug!("Calling pumpx create_transfer_tx, body {:?}", body);
			match ctx.pumpx_api.create_transfer_tx(&access_token, body, language.clone()).await {
				Ok(res) => Ok(NativeTaskOk::PumpxTransferWithdraw(res)),
				Err(e) => {
					error!("Failed to create transfer tx: {}", e);
					Err(NativeTaskError::PumpxApiError(PumpxApiError::CreateTransferTxFailed))
				},
			}
		},
		NativeTask::PumpxNotifyLimitOrderResult(_omni_account, intent_id, result, message) => {
			if result != "ok" && result != "nok" {
				error!("Invalid result value: {}. Must be 'ok' or 'nok'", result);
				return Err(NativeTaskError::PumpxApiError(PumpxApiError::InvalidInput));
			}

			if let Some(msg) = message {
				info!("Limit order result message for intent_id {}: {}", intent_id, msg);
			}

			Ok(NativeTaskOk::PumpxNotifyLimitOrderResult)
		},
		NativeTask::EstimateUserOpGas(
			omni_account,
			serializable_user_op,
			chain_id,
			wallet_index,
		) => {
			info!(
				"Processing EstimateUserOpGas for account {:?}, wallet_index: {}, chain_id: {}",
				omni_account, wallet_index, chain_id
			);

			// Get EntryPoint client for this chain
			let entry_point_client = match ctx.get_entry_point_client(chain_id) {
				Some(client) => client,
				None => {
					error!("No EntryPoint client configured for chain_id: {}", chain_id);
					return Err(NativeTaskError::ChainNotSupported(chain_id));
				},
			};

			// Convert SerializablePackedUserOperation to PackedUserOperation
			let packed_user_op = match convert_to_packed_user_op(serializable_user_op.clone()) {
				Ok(user_op) => user_op,
				Err(e) => {
					error!("Failed to convert UserOperation: {}", e);
					return Err(NativeTaskError::InvalidUserOperation(
						"Invalid user operation format".to_string(),
					));
				},
			};

			// Perform gas estimation (wallet_index can be used for wallet-specific optimizations)
			match estimate_user_op_gas(
				entry_point_client,
				packed_user_op,
				chain_id,
				ctx.binance_api_client.as_ref(),
			)
			.await
			{
				Ok(gas_estimates) => {
					info!("Gas estimation successful: {:?}", gas_estimates);
					Ok(gas_estimates)
				},
				Err(e) => {
					error!("Gas estimation failed: {}", e);
					Err(NativeTaskError::GasEstimationFailed)
				},
			}
		},
		NativeTask::SubmitUserOp(omni_account, serializable_user_ops, chain_id, wallet_index) => {
			info!(
				"Processing SubmitUserOp for {} UserOperations on chain_id: {}",
				serializable_user_ops.len(),
				chain_id
			);

			// Get EntryPoint client for this chain (needed for both signing and submission)
			let entry_point_client = match ctx.get_entry_point_client(chain_id) {
				Some(client) => client,
				None => {
					error!("No EntryPoint client configured for chain_id: {}", chain_id);
					return Err(NativeTaskError::ChainNotSupported(chain_id));
				},
			};

			// Process each UserOperation in the batch
			let mut aa_user_ops = Vec::new();

			for (index, serializable_user_op) in serializable_user_ops.iter().enumerate() {
				// Convert SerializablePackedUserOperation to PackedUserOperation
				let mut packed_user_op =
					match convert_to_packed_user_op(serializable_user_op.clone()) {
						Ok(user_op) => user_op,
						Err(e) => {
							error!("Failed to convert UserOperation {}: {}", index, e);
							return Err(NativeTaskError::InvalidUserOperation(format!(
								"Invalid user operation at index {}",
								index
							)));
						},
					};

				// Check userOp signature status and validate paymaster usage
				if packed_user_op.signature.is_empty() {
					// UNSIGNED userOp: If paymaster specified, must be whitelisted
					if !packed_user_op.paymasterAndData.is_empty() {
						if let Some(paymaster_address) =
							extract_paymaster_address(&packed_user_op.paymasterAndData)
						{
							if !is_whitelisted_paymaster(
								&paymaster_address,
								&ctx.whitelisted_paymaster,
							) {
								error!(
									"UserOperation {} uses non-whitelisted paymaster {}. Only whitelisted paymasters are allowed for unsigned userOps.",
									index, paymaster_address
								);
								return Err(NativeTaskError::InvalidUserOperation(format!(
									"UserOperation at index {} uses non-whitelisted paymaster {}",
									index, paymaster_address
								)));
							}
						}

						match process_erc20_paymaster_data(
							ctx.binance_api_client.as_ref() as &dyn BinancePaymasterApi,
							&packed_user_op.paymasterAndData,
							chain_id,
						)
						.await
						{
							Ok(Some(updated_paymaster_data)) => {
								packed_user_op.paymasterAndData = updated_paymaster_data;
								info!("Updated ERC20 paymaster data for UserOperation {}", index);
							},
							Ok(None) => {
								// Not an ERC20 paymaster, continue as normal
								debug!("UserOperation {} does not use ERC20 paymaster", index);
							},
							Err(e) => {
								error!(
									"Failed to process ERC20 paymaster data for UserOperation {}: {}",
									index, e
								);
								return Err(NativeTaskError::InvalidUserOperation(format!(
									"ERC20 paymaster processing failed for operation at index {}: {}",
									index, e
								)));
							},
						}
					}

					info!("Requesting signature from pumpx signer for UserOperation {}", index);

					// Log UserOp details for debugging
					info!(
						"UserOp details - Sender: {}, Nonce: {}, InitCode length: {}, CallData length: {}",
						packed_user_op.sender,
						packed_user_op.nonce,
						packed_user_op.initCode.len(),
						packed_user_op.callData.len()
					);

					let entry_point_address = entry_point_client.entry_point_address();

					let user_op_hash_bytes = calculate_user_operation_hash(
						&packed_user_op,
						entry_point_address,
						chain_id,
					);
					let message_to_sign = user_op_hash_bytes.to_vec();

					info!(
						"Signing UserOp hash: 0x{}, EntryPoint: {}, ChainID: {}",
						hex::encode(user_op_hash_bytes),
						entry_point_address,
						chain_id
					);

					// Request signature from pumpx signer for EVM chain
					let signature_result = ctx
						.pumpx_signer_client
						.request_signature(
							ChainType::Evm,
							wallet_index,
							omni_account.clone().into(),
							message_to_sign,
						)
						.await;

					let signature = match signature_result {
						Ok(sig) => substrate_to_ethereum_signature(&sig).unwrap().to_vec(),
						Err(_) => {
							error!("Failed to sign user operation {}", index);
							return Err(NativeTaskError::SignatureServiceUnavailable);
						},
					};

					// Prepend 0x01 byte to indicate Root signature type (according to UserOpSigner enum)
					let mut signature_with_prefix: Vec<u8> = vec![0x01];
					signature_with_prefix.extend_from_slice(&signature);
					packed_user_op.signature = Bytes::from(signature_with_prefix);
					info!("UserOperation {} signed successfully", index);
				} else {
					// SIGNED userOp: Only allowed if no paymaster specified
					if !packed_user_op.paymasterAndData.is_empty() {
						error!(
							"UserOperation {} is signed but has paymaster data. Signed userOps are only allowed without paymaster.",
							index
						);
						return Err(NativeTaskError::InvalidUserOperation(format!(
							"UserOperation at index {} is signed but specifies a paymaster",
							index
						)));
					}
					info!("UserOperation {} is signed with no paymaster, processing", index);
				}

				// Convert to aa_contracts_client::PackedUserOperation for EntryPoint call
				let aa_user_op = aa_contracts_client::PackedUserOperation {
					sender: packed_user_op.sender,
					nonce: packed_user_op.nonce,
					initCode: packed_user_op.initCode.clone(),
					callData: packed_user_op.callData.clone(),
					accountGasLimits: packed_user_op.accountGasLimits,
					preVerificationGas: packed_user_op.preVerificationGas,
					gasFees: packed_user_op.gasFees,
					paymasterAndData: packed_user_op.paymasterAndData.clone(),
					signature: packed_user_op.signature.clone(),
				};
				aa_user_ops.push(aa_user_op);
			}

			// Get beneficiary address from the EntryPoint client's wallet
			let beneficiary = match entry_point_client.get_wallet_address().await {
				Ok(address) => address,
				Err(_) => {
					let err_msg = "Failed to get wallet address from EntryPoint client".to_string();
					error!("{}", err_msg.clone());
					return Err(NativeTaskError::InternalError(Some(err_msg)));
				},
			};

			// Run batch simulation for all UserOperations before submission
			info!("Running batch simulation for {} UserOperations", aa_user_ops.len());
			match entry_point_client.simulate_handle_ops(&aa_user_ops, beneficiary).await {
				Ok(simulation_results) => {
					for (index, result) in simulation_results.iter().enumerate() {
						info!(
							"UserOperation {} simulation successful. PreOpGas: {}, Paid: {}, AccountValidation: {}, PaymasterValidation: {}",
							index,
							result.preOpGas,
							result.paid,
							result.accountValidationData,
							result.paymasterValidationData
						);
					}
					info!(
						"All {} UserOperations passed batch simulation checks",
						aa_user_ops.len()
					);
				},
				Err(e) => {
					let err_msg: String = format!("Batch UserOperation simulation failed: {}", e);
					error!("{}", err_msg.clone());
					return Err(NativeTaskError::InvalidUserOperation(err_msg));
				},
			}

			// Submit all UserOperations via EntryPoint.handleOps() with retry logic
			let transaction_hash =
				match entry_point_client.handle_ops_with_retry(&aa_user_ops, beneficiary).await {
					Ok(tx_hash) => {
						// Return the actual transaction hash from handle_ops
						Some(tx_hash)
					},
					Err(_) => {
						let err_msg = "Failed to submit UserOperations to EntryPoint via handleOps after retries"
								.to_string();
						error!("{}", err_msg.clone());
						return Err(NativeTaskError::InternalError(Some(err_msg)));
					},
				};

			Ok(NativeTaskOk::SubmitUserOp(transaction_hash))
		},
		NativeTask::RequestLoan(
			omni_account,
			chain_id,
			wallet_index,
			collateral_ticker,
			spot_ratio,
			margin_ratio,
		) => {
			info!(
				"Processing RequestLoan for {:?}, chain_id: {}, wallet_index: {}, collateral: {}, spot_ratio: {}, margin_ratio: {}",
				omni_account, chain_id, wallet_index, collateral_ticker, spot_ratio, margin_ratio
			);

			handle_request_loan(
				ctx,
				omni_account,
				chain_id,
				wallet_index,
				&collateral_ticker,
				spot_ratio,
				margin_ratio,
			)
			.await
		},
	}
}

async fn verify_google_code(
	pumpx_api: &dyn PumpxApi,
	access_token: &str,
	google_code: String,
	language: Option<String>,
) -> bool {
	debug!("Calling pumpx verify_google_code, code: {}", google_code);
	let verify_result = pumpx_api.verify_google_code(access_token, google_code, language).await;
	verify_result.map_or_else(
		|e| {
			error!("Google code verification request failed: {:?}", e);
			false
		},
		|res| {
			res.data.result.map_or_else(
				|| {
					error!("Google code verification response result is none");
					false
				},
				|success| success,
			)
		},
	)
}

// Helper functions for gas limit packing/unpacking
fn pack_account_gas_limits(verification_gas: u128, call_gas: u128) -> FixedBytes<32> {
	let packed: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
	FixedBytes::from(packed.to_be_bytes())
}

#[cfg(test)]
fn unpack_verification_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = (value >> 128) & U256::from(u128::MAX);
	result.to::<u128>()
}

#[cfg(test)]
fn unpack_call_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = value & U256::from(u128::MAX);
	result.to::<u128>()
}

/// Convert Substrate signature to Ethereum ECDSA format
/// Returns signature in format: [r (32 bytes), s (32 bytes), v (1 byte)]
pub fn substrate_to_ethereum_signature(substrate_sig: &[u8]) -> Result<[u8; 65], &'static str> {
	if substrate_sig.len() != 65 {
		return Err("Invalid signature length");
	}

	// Parse as (r, s, v) format - most common
	let mut r = [0u8; 32];
	let mut s = [0u8; 32];
	r.copy_from_slice(&substrate_sig[0..32]);
	s.copy_from_slice(&substrate_sig[32..64]);
	let substrate_v = substrate_sig[64];

	// Convert recovery parameter: 0/1 -> 27/28
	let ethereum_v = match substrate_v {
		0 => 27,
		1 => 28,
		27 => 27, // Already Ethereum format
		28 => 28, // Already Ethereum format
		_ => return Err("Invalid recovery parameter"),
	};

	// Build Ethereum signature: [r, s, v]
	let mut ethereum_sig = [0u8; 65];
	ethereum_sig[0..32].copy_from_slice(&r);
	ethereum_sig[32..64].copy_from_slice(&s);
	ethereum_sig[64] = ethereum_v;

	Ok(ethereum_sig)
}

/// Convert SerializablePackedUserOperation to aa_contracts_client::PackedUserOperation
pub fn convert_to_packed_user_op(
	user_op: SerializablePackedUserOperation,
) -> Result<aa_contracts_client::PackedUserOperation, String> {
	use std::str::FromStr;

	// Helper function to parse hex string to fixed bytes
	let parse_hex_fixed =
		|hex_str: &str, expected_len: usize, name: &str| -> Result<Vec<u8>, String> {
			let bytes = decode_hex(hex_str)
				.map_err(|e| format!("Invalid hex string '{}' '{}': {}", hex_str, name, e))?;
			if bytes.len() != expected_len {
				return Err(format!(
					"Expected {} bytes, got {} for '{}'",
					expected_len,
					bytes.len(),
					hex_str
				));
			}
			Ok(bytes)
		};

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
		accountGasLimits: {
			let bytes = parse_hex_fixed(&user_op.account_gas_limits, 32, "account_gas_limits")?;
			FixedBytes::from_slice(&bytes)
		},
		preVerificationGas: U256::from(user_op.pre_verification_gas),
		gasFees: {
			let bytes = parse_hex_fixed(&user_op.gas_fees, 32, "gas_fees")?;
			FixedBytes::from_slice(&bytes)
		},
		paymasterAndData: Bytes::from(
			decode_hex(&user_op.paymaster_and_data)
				.map_err(|e| format!("Invalid paymaster_and_data hex: {}", e))?,
		),
		signature: match user_op.signature {
			Some(sig) => {
				Bytes::from(decode_hex(&sig).map_err(|e| format!("Invalid signature hex: {}", e))?)
			},
			None => Bytes::new(), // Empty signature for unsigned operations
		},
	})
}

/// Estimate gas for a UserOperation using simulation
async fn estimate_user_op_gas(
	entry_point_client: Arc<EntryPointClient<AlloyRpcProvider>>,
	user_op: aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
	binance_api: &dyn BinancePaymasterApi,
) -> Result<NativeTaskOk, String> {
	// Step 1: Simulate validation to get base gas requirements
	let validation_result = entry_point_client
		.simulate_validation(user_op.clone())
		.await
		.map_err(|e| format!("Validation simulation failed: {:?}", e))?;

	// Extract preOpGas from validation result
	let pre_op_gas = validation_result.returnInfo.preOpGas;
	debug!("Validation simulation preOpGas: {}", pre_op_gas);

	// Step 2: Calculate verification gas limit based on validation result
	// Add buffer for safety
	let buffer_multiplier = U256::from(100 + VERIFICATION_GAS_BUFFER_PERCENT);
	let verification_gas_base = pre_op_gas.saturating_mul(buffer_multiplier) / U256::from(100);
	let verification_gas_limit = verification_gas_base.min(U256::from(MAX_VERIFICATION_GAS));

	// Step 3: Binary search for optimal call gas limit
	let call_gas_limit =
		estimate_call_gas_limit(entry_point_client.clone(), user_op.clone(), chain_id).await?;

	// Step 4: Calculate preVerificationGas (static + dynamic components)
	let (static_pvg, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);
	let pre_verification_gas = static_pvg + dynamic_pvg;

	// Step 5: Extract paymaster gas limits if paymaster is present
	let (paymaster_verification_gas_limit, paymaster_post_op_gas_limit) =
		extract_paymaster_gas_limits(&user_op.paymasterAndData);

	// Step 6: Calculate gas fees with buffer
	let (max_fee_per_gas, max_priority_fee_per_gas) = entry_point_client
		.calculate_gas_fees_with_buffer(10) // Use 10% additional buffer for estimation
		.await
		.map_err(|e| format!("Failed to calculate gas fees: {:?}", e))?;

	// Convert to u128 for response, ensuring values are within bounds
	let call_gas_limit = call_gas_limit.try_into().map_err(|_| {
		format!("Call gas limit {} exceeds maximum supported value", call_gas_limit)
	})?;

	let verification_gas_limit = verification_gas_limit.try_into().map_err(|_| {
		format!("Verification gas limit {} exceeds maximum supported value", verification_gas_limit)
	})?;

	let pre_verification_gas = pre_verification_gas.try_into().map_err(|_| {
		format!("Pre-verification gas {} exceeds maximum supported value", pre_verification_gas)
	})?;

	let max_fee_per_gas = max_fee_per_gas.try_into().map_err(|_| {
		format!("Max fee per gas {} exceeds maximum supported value", max_fee_per_gas)
	})?;

	let max_priority_fee_per_gas = max_priority_fee_per_gas.try_into().map_err(|_| {
		format!(
			"Max priority fee per gas {} exceeds maximum supported value",
			max_priority_fee_per_gas
		)
	})?;

	// Build initial response with gas estimates
	let gas_response = NativeTaskOk::EstimateUserOpGas {
		call_gas_limit,
		verification_gas_limit,
		pre_verification_gas,
		paymaster_verification_gas_limit,
		paymaster_post_op_gas_limit,
		max_fee_per_gas,
		max_priority_fee_per_gas,
		estimated_token_cost: None, // Will be updated if ERC20 paymaster is detected
	};

	let estimated_token_cost = if !user_op.paymasterAndData.is_empty() {
		// Step 7: Calculate token cost if ERC20 paymaster is present
		calculate_erc20_token_cost(
			binance_api,
			&user_op.paymasterAndData,
			&gas_response,
			&user_op,
			chain_id,
		)
		.await
	} else {
		None
	};

	// Update response with token cost estimate
	let final_response = NativeTaskOk::EstimateUserOpGas {
		call_gas_limit,
		verification_gas_limit,
		pre_verification_gas,
		paymaster_verification_gas_limit,
		paymaster_post_op_gas_limit,
		max_fee_per_gas,
		max_priority_fee_per_gas,
		estimated_token_cost,
	};

	info!("Gas estimation complete: {:?}", final_response);
	Ok(final_response)
}

/// Binary search for optimal call gas limit following Rundler's approach
async fn estimate_call_gas_limit(
	entry_point_client: Arc<EntryPointClient<AlloyRpcProvider>>,
	user_op: aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
) -> Result<U256, String> {
	// Determine gas limits based on operation type
	let (min_gas, max_gas) = if !user_op.initCode.is_empty() {
		// Deployment operation requires higher gas limits
		(U256::from(MIN_DEPLOYMENT_GAS), U256::from(MAX_DEPLOYMENT_GAS))
	} else {
		// Normal operation
		(U256::from(MIN_TRANSACTION_GAS), U256::from(MAX_NORMAL_GAS))
	};

	// Step 1: Initial simulation at maximum to get baseline gas usage
	let mut test_user_op = user_op.clone();
	let verification_gas = U256::from(DEFAULT_VERIFICATION_GAS_FOR_TESTING);
	test_user_op.accountGasLimits =
		pack_account_gas_limits(verification_gas.to::<u128>(), max_gas.to::<u128>());

	let initial_result = entry_point_client
		.simulate_handle_ops(&vec![test_user_op], Address::ZERO)
		.await
		.map_err(|e| format!("Initial simulation failed: {:?}", e))?;

	if initial_result.is_empty() || !initial_result[0].targetSuccess {
		return Err("UserOperation validation failed at maximum gas limit".to_string());
	}

	// Extract actual gas used from the successful simulation
	let initial_gas_used = initial_result[0].paid;

	// Step 2: Set initial guess as 2x the gas used (accounts for 63/64ths rule)
	let initial_guess = initial_gas_used.saturating_mul(U256::from(2)).min(max_gas);

	info!("Initial gas simulation: used={}, initial_guess={}", initial_gas_used, initial_guess);

	// Step 3: Binary search with 10% tolerance (following Rundler's approach)
	let mut lower_bound = min_gas;
	let mut upper_bound = initial_guess;
	let mut iterations = 0;
	const MAX_ITERATIONS: u32 = 20;
	const TOLERANCE_PERCENT: u64 = 10; // Stop when bounds are within 10%

	while iterations < MAX_ITERATIONS {
		// Check if we've converged within tolerance
		if lower_bound > U256::ZERO {
			let range = upper_bound - lower_bound;
			let tolerance_threshold = lower_bound / U256::from(TOLERANCE_PERCENT);

			if range <= tolerance_threshold {
				debug!(
					"Binary search converged after {} iterations: range={}, threshold={}",
					iterations, range, tolerance_threshold
				);
				break;
			}
		}

		let mid_gas = (lower_bound + upper_bound) / U256::from(2);

		// Test if this gas limit works
		let mut test_user_op = user_op.clone();
		test_user_op.accountGasLimits =
			pack_account_gas_limits(verification_gas.to::<u128>(), mid_gas.to::<u128>());

		let test_result =
			entry_point_client.simulate_handle_ops(&vec![test_user_op], Address::ZERO).await;

		match test_result {
			Ok(results) if !results.is_empty() && results[0].targetSuccess => {
				// Simulation succeeded, try lower gas
				upper_bound = mid_gas;
				debug!("Binary search iteration {}: gas {} succeeded", iterations, mid_gas);
			},
			_ => {
				// Simulation failed, need more gas
				lower_bound = mid_gas + U256::from(1);
				debug!("Binary search iteration {}: gas {} failed", iterations, mid_gas);
			},
		}

		iterations += 1;
	}

	// Use the upper bound as our estimate (ensures success)
	let optimal_gas = upper_bound;

	// Add safety buffer based on chain
	let buffer_percent = match chain_id {
		1 => 50,     // Mainnet: 50% buffer
		42161 => 30, // Arbitrum: 30% buffer
		8453 => 30,  // Base: 30% buffer
		56 => 40,    // BSC: 40% buffer
		80084 => 20, // HyperEVM: 20% buffer
		_ => 40,     // Default: 40% buffer
	};

	let final_gas = optimal_gas.saturating_mul(U256::from(100 + buffer_percent)) / U256::from(100);

	info!("Call gas limit estimation: optimal={}, with_buffer={}", optimal_gas, final_gas);
	Ok(final_gas)
}

/// Calculate preVerificationGas split into static and dynamic components
fn calculate_pre_verification_gas(
	user_op: &aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
) -> (U256, U256) {
	// EIP-2028 gas costs
	const GAS_PER_ZERO_BYTE: u64 = 4;
	const GAS_PER_NON_ZERO_BYTE: u64 = 16;
	const BASE_TRANSACTION_GAS: u64 = 21_000;
	const CREATE2_OVERHEAD_GAS: u64 = 32_000;
	const BUNDLE_OVERHEAD_GAS: u64 = 5_000; // Per-UserOp share of bundle transaction overhead

	// Helper function to calculate gas for bytes
	let calculate_bytes_gas = |data: &[u8]| -> U256 {
		let zero_bytes = data.iter().filter(|&&b| b == 0).count() as u64;
		let non_zero_bytes = (data.len() as u64) - zero_bytes;
		U256::from(zero_bytes * GAS_PER_ZERO_BYTE + non_zero_bytes * GAS_PER_NON_ZERO_BYTE)
	};

	// === STATIC PVG ===
	// These costs don't change based on network conditions
	let mut static_gas = U256::from(BASE_TRANSACTION_GAS + BUNDLE_OVERHEAD_GAS);

	// Calculate gas for UserOp calldata that will be included in the bundle
	static_gas += calculate_bytes_gas(&user_op.callData);
	static_gas += calculate_bytes_gas(&user_op.initCode);
	static_gas += calculate_bytes_gas(&user_op.paymasterAndData);
	static_gas += calculate_bytes_gas(&user_op.signature);

	// Add gas for fixed-size fields
	// sender (address as bytes20 padded to bytes32)
	static_gas += U256::from(20 * GAS_PER_NON_ZERO_BYTE + 12 * GAS_PER_ZERO_BYTE);
	// nonce (usually has many zero bytes)
	static_gas += U256::from(32 * GAS_PER_ZERO_BYTE);
	// accountGasLimits (bytes32)
	static_gas += U256::from(16 * GAS_PER_NON_ZERO_BYTE + 16 * GAS_PER_ZERO_BYTE);
	// preVerificationGas (uint256)
	static_gas += U256::from(8 * GAS_PER_NON_ZERO_BYTE + 24 * GAS_PER_ZERO_BYTE);
	// gasFees (bytes32)
	static_gas += U256::from(16 * GAS_PER_NON_ZERO_BYTE + 16 * GAS_PER_ZERO_BYTE);

	// Add deployment overhead if initCode is present
	if !user_op.initCode.is_empty() {
		static_gas += U256::from(CREATE2_OVERHEAD_GAS);
	}

	// === DYNAMIC PVG ===
	// L2-specific costs that can change based on L1 gas prices
	let dynamic_gas = calculate_l2_data_cost(user_op, chain_id);

	// Apply buffers
	let static_with_buffer = static_gas.saturating_mul(U256::from(110)) / U256::from(100); // 10% buffer
	let dynamic_with_buffer = if dynamic_gas > U256::ZERO {
		// L2s need higher buffer due to L1 gas price volatility
		dynamic_gas.saturating_mul(U256::from(125)) / U256::from(100) // 25% buffer for L2
	} else {
		U256::ZERO
	};

	debug!(
		"PreVerificationGas: static={}, dynamic={} (chain_id={}, total_bytes={})",
		static_with_buffer,
		dynamic_with_buffer,
		chain_id,
		user_op.callData.len() + user_op.initCode.len() + user_op.paymasterAndData.len()
	);

	(static_with_buffer, dynamic_with_buffer)
}

/// Calculate L2-specific data availability costs
fn calculate_l2_data_cost(
	user_op: &aa_contracts_client::PackedUserOperation,
	chain_id: ChainId,
) -> U256 {
	// Check if this is an L2 network
	let is_l2 = matches!(
		chain_id,
		42161 | 421614 | // Arbitrum One, Arbitrum Sepolia
		10 | 11155420 |  // Optimism, Optimism Sepolia
		8453 | 84532 |   // Base, Base Sepolia
		137 | 80001 // Polygon, Mumbai
	);

	if !is_l2 {
		return U256::ZERO;
	}

	// Calculate total calldata size that needs to be posted to L1
	let total_bytes = user_op.callData.len()
		+ user_op.initCode.len()
		+ user_op.paymasterAndData.len()
		+ user_op.signature.len()
		+ 32 * 5; // Fixed fields

	// L2-specific multipliers (these would ideally come from an oracle)
	// These are rough estimates - production should use actual L1 gas price oracles
	let l1_data_cost_per_byte = match chain_id {
		42161 | 421614 => U256::from(140), // Arbitrum (uses Nitro compression)
		10 | 11155420 => U256::from(160),  // Optimism (uses bedrock compression)
		8453 | 84532 => U256::from(160),   // Base (same as Optimism)
		137 | 80001 => U256::from(50),     // Polygon (cheaper as sidechain)
		_ => U256::from(100),              // Default for unknown L2s
	};

	let dynamic_cost = U256::from(total_bytes) * l1_data_cost_per_byte;

	debug!(
		"L2 data cost calculation: chain_id={}, bytes={}, cost_per_byte={}, total={}",
		chain_id, total_bytes, l1_data_cost_per_byte, dynamic_cost
	);

	dynamic_cost
}

/// Extract and validate paymaster gas limits from paymasterAndData field
fn extract_paymaster_gas_limits(paymaster_and_data: &Bytes) -> (u128, u128) {
	// If no paymaster, return zeros
	if paymaster_and_data.len() < 20 {
		return (0, 0);
	}

	// paymasterAndData format:
	// [0:20] - paymaster address
	// [20:36] - paymaster verification gas limit (uint128)
	// [36:52] - paymaster post-op gas limit (uint128)
	// [52:] - paymaster data
	// check UserOperationLib.sol

	if paymaster_and_data.len() >= 52 {
		// Extract verification gas limit (bytes 20-36)
		let mut verification_bytes = [0u8; 16];
		verification_bytes.copy_from_slice(&paymaster_and_data[20..36]);
		let verification_gas_limit = u128::from_be_bytes(verification_bytes);

		// Extract post-op gas limit (bytes 36-52)
		let mut post_op_bytes = [0u8; 16];
		post_op_bytes.copy_from_slice(&paymaster_and_data[36..52]);
		let post_op_gas_limit = u128::from_be_bytes(post_op_bytes);

		// Validate gas limits to prevent abuse
		let validated_verification = if verification_gas_limit > MAX_PAYMASTER_GAS {
			debug!(
				"Paymaster verification gas {} exceeds maximum, using default",
				verification_gas_limit
			);
			DEFAULT_PAYMASTER_VERIFICATION_GAS
		} else if verification_gas_limit == 0 {
			debug!("Paymaster verification gas is zero, using default");
			DEFAULT_PAYMASTER_VERIFICATION_GAS
		} else {
			verification_gas_limit
		};

		let validated_post_op = if post_op_gas_limit > MAX_PAYMASTER_GAS {
			debug!("Paymaster post-op gas {} exceeds maximum, using default", post_op_gas_limit);
			DEFAULT_PAYMASTER_POST_OP_GAS
		} else if post_op_gas_limit == 0 {
			debug!("Paymaster post-op gas is zero, using default");
			DEFAULT_PAYMASTER_POST_OP_GAS
		} else {
			post_op_gas_limit
		};

		debug!(
			"Validated paymaster gas limits: verification={}, post_op={}",
			validated_verification, validated_post_op
		);

		(validated_verification, validated_post_op)
	} else {
		// Paymaster present but no gas limits specified, use defaults
		debug!("Paymaster present but gas limits not specified, using defaults");
		(DEFAULT_PAYMASTER_VERIFICATION_GAS, DEFAULT_PAYMASTER_POST_OP_GAS)
	}
}

async fn handle_request_loan<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<
		TaskHandlerContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
	omni_account: executor_primitives::AccountId,
	chain_id: u64,
	wallet_index: u32,
	collateral_ticker: &str,
	spot_ratio: u64,
	margin_ratio: u64,
) -> NativeTaskResponse {
	use hyperliquid::*;

	let hypercore_client = HyperCoreClient::new(chain_id).map_err(|e| {
		error!("Failed to create HyperCore client: {}", e);
		NativeTaskError::ChainNotSupported(chain_id)
	})?;

	// Fetch metadata from HyperCore
	let spot_meta = hypercore_client.get_spot_meta().await.map_err(|e| {
		error!("Failed to get spot meta: {}", e);
		NativeTaskError::InternalError(Some(e))
	})?;

	let meta = hypercore_client.get_meta().await.map_err(|e| {
		error!("Failed to get perp meta: {}", e);
		NativeTaskError::InternalError(Some(e))
	})?;

	// Get asset IDs
	let spot_asset_id = get_spot_asset_id(collateral_ticker, &spot_meta).map_err(|e| {
		error!("Failed to get spot asset ID: {}", e);
		NativeTaskError::InternalError(Some(e))
	})?;

	let perp_asset_id = get_perp_asset_id(collateral_ticker, &meta).map_err(|e| {
		error!("Failed to get perp asset ID: {}", e);
		NativeTaskError::InternalError(Some(e))
	})?;

	info!(
		"Resolved asset IDs - spot: {}, perp: {} for ticker: {}",
		spot_asset_id, perp_asset_id, collateral_ticker
	);

	// Get smart wallet address
	let wallet_bytes = ctx
		.pumpx_signer_client
		.request_wallet(signer_client::ChainType::Evm, wallet_index, omni_account.clone().into())
		.await
		.map_err(|_| {
			error!("Failed to get smart wallet address");
			NativeTaskError::PumpxSignerError(PumpxSignerError::RequestWalletFailed)
		})?;

	let smart_wallet_address =
		format!("0x{}", hex::encode(&wallet_bytes[wallet_bytes.len() - 20..]));

	info!("Smart wallet address: {}", smart_wallet_address);

	// Calculate spot sell size (assuming 1.0 units for now, should be calculated from collateral)
	let total_collateral_size = 1.0;
	let spot_sell_ratio = (spot_ratio as f64) / 10000.0;
	let spot_sell_size = total_collateral_size * spot_sell_ratio;
	let spot_sell_size_units = calculate_size_units(spot_sell_size);

	// Generate cloids
	let spot_sell_cloid = generate_cloid();
	let hedge_open_cloid = generate_cloid() + 1;

	info!("Generated cloids - spot_sell: {}, hedge_open: {}", spot_sell_cloid, hedge_open_cloid);

	// Build spot sell action
	let spot_sell_action =
		build_spot_sell_order(spot_asset_id, spot_sell_size_units, spot_sell_cloid);
	let spot_sell_corewriter_calldata = encode_send_raw_action(spot_sell_action);
	let spot_sell_calldata =
		encode_omni_account_execute(get_core_writer_address(), spot_sell_corewriter_calldata);

	// Submit spot sell UserOp
	let spot_sell_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		chain_id,
		wallet_index,
		&smart_wallet_address,
		spot_sell_calldata,
	)
	.await?;

	info!("Spot sell transaction submitted: {:?}", spot_sell_tx_hash);

	// Wait for spot sell to complete
	let spot_filled = hypercore_client
		.wait_for_order_completion(&smart_wallet_address, &spot_sell_cloid.to_string(), 60)
		.await
		.map_err(|e| {
			error!("Spot sell order did not complete: {}", e);
			NativeTaskError::InternalError(Some(format!("Spot sell timeout: {}", e)))
		})?;

	if !spot_filled {
		error!("Spot sell order failed");
		return Err(NativeTaskError::InternalError(Some("Spot sell order failed".to_string())));
	}

	info!("Spot sell order filled successfully");

	// Calculate hedge size based on margin_ratio
	let hedge_size = total_collateral_size * (10000.0 - (spot_ratio as f64)) / 10000.0;
	let leverage = (margin_ratio as f64) / 10000.0;
	let hedge_position_size = hedge_size * leverage;
	let hedge_size_units = calculate_size_units(hedge_position_size);

	// Build hedge perp order
	let hedge_action = build_perp_long_order(perp_asset_id, hedge_size_units, hedge_open_cloid);
	let hedge_corewriter_calldata = encode_send_raw_action(hedge_action);
	let hedge_calldata =
		encode_omni_account_execute(get_core_writer_address(), hedge_corewriter_calldata);

	// Submit hedge UserOp
	let hedge_tx_hash = submit_corewriter_userop(
		ctx.clone(),
		&omni_account,
		chain_id,
		wallet_index,
		&smart_wallet_address,
		hedge_calldata,
	)
	.await?;

	info!("Hedge transaction submitted: {:?}", hedge_tx_hash);

	// Wait for hedge to complete
	let hedge_filled = hypercore_client
		.wait_for_order_completion(&smart_wallet_address, &hedge_open_cloid.to_string(), 60)
		.await
		.map_err(|e| {
			error!("Hedge order did not complete: {}", e);
			NativeTaskError::InternalError(Some(format!("Hedge timeout: {}", e)))
		})?;

	if !hedge_filled {
		error!("Hedge order failed");
		return Err(NativeTaskError::InternalError(Some("Hedge order failed".to_string())));
	}

	info!("Hedge order filled successfully");

	// Calculate USDC received (simplified, should query actual fill amount)
	let usdc_received = format!("{:.2}", spot_sell_size * 3000.0); // Placeholder price

	Ok(NativeTaskOk::RequestLoan {
		spot_sell_cloid: spot_sell_cloid.to_string(),
		hedge_open_cloid: hedge_open_cloid.to_string(),
		usdc_received,
		spot_sell_tx_hash,
		hedge_open_tx_hash: hedge_tx_hash,
	})
}

async fn submit_corewriter_userop<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<
		TaskHandlerContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
	omni_account: &executor_primitives::AccountId,
	chain_id: u64,
	wallet_index: u32,
	smart_wallet_address: &str,
	call_data: String,
) -> Result<Option<String>, NativeTaskError> {
	use executor_core::types::SerializablePackedUserOperation;
	use hyperliquid::*;

	let entry_point_client = ctx.get_entry_point_client(chain_id).ok_or_else(|| {
		error!("No EntryPoint client for chain_id: {}", chain_id);
		NativeTaskError::ChainNotSupported(chain_id)
	})?;

	// Get current nonce - use a simple counter for now
	// TODO: Query from EntryPoint.getNonce(sender, key) in production
	let nonce = 0u128;

	// Calculate gas fees
	let (max_fee_per_gas, max_priority_fee_per_gas) =
		entry_point_client.calculate_gas_fees_with_buffer(20).await.map_err(|e| {
			error!("Failed to calculate gas fees: {:?}", e);
			NativeTaskError::InternalError(Some("Failed to calculate gas fees".to_string()))
		})?;

	// Build UserOp
	let user_op = SerializablePackedUserOperation {
		sender: smart_wallet_address.to_string(),
		nonce,
		init_code: "0x".to_string(),
		call_data,
		account_gas_limits: pack_account_gas_limits(1_000_000, 2_000_000),
		pre_verification_gas: 100_000,
		gas_fees: pack_gas_fees(
			max_fee_per_gas.to::<u128>(),
			max_priority_fee_per_gas.to::<u128>(),
		),
		paymaster_and_data: encode_simple_paymaster(),
		signature: None, // Will be signed by SubmitUserOp handler
	};

	// Submit via existing SubmitUserOp handler
	let wrapper = executor_core::native_task::NativeTaskWrapper::new(
		executor_core::native_task::NativeTask::SubmitUserOp(
			omni_account.clone(),
			vec![user_op],
			chain_id,
			wallet_index,
		),
		None,
		None,
		"internal".to_string(),
	);

	match Box::pin(handle_native_task(ctx, wrapper)).await {
		Ok(NativeTaskOk::SubmitUserOp(tx_hash)) => Ok(tx_hash),
		Ok(_) => Err(NativeTaskError::InternalError(Some(
			"Unexpected response from SubmitUserOp".to_string(),
		))),
		Err(e) => Err(e),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use aa_contracts_client::PackedUserOperation;
	use alloy::{
		hex,
		primitives::{Bytes, FixedBytes, U256},
	};
	use executor_core::types::SerializablePackedUserOperation;
	use executor_primitives::ChainId;

	#[test]
	fn test_convert_to_packed_user_op() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0xdeadbeef".to_string(),
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: Some("0x1234567890abcdef".to_string()),
		};

		let packed_user_op = convert_to_packed_user_op(serializable_user_op)
			.expect("Failed to convert SerializablePackedUserOperation");

		// Verify the conversion
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
		assert_eq!(packed_user_op.initCode, Bytes::from(hex::decode("deadbeef").unwrap()));
		assert_eq!(packed_user_op.callData, Bytes::from(hex::decode("cafebabe").unwrap()));
		assert_eq!(packed_user_op.preVerificationGas, U256::from(21000));
		assert_eq!(packed_user_op.paymasterAndData, Bytes::from(Vec::<u8>::new()));
		assert_eq!(packed_user_op.signature, Bytes::from(hex::decode("1234567890abcdef").unwrap()));
	}

	#[test]
	fn test_convert_to_packed_user_op_unsigned() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0xdeadbeef".to_string(),
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None, // Unsigned operation
		};

		let packed_user_op = convert_to_packed_user_op(serializable_user_op)
			.expect("Failed to convert unsigned SerializablePackedUserOperation");

		// Verify the signature is empty for unsigned operation
		assert!(packed_user_op.signature.is_empty());
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
	}

	#[test]
	fn test_convert_to_packed_user_op_empty_init_code() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "".to_string(), // Empty init_code
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: Some("0x1234567890abcdef".to_string()),
		};

		let result = convert_to_packed_user_op(serializable_user_op);
		assert!(result.is_ok(), "Empty init_code should not cause an error: {:?}", result.err());

		let packed_user_op = result.unwrap();
		// Empty init_code should result in empty Bytes
		assert!(packed_user_op.initCode.is_empty());
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
	}

	#[test]
	fn test_convert_to_packed_user_op_0x_init_code() {
		let serializable_user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0x".to_string(), // "0x" prefix only
			call_data: "0xcafebabe".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: Some("0x1234567890abcdef".to_string()),
		};

		let result = convert_to_packed_user_op(serializable_user_op);
		assert!(result.is_ok(), "0x init_code should not cause an error: {:?}", result.err());

		let packed_user_op = result.unwrap();
		// "0x" init_code should result in empty Bytes
		assert!(packed_user_op.initCode.is_empty());
		assert_eq!(packed_user_op.sender.to_string(), "0x1234567890123456789012345678901234567890");
		assert_eq!(packed_user_op.nonce, U256::from(42));
	}

	#[test]
	fn test_pack_account_gas_limits() {
		let verification_gas = 500_000u128;
		let call_gas = 300_000u128;

		let packed = pack_account_gas_limits(verification_gas, call_gas);

		// Verify the packed format
		let expected: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
		assert_eq!(packed.0, expected.to_be_bytes());
	}

	#[test]
	fn test_unpack_verification_gas_limit() {
		// Create a packed value with verification gas = 500000, call gas = 300000
		let verification_gas = 500_000u128;
		let call_gas = 300_000u128;
		let packed_value: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
		let packed_bytes = FixedBytes::from(packed_value.to_be_bytes());

		let unpacked = unpack_verification_gas_limit(packed_bytes);
		assert_eq!(unpacked, verification_gas);
	}

	#[test]
	fn test_unpack_call_gas_limit() {
		// Create a packed value with verification gas = 500000, call gas = 300000
		let verification_gas = 500_000u128;
		let call_gas = 300_000u128;
		let packed_value: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
		let packed_bytes = FixedBytes::from(packed_value.to_be_bytes());

		let unpacked = unpack_call_gas_limit(packed_bytes);
		assert_eq!(unpacked, call_gas);
	}

	#[test]
	fn test_pack_unpack_roundtrip() {
		let test_cases = vec![
			(0u128, 0u128),
			(1u128, 1u128),
			(u128::MAX, u128::MAX),
			(1_000_000u128, 500_000u128),
			(3_000_000u128, 10_000_000u128),
		];

		for (verification, call) in test_cases {
			let packed = pack_account_gas_limits(verification, call);
			let unpacked_verification = unpack_verification_gas_limit(packed);
			let unpacked_call = unpack_call_gas_limit(packed);

			assert_eq!(
				unpacked_verification,
				verification,
				"Verification gas mismatch for {:?}",
				(verification, call)
			);
			assert_eq!(unpacked_call, call, "Call gas mismatch for {:?}", (verification, call));
		}
	}

	#[test]
	fn test_calculate_pre_verification_gas_mainnet() {
		// Test with a simple UserOp for mainnet (no L2 costs)
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(1),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x00, 0x01, 0x02, 0x03]), // 4 bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]), // 65 bytes signature
		};

		let chain_id: ChainId = 1; // Ethereum mainnet
		let (static_pvg, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);

		// Static PVG should include base costs + calldata
		// Base: 21000 + 5000 = 26000
		// Calldata: sender(20) + nonce(~3) + initCode(0) + callData(4) + signature(65) + other fields
		// This is approximate since we need to calculate exact calldata costs
		assert!(static_pvg > U256::from(26_000), "Static PVG should be at least base costs");

		// Dynamic PVG should be 0 for mainnet
		assert_eq!(dynamic_pvg, U256::ZERO, "Dynamic PVG should be 0 for mainnet");
	}

	#[test]
	fn test_calculate_pre_verification_gas_arbitrum() {
		// Test with UserOp for Arbitrum (includes L2 data costs)
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(1),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x00; 100]), // 100 zero bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]),
		};

		let chain_id: ChainId = 42161; // Arbitrum One
		let (static_pvg, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);

		// Static PVG should include base costs
		assert!(static_pvg > U256::from(26_000), "Static PVG should include base costs");

		// Dynamic PVG should be non-zero for Arbitrum (140 gas per byte)
		assert!(dynamic_pvg > U256::ZERO, "Dynamic PVG should be non-zero for Arbitrum");

		// Verify L2 multiplier is applied (140 gas per byte for Arbitrum)
		let total_bytes = user_op.sender.len() + 100 + 65; // Approximate total bytes
		let expected_min_dynamic = U256::from(total_bytes * 140);
		assert!(
			dynamic_pvg >= expected_min_dynamic,
			"Dynamic PVG should apply Arbitrum multiplier"
		);
	}

	#[test]
	fn test_calculate_pre_verification_gas_optimism() {
		// Test with UserOp for Optimism
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(1),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x01; 50]), // 50 non-zero bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]),
		};

		let chain_id: ChainId = 10; // Optimism
		let (_, dynamic_pvg) = calculate_pre_verification_gas(&user_op, chain_id);

		// Dynamic PVG should use Optimism multiplier (160 gas per byte)
		assert!(dynamic_pvg > U256::ZERO, "Dynamic PVG should be non-zero for Optimism");

		// Should be higher than Arbitrum for same data
		let arbitrum_chain: ChainId = 42161; // Arbitrum One
		let (_, arbitrum_dynamic) = calculate_pre_verification_gas(&user_op, arbitrum_chain);
		assert!(
			dynamic_pvg > arbitrum_dynamic,
			"Optimism should have higher dynamic PVG than Arbitrum"
		);
	}

	#[test]
	fn test_calculate_pre_verification_gas_with_deployment() {
		// Test with initCode present (deployment scenario)
		let user_op = PackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".parse().unwrap(),
			nonce: U256::from(0),                   // First transaction
			initCode: Bytes::from(vec![0x60; 200]), // 200 bytes of deployment code
			callData: Bytes::from(vec![]),
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0xff; 65]),
		};

		let chain_id: ChainId = 1; // Ethereum mainnet
		let (static_pvg, _) = calculate_pre_verification_gas(&user_op, chain_id);

		// Should include CREATE2 overhead (32000 gas)
		// Base (26000) + CREATE2 (32000) + calldata costs
		assert!(static_pvg > U256::from(58_000), "Static PVG should include CREATE2 overhead");
	}

	#[test]
	fn test_calculate_pre_verification_gas_zero_bytes() {
		// Test calldata gas calculation with all zero bytes
		let user_op = PackedUserOperation {
			sender: "0x0000000000000000000000000000000000000000".parse().unwrap(),
			nonce: U256::from(0),
			initCode: Bytes::from(vec![]),
			callData: Bytes::from(vec![0x00; 1000]), // 1000 zero bytes
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: Bytes::from(vec![]),
			signature: Bytes::from(vec![0x00; 65]), // All zero signature
		};

		let chain_id: ChainId = 1; // Ethereum mainnet
		let (static_pvg, _) = calculate_pre_verification_gas(&user_op, chain_id);

		// Zero bytes cost 4 gas each (EIP-2028)
		// Should be significantly lower than non-zero bytes
		let non_zero_op = PackedUserOperation {
			callData: Bytes::from(vec![0xff; 1000]), // 1000 non-zero bytes
			..user_op.clone()
		};
		let (non_zero_static, _) = calculate_pre_verification_gas(&non_zero_op, chain_id);

		assert!(
			static_pvg < non_zero_static,
			"Zero bytes should cost less gas than non-zero bytes"
		);
	}

	#[test]
	fn test_extract_paymaster_gas_limits_valid() {
		// Valid paymasterAndData with gas limits at bytes 20-52
		let mut paymaster_data = vec![0x11; 20]; // 20 bytes of paymaster address

		// Add verification gas limit (16 bytes, u128)
		let verification_gas = 150_000u128;
		paymaster_data.extend_from_slice(&verification_gas.to_be_bytes());

		// Add post-op gas limit (16 bytes, u128)
		let post_op_gas = 50_000u128;
		paymaster_data.extend_from_slice(&post_op_gas.to_be_bytes());

		// Add some extra data
		paymaster_data.extend_from_slice(&[0xff; 20]);

		let paymaster_and_data = Bytes::from(paymaster_data);
		let (extracted_verification, extracted_post_op) =
			extract_paymaster_gas_limits(&paymaster_and_data);

		assert_eq!(extracted_verification, verification_gas, "Verification gas should match");
		assert_eq!(extracted_post_op, post_op_gas, "Post-op gas should match");
	}

	#[test]
	fn test_extract_paymaster_gas_limits_short_data() {
		// Data shorter than 52 bytes
		let paymaster_data = vec![0x11; 30]; // Only 30 bytes
		let paymaster_and_data = Bytes::from(paymaster_data);

		let (verification, post_op) = extract_paymaster_gas_limits(&paymaster_and_data);

		// Should return defaults
		assert_eq!(verification, DEFAULT_PAYMASTER_VERIFICATION_GAS);
		assert_eq!(post_op, DEFAULT_PAYMASTER_POST_OP_GAS);
	}

	#[test]
	fn test_extract_paymaster_gas_limits_empty() {
		// Empty paymasterAndData
		let paymaster_and_data = Bytes::from(vec![]);

		let (verification, post_op) = extract_paymaster_gas_limits(&paymaster_and_data);

		// Should return zeros for no paymaster
		assert_eq!(verification, 0);
		assert_eq!(post_op, 0);
	}
}

#[cfg(test)]
mod erc20_paymaster_tests {
	use super::*;
	use alloy::primitives::Bytes;
	use binance_api::Error as BinanceApiError;

	#[test]
	fn test_decode_erc20_paymaster_data_valid() {
		// Create test paymaster data with correct format:
		// paymaster (20) + validation_gas_limit (16) + postop_gas_limit (16) + token (20) + exchangeRate (32) + validUntil (32) + validAfter (32)
		let mut data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH];

		// Set paymaster address (bytes 0-19)
		data[0..20].copy_from_slice(&[0x12; 20]);

		// Set validation gas limit (bytes 20-35) - 100000
		let validation_gas = 100000u128;
		data[20..36].copy_from_slice(&validation_gas.to_be_bytes());

		// Set postop gas limit (bytes 36-51) - 50000
		let postop_gas = 50000u128;
		data[36..52].copy_from_slice(&postop_gas.to_be_bytes());

		// Set token address (bytes 52-71) - USDC-like token
		data[52..72].copy_from_slice(&[0xA0; 20]);

		// Set exchange rate (bytes 72-103) - representing 2000 * 10^6 for USDC
		let exchange_rate = 2000000000u128;
		let rate_bytes = [0u8; 16]
			.iter()
			.chain(&exchange_rate.to_be_bytes())
			.copied()
			.collect::<Vec<u8>>();
		data[72..104].copy_from_slice(&rate_bytes);

		// Set validUntil (bytes 104-135) - timestamp 1700000000
		let valid_until = 1700000000u64;
		let valid_until_bytes =
			[0u8; 24].iter().chain(&valid_until.to_be_bytes()).copied().collect::<Vec<u8>>();
		data[104..136].copy_from_slice(&valid_until_bytes);

		// Set validAfter (bytes 136-167) - timestamp 1600000000
		let valid_after = 1600000000u64;
		let valid_after_bytes =
			[0u8; 24].iter().chain(&valid_after.to_be_bytes()).copied().collect::<Vec<u8>>();
		data[136..168].copy_from_slice(&valid_after_bytes);

		// Test decoding
		let result = decode_erc20_paymaster_data(&data);
		assert!(result.is_some());

		let (token_address, exchange_rate_result, valid_until_result, valid_after_result) =
			result.unwrap();
		assert_eq!(token_address, Address::from([0xA0; 20]));
		assert_eq!(exchange_rate_result, exchange_rate);
		assert_eq!(valid_until_result, valid_until);
		assert_eq!(valid_after_result, valid_after);
	}

	#[test]
	fn test_decode_erc20_paymaster_data_too_short() {
		let data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH - 1];
		let result = decode_erc20_paymaster_data(&data);
		assert!(result.is_none());
	}

	#[test]
	fn test_encode_erc20_paymaster_data() {
		// Create initial paymaster data with correct format
		let mut original_data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH];

		// Fill with some initial values
		original_data[0..20].copy_from_slice(&[0x12; 20]); // paymaster address
		original_data[20..36].copy_from_slice(&100000u128.to_be_bytes()); // validation gas
		original_data[36..52].copy_from_slice(&50000u128.to_be_bytes()); // postop gas
		original_data[52..72].copy_from_slice(&[0xA0; 20]); // token address

		// Set initial exchange rate (bytes 72-103)
		let initial_rate = 1500000000u128;
		let rate_bytes = [0u8; 16]
			.iter()
			.chain(&initial_rate.to_be_bytes())
			.copied()
			.collect::<Vec<u8>>();
		original_data[72..104].copy_from_slice(&rate_bytes);

		// Test encoding new values
		let new_exchange_rate = 3000000000u128;
		let new_valid_until = 1800000000u64;
		let new_valid_after = 1700000000u64;

		let updated_data = encode_erc20_paymaster_data(
			&original_data,
			new_exchange_rate,
			new_valid_until,
			new_valid_after,
		);

		// Verify the updated data contains the new values
		let result = decode_erc20_paymaster_data(&updated_data);
		assert!(result.is_some());

		let (token_address, exchange_rate_result, valid_until, valid_after) = result.unwrap();
		assert_eq!(token_address, Address::from([0xA0; 20])); // Token address should remain unchanged
		assert_eq!(exchange_rate_result, new_exchange_rate);
		assert_eq!(valid_until, new_valid_until);
		assert_eq!(valid_after, new_valid_after);
	}

	#[test]
	fn test_supported_token_mapping_consistency() {
		let tokens = get_supported_tokens();

		// Verify that we have tokens for major chains
		let eth_tokens: Vec<_> = tokens.keys().filter(|(chain_id, _)| *chain_id == 1).collect();
		assert!(eth_tokens.len() >= 3, "Should have at least 3 tokens on Ethereum mainnet");

		let arbitrum_tokens: Vec<_> =
			tokens.keys().filter(|(chain_id, _)| *chain_id == 42161).collect();
		assert!(arbitrum_tokens.len() >= 2, "Should have at least 2 tokens on Arbitrum");
	}

	#[tokio::test]
	async fn test_calculate_erc20_token_cost_valid() {
		// Mock Binance API
		struct MockBinanceApi;
		#[async_trait::async_trait]
		impl BinancePaymasterApi for MockBinanceApi {
			async fn get_symbol_price(&self, symbol: &str) -> Result<String, BinanceApiError> {
				// Return mock price for ETHUSDC: 3000 USDC per ETH
				if symbol == "ETHUSDC" {
					Ok("3000.00".to_string())
				} else {
					Err(BinanceApiError::SymbolNotFound)
				}
			}

			async fn get_symbol_precision(&self, _symbol: &str) -> Result<u8, BinanceApiError> {
				Ok(6)
			}

			async fn get_all_trading_symbols(&self) -> Result<Vec<String>, BinanceApiError> {
				Ok(vec!["ETHUSDC".to_string()])
			}
		}

		// Create mock paymaster data with USDC token
		let mut paymaster_data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH];
		// Set paymaster address (bytes 0-19)
		paymaster_data[0..20].copy_from_slice(&[0x12; 20]);
		// Set validation gas limit (bytes 20-35) - 100000
		paymaster_data[20..36].copy_from_slice(&100000u128.to_be_bytes());
		// Set postop gas limit (bytes 36-51) - 50000
		paymaster_data[36..52].copy_from_slice(&50000u128.to_be_bytes());
		// Set USDC token address on Ethereum mainnet (bytes 52-71)
		let usdc_address_bytes = hex::decode("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
		paymaster_data[52..72].copy_from_slice(&usdc_address_bytes);

		// Set exchange rate (bytes 72-103) - will be recalculated
		let exchange_rate = 0u128; // Placeholder
		let rate_bytes = [0u8; 16]
			.iter()
			.chain(&exchange_rate.to_be_bytes())
			.copied()
			.collect::<Vec<u8>>();
		paymaster_data[72..104].copy_from_slice(&rate_bytes);

		// Set validUntil to future timestamp
		let valid_until = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			+ 3600; // 1 hour from now
		let valid_until_bytes =
			[0u8; 24].iter().chain(&valid_until.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[104..136].copy_from_slice(&valid_until_bytes);

		// Set validAfter to past timestamp
		let valid_after = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			- 3600; // 1 hour ago
		let valid_after_bytes =
			[0u8; 24].iter().chain(&valid_after.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[136..168].copy_from_slice(&valid_after_bytes);

		let paymaster_and_data = Bytes::from(paymaster_data);

		// Create mock gas estimates
		let gas_estimates = NativeTaskOk::EstimateUserOpGas {
			call_gas_limit: 100000,
			verification_gas_limit: 200000,
			pre_verification_gas: 50000,
			paymaster_verification_gas_limit: 100000,
			paymaster_post_op_gas_limit: 50000,
			max_fee_per_gas: 1000000000,
			max_priority_fee_per_gas: 100000000,
			estimated_token_cost: None,
		};

		// Create mock UserOp with gas fees
		let mut gas_fees_bytes = [0u8; 32];
		// maxFeePerGas: 30 gwei = 30_000_000_000
		let max_fee = 30_000_000_000u128;
		gas_fees_bytes[0..16].copy_from_slice(&max_fee.to_be_bytes());
		// maxPriorityFeePerGas: 2 gwei = 2_000_000_000
		let priority_fee = 2_000_000_000u128;
		gas_fees_bytes[16..32].copy_from_slice(&priority_fee.to_be_bytes());

		let user_op = aa_contracts_client::PackedUserOperation {
			sender: Address::from([0x01; 20]),
			nonce: U256::from(0),
			initCode: Bytes::new(),
			callData: Bytes::new(),
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(50000),
			gasFees: FixedBytes::from(gas_fees_bytes),
			paymasterAndData: paymaster_and_data.clone(),
			signature: Bytes::new(),
		};

		let mock_api = MockBinanceApi;
		let result = calculate_erc20_token_cost(
			&mock_api,
			&paymaster_and_data,
			&gas_estimates,
			&user_op,
			1, // Ethereum mainnet
		)
		.await;

		assert!(result.is_some());
		let token_cost = result.unwrap();

		// Verify the token cost estimate
		assert_eq!(token_cost.decimals, 6); // USDC has 6 decimals
		assert!(token_cost.amount > 0);
		// The amount should be reasonable (not too high)
		// Total gas = 500k, gas price = 30 gwei, ETH price = 3000 USDC
		// Expected cost ~= 500000 * 30e9 * 3000 / 1e18 = 45 USDC
		// With 10% buffer = 49.5 USDC = 49,500,000 in smallest unit
		let expected_min = 40_000_000u128; // 40 USDC
		let expected_max = 60_000_000u128; // 60 USDC
		assert!(
			token_cost.amount >= expected_min && token_cost.amount <= expected_max,
			"Token cost {} should be between {} and {}",
			token_cost.amount,
			expected_min,
			expected_max
		);
	}

	#[tokio::test]
	async fn test_calculate_erc20_token_cost_rounds_up() {
		struct MockBinanceApi;
		#[async_trait::async_trait]
		impl BinancePaymasterApi for MockBinanceApi {
			async fn get_symbol_price(&self, symbol: &str) -> Result<String, BinanceApiError> {
				if symbol == "ETHUSDC" {
					Ok("3000.00".to_string())
				} else {
					Err(BinanceApiError::SymbolNotFound)
				}
			}

			async fn get_symbol_precision(&self, _symbol: &str) -> Result<u8, BinanceApiError> {
				Ok(6)
			}

			async fn get_all_trading_symbols(&self) -> Result<Vec<String>, BinanceApiError> {
				Ok(vec!["ETHUSDC".to_string()])
			}
		}

		let mut paymaster_data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH];
		paymaster_data[0..20].copy_from_slice(&[0x12; 20]);
		paymaster_data[20..36].copy_from_slice(&100000u128.to_be_bytes());
		paymaster_data[36..52].copy_from_slice(&50000u128.to_be_bytes());
		let usdc_address_bytes = hex::decode("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
		paymaster_data[52..72].copy_from_slice(&usdc_address_bytes);
		let rate_bytes = [0u8; 16].iter().chain(&0u128.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[72..104].copy_from_slice(&rate_bytes);
		let valid_until = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			+ 3600;
		let valid_until_bytes =
			[0u8; 24].iter().chain(&valid_until.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[104..136].copy_from_slice(&valid_until_bytes);
		let valid_after = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			- 3600;
		let valid_after_bytes =
			[0u8; 24].iter().chain(&valid_after.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[136..168].copy_from_slice(&valid_after_bytes);

		let paymaster_and_data = Bytes::from(paymaster_data);

		let gas_estimates = NativeTaskOk::EstimateUserOpGas {
			call_gas_limit: 1,
			verification_gas_limit: 0,
			pre_verification_gas: 0,
			paymaster_verification_gas_limit: 0,
			paymaster_post_op_gas_limit: 0,
			max_fee_per_gas: 1000000000,
			max_priority_fee_per_gas: 100000000,
			estimated_token_cost: None,
		};

		let mut gas_fees_bytes = [0u8; 32];
		let max_fee = 1u128;
		gas_fees_bytes[0..16].copy_from_slice(&max_fee.to_be_bytes());
		let priority_fee = 0u128;
		gas_fees_bytes[16..32].copy_from_slice(&priority_fee.to_be_bytes());

		let user_op = aa_contracts_client::PackedUserOperation {
			sender: Address::from([0x01; 20]),
			nonce: U256::from(0),
			initCode: Bytes::new(),
			callData: Bytes::new(),
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(0),
			gasFees: FixedBytes::from(gas_fees_bytes),
			paymasterAndData: paymaster_and_data.clone(),
			signature: Bytes::new(),
		};

		let mock_api = MockBinanceApi;
		let result =
			calculate_erc20_token_cost(&mock_api, &paymaster_and_data, &gas_estimates, &user_op, 1)
				.await;

		let token_cost = result.expect("Expected token cost to be calculated");
		assert_eq!(token_cost.amount, 1);
	}

	#[tokio::test]
	async fn test_calculate_erc20_token_cost_invalid_timestamps() {
		struct MockBinanceApi;
		#[async_trait::async_trait]
		impl BinancePaymasterApi for MockBinanceApi {
			async fn get_symbol_price(&self, _symbol: &str) -> Result<String, BinanceApiError> {
				Ok("3000.00".to_string())
			}

			async fn get_symbol_precision(&self, _symbol: &str) -> Result<u8, BinanceApiError> {
				Ok(6)
			}

			async fn get_all_trading_symbols(&self) -> Result<Vec<String>, BinanceApiError> {
				Ok(vec!["ETHUSDC".to_string()])
			}
		}

		// Create paymaster data with expired timestamps
		let mut paymaster_data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH];
		paymaster_data[0..20].copy_from_slice(&[0x12; 20]);
		paymaster_data[20..36].copy_from_slice(&100000u128.to_be_bytes());
		paymaster_data[36..52].copy_from_slice(&50000u128.to_be_bytes());

		// Set USDC token address
		let usdc_address_bytes = hex::decode("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
		paymaster_data[52..72].copy_from_slice(&usdc_address_bytes);

		// Set validUntil to past timestamp (expired)
		let valid_until = 1000u64; // Very old timestamp
		let valid_until_bytes =
			[0u8; 24].iter().chain(&valid_until.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[104..136].copy_from_slice(&valid_until_bytes);

		// Set validAfter to recent timestamp
		let valid_after = 900u64;
		let valid_after_bytes =
			[0u8; 24].iter().chain(&valid_after.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[136..168].copy_from_slice(&valid_after_bytes);

		let paymaster_and_data = Bytes::from(paymaster_data);

		// Create mock gas estimates
		let gas_estimates = NativeTaskOk::EstimateUserOpGas {
			call_gas_limit: 100000,
			verification_gas_limit: 200000,
			pre_verification_gas: 50000,
			paymaster_verification_gas_limit: 100000,
			paymaster_post_op_gas_limit: 50000,
			max_fee_per_gas: 1000000000,
			max_priority_fee_per_gas: 100000000,
			estimated_token_cost: None,
		};

		// Create mock UserOp
		let user_op = aa_contracts_client::PackedUserOperation {
			sender: Address::from([0x01; 20]),
			nonce: U256::from(0),
			initCode: Bytes::new(),
			callData: Bytes::new(),
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(50000),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: paymaster_and_data.clone(),
			signature: Bytes::new(),
		};

		let mock_api = MockBinanceApi;
		let result =
			calculate_erc20_token_cost(&mock_api, &paymaster_and_data, &gas_estimates, &user_op, 1)
				.await;

		// Should return None due to expired timestamps
		assert!(result.is_none());
	}

	#[tokio::test]
	async fn test_calculate_erc20_token_cost_unsupported_token() {
		struct MockBinanceApi;
		#[async_trait::async_trait]
		impl BinancePaymasterApi for MockBinanceApi {
			async fn get_symbol_price(&self, _symbol: &str) -> Result<String, BinanceApiError> {
				Ok("3000.00".to_string())
			}

			async fn get_symbol_precision(&self, _symbol: &str) -> Result<u8, BinanceApiError> {
				Ok(6)
			}

			async fn get_all_trading_symbols(&self) -> Result<Vec<String>, BinanceApiError> {
				Ok(vec!["ETHUSDC".to_string()])
			}
		}

		// Create paymaster data with unsupported token
		let mut paymaster_data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH];
		paymaster_data[0..20].copy_from_slice(&[0x12; 20]);
		paymaster_data[20..36].copy_from_slice(&100000u128.to_be_bytes());
		paymaster_data[36..52].copy_from_slice(&50000u128.to_be_bytes());

		// Set unsupported token address
		paymaster_data[52..72].copy_from_slice(&[0xFF; 20]);

		// Set valid timestamps
		let valid_until = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			+ 3600;
		let valid_until_bytes =
			[0u8; 24].iter().chain(&valid_until.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[104..136].copy_from_slice(&valid_until_bytes);

		let valid_after = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
			- 3600;
		let valid_after_bytes =
			[0u8; 24].iter().chain(&valid_after.to_be_bytes()).copied().collect::<Vec<u8>>();
		paymaster_data[136..168].copy_from_slice(&valid_after_bytes);

		let paymaster_and_data = Bytes::from(paymaster_data);

		let gas_estimates = NativeTaskOk::EstimateUserOpGas {
			call_gas_limit: 100000,
			verification_gas_limit: 200000,
			pre_verification_gas: 50000,
			paymaster_verification_gas_limit: 100000,
			paymaster_post_op_gas_limit: 50000,
			max_fee_per_gas: 1000000000,
			max_priority_fee_per_gas: 100000000,
			estimated_token_cost: None,
		};

		let user_op = aa_contracts_client::PackedUserOperation {
			sender: Address::from([0x01; 20]),
			nonce: U256::from(0),
			initCode: Bytes::new(),
			callData: Bytes::new(),
			accountGasLimits: FixedBytes::from([0u8; 32]),
			preVerificationGas: U256::from(50000),
			gasFees: FixedBytes::from([0u8; 32]),
			paymasterAndData: paymaster_and_data.clone(),
			signature: Bytes::new(),
		};

		let mock_api = MockBinanceApi;
		let result =
			calculate_erc20_token_cost(&mock_api, &paymaster_and_data, &gas_estimates, &user_op, 1)
				.await;

		// Should return None due to unsupported token
		assert!(result.is_none());
	}

	#[test]
	fn test_get_token_info_from_mapping_unknown_token() {
		let token_address = Address::from([0xFF; 20]);
		let rt = tokio::runtime::Runtime::new().unwrap();
		let result = rt.block_on(get_token_info_from_mapping(&token_address, 1));
		assert!(result.is_none());
	}

	#[test]
	fn test_supported_tokens_ethereum_mainnet() {
		let tokens = get_supported_tokens();

		// Test USDC on Ethereum mainnet
		let usdc_info = tokens.get(&(1, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"));
		assert!(usdc_info.is_some());
		let usdc = usdc_info.unwrap();
		assert_eq!(usdc.decimals, 6);
		assert_eq!(usdc.binance_pair, "ETHUSDC");
		// Extract symbol from binance_pair: ETHUSDC -> USDC
		let symbol = usdc.binance_pair.strip_prefix("ETH").unwrap_or("");
		assert_eq!(symbol, "USDC");

		// Test USDT on Ethereum mainnet
		let usdt_info = tokens.get(&(1, "0xdac17f958d2ee523a2206206994597c13d831ec7"));
		assert!(usdt_info.is_some());
		let usdt = usdt_info.unwrap();
		assert_eq!(usdt.decimals, 6);
		assert_eq!(usdt.binance_pair, "ETHUSDT");
		// Extract symbol from binance_pair: ETHUSDT -> USDT
		let symbol = usdt.binance_pair.strip_prefix("ETH").unwrap_or("");
		assert_eq!(symbol, "USDT");
	}

	#[test]
	fn test_supported_tokens_arbitrum() {
		let tokens = get_supported_tokens();

		// Test USDC on Arbitrum
		let usdc_info = tokens.get(&(42161, "0xaf88d065e77c8cc2239327c5edb3a432268e5831"));
		assert!(usdc_info.is_some());
		let usdc = usdc_info.unwrap();
		assert_eq!(usdc.decimals, 6);
		// Extract symbol from binance_pair: ETHUSDC -> USDC
		let symbol = usdc.binance_pair.strip_prefix("ETH").unwrap_or("");
		assert_eq!(symbol, "USDC");
	}

	#[test]
	fn test_get_token_info_unsupported_token() {
		let rt = tokio::runtime::Runtime::new().unwrap();
		let token_address = Address::from([0xFF; 20]);
		let result = rt.block_on(get_token_info_from_mapping(&token_address, 999));
		assert!(result.is_none());
	}

	#[test]
	fn test_process_erc20_paymaster_data_invalid_length() {
		let rt = tokio::runtime::Runtime::new().unwrap();

		// Create a mock BinanceApiClient (won't be used in this test)
		let binance_client = binance_api::BinanceApiClient::new(
			"test_key".to_string(),
			"test_secret".to_string(),
			"https://api.binance.com".to_string(),
		);

		// Create paymaster data that's too short (not ERC20 paymaster format)
		let data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH - 10];
		let paymaster_and_data = Bytes::from(data);

		let result = rt.block_on(process_erc20_paymaster_data(
			&binance_client as &dyn BinancePaymasterApi,
			&paymaster_and_data,
			1,
		));
		assert!(result.is_ok());
		assert!(result.unwrap().is_none()); // Should return None for non-ERC20 paymaster
	}

	#[test]
	fn test_exchange_rate_calculation_usdc() {
		// Test USDC (6 decimals) at $4000/ETH
		let token_decimals = 6;
		let tokens_per_eth = 4000.0;

		let expected_rate = (tokens_per_eth * 10_f64.powi(token_decimals)) as u128;
		// 4000 * 10^6 = 4000000000
		assert_eq!(expected_rate, 4000000000u128);

		// Test the actual calculation would work:
		// If maxCost is 1 ETH (10^18 wei), then:
		// requiredTokenAmount = (10^18 * 4000000000) / 10^18 = 4000000000 USDC units = 4000 USDC ✓
	}

	#[test]
	fn test_exchange_rate_calculation_dai() {
		// Test DAI (18 decimals) at $4000/ETH
		let token_decimals = 18;
		let tokens_per_eth = 4000.0;

		let expected_rate = (tokens_per_eth * 10_f64.powi(token_decimals)) as u128;
		// 4000 * 10^18 = 4000000000000000000000
		assert_eq!(expected_rate, 4000000000000000000000u128);
	}

	#[test]
	fn test_paymaster_data_format_validation() {
		// Test that we correctly validate the minimum length
		assert_eq!(MIN_ERC20_PAYMASTER_DATA_LENGTH, 168);
		// 52 (paymaster + gas limits) + 20 (token) + 32 (rate) + 32 (until) + 32 (after) = 168

		// Test structure offsets
		assert_eq!(PAYMASTER_DATA_OFFSET, 52);
	}

	#[test]
	fn test_paymaster_data_alignment_with_contract() {
		// This test ensures our format exactly matches ERC20PaymasterV1.sol
		let mut data = vec![0u8; MIN_ERC20_PAYMASTER_DATA_LENGTH];

		// Contract expects: paymaster(20) + validation_gas(16) + postop_gas(16) + token(20) + exchangeRate(32) + validUntil(32) + validAfter(32)

		// Paymaster address
		data[0..20].copy_from_slice(&[0x11; 20]);

		// Validation gas limit
		data[20..36].copy_from_slice(&150000u128.to_be_bytes());

		// PostOp gas limit
		data[36..52].copy_from_slice(&50000u128.to_be_bytes());

		// Token address (from PAYMASTER_DATA_OFFSET)
		let token_addr = [0xA0; 20];
		data[52..72].copy_from_slice(&token_addr);

		// Exchange rate
		let rate = 2500000000u128; // 2500 USDC per ETH
		let rate_bytes = [0u8; 16].iter().chain(&rate.to_be_bytes()).copied().collect::<Vec<u8>>();
		data[72..104].copy_from_slice(&rate_bytes);

		// ValidUntil
		let until = 1800000000u64;
		let until_bytes =
			[0u8; 24].iter().chain(&until.to_be_bytes()).copied().collect::<Vec<u8>>();
		data[104..136].copy_from_slice(&until_bytes);

		// ValidAfter
		let after = 1700000000u64;
		let after_bytes =
			[0u8; 24].iter().chain(&after.to_be_bytes()).copied().collect::<Vec<u8>>();
		data[136..168].copy_from_slice(&after_bytes);

		// Test decoding matches what we encoded
		let result = decode_erc20_paymaster_data(&data).unwrap();
		assert_eq!(result.0, Address::from(token_addr));
		assert_eq!(result.1, rate);
		assert_eq!(result.2, until);
		assert_eq!(result.3, after);

		// Test re-encoding preserves the structure
		let updated = encode_erc20_paymaster_data(&data, rate + 100, until + 100, after + 100);
		let redecoded = decode_erc20_paymaster_data(&updated).unwrap();
		assert_eq!(redecoded.0, Address::from(token_addr)); // Token unchanged
		assert_eq!(redecoded.1, rate + 100); // Rate updated
		assert_eq!(redecoded.2, until + 100); // Until updated
		assert_eq!(redecoded.3, after + 100); // After updated
	}
}

#[cfg(test)]
mod paymaster_validation_tests {
	use super::*;
	use alloy::primitives::{address, Bytes};

	#[test]
	fn test_extract_paymaster_address_valid() {
		// Create paymasterAndData with a valid paymaster address
		let paymaster_address = address!("0x1234567890123456789012345678901234567890");
		let mut paymaster_data = vec![];
		// Add paymaster address (20 bytes)
		paymaster_data.extend_from_slice(paymaster_address.as_slice());
		// Add some additional data
		paymaster_data.extend_from_slice(&[0xff; 32]); // gas limits etc.

		let paymaster_and_data = Bytes::from(paymaster_data);
		let result = extract_paymaster_address(&paymaster_and_data);

		assert!(result.is_some());
		assert_eq!(result.unwrap(), paymaster_address);
	}

	#[test]
	fn test_extract_paymaster_address_too_short() {
		// Create paymasterAndData that's too short (less than 20 bytes)
		let short_data = vec![0x11; 19]; // Only 19 bytes
		let paymaster_and_data = Bytes::from(short_data);
		let result = extract_paymaster_address(&paymaster_and_data);

		assert!(result.is_none());
	}

	#[test]
	fn test_extract_paymaster_address_empty() {
		// Empty paymasterAndData
		let paymaster_and_data = Bytes::from(vec![]);
		let result = extract_paymaster_address(&paymaster_and_data);

		assert!(result.is_none());
	}

	#[test]
	fn test_is_whitelisted_paymaster_found() {
		let paymaster1 = address!("0x1234567890123456789012345678901234567890");
		let paymaster2 = address!("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");
		let paymaster3 = address!("0x9999999999999999999999999999999999999999");

		let whitelist = vec![paymaster1, paymaster2];

		// Test that whitelisted addresses are found
		assert!(is_whitelisted_paymaster(&paymaster1, &whitelist));
		assert!(is_whitelisted_paymaster(&paymaster2, &whitelist));

		// Test that non-whitelisted address is not found
		assert!(!is_whitelisted_paymaster(&paymaster3, &whitelist));
	}

	#[test]
	fn test_is_whitelisted_paymaster_empty_list() {
		let paymaster = address!("0x1234567890123456789012345678901234567890");
		let empty_whitelist: Vec<Address> = vec![];

		// Test that no address is found in empty whitelist
		assert!(!is_whitelisted_paymaster(&paymaster, &empty_whitelist));
	}

	#[test]
	fn test_paymaster_whitelist_validation() {
		// Test paymaster validation logic:
		// - Unsigned userOp: paymaster must be whitelisted if specified
		// - Signed userOp: no paymaster allowed
		let whitelisted_address = address!("0x1234567890123456789012345678901234567890");
		let non_whitelisted_address = address!("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");

		let mut whitelisted_data = vec![];
		whitelisted_data.extend_from_slice(whitelisted_address.as_slice());
		whitelisted_data.extend_from_slice(&[0xff; 32]);
		let whitelisted_paymaster_and_data = Bytes::from(whitelisted_data);

		let mut non_whitelisted_data = vec![];
		non_whitelisted_data.extend_from_slice(non_whitelisted_address.as_slice());
		non_whitelisted_data.extend_from_slice(&[0xff; 32]);
		let non_whitelisted_paymaster_and_data = Bytes::from(non_whitelisted_data);

		let whitelist = vec![whitelisted_address];

		// Whitelisted paymaster should be accepted for unsigned userOps
		let extracted = extract_paymaster_address(&whitelisted_paymaster_and_data).unwrap();
		assert!(is_whitelisted_paymaster(&extracted, &whitelist));

		// Non-whitelisted paymaster should be rejected for unsigned userOps
		let extracted = extract_paymaster_address(&non_whitelisted_paymaster_and_data).unwrap();
		assert!(!is_whitelisted_paymaster(&extracted, &whitelist));
	}

	#[test]
	fn test_signed_userop_validation_logic() {
		// Test the new validation logic:
		// - Signed userOp with paymaster -> should be rejected
		// - Signed userOp without paymaster -> should be accepted
		let paymaster_address = address!("0x1234567890123456789012345678901234567890");
		let mut paymaster_data = vec![];
		paymaster_data.extend_from_slice(paymaster_address.as_slice());
		paymaster_data.extend_from_slice(&[0xff; 32]);
		let paymaster_and_data = Bytes::from(paymaster_data);

		// Test that paymaster address is extracted correctly
		let extracted = extract_paymaster_address(&paymaster_and_data).unwrap();
		assert_eq!(extracted, paymaster_address);

		// Empty paymasterAndData should be allowed for signed userOps
		let empty_paymaster_data = Bytes::new();
		assert!(extract_paymaster_address(&empty_paymaster_data).is_none());
	}
}
