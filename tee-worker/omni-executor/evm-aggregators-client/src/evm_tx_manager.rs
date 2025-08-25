// Copyright 2020-2025 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use crate::common::{
	is_native_token, CreateMarketTx, CROSS_SERVICE_FEE_BPS, CROSS_SERVICE_FEE_PERCENT,
	DECIMALS_TO_VALUE, INCH_DEX_MAPPINGS, INCH_SWAP_APPROVE_ADDRESS, KYBER_DEX_MAPPINGS,
	KYBER_SWAP_APPROVE_ADDRESS, NATIVE_TOKEN_ADDRESS, OKX_DEX_MAPPINGS, OKX_SWAP_APPROVE_ADDRESS,
	SERVICE_FEE_BPS, SERVICE_FEE_PERCENT,
};
use crate::errors::{ClientError, ClientResult};
use crate::inch_client::client::InchSwap;
use crate::inch_client::types::{convert_slippage_to_inch, SwapRequestBuilder};
use crate::kyber_client::client::KyberSwap;
use crate::okx_client::client::OkxSwap;
use ethereum_rpc::RpcProvider;
use rust_decimal::prelude::{Decimal, FromPrimitive, ToPrimitive};
use std::ops::Div;
use std::str::FromStr;
use std::sync::Arc;

use crate::kyber_client::types::{
	GetSwapRouteRequest, SwapRequestBuilder as KyberSwapRequestBuilder,
};
use crate::okx_client::types::{
	convert_slippage_to_okx, get_okx_gas_level, SwapRequestBuilder as OkxSwapRequestBuilder,
};
use alloy::primitives::Uint;
use alloy::{
	primitives::{Address, TxKind},
	rpc::types::{TransactionInput, TransactionRequest},
};
use async_trait::async_trait;
use ethereum_rpc::client::EthereumClient;
use hex::FromHex;
use log::error;
use pumpx::methods::common::SwapType;
use pumpx::methods::create_market_order_unsigned_tx::CreateMarketOrderUnsignedTxBody;
use pumpx::methods::get_pair_info_by_token::GetPairInfoByTokenRequestBody;
use pumpx::methods::get_token_info::GetTokenInfoBody;
use pumpx::{PumpxApi, PumpxApiClient};

/// EVM Transaction Manager
pub struct EvmTxManager<
	EthClient: RpcProvider + ?Sized,
	InchClient: InchSwap + ?Sized,
	KyberClient: KyberSwap + ?Sized,
	OkxClient: OkxSwap + ?Sized,
> {
	pub okx_client: Arc<OkxClient>,
	pub kyber_client: Arc<KyberClient>,
	pub inch_client: Arc<InchClient>,
	pub eth_client: Arc<EthClient>,
	pub fee_receiver: String,
}

#[async_trait]
pub trait ConstructEvmTx: Send + Sync {
	async fn construct_inch_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> ClientResult<TransactionRequest>;
	async fn construct_okx_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> ClientResult<TransactionRequest>;
	async fn construct_kyber_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> ClientResult<TransactionRequest>;
}

#[async_trait]
impl<EthClient, InchClient, KyberClient, OkxClient> ConstructEvmTx
	for EvmTxManager<EthClient, InchClient, KyberClient, OkxClient>
where
	EthClient: RpcProvider + ?Sized,
	InchClient: InchSwap + ?Sized,
	KyberClient: KyberSwap + ?Sized,
	OkxClient: OkxSwap + ?Sized,
{
	async fn construct_inch_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> ClientResult<TransactionRequest> {
		let chain_id = create_market_tx.chain_id;

		// Safely access INCH_DEX_MAPPINGS to prevent panics
		let chain_map = INCH_DEX_MAPPINGS
			.get(&chain_id)
			.ok_or(ClientError::UnsupportedChainId { chain_id })?;

		let dex_id = chain_map.get(&create_market_tx.trade_pool_name).ok_or_else(|| {
			ClientError::UnsupportedTradePool {
				pool_name: create_market_tx.trade_pool_name.clone(),
				chain_id,
			}
		})?;

		// Determine token addresses based on native token handling
		let (from_token_address, to_token_address) =
			if is_native_token(create_market_tx.in_token_ca.as_bytes()) {
				// Case 1: Selling native token (ETH → USDC)
				(NATIVE_TOKEN_ADDRESS.to_string(), create_market_tx.out_token_ca.clone())
			} else {
				// Case 2: Buying native token (USDC → ETH)
				(create_market_tx.in_token_ca.clone(), NATIVE_TOKEN_ADDRESS.to_string())
			};

		// Use SwapRequestBuilder with validation
		let swap_request = SwapRequestBuilder::new()
			.chain_id(chain_id)
			.amount(amount_decimal.to_string())
			.from_token_address(from_token_address)
			.to_token_address(to_token_address)
			.slippage(convert_slippage_to_inch(create_market_tx.slippage))
			.user_wallet_address(create_market_tx.user_wallet_address.clone())
			.fee_percent(SERVICE_FEE_PERCENT.to_string())
			.referrer(self.fee_receiver.clone())
			.dex_ids(dex_id.to_string())
			.build()?;

		let swap_response = self
			.inch_client
			.swap(chain_id, swap_request)
			.await
			.map_err(|e| ClientError::Network { message: format!("1inch API error: {}", e) })?;

		let (data, to, value, gas) = swap_response.get_transaction_data()?;

		let gas_price = self.get_gas_price_by_level(chain_id, create_market_tx.gas_type).await?;
		let gas_price = gas_price.to_u128().ok_or(ClientError::GasCalculation {
			reason: "Gas price conversion overflow".to_string(),
		})?;

		let tx = TransactionRequest {
			nonce: Some(nonce),
			value: Some(value),
			to: Some(TxKind::Call(to)),
			input: TransactionInput { data: Some(data.into()), ..Default::default() },
			gas: Some(gas),
			gas_price: Some(gas_price),
			..Default::default()
		};
		Ok(tx)
	}

	async fn construct_okx_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> ClientResult<TransactionRequest> {
		let chain_id = create_market_tx.chain_id;
		let mut fee_bps = SERVICE_FEE_PERCENT.to_string();
		if create_market_tx.is_pre_cross {
			fee_bps = CROSS_SERVICE_FEE_PERCENT.to_string();
		}

		// Safely access OKX_DEX_MAPPINGS to prevent panics
		let dex_id = OKX_DEX_MAPPINGS.get(&create_market_tx.trade_pool_name).ok_or_else(|| {
			ClientError::UnsupportedDex {
				dex_name: "OKX".to_string(),
				pool_name: create_market_tx.trade_pool_name.clone(),
			}
		})?;

		// Determine token addresses and referrer setup based on native token handling
		let (from_token_address, to_token_address, from_referrer, to_referrer) =
			if is_native_token(create_market_tx.in_token_ca.as_bytes()) {
				// Case 1: Selling native token (ETH → USDC)
				(
					NATIVE_TOKEN_ADDRESS.to_string(),
					create_market_tx.out_token_ca.clone(),
					Some(self.fee_receiver.clone()),
					None,
				)
			} else {
				// Case 2: Buying native token (USDC → ETH)
				(
					create_market_tx.in_token_ca.clone(),
					NATIVE_TOKEN_ADDRESS.to_string(),
					None,
					Some(self.fee_receiver.clone()),
				)
			};

		// Use OkxSwapRequestBuilder with validation
		let mut builder = OkxSwapRequestBuilder::new()
			.chain_id(chain_id.to_string())
			.amount(amount_decimal.to_string())
			.from_token_address(from_token_address)
			.to_token_address(to_token_address)
			.slippage(convert_slippage_to_okx(create_market_tx.slippage).to_string())
			.user_wallet_address(create_market_tx.user_wallet_address)
			.fee_percent(fee_bps)
			.gas_level(get_okx_gas_level(create_market_tx.gas_type).to_string())
			.dex_ids(dex_id.to_string());

		// Set referrer addresses based on native token handling
		if let Some(from_ref) = from_referrer {
			builder = builder.from_token_referrer_wallet_address(from_ref);
		}
		if let Some(to_ref) = to_referrer {
			builder = builder.to_token_referrer_wallet_address(to_ref);
		}

		let swap_request = builder.build()?;

		let swap_response = self
			.okx_client
			.swap(swap_request)
			.await
			.map_err(|e| ClientError::Network { message: format!("OKX API error: {}", e) })?;

		let (data, to, value, gas) = swap_response.get_transaction_data()?;

		let gas_price = self.get_gas_price_by_level(chain_id, create_market_tx.gas_type).await?;
		let gas_price = gas_price.to_u128().ok_or(ClientError::GasCalculation {
			reason: "Gas price conversion overflow".to_string(),
		})?;

		let tx = TransactionRequest {
			nonce: Some(nonce),
			value: Some(value),
			to: Some(TxKind::Call(to)),
			input: TransactionInput { data: Some(data.into()), ..Default::default() },
			gas: Some(gas),
			gas_price: Some(gas_price),
			..Default::default()
		};
		Ok(tx)
	}

	async fn construct_kyber_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> ClientResult<TransactionRequest> {
		let chain_id = create_market_tx.chain_id;
		let mut fee_bps = SERVICE_FEE_BPS.to_string();
		if create_market_tx.is_pre_cross {
			fee_bps = CROSS_SERVICE_FEE_BPS.to_string();
		}

		// Safely access KYBER_DEX_MAPPINGS to prevent panics
		let dex_id =
			KYBER_DEX_MAPPINGS
				.get(create_market_tx.trade_pool_name.as_str())
				.ok_or_else(|| ClientError::UnsupportedDex {
					dex_name: "Kyber".to_string(),
					pool_name: create_market_tx.trade_pool_name.clone(),
				})?;

		let mut swap_route_request = GetSwapRouteRequest {
			chain_id,
			amount: amount_decimal.to_string(),
			from_token_address: create_market_tx.in_token_ca.clone(),
			to_token_address: create_market_tx.out_token_ca.clone(),
			fee_bps,
			referrer: self.fee_receiver.clone(),
			dex_ids: dex_id.to_string(),
			is_from_token_referrer: false,
		};

		// Handle native token swaps: Native tokens (ETH, BNB, MATIC) require special handling
		// because they are not ERC-20 contracts and need to be represented using the special
		// NATIVE_TOKEN_ADDRESS marker (0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee) for DEX APIs.
		//
		// Two scenarios:
		// 1. Selling native token (ETH → USDC): Use NATIVE_TOKEN_ADDRESS for from_token and collect fees from input
		// 2. Buying native token (USDC → ETH): Use NATIVE_TOKEN_ADDRESS for to_token and collect fees from output
		if is_native_token(create_market_tx.in_token_ca.as_bytes()) {
			// Case 1: User is selling native token (e.g., ETH → USDC)
			// Replace wrapped token address (e.g., WETH) with native marker
			swap_route_request.from_token_address = NATIVE_TOKEN_ADDRESS.to_string();
			// Set fee collection from the native token being sold (more gas efficient)
			swap_route_request.is_from_token_referrer = true;
		} else {
			// Case 2: User is buying native token (e.g., USDC → ETH)
			// Set the output token to use the native marker
			swap_route_request.to_token_address = NATIVE_TOKEN_ADDRESS.to_string();
			// Keep default fee collection behavior (from output token)
		}

		let swap_route_response = self
			.kyber_client
			.get_swap_route(chain_id, swap_route_request)
			.await
			.map_err(|e| ClientError::Network {
				message: format!("Kyber route API error: {}", e),
			})?;

		// Calculate deadline (current time + 60 seconds)
		let deadline = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map_err(|e| ClientError::SystemTime { message: e.to_string() })?
			.as_secs()
			+ 60;

		// Use KyberSwapRequestBuilder with validation
		let swap_request = KyberSwapRequestBuilder::new()
			.route_summary(swap_route_response.route_summary)
			.sender(create_market_tx.user_wallet_address.clone())
			.recipient(create_market_tx.user_wallet_address.clone())
			.deadline(deadline as i64)
			.slippage_bps(create_market_tx.slippage as i64)
			.enable_gas_estimation(true)
			.ignore_capped_slippage(true)
			.build()?;

		let swap_response = self.kyber_client.swap(chain_id, swap_request).await.map_err(|e| {
			ClientError::Network { message: format!("Kyber swap API error: {}", e) }
		})?;

		let (data, to, value, gas) = swap_response.get_transaction_data()?;

		let gas_price = self.get_gas_price_by_level(chain_id, create_market_tx.gas_type).await?;
		let gas_price = gas_price.to_u128().ok_or(ClientError::GasCalculation {
			reason: "Gas price conversion overflow".to_string(),
		})?;

		let tx = TransactionRequest {
			nonce: Some(nonce),
			value: Some(value),
			to: Some(TxKind::Call(to)),
			input: TransactionInput { data: Some(data.into()), ..Default::default() },
			gas: Some(gas),
			gas_price: Some(gas_price),
			..Default::default()
		};

		Ok(tx)
	}
}

impl<EthClient, InchClient, KyberClient, OkxClient>
	EvmTxManager<EthClient, InchClient, KyberClient, OkxClient>
where
	EthClient: RpcProvider + ?Sized,
	InchClient: InchSwap + ?Sized,
	KyberClient: KyberSwap + ?Sized,
	OkxClient: OkxSwap + ?Sized,
{
	pub async fn get_gas_price_by_level(&self, chain_id: u64, level: i32) -> ClientResult<Decimal> {
		let gas_price =
			self.okx_client
				.get_gas_price(chain_id)
				.await
				.map_err(|e| ClientError::Network {
					message: format!("OKX gas price API error: {}", e),
				})?;

		match level {
			1_i32 => Decimal::from_str(&gas_price.min)
				.map_err(|_| ClientError::InvalidDecimal { value: gas_price.min.clone() }),
			2_i32 => Decimal::from_str(&gas_price.normal)
				.map_err(|_| ClientError::InvalidDecimal { value: gas_price.normal.clone() }),
			3_i32 => Decimal::from_str(&gas_price.max)
				.map_err(|_| ClientError::InvalidDecimal { value: gas_price.max.clone() }),
			_ => {
				error!("Invalid gas level: {}", level);
				Err(ClientError::InvalidGasLevel { level: level.to_string() })
			},
		}
	}
}

impl<EthClient, InchClient, KyberClient, OkxClient>
	EvmTxManager<EthClient, InchClient, KyberClient, OkxClient>
where
	EthClient: RpcProvider<Addr = Address> + ?Sized + EthereumClient,
	InchClient: InchSwap + ?Sized,
	KyberClient: KyberSwap + ?Sized,
	OkxClient: OkxSwap + ?Sized,
{
	#[allow(dead_code)]
	async fn construct_unsigned_market_tx(
		&self,
		tx: CreateMarketTx,
	) -> Result<Vec<TransactionRequest>, ClientError> {
		let amount_decimal = Decimal::from_str(&tx.amount_in)
			.map_err(|_| ClientError::InvalidDecimal { value: tx.amount_in.clone() })?;
		let multiplier = DECIMALS_TO_VALUE
			.get(&tx.in_decimal)
			.cloned()
			.ok_or(ClientError::UnsupportedDecimals { decimals: tx.in_decimal })?;
		let amount_decimal = amount_decimal * Decimal::from(multiplier);

		let from = Address::from_hex(&tx.user_wallet_address)
			.map_err(|_| ClientError::InvalidAddress { address: tx.user_wallet_address.clone() })?;

		let mut balance = self.eth_client.get_balance(from).await.map_err(|_| {
			ClientError::Network { message: format!("Could not find balance for {}", from) }
		})?;

		let mut nonce =
			self.eth_client
				.get_pending_nonce(from)
				.await
				.map_err(|_| ClientError::Network {
					message: format!("Could not find nonce for {}", from),
				})?;

		let is_buy = is_native_token(tx.in_token_ca.as_bytes());
		let platform = Platform::KyberSwap;

		let amount_u128 = amount_decimal.to_u128().ok_or(ClientError::AmountOverflow)?;

		let amount = Uint::try_from(amount_u128).map_err(|e| ClientError::ConversionError {
			message: format!("Failed to convert amount_u128 to Uint: {:?}", e),
		})?;

		let mut transactions: Vec<TransactionRequest> = Vec::new();
		if is_buy {
			if balance < amount {
				return Err(ClientError::InsufficientBalance {
					required: amount.to_string(),
					available: balance.to_string(),
				});
			}
			balance -= amount;
		} else {
			let approve_addr: Address = match platform {
				Platform::KyberSwap => {
					Address::from_hex(KYBER_SWAP_APPROVE_ADDRESS).map_err(|_| {
						ClientError::InvalidAddress {
							address: KYBER_SWAP_APPROVE_ADDRESS.to_string(),
						}
					})?
				},
				Platform::Inch => Address::from_hex(INCH_SWAP_APPROVE_ADDRESS).map_err(|_| {
					ClientError::InvalidAddress { address: INCH_SWAP_APPROVE_ADDRESS.to_string() }
				})?,
				Platform::Okx => Address::from_hex(OKX_SWAP_APPROVE_ADDRESS).map_err(|_| {
					ClientError::InvalidAddress { address: OKX_SWAP_APPROVE_ADDRESS.to_string() }
				})?,
			};

			let approve_tx = self
				.eth_client
				.construct_approve_erc20_tx(
					approve_addr,
					amount,
					Address::from_hex(&tx.in_token_ca).map_err(|_| {
						ClientError::InvalidAddress { address: tx.in_token_ca.clone() }
					})?,
					nonce,
				)
				.await
				.map_err(|_| ClientError::Network {
					message: "Failed to create approve tx".to_string(),
				})?;

			transactions.push(approve_tx);
			nonce += 1;
		}

		let unsigned_tx: TransactionRequest = match platform {
			Platform::KyberSwap => self.construct_kyber_tx(tx, nonce, amount_decimal).await?,
			Platform::Inch => self.construct_inch_tx(tx, nonce, amount_decimal).await?,
			Platform::Okx => self.construct_okx_tx(tx, nonce, amount_decimal).await?,
		};

		let gas = unsigned_tx
			.gas
			.ok_or_else(|| ClientError::TransactionFieldNotSet { field: "gas".to_string() })?;

		let gas_price_u64 = unsigned_tx
			.gas_price
			.ok_or_else(|| ClientError::TransactionFieldNotSet { field: "gas_price".to_string() })?
			.to_u64()
			.ok_or_else(|| ClientError::ConversionError {
				message: "Failed to convert gas price to u64".to_string(),
			})?;

		let gas_fee_u128 = gas.checked_mul(gas_price_u64).ok_or_else(|| {
			ClientError::GasCalculation { reason: "Gas fee multiplication overflow".to_string() }
		})?;

		let gas_fee = Uint::try_from(gas_fee_u128).map_err(|e| ClientError::ConversionError {
			message: format!("Failed to convert gas fee to Uint: {:?}", e),
		})?;

		if balance < gas_fee {
			return Err(ClientError::InsufficientBalance {
				required: gas_fee.to_string(),
				available: balance.to_string(),
			});
		}

		transactions.push(unsigned_tx);
		Ok(transactions)
	}
}

/// Validates order parameters and pair info
async fn validate_order_params(
	order: &CreateMarketOrderUnsignedTxBody,
	pumpx_api_client: &PumpxApiClient,
	access_token: &str,
) -> ClientResult<pumpx::methods::get_pair_info_by_token::GetPairInfoByTokenResponse> {
	// Get pair info from pumpx api
	let pair_info_response = pumpx_api_client
		.get_pair_info_by_token(
			access_token,
			GetPairInfoByTokenRequestBody {
				chain_id: order.chain_id as i64,
				token_address: order.token_ca.clone(),
			},
		)
		.await
		.map_err(|e| {
			log::error!("Failed to get pair info: {:?}", e);
			ClientError::Network { message: "Failed to get pair info".to_string() }
		})?;

	// Validate FDV is not zero
	if pair_info_response.data.fdv == 0.0 {
		error!("The pair fdv is 0");
		return Err(ClientError::Network { message: "FDV cannot be zero".to_string() });
	}

	Ok(pair_info_response.data)
}

/// Calculates token price information from pair info
fn calculate_token_prices(
	pair_info: &pumpx::methods::get_pair_info_by_token::GetPairInfoByTokenResponse,
) -> ClientResult<Decimal> {
	let base_token_price = Decimal::from_f64(pair_info.base_token_price).ok_or_else(|| {
		ClientError::InvalidDecimal { value: pair_info.base_token_price.to_string() }
	})?;
	let token_price_usd_decimal = Decimal::from_f64(pair_info.token_price)
		.ok_or_else(|| ClientError::InvalidDecimal { value: pair_info.token_price.to_string() })?;

	Ok(token_price_usd_decimal.div(&base_token_price))
}

/// Determines the correct wallet address from order
fn determine_wallet_address(order: &CreateMarketOrderUnsignedTxBody) -> String {
	if !order.recipient_address.is_empty() && order.recipient_address != order.address {
		order.recipient_address.clone()
	} else {
		order.address.clone()
	}
}

/// Determines token addresses and decimals based on swap type
fn determine_token_config(
	order: &CreateMarketOrderUnsignedTxBody,
	pair_info: &pumpx::methods::get_pair_info_by_token::GetPairInfoByTokenResponse,
) -> (String, String, u8, u8) {
	let mut in_token_address = pair_info.base_token_address.clone();
	let mut out_token_address = pair_info.token_address.clone();
	let mut in_token_decimal = pair_info.base_token_decimal as u8;
	let mut out_token_decimal = pair_info.token_decimal as u8;

	if order.swap_type == SwapType::Sell {
		(in_token_address, out_token_address, in_token_decimal, out_token_decimal) =
			(out_token_address, in_token_address, out_token_decimal, in_token_decimal);
	}

	(in_token_address, out_token_address, in_token_decimal, out_token_decimal)
}

/// Creates the market transaction parameters
fn build_create_market_tx(
	order: &CreateMarketOrderUnsignedTxBody,
	amount_decimal: &Decimal,
	pair_info: &pumpx::methods::get_pair_info_by_token::GetPairInfoByTokenResponse,
	wallet_address: String,
	in_token_address: String,
	out_token_address: String,
	in_token_decimal: u8,
) -> CreateMarketTx {
	CreateMarketTx {
		chain_id: order.chain_id as u64,
		user_wallet_address: wallet_address,
		amount_in: amount_decimal.to_string(),
		slippage: order.slippage,
		gas_type: order.gas_type.clone() as i32,
		trade_pool_name: pair_info.name.clone(),
		in_decimal: in_token_decimal,
		in_token_ca: in_token_address,
		out_token_ca: out_token_address,
		is_pre_cross: false,
	}
}

/// This is the entry point for constructing EVM Transaction
/// it takes the CreateMarketOrderUnsignedTxBody and returns Vec<Transaction>
pub async fn create_market_tx_unsigned<EthClient, InchClient, KyberClient, OkxClient>(
	order: CreateMarketOrderUnsignedTxBody,
	evm_tx_manager: EvmTxManager<EthClient, InchClient, KyberClient, OkxClient>,
	pumpx_api_client: PumpxApiClient,
	access_token: String,
) -> Result<Vec<TransactionRequest>, ClientError>
where
	EthClient: RpcProvider<Addr = Address> + ?Sized + EthereumClient,
	InchClient: InchSwap + ?Sized,
	KyberClient: KyberSwap + ?Sized,
	OkxClient: OkxSwap + ?Sized,
{
	// Parse amount
	let amount_decimal = Decimal::from_str(&order.amount_in).map_err(|e| {
		log::error!("Failed to parse amount '{}': {}", order.amount_in, e);
		ClientError::InvalidDecimal { value: order.amount_in.clone() }
	})?;

	// Validate order and get pair info
	let pair_info = validate_order_params(&order, &pumpx_api_client, &access_token).await?;

	// Calculate token prices (currently unused but available for future use)
	let _token_price_decimal = calculate_token_prices(&pair_info)?;

	// Determine wallet address
	let wallet_address = determine_wallet_address(&order);

	// Get token info (for potential future use)
	let _token_info = pumpx_api_client
		.get_token_info(
			access_token.as_str(),
			GetTokenInfoBody {
				chain_id: order.chain_id as i64,
				token_address: order.token_ca.clone(),
			},
		)
		.await
		.map_err(|e| {
			log::error!("Failed to get token info: {:?}", e);
			ClientError::Network { message: "Failed to get token info".to_string() }
		})?
		.data;

	// Determine token configuration based on swap type
	let (in_token_address, out_token_address, in_token_decimal, _out_token_decimal) =
		determine_token_config(&order, &pair_info);

	// Build market transaction parameters
	let create_market_tx = build_create_market_tx(
		&order,
		&amount_decimal,
		&pair_info,
		wallet_address,
		in_token_address,
		out_token_address,
		in_token_decimal,
	);

	// Construct the unsigned transaction
	evm_tx_manager.construct_unsigned_market_tx(create_market_tx).await
}

pub enum Platform {
	Inch,
	Okx,
	KyberSwap,
}

/// Helper function to create the native token swap route request logic.
/// This function encapsulates the native token handling logic for testing purposes.
///
/// # Arguments
/// * `in_token_ca` - Input token contract address
/// * `base_request` - Base swap route request to modify
///
/// # Returns
/// Modified swap route request with proper native token handling
pub fn configure_native_token_swap(
	in_token_ca: &str,
	mut base_request: GetSwapRouteRequest,
) -> GetSwapRouteRequest {
	if is_native_token(in_token_ca.as_bytes()) {
		// Case 1: User is selling native token (e.g., ETH → USDC)
		base_request.from_token_address = NATIVE_TOKEN_ADDRESS.to_string();
		base_request.is_from_token_referrer = true;
	} else {
		// Case 2: User is buying native token (e.g., USDC → ETH)
		base_request.to_token_address = NATIVE_TOKEN_ADDRESS.to_string();
	}
	base_request
}

#[cfg(test)]
mod tests {
	use super::*;

	fn create_test_swap_route_request() -> GetSwapRouteRequest {
		GetSwapRouteRequest {
			chain_id: 1,
			amount: "1000000000000000000".to_string(), // 1 ETH in wei
			from_token_address: "".to_string(),        // Will be set by test
			to_token_address: "".to_string(),          // Will be set by test
			fee_bps: "100".to_string(),
			referrer: "0x742d35Cc6641b4Fc7b05cC38f69Cc8D7C2B6B444".to_string(),
			dex_ids: "uniswap".to_string(),
			is_from_token_referrer: false,
		}
	}

	#[test]
	fn test_native_token_swap_selling_eth() {
		// Test Case 1: Selling ETH for USDC (ETH → USDC)
		let weth_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH on Ethereum
		let usdc_address = "0xA0b86a33E6417c8f7851efA37A9f7F1A5d8C8f6E"; // USDC

		let mut base_request = create_test_swap_route_request();
		base_request.from_token_address = weth_address.to_string();
		base_request.to_token_address = usdc_address.to_string();

		let result = configure_native_token_swap(weth_address, base_request);

		// Assertions for selling native token
		assert_eq!(
			result.from_token_address, NATIVE_TOKEN_ADDRESS,
			"When selling native token, from_token_address should be NATIVE_TOKEN_ADDRESS"
		);
		assert_eq!(
			result.to_token_address, usdc_address,
			"When selling native token, to_token_address should remain unchanged"
		);
		assert_eq!(
			result.is_from_token_referrer, true,
			"When selling native token, fees should be collected from input token"
		);
	}

	#[test]
	fn test_native_token_swap_buying_eth() {
		// Test Case 2: Buying ETH with USDC (USDC → ETH)
		let usdc_address = "0xA0b86a33E6417c8f7851efA37A9f7F1A5d8C8f6E"; // USDC
		let weth_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH on Ethereum

		let mut base_request = create_test_swap_route_request();
		base_request.from_token_address = usdc_address.to_string();
		base_request.to_token_address = weth_address.to_string();

		let result = configure_native_token_swap(usdc_address, base_request);

		// Assertions for buying native token
		assert_eq!(
			result.from_token_address, usdc_address,
			"When buying native token, from_token_address should remain unchanged"
		);
		assert_eq!(
			result.to_token_address, NATIVE_TOKEN_ADDRESS,
			"When buying native token, to_token_address should be NATIVE_TOKEN_ADDRESS"
		);
		assert_eq!(
			result.is_from_token_referrer, false,
			"When buying native token, fees should be collected from output token"
		);
	}

	#[test]
	fn test_native_token_swap_bsc_selling_bnb() {
		// Test BSC: Selling BNB for BUSD (BNB → BUSD)
		let wbnb_address = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"; // WBNB on BSC
		let busd_address = "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56"; // BUSD

		let mut base_request = create_test_swap_route_request();
		base_request.chain_id = 56; // BSC chain ID
		base_request.from_token_address = wbnb_address.to_string();
		base_request.to_token_address = busd_address.to_string();

		let result = configure_native_token_swap(wbnb_address, base_request);

		// Assertions for BSC native token
		assert_eq!(
			result.from_token_address, NATIVE_TOKEN_ADDRESS,
			"When selling BNB, from_token_address should be NATIVE_TOKEN_ADDRESS"
		);
		assert_eq!(
			result.to_token_address, busd_address,
			"When selling BNB, to_token_address should remain unchanged"
		);
		assert_eq!(
			result.is_from_token_referrer, true,
			"When selling BNB, fees should be collected from input token"
		);
	}

	#[test]
	fn test_native_token_swap_base_chain() {
		// Test Base Chain: Selling ETH for USDC (ETH → USDC)
		let base_weth = "0x4200000000000000000000000000000000000006"; // WETH on Base
		let base_usdc = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"; // USDC on Base

		let mut base_request = create_test_swap_route_request();
		base_request.chain_id = 8453; // Base chain ID
		base_request.from_token_address = base_weth.to_string();
		base_request.to_token_address = base_usdc.to_string();

		let result = configure_native_token_swap(base_weth, base_request);

		// Assertions for Base chain native token
		assert_eq!(
			result.from_token_address, NATIVE_TOKEN_ADDRESS,
			"When selling ETH on Base, from_token_address should be NATIVE_TOKEN_ADDRESS"
		);
		assert_eq!(
			result.to_token_address, base_usdc,
			"When selling ETH on Base, to_token_address should remain unchanged"
		);
		assert_eq!(
			result.is_from_token_referrer, true,
			"When selling ETH on Base, fees should be collected from input token"
		);
	}

	#[test]
	fn test_erc20_to_erc20_swap() {
		// Test ERC-20 to ERC-20 swap (no native tokens involved)
		let usdc_address = "0xA0b86a33E6417c8f7851efA37A9f7F1A5d8C8f6E"; // USDC
		let usdt_address = "0xdAC17F958D2ee523a2206206994597C13D831ec7"; // USDT

		let mut base_request = create_test_swap_route_request();
		base_request.from_token_address = usdc_address.to_string();
		base_request.to_token_address = usdt_address.to_string();

		let result = configure_native_token_swap(usdc_address, base_request);

		// When no native tokens are involved, only to_token should be set to NATIVE_TOKEN_ADDRESS
		// This is because the function assumes at least one token must be native
		assert_eq!(
			result.from_token_address, usdc_address,
			"For ERC-20 to ERC-20, from_token_address should remain unchanged"
		);
		assert_eq!(result.to_token_address, NATIVE_TOKEN_ADDRESS,
			"For ERC-20 to ERC-20, to_token_address gets set to NATIVE_TOKEN_ADDRESS (function assumption)");
		assert_eq!(
			result.is_from_token_referrer, false,
			"For ERC-20 to ERC-20, should use default fee collection"
		);
	}

	#[test]
	fn test_native_token_detection_edge_cases() {
		// Test with various native token formats
		let native_token_variants = vec![
			"0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", // NATIVE_TOKEN constant
			"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
			"0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c", // WBNB
			"0x4200000000000000000000000000000000000006", // Base WETH
		];

		let non_native_token = "0xA0b86a33E6417c8f7851efA37A9f7F1A5d8C8f6E"; // USDC

		for native_variant in native_token_variants {
			let mut base_request = create_test_swap_route_request();
			base_request.from_token_address = native_variant.to_string();
			base_request.to_token_address = non_native_token.to_string();

			let result = configure_native_token_swap(native_variant, base_request);

			assert_eq!(
				result.from_token_address, NATIVE_TOKEN_ADDRESS,
				"Native token variant {} should be converted to NATIVE_TOKEN_ADDRESS",
				native_variant
			);
			assert_eq!(
				result.is_from_token_referrer, true,
				"Native token variant {} should set is_from_token_referrer to true",
				native_variant
			);
		}
	}
}
