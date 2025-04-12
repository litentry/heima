use crate::{
	error_code::*, oneshot, server::RpcContext, verify_auth::verify_auth_token_authentication,
	Decode, Deserialize, ErrorCode,
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
use native_task_handler::{NativeTaskOk, NativeTaskResponse};
use pumpx::constants::*;
use pumpx::types::{MarketOrderTxResponse, OrderInfoResponse, SwapType};
use serde::Serialize;

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
				log::error!("Unsupported chain id: {}", chain_id);
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
	pub market_order_response: Option<MarketOrderTxResponse>,
}

pub fn register_submit_swap_order(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_submitSwapOrder", |params, ctx, _| async move {
			let params =
				params.parse::<SubmitSwapOrderParams>().map_err(|_| ErrorCode::ParseError)?;
			let user_identity =
				Identity::from_web2_account(&params.user_email, Web2IdentityType::Email);
			if verify_auth_token_authentication(ctx.clone(), &user_identity, &params.auth_token)
				.is_err()
			{
				log::error!("Failed to verify auth token");
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE));
			}

			let from_chain_asset = params.try_get_from_chain_asset().map_err(|_| {
				log::error!("Failed to get from chain asset");
				ErrorCode::InvalidParams
			})?;
			let to_chain_asset = params.try_get_to_chain_asset().map_err(|_| {
				log::error!("Failed to get to chain asset");
				ErrorCode::InvalidParams
			})?;

			if params.order_type == PumpxOrderType::Limit
				&& !from_chain_asset.is_same_chain(&to_chain_asset)
			{
				log::error!("Limit order must be on the same chain");
				return Err(ErrorCode::InvalidParams);
			}

			let from_amount = BoundedVec::try_from(params.from_amount.as_bytes().to_vec())
				.map_err(|_| {
					log::error!("Failed to convert from_amount to BoundedVec");
					ErrorCode::InvalidParams
				})?;

			let swap_order = SwapOrder {
				from_asset: from_chain_asset,
				to_asset: to_chain_asset,
				from_amount,
				to_address: None,
			};

			let storage = PumpxJwtStorage::new(ctx.storage_db.clone());
			let Some(access_token) =
				storage.get(&(user_identity.to_omni_account(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				log::error!("Failed to get access token from storage");
				return Err(ErrorCode::InternalError);
			};
			let user_trade_info =
				ctx.pumpx_api.get_user_trade_info(&access_token).await.map_err(|_| {
					log::error!("Failed to get user trade info");
					ErrorCode::InvalidParams
				})?;
			let gas_type = match params.to_chain_id {
				BASE_CHAIN_ID => user_trade_info.data.gas_type_base.to_number() as u32,
				ETHEREUM_CHAIN_ID => user_trade_info.data.gas_type_eth.to_number() as u32,
				BSC_CHAIN_ID => user_trade_info.data.gas_type_bsc.to_number() as u32,
				SOLANA_CHAIN_ID => user_trade_info.data.gas_type_base.to_number() as u32,
				_ => {
					log::error!("Unsupported chain id: {}", params.to_chain_id);
					return Err(ErrorCode::InvalidParams);
				},
			};
			let token_ca =
				BoundedVec::try_from(params.to_token_ca.unwrap_or_default().as_bytes().to_vec())
					.map_err(|_| ErrorCode::InvalidParams)?;
			let token_cap = params
				.token_cap
				.map(|token_ca| {
					BoundedVec::try_from(token_ca.as_bytes().to_vec())
						.map_err(|_| ErrorCode::InvalidParams)
				})
				.transpose()?;
			let price_usd = params
				.price_usd
				.map(|price_usd| {
					BoundedVec::try_from(price_usd.as_bytes().to_vec())
						.map_err(|_| ErrorCode::InvalidParams)
				})
				.transpose()?;
			let usd_worth = BoundedVec::try_from(params.usd_worth.as_bytes().to_vec())
				.map_err(|_| ErrorCode::InvalidParams)?;

			let pumpx_config = PumpxConfig {
				order_type: params.order_type.clone(),
				swap_type: params.swap_type.to_number() as u32,
				chain_id: params.to_chain_id,
				token_ca,
				double_out: params.double_out,
				is_one_click: params.is_one_click,
				is_anti_mev: user_trade_info.data.is_anti_mev,
				is_auto_slippage: user_trade_info.data.is_auto_slippage,
				gas_type,
				slippage: user_trade_info.data.slippage,
				wallet_index: params.wallet_index,
				token_cap,
				price_usd,
				usd_worth,
				trailing_percent: params.trailing_percent,
			};
			let scs_provider = SingleChainSwapProvider::Pumpx(pumpx_config);
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

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError);
			}
			match response_receiver.await {
				Ok(response) => {
					let native_task_response: NativeTaskResponse =
						Decode::decode(&mut response.as_slice())
							.map_err(|_| ErrorCode::InternalError)?;

					match native_task_response {
						Ok(NativeTaskOk::IntentSwapResponse(swap_response)) => {
							if params.order_type == PumpxOrderType::Market {
								let market_order_response: MarketOrderTxResponse =
									Decode::decode(&mut swap_response.as_slice())
										.map_err(|_| ErrorCode::InternalError)?;
								let response = PumpxSubmitSwapOrderResponse {
									backend_response: BackendResponse {
										limit_order_response: None,
										market_order_response: Some(market_order_response),
									},
								};
								Ok(response)
							} else {
								let limit_order_response: OrderInfoResponse =
									Decode::decode(&mut swap_response.as_slice())
										.map_err(|_| ErrorCode::InternalError)?;
								let response = PumpxSubmitSwapOrderResponse {
									backend_response: BackendResponse {
										limit_order_response: Some(limit_order_response),
										market_order_response: None,
									},
								};
								Ok(response)
							}
						},
						Err(native_task_err) => {
							log::error!("Failed to execute native task: {:?}", native_task_err);
							Err(ErrorCode::InternalError)
						},
						_ => {
							log::error!("Unexpected response type");
							Err(ErrorCode::InternalError)
						},
					}
				},
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError)
				},
			}
		})
		.expect("Failed to register pumpx_submitSwapOrder method");
}
