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

use crate::utils::{
	calculate_amount_in, do_binance_swap, do_payout, estimate_asset_value_in_usdt,
	get_binance_deposit_info, get_token_available_amount,
};
use crate::*;
use alloy::primitives::Address;
use oe_primitives::SwapOrder;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{debug, error, info};

impl<
		BinanceClient: BinanceApi,
		EthereumClient: EthereumClientTrait,
		SolanaClient: SolanaClientTrait,
	> CrossChainIntentExecutor<BinanceClient, EthereumClient, SolanaClient>
{
	pub async fn execute_omni_cross_chain_swap(
		&self,
		account_id: &AccountId,
		omni_account: [u8; 32],
		intent_id: IntentId,
		swap_order: &SwapOrder,
		from_address: String,
		from_wallet: Vec<u8>,
		to_address: String,
		from_amount: String,
	) -> Result<(String, String, Option<InstantFlowDetails>), ()> {
		debug!("executing omni cross chain swap");
		debug!(
			"intent_id: {}, from_address: {}, to_address: {}",
			intent_id, from_address, to_address
		);

		info!(
			"Omni cross chain swap from {:?} to {:?}",
			swap_order.from_asset, swap_order.to_asset
		);

		let from_asset_binance_coin = BinanceAsset::from_chain_asset(&swap_order.from_asset)?.coin;

		let from_amount_decimal = Decimal::from_str(&from_amount).map_err(|_| {
			error!("Failed to parse from_amount_string");
		})?;

		let usdt_trade_symbol = match &swap_order.from_asset {
			ChainAsset::Solana(_) => "SOLUSDT",
			ChainAsset::Ethereum(_, _) => "BNBUSDT",
		};

		let estimated_from_amount_in_usdt = estimate_asset_value_in_usdt(
			&self.oe_client_binance,
			usdt_trade_symbol,
			from_asset_binance_coin.name(),
			from_amount_decimal,
		)
		.await?;

		debug!(
			"Checking instant payout threshold: {:?}, estimated usdt amount: {:?}",
			self.instant_payout_threshold, estimated_from_amount_in_usdt
		);

		let instant = estimated_from_amount_in_usdt <= self.instant_payout_threshold;
		debug!("Instant: {:?}", instant);

		let amount_to_lock = AmountType::from_str(
			&(from_amount_decimal * from_asset_binance_coin.decimal_value())
				.normalize()
				.to_string(),
		)
		.unwrap();

		if instant {
			let available_amount = get_token_available_amount(
				&swap_order.from_asset,
				from_wallet.clone(),
				&self.rpc_endpoint_registry,
				&self.solana_client,
			)
			.await?;

			self.account_asset_lock.check_and_insert(
				account_id.clone(),
				swap_order.from_asset.clone(),
				amount_to_lock,
				available_amount,
			)?;
		}

		match (&swap_order.from_asset, &swap_order.to_asset) {
			// SOL to BSC
			(ChainAsset::Solana(_), ChainAsset::Ethereum(56, _)) => {
				// BSC chain ID is 56
				let payout_address = Address::from_str(&to_address).map_err(|_| {
					error!("Failed to parse payout address");
				})?;
				debug!("omni cross chain swap details: intent_id: {}, from_amount: {}, from_address: {}, payout_address: {}", intent_id, from_amount, from_address, payout_address);

				if instant {
					let (_, binance_asset, from_amount_decimal, _) = get_binance_deposit_info(
						self.oe_client_binance.clone(),
						&swap_order.from_asset,
						&from_amount,
					)
					.await?;

					let binance_network = binance_asset.network;
					let binance_coin = binance_asset.coin;

					let (trade_symbol, _) = determine_trade_symbol_and_order_side(
						binance_network.clone(),
						binance_coin.clone(),
						BinanceNetwork::Bsc,
					)?;

					// init `payout_amount` with estimated-amount-to-receive
					let payout_amount = estimate_payout_amount(
						&self.oe_client_binance,
						&trade_symbol,
						binance_coin.clone(),
						from_amount_decimal,
					)
					.await?;

					Ok((
						self.apply_omni_gas_fee(&payout_amount, BinanceCoin::Bnb).await?,
						self.evm_oe_client_accounting.get_signer_address().await.to_string(),
						Some(InstantFlowDetails {
							omni_account,
							from_asset: swap_order.from_asset.clone(),
							from_amount: from_amount_decimal,
							from_address: from_address.clone(),
							wallet_index: 0, // omni doesn't use wallet index concept
							locked_amount: amount_to_lock,
						}),
					))
				} else {
					let (payout_amount, payout_amount_u256) = do_binance_swap(
						self.oe_client_binance.clone(),
						&self.evm_oe_client_accounting,
						swap_order.from_asset.clone(),
						from_amount.clone(),
						BinanceNetwork::Bsc,
						BinanceCoin::Bnb,
					)
					.await?;

					do_payout(&self.evm_oe_client_accounting, payout_address, payout_amount_u256)
						.await?;

					Ok((
						self.apply_omni_gas_fee(&payout_amount, BinanceCoin::Bnb).await?,
						to_address,
						None,
					))
				}
			},
			// BSC to SOL
			(ChainAsset::Ethereum(56, _), ChainAsset::Solana(_)) => {
				// BSC chain ID is 56
				let payout_address = Pubkey::from_str(&to_address).map_err(|_| {
					error!("Failed to parse payout address");
				})?;
				debug!("omni cross chain swap details: intent_id: {}, from_amount: {}, from_address: {}, payout_address: {}", intent_id, from_amount, from_address, payout_address);

				let (payout_amount, payout_amount_u256) = do_binance_swap(
					self.oe_client_binance.clone(),
					&self.solana_oe_client_accounting,
					swap_order.from_asset.clone(),
					from_amount.clone(),
					BinanceNetwork::Sol,
					BinanceCoin::Sol,
				)
				.await?;

				do_payout(&self.solana_oe_client_accounting, payout_address, payout_amount_u256)
					.await?;

				Ok((
					self.apply_omni_gas_fee(&payout_amount, BinanceCoin::Sol).await?,
					to_address,
					None,
				))
			},
			_ => {
				error!(
					"Unsupported omni cross chain swap from {:?} to {:?}",
					swap_order.from_asset, swap_order.to_asset
				);
				Err(())
			},
		}
	}

	async fn apply_omni_gas_fee(
		&self,
		payout_amount: &str,
		payout_coin: BinanceCoin,
	) -> Result<String, ()> {
		debug!("Applying omni gas fee calculation");

		// For omni, we use a simple fixed gas fee calculation
		let gas_fee = match payout_coin {
			BinanceCoin::Bnb => "0.001", // 0.001 BNB
			BinanceCoin::Sol => "0.01",  // 0.01 SOL
			_ => "0.001",                // default
		};

		debug!("Using fixed gas fee: {} for coin: {:?}", gas_fee, payout_coin);

		calculate_amount_in(payout_amount, gas_fee, payout_coin.decimals())
	}
}
