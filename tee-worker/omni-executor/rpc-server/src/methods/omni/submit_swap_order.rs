use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE, PARSE_ERROR_CODE, *},
	methods::omni::{check_auth, check_omni_api_response},
	server::RpcContext,
	Decode, Deserialize, RpcResult,
};
use executor_core::intent_executor::IntentExecutor;
use executor_storage::{HeimaJwtStorage, IntentIdStorage, Storage};
use heima_authentication::constants::AUTH_TOKEN_ACCESS_TYPE;
use heima_primitives::{
	AccountId, Address20, Address32, BinanceConfig, BoundedVec, ChainAsset, CrossChainSwapProvider,
	EthereumToken, Intent, PumpxConfig, PumpxOrderType, SingleChainSwapProvider, SolanaToken,
	SwapOrder,
};
use heima_utils::decode_hex;
use jsonrpsee::RpcModule;
use pumpx::constants::*;
use pumpx::methods::common::{OrderInfoResponse, SwapType};
use pumpx::methods::send_order_tx::SendOrderTxResponse;
use serde::Serialize;
use std::str::FromStr;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct SubmitSwapOrderParams {
	pub intent_id: u32,
	pub order_type: PumpxOrderType,
	pub swap_type: SwapType,
	pub from_chain_id: u32,
	pub from_token_ca: Option<String>,
	pub from_amount: String,
	pub to_chain_id: u32,
	pub to_token_ca: Option<String>,
	pub double_out: bool,
	pub is_one_click: bool,
	pub token_cap: Option<String>,
	pub price_usd: Option<String>,
	pub usd_worth: String,
	pub trailing_percent: Option<u32>,
	pub wallet_index: u32,
}

impl SubmitSwapOrderParams {
	pub fn try_get_from_chain_asset(&self) -> Result<ChainAsset, ()> {
		let from_chain_id = self.from_chain_id;
		let from_token_ca = self.from_token_ca.clone();
		Self::try_get_chain_asset(from_chain_id, from_token_ca)
	}

	pub fn try_get_to_chain_asset(&self) -> Result<ChainAsset, ()> {
		let to_chain_id = self.to_chain_id;
		let to_token_ca = self.to_token_ca.clone();
		Self::try_get_chain_asset(to_chain_id, to_token_ca)
	}

	fn try_get_chain_asset(chain_id: u32, token_ca: Option<String>) -> Result<ChainAsset, ()> {
		match chain_id {
			SOLANA_CHAIN_ID => {
				let solana_token = match token_ca {
					None => SolanaToken::Native,
					Some(ref s) if s.is_empty() => SolanaToken::Native,
					Some(ref mint_address) => {
						let mint_address: Address32 = mint_address.as_str().try_into()?;
						SolanaToken::SPL(mint_address)
					},
				};
				Ok(ChainAsset::Solana(solana_token))
			},
			ETHEREUM_CHAIN_ID | BSC_CHAIN_ID | BASE_CHAIN_ID => {
				let eth_token = match token_ca {
					None => EthereumToken::Native,
					Some(ref s) if s.is_empty() => EthereumToken::Native,
					Some(ref token_address) => {
						let token_address: Address20 = decode_hex(token_address)
							.map_err(|_| ())?
							.as_slice()
							.try_into()
							.map_err(|_| ())?;
						EthereumToken::ERC20(token_address)
					},
				};
				Ok(ChainAsset::Ethereum(chain_id, eth_token))
			},
			_ => {
				error!("Unsupported chain id: {}", chain_id);
				Err(())
			},
		}
	}
}

// TODO: refactor this response to make it more generic and also support binance swaps responses
#[derive(Serialize, Clone)]
pub struct PumpxSubmitSwapOrderResponse {
	backend_response: BackendResponse,
}

#[derive(Serialize, Clone)]
struct BackendResponse {
	pub limit_order_response: Option<OrderInfoResponse>,
	pub market_order_response: Option<SendOrderTxResponse>,
}

pub fn register_submit_swap_order<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_submitSwapOrder", |params, ctx, ext| async move {
			let omni_account = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_suggestion("Please check your authentication credentials")
				.to_rpc_error()
			})?;

			let params = params.parse::<SubmitSwapOrderParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
					.to_rpc_error()
			})?;

			debug!("Received omni_submitSwapOrder, params: {:?}", params);

			let Ok(omni_account_id) = AccountId::from_str(&omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("Failed to parse omni account from authentication token")
					.to_rpc_error());
			};

			let from_chain_asset = params.try_get_from_chain_asset().map_err(|_| {
				error!("Failed to get from chain asset");
				DetailedError::new(INVALID_PARAMS_CODE, "Invalid params")
					.with_suggestion("Invalid method parameters")
					.to_rpc_error()
			})?;
			let to_chain_asset = params.try_get_to_chain_asset().map_err(|_| {
				error!("Failed to get to chain asset");
				DetailedError::new(INVALID_PARAMS_CODE, "Invalid params")
					.with_suggestion("Invalid method parameters")
					.to_rpc_error()
			})?;

			if params.order_type == PumpxOrderType::Limit
				&& !from_chain_asset.is_same_chain(&to_chain_asset)
			{
				error!("Limit order must be on the same chain");
				return Err(DetailedError::new(INVALID_PARAMS_CODE, "Invalid params")
					.with_suggestion("Invalid method parameters")
					.to_rpc_error());
			}

			let from_amount = BoundedVec::try_from(params.from_amount.as_bytes().to_vec())
				.map_err(|_| {
					error!("Failed to convert from_amount to BoundedVec");
					DetailedError::new(INVALID_PARAMS_CODE, "Invalid params")
						.with_suggestion("Invalid method parameters")
						.to_rpc_error()
				})?;

			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(omni_account_id.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get access token from storage");
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("Failed to get access token from storage")
					.to_rpc_error());
			};

			let swap_order = SwapOrder {
				from_asset: from_chain_asset,
				to_asset: to_chain_asset,
				from_amount,
				to_address: None,
			};

			debug!("Calling pumpx get_user_trade_info");
			let user_trade_info =
				ctx.pumpx_api.get_user_trade_info(&access_token).await.map_err(|e| {
					error!("Failed to get user trade info: {:?}", e);
					DetailedError::new(INVALID_PARAMS_CODE, "Invalid params")
						.with_suggestion("Invalid method parameters")
						.to_rpc_error()
				})?;

			debug!("Response pumpx get_user_trade_info: {:?}", user_trade_info);

			let gas_type_base = check_and_get_option_user_trade_info_field(
				user_trade_info.data.gas_type_base,
				"gas_type_base",
			)?;
			let gas_type_bsc = check_and_get_option_user_trade_info_field(
				user_trade_info.data.gas_type_bsc,
				"gas_type_bsc",
			)?;
			let gas_type_eth = check_and_get_option_user_trade_info_field(
				user_trade_info.data.gas_type_eth,
				"gas_type_eth",
			)?;
			let gas_type_sol = check_and_get_option_user_trade_info_field(
				user_trade_info.data.gas_type_sol,
				"gas_type_sol",
			)?;

			let is_anti_mev = check_and_get_option_user_trade_info_field(
				user_trade_info.data.is_anti_mev,
				"is_anti_mev",
			)?;
			let is_auto_slippage = check_and_get_option_user_trade_info_field(
				user_trade_info.data.is_auto_slippage,
				"is_auto_slippage",
			)?;
			let slippage = check_and_get_option_user_trade_info_field(
				user_trade_info.data.slippage,
				"slippage",
			)?;

			let gas_type = match params.to_chain_id {
				BASE_CHAIN_ID => gas_type_base.to_number() as u32,
				ETHEREUM_CHAIN_ID => gas_type_eth.to_number() as u32,
				BSC_CHAIN_ID => gas_type_bsc.to_number() as u32,
				SOLANA_CHAIN_ID => gas_type_sol.to_number() as u32,
				_ => {
					error!("Unsupported chain id: {}", params.to_chain_id);
					return Err(DetailedError::new(INVALID_PARAMS_CODE, "Invalid params")
						.with_suggestion("Invalid method parameters")
						.to_rpc_error());
				},
			};

			let omni_config = PumpxConfig {
				order_type: params.order_type.clone(),
				swap_type: params.swap_type.to_number() as u32,
				from_chain_id: params.from_chain_id,
				from_token_ca: params.from_token_ca.unwrap_or("".to_string()),
				to_chain_id: params.to_chain_id,
				to_token_ca: params.to_token_ca.unwrap_or("".to_string()),
				from_amount: params.from_amount,
				double_out: params.double_out,
				is_one_click: params.is_one_click,
				is_anti_mev,
				is_auto_slippage,
				gas_type,
				slippage,
				wallet_index: params.wallet_index,
				token_cap: params.token_cap,
				price_usd: params.price_usd,
				usd_worth: params.usd_worth,
				trailing_percent: params.trailing_percent,
			};
			let scs_provider = SingleChainSwapProvider::Pumpx(omni_config);
			let mut ccs_provider: Option<CrossChainSwapProvider> = None;

			if params.from_chain_id != params.to_chain_id {
				// TODO: figure out the binance config
				ccs_provider = Some(CrossChainSwapProvider::Binance(BinanceConfig {}));
			}

			let intent = Intent::Swap(
				swap_order,
				ccs_provider,
				scs_provider.try_into().map_err(|_| {
					error!("Failed to convert single chain swap provider");
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to convert single chain swap provider")
						.to_rpc_error()
				})?,
			);

			// Inlined handler logic from handle_request_intent
			debug!("Intent requested, intent_id: {}", params.intent_id);

			let intent_id_storage = IntentIdStorage::new(ctx.storage_db.clone());
			let stored_intent_id = match intent_id_storage.get(&omni_account_id) {
				Ok(id) => id.unwrap_or_default(),
				Err(_) => {
					error!("Failed to read intent from store");
					return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to read intent from store")
						.to_rpc_error());
				},
			};

			if params.intent_id == stored_intent_id + 1 {
				if intent_id_storage.insert(&omni_account_id, params.intent_id).is_err() {
					error!("Failed to save intent id");
					return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to save intent id")
						.to_rpc_error());
				}
			} else {
				error!(
					"Intent id different than expected, expected: {:?}, got: {:?}",
					stored_intent_id + 1,
					params.intent_id
				);
				return Err(DetailedError::new(INVALID_PARAMS_CODE, "Intent nonce mismatch")
					.with_reason("Intent ID does not match expected value")
					.to_rpc_error());
			}

			let swap_response = match intent {
				Intent::SystemRemark(_)
				| Intent::TransferNative(_)
				| Intent::CallEthereum(_)
				| Intent::TransferEthereum(_)
				| Intent::TransferSolana(_) => {
					info!("Intent temporarily rejected, intent_id: {}", params.intent_id);
					return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("This intent type is temporarily not supported")
						.into());
				},
				Intent::Swap(..) => {
					let response = match ctx
						.cross_chain_intent_executor
						.execute(&omni_account_id, params.intent_id, intent.clone())
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
						response
					} else {
						return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Intent execution failed")
							.to_rpc_error());
					}
				},
			};

			// Process the swap response based on order type
			if params.order_type == PumpxOrderType::Market {
				let market_order_response: SendOrderTxResponse =
					Decode::decode(&mut swap_response.as_slice()).map_err(|e| {
						error!("Failed to decode market order response: {:?}", e);
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Failed to decode market order response: {:?}", e))
							.to_rpc_error()
					})?;
				check_omni_api_response(market_order_response.clone(), "Market order".into())?;
				let response = PumpxSubmitSwapOrderResponse {
					backend_response: BackendResponse {
						limit_order_response: None,
						market_order_response: Some(market_order_response),
					},
				};
				Ok(response)
			} else {
				let limit_order_response: OrderInfoResponse =
					Decode::decode(&mut swap_response.as_slice()).map_err(|e| {
						error!("Failed to decode limit order response: {:?}", e);
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason(format!("Failed to decode limit order response: {:?}", e))
							.to_rpc_error()
					})?;
				check_omni_api_response(limit_order_response.clone(), "Limit order".into())?;
				let response = PumpxSubmitSwapOrderResponse {
					backend_response: BackendResponse {
						limit_order_response: Some(limit_order_response),
						market_order_response: None,
					},
				};
				Ok(response)
			}
		})
		.expect("Failed to register omni_submitSwapOrder method");
}

fn check_and_get_option_user_trade_info_field<T>(
	field_value: Option<T>,
	field_name: &str,
) -> RpcResult<T> {
	field_value.ok_or_else(|| {
		error!("Response data.{} of call get_user_trade_info is none", field_name);
		DetailedError::new(
			PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE,
			"Failed to get user trade info from API",
		)
		.with_suggestion("User trade info retrieval failed. Please try again later.")
		.to_rpc_error()
	})
}
