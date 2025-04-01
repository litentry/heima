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
#![allow(opaque_hidden_inferred_bound)]

use itp_rpc::Id;
use lc_data_providers::nodereal_jsonrpc::{
	GetNFTHoldingsResult, GetNFTHoldingsResultDetail, GetNFTInventoryResult,
	GetNFTInventoryResultDetail, RpcResponse,
};
use warp::{http::Response, hyper::body::Bytes, Filter};
const RES_BODY_OK_GET_TOKEN_HOLDINGS: &str = r#"
{
	"id": "1",
	"jsonrpc": "2.0",
	"result": {
		"totalCount": "0x34",
		"nativeTokenBalance": "0x0",
		"details": [
			{
				"tokenAddress": "0xfcb5DF42e06A39E233dc707bb3a80311eFD11576",
				"tokenBalance": "0x0000000000000000000000000000000000000000f",
				"tokenName": "www.METH.co.in",
				"tokenSymbol": "METH"
			}
		]
	}
}
"#;

const RES_BODY_OK_GET_TRANSACTION_COUNT: &str = r#"
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": "0x1"
}
"#;

pub(crate) fn query() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::post()
		.and(warp::path!("nodereal_jsonrpc" / "v1" / String))
		.and(warp::body::bytes())
		.map(|_, body: Bytes| {
			let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
			let method = json.get("method").unwrap().as_str().unwrap();

			let params: Vec<String> = json
				.get("params")
				.unwrap()
				.as_array()
				.unwrap()
				.iter()
				.filter_map(|v| v.as_str().map(String::from))
				.collect();
			match method {
				"nr_getNFTHoldings" => {
					let result = GetNFTHoldingsResult {
						total_count: "0x1".into(),
						details: vec![GetNFTHoldingsResultDetail {
							token_address: "0x9401518f4EBBA857BAA879D9f76E1Cc8b31ed197".into(),
							token_id_num: "0x12".into(),
							token_name: "Pancake Lottery Ticket".into(),
							token_symbol: "PLT".into(),
						}],
					};
					let body = RpcResponse {
						jsonrpc: "2.0".into(),
						id: Id::Number(1),
						result: serde_json::to_value(result).unwrap(),
					};
					Response::builder().body(serde_json::to_string(&body).unwrap())
				},
				"nr_getTokenBalance721" => {
					let body = RpcResponse {
						jsonrpc: "2.0".into(),
						id: Id::Number(1),
						result: serde_json::to_value("0x1").unwrap(),
					};
					Response::builder().body(serde_json::to_string(&body).unwrap())
				},
				"nr_getTokenBalance1155" => {
					let body = RpcResponse {
						jsonrpc: "2.0".into(),
						id: Id::Number(1),
						result: serde_json::to_value("0x1").unwrap(),
					};
					Response::builder().body(serde_json::to_string(&body).unwrap())
				},
				"nr_getTokenBalance20" => {
					let value = match params[1].as_str() {
						// 100_000_000_000
						"0x85be4e2ccc9c85be8783798b6e8a101bdac6467f" => "0x174876E800",
						// 3000
						"0x90d53026a47ac20609accc3f2ddc9fb9b29bb310" => "0xBB8",
						// 50 * 10^18
						"0x45cdb67696802b9d01ed156b883269dbdb9c6239" => "0x2b5e3af16b1880000",
						// 400 * 10^18
						"0x75438d34c9125839c8b08d21b7f3167281659e7c" => "0x15af1d78b58c400000",
						// 2199 * 10^18
						"0xba359c153ad11aa17c3122b05a4db8b46bb3191b" => "0x7735416132dbfc0000",
						// 0
						"0x75438d34c9125839c8b08d21b7f3167281659e0c" => "0x0",
						// 0.01 * 10^18
						"0x75438d34c9125839c8b08d21b7f3167281659e1c" => "0x2386f26fc10000",
						// 20 * 10^18
						"0x75438d34c9125839c8b08d21b7f3167281659e2c" => "0x1158e460913d00000",
						// 5000 * 10^18
						"0x75438d34c9125839c8b08d21b7f3167281659e3c" => "0x10f0cf064dd59200000",
						// 120_000 * 10^18
						"0x75438d34c9125839c8b08d21b7f3167281659e4c" => "0x1969368974c05b000000",
						// 1_500 * 10 ^ 18
						"0x75438d34c9125839c8b08d21b7f3167281659e5c" => "0x5150ae84a8cdf00000",
						_ => "0x320",
					};
					let body = RpcResponse {
						jsonrpc: "2.0".into(),
						id: Id::Number(1),
						result: serde_json::to_value(value).unwrap(),
					};
					Response::builder().body(serde_json::to_string(&body).unwrap())
				},
				"nr_getNFTInventory" => {
					let result = GetNFTInventoryResult {
						page_key: "100_342".into(),
						details: vec![GetNFTInventoryResultDetail {
							token_address: "0x5e74094cd416f55179dbd0e45b1a8ed030e396a1".into(),
							token_id: "0x0000000000000000000000000000000000000000f".into(),
							balance: "0x00000000000000000000000000000000000000001".into(),
						}],
					};
					let body = RpcResponse {
						jsonrpc: "2.0".into(),
						id: Id::Number(1),
						result: serde_json::to_value(result).unwrap(),
					};
					Response::builder().body(serde_json::to_string(&body).unwrap())
				},
				"eth_getBalance" => {
					let body = RpcResponse {
						jsonrpc: "2.0".into(),
						id: Id::Number(1),
						// 1 * 10^18
						result: serde_json::to_value("0xde0b6b3a7640000").unwrap(),
					};
					Response::builder().body(serde_json::to_string(&body).unwrap())
				},
				"nr_getTokenHoldings" => {
					Response::builder().body(RES_BODY_OK_GET_TOKEN_HOLDINGS.to_string())
				},
				"eth_getTransactionCount" => {
					Response::builder().body(RES_BODY_OK_GET_TRANSACTION_COUNT.to_string())
				},
				_ => Response::builder().status(404).body(String::from("Error query")),
			}
		})
}
