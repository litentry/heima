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

use crate::common::FourMemeContract::FourMemeContractInstance;
use crate::common::FourMemeHelperContract::FourMemeHelperContractInstance;
use crate::common::FourMemeManagerContract::FourMemeManagerContractInstance;
use crate::common::{
	is_native_token, CreateMarketTx, ALL_BP_DECIMAL, CROSS_SERVICE_FEE_BPS,
	CROSS_SERVICE_FEE_PERCENT, DECIMALS_TO_VALUE, FOUR_MEME, FOUR_MEME_HELPER, FOUR_MEME_MANAGER,
	GAS_LIMIT, INCH_DEX_IDS_MAP, INCH_SWAP_APPROVE_ADDRESS, KYBER_SWAP_APPROVE_ADDRESS,
	KYBER_SWAP_DEX_ID_MAP, NATIVE_ADDRESS, OKX_DEX_IDS_MAP, OKX_SWAP_APPROVE_ADDRESS,
	SERVICE_FEE_BPS, SERVICE_FEE_PERCENT,
};
use crate::inch_client::client::InchSwap;
use crate::inch_client::types::{convert_slippage_to_inch, SwapRequest};
use crate::kyber_client::client::KyberSwap;
use crate::kyber_client::types::GetSwapRouteRequest;
use crate::okx_client::client::OkxSwap;
use crate::okx_client::types::{convert_slippage_to_okx, get_okx_gas_level};
use alloy::primitives::{Uint, U256};
use alloy::{
	primitives::{Address, TxKind},
	rpc::types::{TransactionInput, TransactionRequest},
};
use async_trait::async_trait;
use ethereum_rpc::client::EthereumClient;
use ethereum_rpc::RpcProvider;
use hex::FromHex;
use log::error;
use rust_decimal::prelude::{Decimal, FromPrimitive, ToPrimitive};
use std::ops::Div;
use std::str::FromStr;
use std::sync::Arc;

/// EVM Transaction Manager
pub struct EvmTxManager<
	EthClient: RpcProvider<Transaction = TransactionRequest> + ?Sized,
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
	async fn construct_four_meme_v2_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> Result<TransactionRequest, ()>;
}

#[async_trait]
impl<EthClient, InchClient, KyberClient, OkxClient> ConstructEvmTx
	for EvmTxManager<EthClient, InchClient, KyberClient, OkxClient>
where
	EthClient: RpcProvider<Transaction = TransactionRequest> + ?Sized,
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
		let chain_id = create_market_tx.chain_id;

		let mut swap_request = SwapRequest {
			chain_id,
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

		if is_native_token(&create_market_tx.in_token_ca.into_bytes()) {
			swap_request.from_token_address = NATIVE_ADDRESS.to_string();
		} else {
			swap_request.to_token_address = NATIVE_ADDRESS.to_string();
		}

		let swap_response = self
			.inch_client
			.swap(chain_id, swap_request)
			.await
			.map_err(|e| error!("Failed to get swap response from 1inch due to: {:?}", e))?;

		let (data, to, value, gas) = swap_response.get_transaction_data().map_err(|_| {
			error!("Failed to extract transaction details from swap response");
		})?;

		let gas_price = self
			.get_gas_price_by_level(chain_id, create_market_tx.gas_type)
			.await
			.map_err(|_| {
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
		let chain_id = create_market_tx.chain_id;
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

		if is_native_token(&create_market_tx.in_token_ca.into_bytes()) {
			swap_request.from_token_referrer_wallet_address = self.fee_receiver.clone();
			swap_request.from_token_address = NATIVE_ADDRESS.to_string();
		} else {
			swap_request.to_token_referrer_wallet_address = self.fee_receiver.clone();
			swap_request.to_token_address = NATIVE_ADDRESS.to_string();
		}

		let swap_response = self.okx_client.swap(swap_request).await.map_err(|_| {
			error!("Failed to get swap response from okx");
		})?;

		let (data, to, value, gas) = swap_response.get_transaction_data().map_err(|_| {
			error!("Failed to extract transaction details from swap response");
		})?;

		let gas_price = self
			.get_gas_price_by_level(chain_id, create_market_tx.gas_type)
			.await
			.map_err(|_| {
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
		let chain_id = create_market_tx.chain_id;
		let mut fee_bps = SERVICE_FEE_BPS.to_string();
		if create_market_tx.is_pre_cross {
			fee_bps = CROSS_SERVICE_FEE_BPS.to_string();
		}

		let mut swap_route_request = GetSwapRouteRequest {
			chain_id,
			amount: amount_decimal.to_string(),
			from_token_address: create_market_tx.in_token_ca.clone(),
			to_token_address: create_market_tx.out_token_ca.clone(),
			fee_bps,
			referrer: self.fee_receiver.clone(),
			dex_ids: KYBER_SWAP_DEX_ID_MAP[&create_market_tx.trade_pool_name].to_string(),
			is_from_token_referrer: false,
		};

		if is_native_token(&create_market_tx.in_token_ca.into_bytes()) {
			swap_route_request.from_token_address = NATIVE_ADDRESS.to_string();
			swap_route_request.is_from_token_referrer = true;
		} else {
			swap_route_request.to_token_address = NATIVE_ADDRESS.to_string();
		}

		let swap_route_response = self
			.kyber_client
			.get_swap_route(chain_id, swap_route_request)
			.await
			.map_err(|e| {
				error!("Failed to get swap route response from kyber: {}", e);
			})?;

		let swap_request = crate::kyber_client::types::SwapRequest {
			route_summary: swap_route_response.route_summary,
			sender: Some(create_market_tx.user_wallet_address.clone()),
			recipient: Some(create_market_tx.user_wallet_address.clone()),
			deadline: Some(
				(std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.expect("Time went backwards")
					.as_secs() + 60) as i64,
			),
			slippage_bps: create_market_tx.slippage.to_i64(),
			enable_gas_estimation: Some(true),
			ignore_capped_slippage: Some(true),
			..Default::default()
		};

		let swap_response = self.kyber_client.swap(chain_id, swap_request).await.map_err(|e| {
			error!("Failed to get swap response from kyber: {}", e);
		})?;

		let (data, to, value, gas) = swap_response.get_transaction_data().map_err(|_| {
			error!("Failed to extract transaction details from swap response");
		})?;

		let gas_price = self
			.get_gas_price_by_level(chain_id, create_market_tx.gas_type)
			.await
			.map_err(|_| {
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

	async fn construct_four_meme_v2_tx(
		&self,
		create_market_tx: CreateMarketTx,
		nonce: u64,
		amount_decimal: Decimal,
	) -> Result<TransactionRequest, ()> {
		let mut to = Address::from_str(FOUR_MEME).map_err(|e| {
			error!("Error converting user wallet address into Ethereum Address: {}", e);
		})?;
		let from = Address::from_str(&create_market_tx.user_wallet_address).map_err(|e| {
			error!("Error converting user wallet address into Ethereum Address: {}", e);
		})?;
		let chain_id = create_market_tx.chain_id;
		let is_buy = is_native_token(create_market_tx.in_token_ca.as_ref());

		let amount = U256::from_str(&amount_decimal.trunc().to_string()).map_err(|e| {
			error!("Error converting amount to U256: {}", e);
		})?;
		let fee_rate = U256::from_str(SERVICE_FEE_BPS).map_err(|e| {
			error!("Error converting fee rate to U256: {}", e);
		})?;
		let fee_receiver = Address::from_str(&create_market_tx.recipient_address).map_err(|e| {
			error!("Error converting token contract to Ethereum Address: {}", e);
		})?;
		let mut value = U256::from(0);

		let mut call = if is_buy {
			value = amount;
			let target_token = Address::from_str(&create_market_tx.out_token_ca).map_err(|e| {
				error!("Error converting token contract to Ethereum Address: {}", e);
			})?;
			let recipient_address = Address::from_str(&create_market_tx.recipient_address)
				.map_err(|e| {
					error!("Error converting token contract to Ethereum Address: {}", e);
				})?;
			let min_amount = self
				.calculate_min_amount(
					target_token,
					amount_decimal,
					i64::from_u32(create_market_tx.slippage).unwrap(),
					is_buy,
				)
				.await?;
			let four_meme_address = Address::from_str(FOUR_MEME).map_err(|e| {
				error!("Error converting user wallet address into Ethereum Address: {}", e);
			})?;
			let four_meme_contract = FourMemeContractInstance::new(
				four_meme_address,
				self.eth_client.get_provider().await?,
			);
			four_meme_contract
				.buyTokenAMAP(
					target_token,
					recipient_address,
					amount,
					min_amount,
					fee_rate,
					fee_receiver,
				)
				.into_transaction_request()
		} else {
			to = Address::from_str(FOUR_MEME_MANAGER).map_err(|e| {
				error!("Error converting four meme manager address into Ethereum Address: {}", e);
			})?;
			let target_token = Address::from_str(&create_market_tx.in_token_ca).map_err(|e| {
				error!("Error converting token contract to Ethereum Address: {}", e);
			})?;
			let min_amount = self
				.calculate_min_amount(
					target_token,
					amount_decimal,
					i64::from_u32(create_market_tx.slippage).unwrap(),
					is_buy,
				)
				.await?;
			let four_meme_manager_address = Address::from_str(FOUR_MEME_MANAGER).map_err(|e| {
				error!("Error converting token contract to Ethereum Address: {}", e);
			})?;
			let four_meme_manager_contract = FourMemeManagerContractInstance::new(
				four_meme_manager_address,
				self.eth_client.get_provider().await?,
			);
			four_meme_manager_contract
				.sellToken(
					U256::from(0),
					target_token,
					from,
					amount,
					min_amount,
					fee_rate,
					fee_receiver,
				)
				.into_transaction_request()
		};

		let gas_price = self
			.get_gas_price_by_level(chain_id, create_market_tx.gas_type)
			.await
			.map_err(|_| {
				error!("Failed to get gas gas price by level");
			})?;

		let gas_price = gas_price.to_u128().ok_or_else(|| {
			error!("Failed to convert gas price to u128");
		})?;

		call.gas_price = Some(gas_price);
		call.to = Some(TxKind::Call(to));
		call.nonce = Some(nonce);
		call.value = Some(value);

		let gas = self.eth_client.estimate_gas(call.clone()).await?;

		let mut adjusted_gas = gas + gas.div(2);
		if adjusted_gas > *GAS_LIMIT {
			adjusted_gas = *GAS_LIMIT;
		}

		call.gas = Some(adjusted_gas);
		Ok(call)
	}
}

impl<EthClient, InchClient, KyberClient, OkxClient>
	EvmTxManager<EthClient, InchClient, KyberClient, OkxClient>
where
	EthClient: RpcProvider<Transaction = TransactionRequest> + ?Sized,
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

	pub async fn calculate_min_amount(
		&self,
		token: Address,
		amount_decimal: Decimal,
		slippage: i64,
		is_buy: bool,
	) -> Result<U256, ()> {
		let four_meme_helper = Address::from_str(FOUR_MEME_HELPER).map_err(|e| {
			error!("Failed to get four meme helper ethereum address due to: {:?}", e);
		})?;
		let four_meme_manager = Address::from_str(FOUR_MEME_MANAGER).map_err(|e| {
			error!("Failed to get four meme helper ethereum address due to: {:?}", e);
		})?;
		let four_meme_contract = FourMemeHelperContractInstance::new(
			four_meme_helper,
			self.eth_client.get_provider().await?,
		);
		let amount = U256::from_str(amount_decimal.trunc().to_string().as_str()).map_err(|e| {
			error!("Failed to get amount due to: {:?}", e);
		})?;
		let amount_out: U256 = if is_buy {
			let try_buy_response =
				four_meme_contract.tryBuy(token, U256::from(0), amount).call().await.map_err(
					|e| {
						error!("Failed to get try_buy call due to: {:?}", e);
					},
				)?;

			if try_buy_response.tokenManager != four_meme_manager {
				error!("invalid token manager");
				return Err(());
			}

			try_buy_response.estimatedAmount
		} else {
			let try_sell_response =
				four_meme_contract.trySell(token, amount).call().await.map_err(|e| {
					error!("Failed to get try_buy call due to: {:?}", e);
				})?;

			if try_sell_response.tokenManager != four_meme_manager {
				error!("invalid token manager");
				return Err(());
			}

			try_sell_response.funds
		};

		let slippage_bp: Decimal = slippage.into();

		let total_bp = *ALL_BP_DECIMAL;
		let total_bp_u256 = U256::from_str(&ALL_BP_DECIMAL.clone().trunc().to_string()).unwrap();

		let adj_bp = total_bp
			.checked_sub(slippage_bp)
			.ok_or_else(|| {
				error!("Slippage {} exceeds total basis points {}", slippage_bp, total_bp)
			})?
			.trunc()
			.to_string();
		let adj_bp =
			U256::from_str(&adj_bp).map_err(|e| error!("Failed to parse U256 value: {}", e))?;

		let numerator = amount_out
			.checked_mul(adj_bp)
			.ok_or_else(|| error!("Overflow multiplying amount_out by adjusted BP"))?;

		let min_amount_out = numerator
			.checked_div(total_bp_u256)
			.ok_or_else(|| error!("Division by zero in basis points"))?;

		Ok(min_amount_out)
	}
}

impl<EthClient, InchClient, KyberClient, OkxClient>
	EvmTxManager<EthClient, InchClient, KyberClient, OkxClient>
where
	EthClient:
		RpcProvider<Addr = Address, Transaction = TransactionRequest> + ?Sized + EthereumClient,
	InchClient: InchSwap + ?Sized,
	KyberClient: KyberSwap + ?Sized,
	OkxClient: OkxSwap + ?Sized,
{
	#[allow(dead_code)]
	async fn construct_unsigned_market_tx(
		&self,
		tx: CreateMarketTx,
	) -> Result<Vec<TransactionRequest>, ()> {
		let amount_decimal = Decimal::from_str(&tx.amount_in).unwrap();
		let multiplier = DECIMALS_TO_VALUE.get(&tx.in_decimal).cloned().unwrap_or(1);
		let amount_decimal = amount_decimal * Decimal::from(multiplier);

		let from = Address::from_hex(&tx.user_wallet_address)
			.map_err(|e| error!("Failed to convert user wallet to address: {}", e))?;

		let mut balance = self.eth_client.get_balance(from).await.map_err(|_| {
			error!("Couldn't get balance");
		})?;

		let mut nonce = self.eth_client.get_pending_nonce(from).await.map_err(|_| {
			error!("Couldn't get pending nonce");
		})?;

		let is_buy = is_native_token(&tx.in_token_ca.clone().into_bytes());
		let platform = Platform::KyberSwap;

		let amount_u128 = amount_decimal.to_u128().ok_or_else(|| {
			error!("amount_decimal could not be converted to u128: {}", amount_decimal);
		})?;

		let amount = Uint::try_from(amount_u128).map_err(|e| {
			error!("Failed to convert amount_u128 to Uint: {:?}", e);
		})?;

		let mut transactions: Vec<TransactionRequest> = Vec::new();
		if is_buy {
			if balance < amount {
				error!("Balance is too low");
				return Err(());
			}
			balance -= amount;
		} else {
			let approve_addr: Address = match platform {
				Platform::KyberSwap => {
					Address::from_hex(KYBER_SWAP_APPROVE_ADDRESS).map_err(|_| {
						error!("Failed to convert address from hex to address");
					})?
				},
				Platform::Inch => Address::from_hex(INCH_SWAP_APPROVE_ADDRESS).map_err(|_| {
					error!("Failed to convert address from hex to address");
				})?,
				Platform::Okx => Address::from_hex(OKX_SWAP_APPROVE_ADDRESS).map_err(|_| {
					error!("Failed to convert address from hex to address");
				})?,
			};

			let approve_tx = self
				.eth_client
				.construct_approve_erc20_tx(
					approve_addr,
					amount,
					Address::from_hex(tx.in_token_ca.clone()).unwrap(),
					nonce,
				)
				.await
				.map_err(|_| {
					error!("Failed to create approve tx");
				})?;

			transactions.push(approve_tx);
			nonce += 1;
		}

		let unsigned_tx: TransactionRequest = match platform {
			Platform::KyberSwap => {
				self.construct_kyber_tx(tx, nonce, amount_decimal).await.map_err(|_| {
					error!("Failed to create unsigned tx for kyber swap");
				})?
			},
			Platform::Inch => {
				self.construct_inch_tx(tx, nonce, amount_decimal).await.map_err(|_| {
					error!("Failed to create unsigned tx for 1inch swap");
				})?
			},
			Platform::Okx => {
				self.construct_okx_tx(tx, nonce, amount_decimal).await.map_err(|_| {
					error!("Failed to create unsigned tx for okx swap");
				})?
			},
		};

		let gas = unsigned_tx.gas.ok_or_else(|| {
			error!("Gas not set in transaction");
		})?;

		let gas_price_u64 = unsigned_tx
			.gas_price
			.ok_or_else(|| {
				error!("Gas price not set in transaction");
			})?
			.to_u64()
			.ok_or_else(|| {
				error!("Failed to convert gas price to u64");
			})?;

		let gas_fee_u128 = gas.checked_mul(gas_price_u64).ok_or_else(|| {
			error!("Gas fee multiplication overflow");
		})?;

		let gas_fee = Uint::try_from(gas_fee_u128).map_err(|e| {
			error!("Failed to convert gas fee to Uint: {:?}", e);
		})?;

		if balance < gas_fee {
			error!("Balance is too low for gas fee");
			return Err(());
		}

		transactions.push(unsigned_tx);
		Ok(transactions)
	}
}

pub enum Platform {
	Inch,
	Okx,
	KyberSwap,
}
