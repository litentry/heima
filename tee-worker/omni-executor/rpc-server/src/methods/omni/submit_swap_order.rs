use crate::methods::omni::PumpxRpcError;
use crate::{
	error_code::*, server::RpcContext, verify_auth::verify_auth_token_authentication, Decode,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use executor_storage::{PumpxJwtStorage, Storage};
use heima_authentication::auth_token::AUTH_TOKEN_ACCESS_TYPE;
use heima_hex_utils::decode_hex;
use heima_primitives::{
	Address20, Address32, BinanceConfig, BoundedVec, ChainAsset, CrossChainSwapProvider,
	EthereumToken, Identity, Intent, PumpxConfig, PumpxOrderType, SingleChainSwapProvider,
	SolanaToken, SwapOrder, Web2IdentityType,
};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::constants::*;
use pumpx::methods::common::{OrderInfoResponse, SwapType};
use pumpx::methods::create_market_order_tx::CreateMarketOrderTxResponse;
use serde::Serialize;

use super::common::{check_omni_api_response, handle_omni_native_task};

#[derive(Debug, Deserialize)]
pub struct SubmitSwapOrderParams {
	pub user_email: String,
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
	pub auth_token: String,
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
				tracing::log::error!("Unsupported chain id: {}", chain_id);
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
	pub market_order_response: Option<CreateMarketOrderTxResponse>,
}

pub fn register_submit_swap_order(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_submitSwapOrder", |params, ctx, _| async move {
			let params = params.parse::<SubmitSwapOrderParams>().map_err(|e| {
				tracing::log::error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			tracing::log::debug!("Received omni_submitSwapOrder, user_email: {}, intent_id: {}, order_type: {:?}, swap_type: {:?}, from_chain_id: {}, from_token_ca: {:?}, from_amount: {}, to_chain_id: {}, to_token_ca: {:?}, wallet_index: {}",
			params.user_email, params.intent_id, params.order_type, params.swap_type, params.from_chain_id, params.from_token_ca, params.from_amount, params.to_chain_id, params.to_token_ca, params.wallet_index);

			let user_identity =
				Identity::from_web2_account(&params.user_email, Web2IdentityType::Email);
			if verify_auth_token_authentication(ctx.clone(), &user_identity, &params.auth_token)
				.is_err()
			{
				tracing::log::error!("Failed to verify auth token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			}

			let from_chain_asset = params.try_get_from_chain_asset().map_err(|_| {
				tracing::log::error!("Failed to get from chain asset");
				PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
			})?;
			let to_chain_asset = params.try_get_to_chain_asset().map_err(|_| {
				tracing::log::error!("Failed to get to chain asset");
				PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
			})?;

			if params.order_type == PumpxOrderType::Limit
				&& !from_chain_asset.is_same_chain(&to_chain_asset)
			{
				tracing::log::error!("Limit order must be on the same chain");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InvalidParams));
			}

			let from_amount = BoundedVec::try_from(params.from_amount.as_bytes().to_vec())
				.map_err(|_| {
					tracing::log::error!("Failed to convert from_amount to BoundedVec");
					PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
				})?;

			let storage = PumpxJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(user_identity.to_omni_account(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				tracing::log::error!("Failed to get access token from storage");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			let swap_order = SwapOrder {
				from_asset: from_chain_asset,
				to_asset: to_chain_asset,
				from_amount,
				to_address: None,
			};

			tracing::log::debug!("Calling pumpx get_user_trade_info, user_email: {}", params.user_email);
			let user_trade_info =
				ctx.pumpx_api.get_user_trade_info(&access_token).await.map_err(|e| {
					tracing::log::error!("Failed to get user trade info: {:?}", e);
					PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
				})?;

			tracing::log::debug!("Response pumpx get_user_trade_info: {:?}", user_trade_info);

			let gas_type_base = check_and_get_option_user_trade_info_field(user_trade_info.data.gas_type_base, "gas_type_base")?;
			let gas_type_bsc = check_and_get_option_user_trade_info_field(user_trade_info.data.gas_type_bsc, "gas_type_bsc")?;
			let gas_type_eth = check_and_get_option_user_trade_info_field(user_trade_info.data.gas_type_eth, "gas_type_eth")?;
			let is_anti_mev = check_and_get_option_user_trade_info_field(user_trade_info.data.is_anti_mev, "is_anti_mev")?;
			let is_auto_slippage = check_and_get_option_user_trade_info_field(user_trade_info.data.is_auto_slippage, "is_auto_slippage")?;
			let slippage = check_and_get_option_user_trade_info_field(user_trade_info.data.slippage, "slippage")?;

			let gas_type = match params.to_chain_id {
				BASE_CHAIN_ID => gas_type_base.to_number() as u32,
				ETHEREUM_CHAIN_ID => gas_type_eth.to_number() as u32,
				BSC_CHAIN_ID => gas_type_bsc.to_number() as u32,
				SOLANA_CHAIN_ID => gas_type_base.to_number() as u32,
				_ => {
					tracing::log::error!("Unsupported chain id: {}", params.to_chain_id);
					return Err(PumpxRpcError::from_error_code(ErrorCode::InvalidParams));
				},
			};
			let token_cap = params
				.token_cap
				.map(|token_ca| {
					BoundedVec::try_from(token_ca.as_bytes().to_vec()).map_err(|_| {
						tracing::log::error!("Failed to convert token_cap");
						PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
					})
				})
				.transpose()?;
			let price_usd = params
				.price_usd
				.map(|price_usd| {
					BoundedVec::try_from(price_usd.as_bytes().to_vec()).map_err(|_| {
						tracing::log::error!("Failed to convert price_usd");
						PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
					})
				})
				.transpose()?;
			let usd_worth =
				BoundedVec::try_from(params.usd_worth.as_bytes().to_vec()).map_err(|_| {
					tracing::log::error!("Failed to convert usd_worth");
					PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
				})?;

			let omni_config = PumpxConfig {
				order_type: params.order_type.clone(),
				swap_type: params.swap_type.to_number() as u32,
				from_chain_id: params.from_chain_id,
				from_token_ca: BoundedVec::try_from(
					params.from_token_ca.unwrap_or("".to_string()).as_bytes().to_vec(),
				)
				.map_err(|_| {
					tracing::log::error!("Failed to convert from_token_ca");
					PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
				})?,
				to_chain_id: params.to_chain_id,
				to_token_ca: BoundedVec::try_from(
					params.to_token_ca.unwrap_or("".to_string()).as_bytes().to_vec(),
				)
				.map_err(|_| {
					tracing::log::error!("Failed to convert to_token_ca");
					PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
				})?,
				from_amount: BoundedVec::try_from(params.from_amount.as_bytes().to_vec()).map_err(
					|_| {
						tracing::log::error!("Failed to convert from_amount");
						PumpxRpcError::from_error_code(ErrorCode::InvalidParams)
					},
				)?,
				double_out: params.double_out,
				is_one_click: params.is_one_click,
				is_anti_mev,
				is_auto_slippage,
				gas_type,
				slippage,
				wallet_index: params.wallet_index,
				token_cap,
				price_usd,
				usd_worth,
				trailing_percent: params.trailing_percent,
			};
			let scs_provider = SingleChainSwapProvider::Pumpx(omni_config);
			let mut ccs_provider: Option<CrossChainSwapProvider> = None;

			if params.from_chain_id != params.to_chain_id {
				// TODO: figure out the binance config
				ccs_provider = Some(CrossChainSwapProvider::Binance(BinanceConfig {}));
			}

			let intent = Intent::Swap(swap_order, ccs_provider, scs_provider);
			let wrapper = NativeTaskWrapper {
				task: NativeTask::RequestIntent(user_identity, params.intent_id, intent),
				nonce: None,
				auth: Some(OmniAuth::AuthToken(params.auth_token)),
			};

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::IntentSwapResponse(swap_response) => {
					if params.order_type == PumpxOrderType::Market {
						let market_order_response: CreateMarketOrderTxResponse =
							Decode::decode(&mut swap_response.as_slice()).map_err(|e| {
								tracing::log::error!("Failed to decode market order response: {:?}", e);
								PumpxRpcError::from_error_code(ErrorCode::InternalError)
							})?;
						check_omni_api_response(
							market_order_response.clone(),
							"Market order".into(),
						)?;
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
								tracing::log::error!("Failed to decode limit order response: {:?}", e);
								PumpxRpcError::from_error_code(ErrorCode::InternalError)
							})?;
						check_omni_api_response(
							limit_order_response.clone(),
							"Limit order".into(),
						)?;
						let response = PumpxSubmitSwapOrderResponse {
							backend_response: BackendResponse {
								limit_order_response: Some(limit_order_response),
								market_order_response: None,
							},
						};
						Ok(response)
					}
				},
				_ => {
					tracing::log::error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_submitSwapOrder method");
}

fn check_and_get_option_user_trade_info_field<T>(
	field_value: Option<T>,
	field_name: &str,
) -> Result<T, PumpxRpcError> {
	field_value.ok_or_else(|| {
		tracing::log::error!("Response data.{} of call get_user_trade_info is none", field_name);
		PumpxRpcError::from_error_code(ErrorCode::ServerError(
			PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE,
		))
	})
}
