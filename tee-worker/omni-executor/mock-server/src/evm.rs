#![allow(opaque_hidden_inferred_bound)]

use serde_json::{json, Value};
use warp::{http::Response, Filter};

const BASE_PATH: &str = "evm";

fn build_success_response(response: Value) -> Response<String> {
	Response::builder()
		.status(200)
		.header("content-type", "application/json")
		.body(response.to_string())
		.unwrap()
}

pub(crate) fn handle() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	// Single endpoint that handles all EVM RPC calls with dynamic network support
	warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path::param::<String>()) // Capture network name (eth, bsc, etc.)
		.and(warp::body::json())
		.map(|network: String, body: serde_json::Value| {
			log::info!("Received EVM RPC request for network {}: {:?}", network, body);

			let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
			let default_id = json!(1);
			let id = body.get("id").unwrap_or(&default_id);
			let params = body.get("params").and_then(|p| p.as_array());

			match method {
				// All mocks below is for AccountingContractClient.execute_pay_out_request
				"eth_estimateGas" => {
					// Mock gas estimate
					let response = json!({
						"jsonrpc": "2.0",
						"result": "0x5208", // 21000 gas
						"id": id
					});
					build_success_response(response)
				},
				"eth_gasPrice" => {
					// Mock gas price
					let response = json!({
						"jsonrpc": "2.0",
						"result": "0x4a817c800", // 20 Gwei
						"id": id
					});
					build_success_response(response)
				},
				"eth_getTransactionCount" => {
					// Mock transaction count (nonce)
					let response = json!({
						"jsonrpc": "2.0",
						"result": "0x1", // nonce = 1
						"id": id
					});
					build_success_response(response)
				},
				"eth_chainId" => {
					// Mock chain ID based on network
					let chain_id = match network.as_str() {
						"eth" => "0x1",     // Ethereum mainnet
						"bsc" => "0x38",    // BSC mainnet
						"polygon" => "0x89", // Polygon mainnet
						"arbitrum" => "0xa4b1", // Arbitrum One
						"optimism" => "0xa",   // Optimism
						_ => "0x1",          // Default to Ethereum
					};
					let response = json!({
						"jsonrpc": "2.0",
						"result": chain_id,
						"id": id
					});
					build_success_response(response)
				},
				"eth_sendRawTransaction" => {
					// Mock successful raw transaction
					let response = json!({
						"jsonrpc": "2.0",
						"result": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
						"id": id
					});
					build_success_response(response)
				},
				"eth_blockNumber" => {
					// Mock current block number
					let response = json!({
						"jsonrpc": "2.0",
						"result": "0x1", // block number = 1
						"id": id
					});
					build_success_response(response)
				},
				"eth_getBlockByNumber" => {
					let number = params
						.and_then(|p| p.first())
						.and_then(|num| num.as_str())
						.unwrap_or("");

					// Mock block data
					let response = json!({
						"jsonrpc": "2.0",
						"result": {
							"number": number,
							"hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
							"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
							"timestamp": "0x61bc7e3c",
							"gasLimit": "0x1c9c380",
							"gasUsed": "0x5208",
							"transactions": []
						},
						"id": id
					});
					build_success_response(response)
				},
				"eth_getTransactionReceipt" => {
					let transaction_hash = params
						.and_then(|p| p.first())
						.and_then(|hash| hash.as_str())
						.unwrap_or("");

					// Mock transaction receipt
					let response = json!({
						"jsonrpc": "2.0",
						"result": {
							"transactionHash": transaction_hash,
							"transactionIndex": "0x0",
							"blockHash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
							"blockNumber": "0x1",
							"from": "0x70997970c51812dc3a010c7d01b50e0d17dc79c8",
							"to": "0x1234567890123456789012345678901234567890",
							"gasUsed": "0x5208",
							"cumulativeGasUsed": "0x5208",
							"contractAddress": null,
							"logs": [],
							"logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
							"status": "0x1",
							"type": "0x2"
						},
						"id": id
					});
					build_success_response(response)
				},
				// All mocks above is for AccountingContractClient.execute_pay_out_request
				"eth_call" => {
                    let (data, to) = params
                        .and_then(|arr| arr.first())
                        .map(|tx| {
                            (
                                tx.get("data").and_then(|d| d.as_str()).unwrap_or(""),
                                tx.get("to").and_then(|d| d.as_str()).unwrap_or("")
                            )
                        })
                        .unwrap_or(("", ""));

					let result = if data.starts_with("0x12065fe0") { // getBalance method signature
						match to {
							"0x1234567890123456789012345678901234567890" => "0x00000000000000000000000000000000000000000000003635c9adc5dea00000", // 1000 ETH
							_ => "0x0000000000000000000000000000000000000000000000000000000000000000" // 0 ETH
						}
					} else if data.starts_with("0x2d0335ab") { // getNonce method signature
						match to {
							"0x1234567890123456789012345678901234567890" => "0x0000000000000000000000000000000000000000000000000000000000000001", // nonce = 1
							_ => "0x0000000000000000000000000000000000000000000000000000000000000000" // nonce = 0
						}
					} else {
						"0x0"
					};

					let response = json!({
						"jsonrpc": "2.0",
						"result": result,
						"id": id
					});
					build_success_response(response)
				},
				_ => {
					// Default response for unknown methods
					let response = json!({
						"jsonrpc": "2.0",
						"result": null,
						"id": id
					});
					build_success_response(response)
				},
			}
		},
	)
}
