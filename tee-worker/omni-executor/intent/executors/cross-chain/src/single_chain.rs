use super::*;
use alloy::consensus::transaction::RlpEcdsaEncodableTx;
use alloy::primitives::ChainId;
use executor_primitives::PumpxConfig;
use pumpx::methods::create_market_order_tx::CreateMarketOrderTxBody;
use sp_core::keccak_256;
use tracing::{debug, error};

impl<BinanceClient: BinanceApi, SolanaClient: SolanaClientTrait>
	CrossChainIntentExecutor<BinanceClient, SolanaClient>
{
	pub async fn execute_single_chain_swap(
		&self,
		omni_account: [u8; 32],
		intent_id: IntentId,
		access_token: &str,
		amount: String,
		pumpx_config: &PumpxConfig,
	) -> Result<Vec<u8>, ()> {
		debug!("executing single chain swap");

		let Some(to_chain_type) = ChainType::from_pumpx_chain_id(pumpx_config.to_chain_id) else {
			error!("Unsupported to_chain_id: {}", pumpx_config.to_chain_id);
			return Err(());
		};

		let to_wallet = self
			.pumpx_signer_client
			.request_wallet(to_chain_type, pumpx_config.wallet_index, omni_account)
			.await
			.map_err(|e| error!("Could not get to_wallet from pumpx-signer: {:?}", e))?;

		let to_address = pubkey_to_address(to_chain_type, &to_wallet)?;

		// always use `to_token_ca` from RPC
		let token_ca = std::str::from_utf8(&pumpx_config.to_token_ca)
			.map_err(|_| {
				error!("Failed to parse to_token_ca");
			})
			.map(|v| v.to_string())?;

		match pumpx_config.order_type {
			PumpxOrderType::Market => {
				debug!("Doing market order");
				self.worker_do_market_order(
					omni_account,
					intent_id,
					to_address.clone(),
					amount,
					token_ca,
					to_address,
					access_token,
					pumpx_config,
					to_chain_type,
				)
				.await
			},
			PumpxOrderType::Limit => {
				debug!("Doing limit order");
				let token_cap = match pumpx_config.token_cap {
					Some(ref token_cap) => Some(
						std::str::from_utf8(token_cap)
							.map_err(|_| {
								error!("Failed to parse token_cap");
							})
							.map(|v| v.to_string())?,
					),
					None => None,
				};
				let price_usd = match pumpx_config.price_usd {
					Some(ref price_usd) => Some(
						std::str::from_utf8(price_usd)
							.map_err(|_| {
								error!("Failed to parse price_usd");
							})
							.map(|v| v.to_string())?,
					),
					None => None,
				};

				self.do_limit_order(
					intent_id,
					amount,
					token_ca,
					to_address,
					token_cap,
					price_usd,
					access_token,
					pumpx_config,
				)
				.await
			},
		}
	}

	// call backend API in one step
	// currently unused, will be deleted once we remove the signer signing access for backend
	#[allow(unused)]
	async fn pumpx_do_market_order(
		&self,
		intent_id: IntentId,
		amount: String,
		token_ca: String,
		recipient_address: String,
		access_token: &str,
		pumpx_config: &PumpxConfig,
	) -> Result<Vec<u8>, ()> {
		debug!("executing pumpx_do_market_order");
		let body = CreateMarketOrderTxBody {
			request_id: intent_id,
			chain_id: pumpx_config.to_chain_id,
			token_ca,
			swap_type: match pumpx_config.swap_type {
				1 => SwapType::Buy,
				2 => SwapType::Sell,
				_ => {
					error!("Unsupported swap type: {}", pumpx_config.swap_type);
					return Err(());
				},
			},
			amount_in: amount,
			double_out: pumpx_config.double_out,
			is_one_click: pumpx_config.is_one_click,
			address: recipient_address,
			is_anti_mev: pumpx_config.is_anti_mev,
			is_auto_slippage: pumpx_config.is_auto_slippage,
			gas_type: match pumpx_config.gas_type {
				1 => GasType::Slow,
				2 => GasType::Medium,
				3 => GasType::Fast,
				_ => {
					error!("Unsupported gas type: {}", pumpx_config.gas_type);
					return Err(());
				},
			},
			slippage: pumpx_config.slippage,
			wallet_index: pumpx_config.wallet_index,
		};
		debug!("Calling pumpx create_market_order_tx, body: {:?}", body);
		let response =
			self.pumpx_api.create_market_order_tx(access_token, body).await.map_err(|_| {
				error!("Failed to create market order tx");
			})?;

		debug!("Response create_market_order_tx: {:?}", response);
		Ok(response.encode())
	}

	// call backend API in two steps, only worker has signing access to signer in this case
	async fn worker_do_market_order(
		&self,
		omni_account: [u8; 32],
		intent_id: IntentId,
		initiator_address: String, // the address that initiates/places the market order
		amount: String,
		token_ca: String,
		recipient_address: String,
		access_token: &str,
		pumpx_config: &PumpxConfig,
		to_chain_id: ChainType,
	) -> Result<Vec<u8>, ()> {
		debug!("executing worker_do_market_order");
		let body = CreateMarketOrderUnsignedTxBody {
			request_id: intent_id,
			chain_id: pumpx_config.to_chain_id,
			token_ca,
			swap_type: match pumpx_config.swap_type {
				1 => SwapType::Buy,
				2 => SwapType::Sell,
				_ => {
					error!("Unsupported swap type: {}", pumpx_config.swap_type);
					return Err(());
				},
			},
			amount_in: amount,
			double_out: pumpx_config.double_out,
			is_one_click: pumpx_config.is_one_click,
			address: initiator_address,
			is_anti_mev: pumpx_config.is_anti_mev,
			is_auto_slippage: pumpx_config.is_auto_slippage,
			gas_type: match pumpx_config.gas_type {
				1 => GasType::Slow,
				2 => GasType::Medium,
				3 => GasType::Fast,
				_ => {
					error!("Unsupported gas type: {}", pumpx_config.gas_type);
					return Err(());
				},
			},
			slippage: pumpx_config.slippage,
			wallet_index: pumpx_config.wallet_index,
			recipient_address,
		};

		let response = self
			.pumpx_api
			.create_market_order_unsigned_tx(access_token, body)
			.await
			.map_err(|_| error!("Failed to get unsigned market order tx"))?;

		debug!("Response create_market_order_unsigned_tx: {:?}", response);

		let unsigned_tx_string = response.data.tx_data.ok_or_else(|| {
			error!("Failed to unwrap tx_data");
		})?;
		let order_id = response.data.order_id.ok_or_else(|| {
			error!("Failed to unwrap order_id");
		})?;
		let chain_id = response.data.chain_id.ok_or_else(|| {
			error!("Failed to unwrap chain_id");
		})?;

		match to_chain_id {
			ChainType::Evm => {
				self.do_worker_evm_order(
					unsigned_tx_string,
					access_token,
					pumpx_config,
					omni_account,
					order_id,
					chain_id,
				)
				.await
			},
			ChainType::Solana => {
				self.do_worker_solana_order(
					unsigned_tx_string,
					access_token,
					pumpx_config,
					omni_account,
					order_id,
					chain_id,
				)
				.await
			},
			_ => {
				error!("Unsupported chain type");
				Err(())
			},
		}
	}

	async fn do_worker_evm_order(
		&self,
		unsigned_tx_string: Vec<String>,
		access_token: &str,
		pumpx_config: &PumpxConfig,
		omni_account: [u8; 32],
		order_id: u32,
		chain_id: u32,
	) -> Result<Vec<u8>, ()> {
		let unsigned_tx_bytes = unsigned_tx_string
			.iter()
			.map(|s| {
				let hex_str = s.trim_start_matches("0x");
				let hex_decoded = hex::decode(hex_str).expect("Invalid hex string");
				let mut unsigned_tx = TxLegacy::decode(&mut &hex_decoded[..])
					.map_err(|_| error!("Failed to decode legacy tx"))?;

				unsigned_tx.chain_id = Some(ChainId::from(chain_id));
				let mut rlp_encoded_tx = vec![];
				unsigned_tx.rlp_encode(&mut rlp_encoded_tx);

				Ok(keccak_256(&rlp_encoded_tx).to_vec())
			})
			.collect::<Result<Vec<Vec<u8>>, ()>>()?;

		let signatures = self
			.pumpx_signer_client
			.request_signatures(
				ChainType::Evm,
				pumpx_config.wallet_index,
				omni_account,
				unsigned_tx_bytes,
			)
			.await?;

		let mut tx_data: Vec<String> = vec![];
		for (x, y) in unsigned_tx_string.into_iter().zip(signatures.into_iter()) {
			let bytes = hex::decode(x.trim_start_matches("0x"))
				.map_err(|_| error!("invalid hex string"))?;
			// We should be able to decode it to Legacy Transaction
			// As it is RLP Encoded Bytes which adheres to string encoding rules
			let mut unsigned_tx = TxLegacy::decode(&mut &bytes[..])
				.map_err(|_| error!("Failed to decode legacy tx"))?;
			// We need to explicitly set the chain id
			unsigned_tx.chain_id = Some(ChainId::from(chain_id));
			let signature = Signature::try_from(y.as_ref())
				.map_err(|_| error!("Failed to create Typed signature"))?;

			let signed_tx = unsigned_tx.into_signed(signature);
			let mut encoded_signed_tx = vec![];
			signed_tx.rlp_encode(&mut encoded_signed_tx);
			tx_data.push(format!("0x{}", hex::encode(encoded_signed_tx)));
		}

		let response = self
			.pumpx_api
			.send_order_tx(access_token, SendOrderTxBody { order_id, chain_id, tx_data })
			.await
			.map_err(|_| error!("Failed to send order tx"))?;

		debug!("Response send_order_tx: {:?}", response);

		Ok(response.encode())
	}

	async fn do_worker_solana_order(
		&self,
		unsigned_tx_string: Vec<String>,
		access_token: &str,
		pumpx_config: &PumpxConfig,
		omni_account: [u8; 32],
		order_id: u32,
		chain_id: u32,
	) -> Result<Vec<u8>, ()> {
		// Note: Based on the go code, It is going to be mostly a single Transaction
		let unsigned_tx: Vec<solana_sdk::transaction::Transaction> = unsigned_tx_string
			.iter()
			.map(|tx| {
				let unsigned_tx_bytes = hex::decode(tx.trim_start_matches("0x")).unwrap();
				bincode::deserialize(&unsigned_tx_bytes[..])
					.map_err(|e| error!("Failed to deserialize string: {:?}", e))
			})
			.collect::<Result<Vec<solana_sdk::transaction::Transaction>, _>>()?;

		let messages_to_sign: Vec<Vec<u8>> =
			unsigned_tx.iter().map(|tx| tx.message_data()).collect();

		let signatures = self
			.pumpx_signer_client
			.request_signatures(
				ChainType::Solana,
				pumpx_config.wallet_index,
				omni_account,
				messages_to_sign,
			)
			.await?;

		let mut tx_data: Vec<String> = vec![];
		for (mut tx, sig) in unsigned_tx.into_iter().zip(signatures.into_iter()) {
			let signature = solana_sdk::signature::Signature::try_from(sig.as_ref())
				.map_err(|_| error!("Failed to convert to Solana Signature"))?;
			let num_required_signatures: usize = tx.message.header.num_required_signatures as usize;
			// Note: this is being done in the Go code as well, so although we technically have
			// only one signature we are still filling all the required placeholder
			// with the same signature
			for i in 0_usize..num_required_signatures {
				tx.signatures[i] = signature;
			}
			tx.verify().map_err(|e| {
				error!("Solana transaction verification failed: {:?}", e);
			})?;
			let encoded_tx = bincode::serialize(&tx).map_err(|e| {
				error!("Failed to serialize Solana transaction: {:?}", e);
			})?;
			tx_data.push(format!("0x{}", hex::encode(encoded_tx)));
		}

		let response = self
			.pumpx_api
			.send_order_tx(access_token, SendOrderTxBody { order_id, chain_id, tx_data })
			.await
			.map_err(|_| error!("Failed to send order tx"))?;

		debug!("Response send_order_tx: {:?}", response);

		Ok(response.encode())
	}

	async fn do_limit_order(
		&self,
		intent_id: IntentId,
		amount: String,
		token_ca: String,
		recipient_address: String,
		token_cap: Option<String>,
		price_usd: Option<String>,
		access_token: &str,
		pumpx_config: &PumpxConfig,
	) -> Result<Vec<u8>, ()> {
		let new_limit_order = CreateLimitOrderBody {
			request_id: intent_id,
			chain_id: pumpx_config.to_chain_id,
			token_ca,
			amount,
			swap_type: match pumpx_config.swap_type {
				1 => SwapType::Buy,
				2 => SwapType::Sell,
				_ => {
					error!("Unsupported swap type: {}", pumpx_config.swap_type);
					return Err(());
				},
			},
			double_out: pumpx_config.double_out,
			token_cap,
			price_usd,
			trailing_percent: pumpx_config.trailing_percent.map(|v| v.to_string()),
			address: recipient_address,
			is_anti_mev: pumpx_config.is_anti_mev,
			is_auto_slippage: pumpx_config.is_auto_slippage,
			gas_type: match pumpx_config.gas_type {
				1 => GasType::Slow,
				2 => GasType::Medium,
				3 => GasType::Fast,
				_ => {
					error!("Unsupported gas type: {}", pumpx_config.gas_type);
					return Err(());
				},
			},
			slippage: pumpx_config.slippage,
			wallet_index: pumpx_config.wallet_index,
		};
		debug!("Calling pumpx create_limit_order, order: {:?}", new_limit_order);
		let response =
			self.pumpx_api.create_limit_order(access_token, new_limit_order).await.map_err(
				|_| {
					error!("Failed to create limit order");
				},
			)?;

		debug!("Response create_limit_order: {:?}", response);
		Ok(response.encode())
	}
}
