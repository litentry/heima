// Copyright 2020-2024 Trust Computing GmbH.
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

use crate::*;
use accounting_contract_client::{
	solana::AccountingContractApi as SolanaAccountingContractApi,
	AccountingContractApi as EvmAccountingContractApi,
};
use alloy::primitives::Address;
use ethereum_rpc::AlloyRpcProvider;
use executor_primitives::Chain;
use intent_token_query::{query_ethereum, query_solana, EthereumAddress, SolanaPubkey};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error};

pub async fn estimate_asset_value_in_usdt<BinanceClient: BinanceApi>(
	binance_api: &Arc<BinanceClient>,
	trade_symbol: &str,
	binance_coin_name: &str,
	from_amount_decimal: Decimal,
) -> Result<Decimal, ()> {
	let price_str = SpotTradingApi::new(binance_api.as_ref())
		.get_symbol_price(trade_symbol)
		.await
		.map_err(|_| {
			error!("Failed to get symbol price for {}", trade_symbol);
		})?;

	let price = if binance_coin_name == "SOL" {
		// For SOL, we sell SOL to get BNB, so get SOLBNB price
		Decimal::from_str(&price_str).map_err(|_| {
			error!("Failed to parse symbol price {}", price_str);
		})?
	} else {
		// For USDC/USDT, we buy BNB with USDC/USDT, so get BNBUSDC/BNBUSDT price and invert
		let price = Decimal::from_str(&price_str).map_err(|_| {
			error!("Failed to parse symbol price {}", price_str);
		})?;
		if price.is_zero() {
			error!("Symbol price is zero for {}", trade_symbol);
			return Err(());
		}
		Decimal::ONE / price
	};

	let amount = from_amount_decimal * price;

	debug!("From amount in usdt: {}", amount);
	Ok(amount)
}

pub fn calculate_amount_in(amount: &str, gas: &str, decimals: u32) -> Result<String, ()> {
	let amount = Decimal::from_str(amount).map_err(|_| ())?;
	let gas = Decimal::from_str(gas).map_err(|_| ())?;
	if amount <= gas {
		Err(())
	} else {
		let amount = amount - gas;
		Ok(amount.trunc_with_scale(decimals).to_string())
	}
}

pub fn str_to_u256(amount: &str, decimals: u32) -> Result<U256, ()> {
	let amount = Decimal::from_str(amount).map_err(|_| ())?;
	let factor = Decimal::from(10u64.pow(decimals));
	let result = amount * factor;
	let result_str = result.trunc().to_string();
	U256::from_str(&result_str).map_err(|_| ())
}

pub fn determine_trade_symbol_and_order_side(
	from_network: BinanceNetwork,
	from_coin: BinanceCoin,
	to_network: BinanceNetwork,
) -> Result<(String, BinanceOrderSide), ()> {
	match (&from_network, &from_coin, &to_network) {
		// SOL to BSC
		(BinanceNetwork::Sol, BinanceCoin::Sol, BinanceNetwork::Bsc) => {
			Ok(("SOLBNB".to_string(), BinanceOrderSide::SELL))
		},
		(BinanceNetwork::Sol, BinanceCoin::Usdc, BinanceNetwork::Bsc) => {
			Ok(("BNBUSDC".to_string(), BinanceOrderSide::BUY))
		},
		(BinanceNetwork::Sol, BinanceCoin::Usdt, BinanceNetwork::Bsc) => {
			Ok(("BNBUSDT".to_string(), BinanceOrderSide::BUY))
		},
		// BSC to SOL
		(BinanceNetwork::Bsc, BinanceCoin::Bnb, BinanceNetwork::Sol) => {
			Ok(("SOLBNB".to_string(), BinanceOrderSide::BUY))
		},
		(BinanceNetwork::Bsc, BinanceCoin::Usdc, BinanceNetwork::Sol) => {
			Ok(("SOLUSDC".to_string(), BinanceOrderSide::BUY))
		},
		(BinanceNetwork::Bsc, BinanceCoin::Usdt, BinanceNetwork::Sol) => {
			Ok(("SOLUSDT".to_string(), BinanceOrderSide::BUY))
		},
		_ => {
			error!(
				"Unsupported trade pair: {:?} {:?} -> {:?}",
				from_network, from_coin, to_network
			);
			Err(())
		},
	}
}

pub async fn estimate_payout_amount<BinanceClient: BinanceApi>(
	binance_api: &Arc<BinanceClient>,
	trade_symbol: &str,
	binance_coin: BinanceCoin,
	from_amount_decimal: Decimal,
) -> Result<String, ()> {
	let price_str = SpotTradingApi::new(binance_api.as_ref())
		.get_symbol_price(trade_symbol)
		.await
		.map_err(|_| {
			error!("Failed to get symbol price for {}", trade_symbol);
		})?;

	let price = Decimal::from_str(&price_str).map_err(|_| {
		error!("Failed to parse symbol price {}", price_str);
	})?;

	let payout_amount = match binance_coin {
		BinanceCoin::Sol => from_amount_decimal * price,
		BinanceCoin::Bnb => from_amount_decimal / price,
		BinanceCoin::Fc => from_amount_decimal * price,
		BinanceCoin::Usdc | BinanceCoin::Usdt => from_amount_decimal * price,
	};

	debug!("Estimated payout amount: {}", payout_amount);
	Ok(payout_amount.trunc_with_scale(binance_coin.decimals()).to_string())
}

pub async fn get_binance_deposit_info<BinanceClient: BinanceApi>(
	binance_api: Arc<BinanceClient>,
	swap_order_from_asset: &ChainAsset,
	from_amount: &str,
) -> Result<(String, BinanceAsset, Decimal, Decimal), ()> {
	let coins_info =
		WalletApi::new(binance_api.as_ref()).get_all_coins_info().await.map_err(|e| {
			error!("Failed to get all coins info, {:?}", e);
		})?;

	let binance_asset = BinanceAsset::from_chain_asset(swap_order_from_asset)?;

	debug!(
		"from_network_name: {}, binance_coin_name: {}, token_address: {}",
		binance_asset.network.name(),
		binance_asset.coin.name(),
		binance_asset.address
	);

	let Some(binance_coin_info) = coins_info.iter().find(|c| c.coin == binance_asset.coin.name())
	else {
		error!("Coin not found: {}", binance_asset.coin.name());
		return Err(());
	};

	let Some(_binance_network_info) = binance_coin_info
		.network_list
		.iter()
		.find(|n| n.network == binance_asset.network.name())
	else {
		error!(
			"Network not found: {} for coin: {}",
			binance_asset.network.name(),
			binance_asset.coin.name()
		);
		return Err(());
	};

	let deposit_address = WalletApi::new(binance_api.as_ref())
		.get_deposit_address(binance_asset.coin.name(), binance_asset.network.name())
		.await
		.map_err(|e| {
			error!("Failed to get deposit address, {:?}", e);
		})?;

	let from_amount_decimal = Decimal::from_str(from_amount).map_err(|_| {
		error!("Failed to parse from_amount_string");
	})?;

	debug!("Binance deposit address: {}, from_amount: {}", deposit_address, from_amount);

	let asset_decimal_multiplier = binance_asset.coin.decimal_value();

	let amount_to_transfer = from_amount_decimal * asset_decimal_multiplier;

	Ok((deposit_address, binance_asset, from_amount_decimal, amount_to_transfer))
}

pub async fn do_payout_sol_to_bsc(
	payout_address: &Address,
	payout_amount: &U256,
	evm_accounting_contract_client: &Arc<Box<dyn EvmAccountingContractApi>>,
) -> Result<(), ()> {
	debug!("Getting {:?} nonce for payout request", payout_address);
	let user_nonce =
		evm_accounting_contract_client.get_nonce(*payout_address).await.map_err(|_| {
			error!("Failed to get nonce");
		})?;

	debug!("Received {:?} nonce", user_nonce);
	let user_nonce = user_nonce + U256::from(1u64);
	debug!(
		"Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}",
		payout_address, user_nonce, payout_amount
	);

	evm_accounting_contract_client
		.execute_pay_out_request(*payout_address, user_nonce, *payout_amount)
		.await
		.map_err(|_| {
			error!("Failed to execute pay out request");
		})
}

pub async fn do_payout_bsc_to_sol(
	payout_address: Pubkey,
	payout_amount: U256,
	solana_accounting_contract_client: &Arc<Box<dyn SolanaAccountingContractApi>>,
) -> Result<(), ()> {
	debug!("Getting {:?} nonce for payout request", payout_address);
	let user_nonce =
		solana_accounting_contract_client.get_nonce(payout_address).await.map_err(|_| {
			error!("Failed to get nonce");
		})?;

	debug!("Received {:?} nonce", user_nonce);
	let user_nonce = user_nonce + 1u64;
	debug!(
		"Calling accounting contract payout with address {:?}, nonce {:?} and amount {:?}",
		payout_address, user_nonce, payout_amount
	);

	solana_accounting_contract_client
		.execute_pay_out_request(payout_address, user_nonce, payout_amount)
		.await
		.map_err(|_| {
			error!("Failed to execute pay out request");
		})
}

pub async fn do_binance_swap_sol_to_bsc<BinanceClient: BinanceApi>(
	binance_api: Arc<BinanceClient>,
	evm_accounting_contract_client: &Arc<Box<dyn EvmAccountingContractApi>>,
	from_asset: ChainAsset,
	from_amount: String,
) -> Result<(String, U256), ()> {
	let (deposit_address, binance_asset, from_amount_decimal, _amount_to_transfer_decimal) =
		get_binance_deposit_info(binance_api.clone(), &from_asset, &from_amount).await?;

	let binance_network = binance_asset.network;
	let binance_coin = binance_asset.coin;

	let (trade_symbol, _order_side) = determine_trade_symbol_and_order_side(
		binance_network.clone(),
		binance_coin.clone(),
		BinanceNetwork::Bsc,
	)?;

	// init `payout_amount` with estimated-amount-to-receive
	let payout_amount = estimate_payout_amount(
		&binance_api,
		&trade_symbol,
		binance_coin.clone(),
		from_amount_decimal,
	)
	.await?;

	let payout_amount_u256 = str_to_u256(&payout_amount, BinanceCoin::Bnb.decimals())?;

	// Fetch contract balance
	let balance = evm_accounting_contract_client.get_balance().await?;
	if balance < payout_amount_u256 {
		error!(
			"There is not enough balance in the accounting contract, {} < {}",
			balance, payout_amount_u256
		);
		return Err(());
	}

	// For omni, we simulate the SOL transfer instead of actually performing it
	debug!("Omni: Simulating SOL transfer to binance deposit address: {}", deposit_address);

	Ok((payout_amount, payout_amount_u256))
}

pub async fn do_binance_swap_bsc_to_sol<BinanceClient: BinanceApi>(
	binance_api: Arc<BinanceClient>,
	solana_accounting_contract_client: &Arc<Box<dyn SolanaAccountingContractApi>>,
	from_asset: ChainAsset,
	from_amount: String,
) -> Result<(String, U256), ()> {
	let (deposit_address, binance_asset, from_amount_decimal, _amount_to_transfer_decimal) =
		get_binance_deposit_info(binance_api.clone(), &from_asset, &from_amount).await?;

	let binance_network = binance_asset.network;
	let binance_coin = binance_asset.coin;

	let (trade_symbol, _order_side) = determine_trade_symbol_and_order_side(
		binance_network.clone(),
		binance_coin.clone(),
		BinanceNetwork::Sol,
	)?;

	// init `payout_amount` with estimated-amount-to-receive
	let payout_amount = estimate_payout_amount(
		&binance_api,
		&trade_symbol,
		binance_coin.clone(),
		from_amount_decimal,
	)
	.await?;

	let payout_amount_u256 = str_to_u256(&payout_amount, BinanceCoin::Sol.decimals())?;

	// Fetch contract balance
	let balance = solana_accounting_contract_client.get_balance().await?;
	if balance < payout_amount_u256 {
		error!(
			"There is not enough balance in the accounting contract, {} < {}",
			balance, payout_amount_u256
		);
		return Err(());
	}

	// For omni, we simulate the BNB transfer instead of actually performing it
	debug!("Omni: Simulating BNB transfer to binance deposit address: {}", deposit_address);

	Ok((payout_amount, payout_amount_u256))
}

pub async fn get_token_available_amount<SolanaClient: SolanaClientTrait>(
	asset: &ChainAsset,
	address: Vec<u8>,
	rpc_endpoint_registry: &RpcEndpointRegistry,
	solana_client: &Arc<SolanaClient>,
) -> Result<U256, ()> {
	match &asset {
		ChainAsset::Ethereum(chain_id, token) => {
			let rpc_url = rpc_endpoint_registry
				.get(&Chain::Evm(*chain_id as u64))
				.ok_or(error!("No RPC endpoint in registry for Ethereum chain"))?;
			let provider = AlloyRpcProvider::new(rpc_url);
			query_ethereum(&provider, EthereumAddress::from_slice(&address), token).await
		},
		ChainAsset::Solana(token) => {
			let pubkey = SolanaPubkey::try_from(address).map_err(|e| {
				error!("Could not create solana pubkey from wallet address: {:?}", e)
			})?;
			query_solana(solana_client.deref(), &pubkey, token)
				.await
				.map(|v| AmountType::from(v))
		},
	}
}
