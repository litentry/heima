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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::{
	build_client_with_cert, convert_balance_hex_json_value_to_u128, DataProviderConfig, Error,
	HttpError, ReqPath, RetryOption, RetryableRestPost,
};
use http::header::CONNECTION;
use http_req::response::Headers;
use itc_rest_client::{
	http_client::{HttpClient, SendWithCertificateVerification},
	rest_client::RestClient,
	RestPath,
};
use itp_rpc::{Id, RpcRequest};
use litentry_primitives::Web3Network;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
	format, str,
	string::{String, ToString},
	vec,
	vec::Vec,
};

pub trait Web3NetworkNoderealJsonrpcClient {
	fn create_nodereal_jsonrpc_client(
		&self,
		data_provider_config: &DataProviderConfig,
	) -> Option<NoderealJsonrpcClient>;
}

impl Web3NetworkNoderealJsonrpcClient for Web3Network {
	fn create_nodereal_jsonrpc_client(
		&self,
		data_provider_config: &DataProviderConfig,
	) -> Option<NoderealJsonrpcClient> {
		match self {
			Web3Network::Bsc => {
				Some(NoderealJsonrpcClient::new(NoderealChain::Bsc, data_provider_config))
			},
			Web3Network::Ethereum => {
				Some(NoderealJsonrpcClient::new(NoderealChain::Eth, data_provider_config))
			},
			Web3Network::Polygon => {
				Some(NoderealJsonrpcClient::new(NoderealChain::Polygon, data_provider_config))
			},
			Web3Network::Combo => {
				Some(NoderealJsonrpcClient::new(NoderealChain::Combo, data_provider_config))
			},
			_ => None,
		}
	}
}

// https://docs.nodereal.io/reference/getting-started-with-your-api
pub enum NoderealChain {
	// BNB Smart Chain
	Bsc,
	// opBNB
	Opbnb,
	// Ethereum
	Eth,
	// ETH Beacon Chain
	Eth2Beacon,
	// Aptos
	Aptos,
	// Optimism
	Opt,
	// Polygon
	Polygon,
	// Combo
	Combo,
}

impl NoderealChain {
	pub fn to_string(&self) -> &'static str {
		match self {
			NoderealChain::Bsc => "bsc",
			NoderealChain::Opbnb => "opbnb",
			NoderealChain::Eth => "eth",
			NoderealChain::Eth2Beacon => "eth2-beacon",
			NoderealChain::Aptos => "aptos",
			NoderealChain::Opt => "opt",
			NoderealChain::Polygon => "polygon",
			NoderealChain::Combo => "combo",
		}
	}
}

impl<'a> RestPath<ReqPath<'a>> for RpcRequest {
	fn get_path(req: ReqPath) -> core::result::Result<String, HttpError> {
		Ok(req.path.into())
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcResponse {
	pub id: Id,
	pub jsonrpc: String,
	pub result: serde_json::Value,
}

pub struct NoderealJsonrpcClient {
	path: String,
	retry_option: RetryOption,
	client: RestClient<HttpClient<SendWithCertificateVerification>>,
}

impl NoderealJsonrpcClient {
	pub fn new(chain: NoderealChain, data_provider_config: &DataProviderConfig) -> Self {
		let api_key = data_provider_config.nodereal_api_key.clone();
		let api_retry_delay = data_provider_config.nodereal_api_retry_delay;
		let api_retry_times = data_provider_config.nodereal_api_retry_times;
		let api_url = data_provider_config.nodereal_api_chain_network_url.clone();
		let base_url = api_url.replace("{chain}", chain.to_string());
		let retry_option =
			RetryOption { retry_delay: Some(api_retry_delay), retry_times: Some(api_retry_times) };
		let path = format!("v1/{}", api_key);

		let mut headers = Headers::new();
		headers.insert(CONNECTION.as_str(), "close");
		let client = build_client_with_cert(base_url.as_str(), headers);

		NoderealJsonrpcClient { path, retry_option, client }
	}

	fn post(&mut self, body: &RpcRequest, fast_fail: bool) -> Result<RpcResponse, Error> {
		let path = ReqPath::new(self.path.as_str());
		let retry_option = if fast_fail { None } else { Some(self.retry_option.clone()) };
		self.client
			.post_capture_retry::<ReqPath, RpcRequest, RpcResponse>(path, body, retry_option)
	}
}

#[derive(Serialize, Debug)]
pub struct GetNFTHoldingsParam {
	// The address of the account in hex format
	pub account_address: String,
	// Please specify the type of token you query for, e.g. "ERC721", "ERC1155", etc.
	pub token_type: String,
	// page number in hex format
	pub page: usize,
	// pageSize is hex encoded and should be less equal than 100 (each page return at most pageSize items)
	pub page_size: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNFTHoldingsResult {
	// number in hex format
	pub total_count: String,
	pub details: Vec<GetNFTHoldingsResultDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNFTHoldingsResultDetail {
	// the address of the token
	pub token_address: String,
	// the name of the token
	pub token_name: String,
	// the symbol of the token
	pub token_symbol: String,
	// the id number of the token
	pub token_id_num: String,
}

#[derive(Serialize, Debug)]
pub struct GetTokenBalance721Param {
	// The address of the ERC721/BEP721 token
	pub token_address: String,
	// Account address whose balance will be checked
	pub account_address: String,
	// The block number in hex format or the string 'latest' or 'earliest' on which the balance will be checked
	pub block_number: String,
}

#[derive(Serialize, Debug)]
pub struct GetTokenBalance1155Param {
	// The address of the ERC1155/BEP1155 token
	pub token_address: String,
	// Account address whose balance will be checked
	pub account_address: String,
	// The block number in hex format or the string 'latest' or 'earliest' on which the balance will be checked
	pub block_number: String,
	// The tokenId in hex format of the ERC1155/BEP1155 token
	pub token_id: String,
}

#[derive(Serialize, Debug)]
pub struct GetNFTInventoryParam {
	// The address of the account in hex format
	pub account_address: String,
	// The address of the contract
	pub contract_address: String,
	// pageSize is hex encoded and should be less equal than 100 (each page return at most pageSize items)
	pub page_size: String,
	// It should be empty for the first page. If more results are available, a pageKey will be returned in the response. Pass the pageKey to fetch the next pageSize items.
	pub page_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNFTInventoryResult {
	// example: 100_342
	pub page_key: String,
	pub details: Vec<GetNFTInventoryResultDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNFTInventoryResultDetail {
	// the address of the token
	pub token_address: String,
	// the id of the token
	pub token_id: String,
	// the balance of the token
	pub balance: String,
}

pub trait NftApiList {
	fn get_nft_holdings(
		&mut self,
		param: &GetNFTHoldingsParam,
		fast_fail: bool,
	) -> Result<GetNFTHoldingsResult, Error>;

	fn get_token_balance_721(
		&mut self,
		param: &GetTokenBalance721Param,
		fast_fail: bool,
	) -> Result<u128, Error>;

	fn get_token_balance_1155(
		&mut self,
		param: &GetTokenBalance1155Param,
		fast_fail: bool,
	) -> Result<u128, Error>;

	fn get_token_nft_inventory(
		&mut self,
		param: &GetNFTInventoryParam,
		fast_fail: bool,
	) -> Result<GetNFTInventoryResult, Error>;
}

// NFT API
impl NftApiList for NoderealJsonrpcClient {
	// https://docs.nodereal.io/reference/nr_getnftholdings
	fn get_nft_holdings(
		&mut self,
		param: &GetNFTHoldingsParam,
		fast_fail: bool,
	) -> Result<GetNFTHoldingsResult, Error> {
		let params: Vec<String> = vec![
			param.account_address.clone(),
			param.token_type.clone(),
			format!("0x{:X}", param.page),
			format!("0x{:X}", param.page_size),
		];
		debug!("get_nft_holdings: {:?}", param);
		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "nr_getNFTHoldings".to_string(),
			params,
			id: Id::Number(1),
		};
		self.post(&req_body, fast_fail)
			.map_err(|e| {
				debug!("get_nft_holdings, error: {:?}", e);
				e
			})
			.map(|resp: RpcResponse| {
				debug!("get_nft_holdings, response: {:?}", resp);
				serde_json::from_value(resp.result).unwrap()
			})
	}

	// https://docs.nodereal.io/reference/nr_gettokenbalance721
	fn get_token_balance_721(
		&mut self,
		param: &GetTokenBalance721Param,
		fast_fail: bool,
	) -> Result<u128, Error> {
		let params: Vec<String> = vec![
			param.token_address.clone(),
			param.account_address.clone(),
			param.block_number.clone(),
		];
		debug!("get_token_balance_721: {:?}", param);
		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "nr_getTokenBalance721".to_string(),
			params,
			id: Id::Number(1),
		};

		match self.post(&req_body, fast_fail) {
			Ok(resp) => {
				// result example: '0x', '0x8'
				debug!("get_token_balance_721, response: {:?}", resp);
				convert_balance_hex_json_value_to_u128(resp.result)
			},
			Err(e) => {
				debug!("get_token_balance_721, error: {:?}", e);
				Err(e)
			},
		}
	}

	// https://docs.nodereal.io/reference/nr_gettokenbalance1155
	fn get_token_balance_1155(
		&mut self,
		param: &GetTokenBalance1155Param,
		fast_fail: bool,
	) -> Result<u128, Error> {
		let params: Vec<String> = vec![
			param.token_address.clone(),
			param.account_address.clone(),
			param.block_number.clone(),
			param.token_id.clone(),
		];
		debug!("get_token_balance_1155: {:?}", param);
		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "nr_getTokenBalance1155".to_string(),
			params,
			id: Id::Number(1),
		};

		match self.post(&req_body, fast_fail) {
			Ok(resp) => {
				// result example: '0x', '0x8'
				debug!("get_token_balance_1155, response: {:?}", resp);
				convert_balance_hex_json_value_to_u128(resp.result)
			},
			Err(e) => {
				debug!("get_token_balance_1155, error: {:?}", e);
				Err(e)
			},
		}
	}

	// https://docs.nodereal.io/reference/nr_getnftinventory
	fn get_token_nft_inventory(
		&mut self,
		param: &GetNFTInventoryParam,
		fast_fail: bool,
	) -> Result<GetNFTInventoryResult, Error> {
		let params: Vec<String> = vec![
			param.account_address.clone(),
			param.contract_address.clone(),
			param.page_size.clone(),
			param.page_key.clone(),
		];
		debug!("get_token_nft_inventory: {:?}", param);
		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "nr_getNFTInventory".to_string(),
			params,
			id: Id::Number(1),
		};

		match self.post(&req_body, fast_fail) {
			Ok(resp) => {
				debug!("get_token_nft_inventory, response: {:?}", resp);
				match serde_json::from_value::<GetNFTInventoryResult>(resp.result) {
					Ok(result) => Ok(result),
					Err(e) => Err(Error::RequestError(format!("{:?}", e))),
				}
			},
			Err(e) => {
				debug!("get_token_nft_inventory, error: {:?}", e);
				Err(e)
			},
		}
	}
}

#[derive(Serialize, Debug)]
pub struct GetTokenBalance20Param {
	// The address of the contract
	pub contract_address: String,
	// Target address
	pub address: String,
	// The block number in hex format or the string 'latest' or 'earliest' on which the balance will be checked
	pub block_number: String,
}

// Fungible Tokens API
pub trait FungibleApiList {
	fn get_token_balance_20(
		&mut self,
		param: &GetTokenBalance20Param,
		fast_fail: bool,
	) -> Result<u128, Error>;
	fn get_token_holdings(&mut self, address: &str, fast_fail: bool) -> Result<RpcResponse, Error>;
}

impl FungibleApiList for NoderealJsonrpcClient {
	// https://docs.nodereal.io/reference/nr_gettokenbalance20
	fn get_token_balance_20(
		&mut self,
		param: &GetTokenBalance20Param,
		fast_fail: bool,
	) -> Result<u128, Error> {
		let params: Vec<String> =
			vec![param.contract_address.clone(), param.address.clone(), param.block_number.clone()];
		debug!("get_token_balance_20: {:?}", param);
		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "nr_getTokenBalance20".to_string(),
			params,
			id: Id::Number(1),
		};

		match self.post(&req_body, fast_fail) {
			Ok(resp) => {
				// result example: '0x', '0x8'
				debug!("get_token_balance_20, response: {:?}", resp);
				convert_balance_hex_json_value_to_u128(resp.result)
			},
			Err(e) => {
				debug!("get_token_balance_20, error: {:?}", e);
				Err(e)
			},
		}
	}

	fn get_token_holdings(&mut self, address: &str, fast_fail: bool) -> Result<RpcResponse, Error> {
		let params: Vec<String> = vec![address.to_string(), "0x1".to_string(), "0x64".to_string()];
		debug!("get_token_holdings: {:?}", params);

		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "nr_getTokenHoldings".to_string(),
			params: params.to_vec(),
			id: Id::Number(1),
		};

		match self.post(&req_body, fast_fail) {
			Ok(resp) => {
				debug!("get_token_holdings, response: {:?}", resp);
				Ok(resp)
			},
			Err(e) => {
				debug!("get_token_holdings, error: {:?}", e);
				Err(e)
			},
		}
	}
}

pub trait EthBalance {
	fn get_balance(&mut self, address: &str, fast_fail: bool) -> Result<u128, Error>;
}

impl EthBalance for NoderealJsonrpcClient {
	fn get_balance(&mut self, address: &str, fast_fail: bool) -> Result<u128, Error> {
		let params = vec![address.to_string(), "latest".to_string()];

		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "eth_getBalance".to_string(),
			params,
			id: Id::Number(1),
		};

		match self.post(&req_body, fast_fail) {
			Ok(resp) => {
				// result example: '0x', '0x8'
				debug!("eth_getBalance, response: {:?}", resp);
				convert_balance_hex_json_value_to_u128(resp.result)
			},
			Err(e) => {
				debug!("eth_getBalance, error: {:?}", e);
				Err(e)
			},
		}
	}
}

pub trait TransactionCount {
	fn get_transaction_count(&mut self, address: &str, fast_fail: bool) -> Result<u64, Error>;
}

impl TransactionCount for NoderealJsonrpcClient {
	fn get_transaction_count(&mut self, address: &str, fast_fail: bool) -> Result<u64, Error> {
		let params = vec![address.to_string(), "latest".to_string()];

		let req_body = RpcRequest {
			jsonrpc: "2.0".to_string(),
			method: "eth_getTransactionCount".to_string(),
			params,
			id: Id::Number(1),
		};

		match self.post(&req_body, fast_fail) {
			Ok(resp) => {
				// result example: '0x', '0x8'
				debug!("eth_getTransactionCount, response: {:?}", resp);
				match resp.result.as_str() {
					Some(result) => match u64::from_str_radix(&result[2..], 16) {
						Ok(balance) => Ok(balance),
						Err(_) => Err(Error::RequestError(format!(
							"Cannot parse result {:?} to u64",
							result
						))),
					},
					None => Err(Error::RequestError(format!(
						"Cannot transform response result {:?} to &str",
						resp.result
					))),
				}
			},
			Err(e) => {
				debug!("get_transaction_count, error: {:?}", e);
				Err(e)
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use lc_mock_server::run;

	fn init() -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap() + "/nodereal_jsonrpc/";

		let mut config = DataProviderConfig::new().unwrap();
		config.set_nodereal_api_key("d416f55179dbd0e45b1a8ed030e3".to_string());
		config.set_nodereal_api_chain_network_url(url).unwrap();
		config
	}

	#[test]
	fn does_get_nft_holdings_works() {
		let config = init();
		let mut client = NoderealJsonrpcClient::new(NoderealChain::Eth, &config);
		let param = GetNFTHoldingsParam {
			account_address: "0x49AD262C49C7aA708Cc2DF262eD53B64A17Dd5EE".into(),
			token_type: "ERC721".into(),
			page: 1,
			page_size: 2,
		};
		let result = client.get_nft_holdings(&param, false).unwrap();
		assert_eq!(result.total_count, "0x1");
		assert_eq!(result.details.len(), 1);
		assert_eq!(result.details[0].token_address, "0x9401518f4EBBA857BAA879D9f76E1Cc8b31ed197");
		assert_eq!(result.details[0].token_id_num, "0x12");
		assert_eq!(result.details[0].token_name, "Pancake Lottery Ticket");
		assert_eq!(result.details[0].token_symbol, "PLT");
	}

	#[test]
	fn does_get_token_balance_721_works() {
		let config = init();
		let mut client = NoderealJsonrpcClient::new(NoderealChain::Eth, &config);
		let param = GetTokenBalance721Param {
			token_address: "0x07D971C03553011a48E951a53F48632D37652Ba1".into(),
			account_address: "0x49AD262C49C7aA708Cc2DF262eD53B64A17Dd5EE".into(),
			block_number: "latest".into(),
		};
		let result = client.get_token_balance_721(&param, false).unwrap();
		assert_eq!(result, 1);
	}

	#[test]
	fn does_get_token_balance_20_works() {
		let config = init();
		let mut client = NoderealJsonrpcClient::new(NoderealChain::Eth, &config);
		let param = GetTokenBalance20Param {
			contract_address: "0x76A797A59Ba2C17726896976B7B3747BfD1d220f".into(),
			address: "0x85Be4e2ccc9c85BE8783798B6e8A101BDaC6467F".into(),
			block_number: "latest".into(),
		};
		let result = client.get_token_balance_20(&param, false).unwrap();
		assert_eq!(result, 800);
	}

	#[test]
	fn does_get_token_balance_1155_works() {
		let config = init();
		let mut client = NoderealJsonrpcClient::new(NoderealChain::Eth, &config);
		let param = GetTokenBalance1155Param {
			token_address: "0x07D971C03553011a48E951a53F48632D37652Ba1".into(),
			account_address: "0x49AD262C49C7aA708Cc2DF262eD53B64A17Dd5EE".into(),
			block_number: "latest".into(),
			token_id: "0x0000000000000000000000000000000000000000f".into(),
		};
		let result = client.get_token_balance_1155(&param, false).unwrap();
		assert_eq!(result, 1);
	}

	#[test]
	fn does_get_token_nft_inventory_works() {
		let config = init();
		let mut client = NoderealJsonrpcClient::new(NoderealChain::Eth, &config);
		let param = GetNFTInventoryParam {
			account_address: "0x0042f9b78c67eb30c020a56d07f9a2fc83bc2514".into(),
			contract_address: "0x64aF96778bA83b7d4509123146E2B3b07F7deF52".into(),
			page_size: "0x14".into(),
			page_key: "".into(),
		};
		let result = client.get_token_nft_inventory(&param, false).unwrap();
		assert_eq!(result.page_key, "100_342");
		assert_eq!(result.details.len(), 1);
		assert_eq!(result.details[0].token_address, "0x5e74094cd416f55179dbd0e45b1a8ed030e396a1");
		assert_eq!(result.details[0].token_id, "0x0000000000000000000000000000000000000000f");
		assert_eq!(result.details[0].balance, "0x00000000000000000000000000000000000000001");
	}
}
