use crate::common::{is_native_token, CreateMarketTx, CROSS_SERVICE_FEE_BPS, CROSS_SERVICE_FEE_PERCENT, GAS_LIMIT, INCH_DEX_IDS_MAP, KYBER_SWAP_DEX_ID_MAP, NATIVE_ADDRESS, OKX_DEX_IDS_MAP, SERVICE_FEE_BPS, SERVICE_FEE_PERCENT};
use crate::inch_client::client::{InchClient, InchSwap};
use crate::inch_client::types::{convert_slippage_to_inch, SwapRequest};
use crate::kyber_client::client::KyberSwap;
use crate::okx_client::client::OkxSwap;
use ethereum_rpc::RpcProvider;
use rust_decimal::prelude::{Decimal, ToPrimitive};
use std::str::FromStr;
use std::sync::Arc;

use crate::kyber_client::types::GetSwapRouteRequest;
use crate::okx_client::types::{convert_slippage_to_okx, get_okx_gas_level};
use alloy::{
	primitives::{Address, TxKind, U256},
	rpc::types::{TransactionInput, TransactionRequest},
};
use log::error;

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

pub trait ConstructEvmTx {
	async fn construct_inch_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> Result<TransactionRequest, ()>;
	async fn construct_okx_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> Result<TransactionRequest, ()>;
	async fn construct_kyber_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> Result<TransactionRequest, ()>;
}

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
	) -> Result<TransactionRequest, ()> {
		let chain_id = create_market_tx.chain_id.clone();

		let mut swap_request = SwapRequest {
			chain_id: chain_id.clone(),
			amount: amount_decimal.to_string(),
			from_token_address: create_market_tx.in_token_ca.clone(),
			to_token_address: create_market_tx.out_token_ca.clone(),
			slippage: convert_slippage_to_inch(create_market_tx.slippage),
			user_wallet_address: create_market_tx.user_wallet_address.clone(),
			fee_percent: SERVICE_FEE_PERCENT.to_string(),
			dex_ids: INCH_DEX_IDS_MAP[&chain_id][&create_market_tx.trade_pool_name].to_string(),
			referrer: self.fee_receiver.clone(),
			..Default::default()
		};

		if is_native_token(&*create_market_tx.out_token_ca.into_bytes()) {
			swap_request.from_token_address = NATIVE_ADDRESS.to_string();
		} else {
			swap_request.to_token_address = NATIVE_ADDRESS.to_string();
		}

		let swap_response = self
			.inch_client
			.swap(swap_request)
			.await
			.map_err(|e| error!("Failed to get swap response from 1inch due to: {:?}", e))?;

		let (data, to, value, gas) = swap_response.get_transaction_data().map_err(|e| {
			error!("Failed to extract transaction details from swap response");
		})?;

		let gas_price = self.get_gas_price_by_level(chain_id, create_market_tx.gas_type).await.map_err(|_| {
			error!("Failed to get gas gas price by level");
		})?;
		let gas_price = gas_price.to_u128().ok_or_else(|| {
			error!("Failed to convert gas price to u128");
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
	) -> Result<TransactionRequest, ()> {
		let chain_id = create_market_tx.chain_id.clone();
		let mut fee_bps = SERVICE_FEE_PERCENT.to_string();
		if create_market_tx.is_pre_cross {
			fee_bps = CROSS_SERVICE_FEE_PERCENT.to_string();
		}

		let mut swap_request = crate::okx_client::types::SwapRequest {
			chain_id: chain_id.clone().to_string(),
			amount: amount_decimal.to_string(),
			from_token_address: create_market_tx.in_token_ca.clone(),
			to_token_address: create_market_tx.out_token_ca.clone(),
			slippage: convert_slippage_to_okx(create_market_tx.slippage).to_string(),
			user_wallet_address: create_market_tx.user_wallet_address,
			fee_percent: fee_bps,
			gas_level: get_okx_gas_level(create_market_tx.gas_type).to_string(),
			dex_ids: OKX_DEX_IDS_MAP[&create_market_tx.trade_pool_name].to_string(),
			..Default::default()
		};

		if is_native_token(&*create_market_tx.in_token_ca.into_bytes()) {
			swap_request.from_token_referrer_wallet_address = self.fee_receiver.clone();
			swap_request.from_token_address = NATIVE_ADDRESS.to_string();
		} else {
			swap_request.to_token_referrer_wallet_address = self.fee_receiver.clone();
			swap_request.to_token_address = NATIVE_ADDRESS.to_string();
		}

		let swap_response = self.okx_client.swap(swap_request).await.map_err(|e| {
			error!("Failed to get swap response from okx");
		})?;

		let (data, to, value, gas) = swap_response.get_transaction_data().map_err(|e| {
			error!("Failed to extract transaction details from swap response");
		})?;

		let gas_price = self.get_gas_price_by_level(chain_id, create_market_tx.gas_type).await.map_err(|_| {
			error!("Failed to get gas gas price by level");
		})?;
		let gas_price = gas_price.to_u128().ok_or_else(|| {
			error!("Failed to convert gas price to u128");
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
	) -> Result<TransactionRequest, ()> {
		let chain_id = create_market_tx.chain_id.clone();
		let mut fee_bps = SERVICE_FEE_BPS.to_string();
		if create_market_tx.is_pre_cross {
			fee_bps = CROSS_SERVICE_FEE_BPS.to_string();
		}

		let mut swap_route_request = GetSwapRouteRequest {
			chain_id: chain_id.clone(),
			amount: amount_decimal.to_string(),
			from_token_address: create_market_tx.in_token_ca.clone(),
			to_token_address: create_market_tx.out_token_ca.clone(),
			fee_bps,
			referrer: self.fee_receiver.clone(),
			dex_ids: KYBER_SWAP_DEX_ID_MAP[&create_market_tx.trade_pool_name].to_string(),
			is_from_token_referrer: false,
			..Default::default()
		};

		if is_native_token(&*create_market_tx.in_token_ca.into_bytes()) {
			swap_route_request.from_token_address = NATIVE_ADDRESS.to_string();
			swap_route_request.is_from_token_referrer = true;
		} else {
			swap_route_request.to_token_address = NATIVE_ADDRESS.to_string();
		}

		let swap_route_response =
			self.kyber_client.get_swap_route(swap_route_request).await.map_err(|e| {
				error!("Failed to get swap route response from kyber: {}", e);
			})?;

		let swap_request = crate::kyber_client::types::SwapRequest {
			route_summary: swap_route_response.route_summary,
			sender: Some(create_market_tx.user_wallet_address.clone()),
			recipient: Some(create_market_tx.user_wallet_address.clone()),
			deadline: Some((std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.expect("Time went backwards")
				.as_secs() + 60) as i64),
			slippage_bps: create_market_tx.slippage.to_i64(),
			enable_gas_estimation: Some(true),
			ignore_capped_slippage: Some(true),
			..Default::default()
		};

		let swap_response = self.kyber_client.swap(swap_request).await.map_err(|e| {
			error!("Failed to get swap response from kyber: {}", e);
		})?;

		let (data, to, value, gas) = swap_response.get_transaction_data().map_err(|e| {
			error!("Failed to extract transaction details from swap response");
		})?;

		let gas_price = self.get_gas_price_by_level(chain_id, create_market_tx.gas_type).await.map_err(|_| {
			error!("Failed to get gas gas price by level");
		})?;
		let gas_price = gas_price.to_u128().ok_or_else(|| {
			error!("Failed to convert gas price to u128");
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
	pub async fn get_gas_price_by_level(&self, chain_id: u64, level: i32) -> Result<Decimal, ()> {
		let gas_price = self
			.okx_client
			.get_gas_price(chain_id)
			.await
			.map_err(|e| error!("Failed to get price by level due to: {:?}", e))?;

		match level {
			1_i32 => Decimal::from_str(&gas_price.min)
				.map_err(|_| error!("Failed to get price by level")),
			2_i32 => Decimal::from_str(&gas_price.normal)
				.map_err(|_| error!("Failed to get price by level")),
			3_i32 => Decimal::from_str(&gas_price.max)
				.map_err(|_| error!("Failed to get price by level")),
			_ => {
				error!("Level {} not found", level);
				Err(())
			},
		}
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
	fn construct_unsigned_market_tx() {

	}
}
