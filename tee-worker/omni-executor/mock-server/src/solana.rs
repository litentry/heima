#![allow(opaque_hidden_inferred_bound)]

use serde_json::{json, Value};
use warp::{http::Response, Filter};

const BASE_PATH: &str = "solana";

fn build_success_response(response: Value) -> Response<String> {
	Response::builder()
		.status(200)
		.header("content-type", "application/json")
		.body(response.to_string())
		.unwrap()
}

pub(crate) fn handle() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	// Single endpoint that handles all Solana RPC calls
	warp::post().and(warp::path(BASE_PATH)).and(warp::body::json()).map(
		|body: serde_json::Value| {
			log::info!("Received Solana RPC request: {:?}", body);

			let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
			let default_id = json!(1);
			let id = body.get("id").unwrap_or(&default_id);

			match method {
				"getBalance" => {
					// Example request body: {"id": Number(0), "jsonrpc": String("2.0"), "method": String("getBalance"), "params": Array [String("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9"), Object {"commitment": String("confirmed")}]}
					// Extract the address from params
					let address = body
						.get("params")
						.and_then(|params| params.as_array())
						.and_then(|arr| arr.first())
						.and_then(|addr| addr.as_str())
						.unwrap_or("");

					// Check if the address matches the specific test address
					let balance = if address == "AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9" {
						1000000000 // 1 SOL for the specific test address
					} else {
						0 // 0 SOL for all other addresses
					};

					// Mock response for getBalance
					let response = json!({
						"jsonrpc": "2.0",
						"result": {
							"context": {
								"slot": 123456789
							},
							"value": balance
						},
						"id": id
					});

					Response::builder()
						.status(200)
						.header("content-type", "application/json")
						.body(response.to_string())
						.unwrap()
				},
				"sendTransaction" => {
					// Example request body: {"id": Number(1), "jsonrpc": String("2.0"), "method": String("sendTransaction"), "params": Array [String("AaV8xKPN0GfY3XVuTiQsxFPDOFlZhXhWwbcVMBEGd2kUTgid2Ss9GUNqJ1S01iTyvDKCWea8sRMwlmlZ1A8zDAEBAAMIiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zG8DcMUx1vBnO2Al4wQ7nCKjwgl/c5iVbUF2HhIwZJ3ez0N0UQ/aETrzeokAeTDaVjdVNHLlFhPu0cKQjhxBso7UkoxijRwsbq6QM4kFmVYSlZJzpcY/k2NsFGFKyHN9Hx9pMmCdAce72TK7SkfatcvRdHPxk+YN0BruCENR+3BgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAsuuu6Iwn1QSzfnYrRIa2f/i5FCzjnwDv32SN1j4fahTyq0V7Xk41kUInl1rH0tI9eWEbcpDkAbgz89/ZTaRNLMxJDpKM0uOHO7ND/JXaMxecpg9Nv0bCw26RKZ1V1Oa5AQYHAgEABwQDBRiE8aphpOG6LgEAAAAAAAAAAMqaOwAAAAA="), Object {"encoding": String("base64"), "maxRetries": Null, "minContextSlot": Null, "preflightCommitment": String("confirmed"), "skipPreflight": Bool(false)}]}
					// Extract the encoded tx_string from params
					let tx_string = body
						.get("params")
						.and_then(|params| params.as_array())
						.and_then(|arr| arr.first())
						.and_then(|addr| addr.as_str())
						.unwrap_or("");

					let transaction_id = if tx_string == "AaV8xKPN0GfY3XVuTiQsxFPDOFlZhXhWwbcVMBEGd2kUTgid2Ss9GUNqJ1S01iTyvDKCWea8sRMwlmlZ1A8zDAEBAAMIiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zG8DcMUx1vBnO2Al4wQ7nCKjwgl/c5iVbUF2HhIwZJ3ez0N0UQ/aETrzeokAeTDaVjdVNHLlFhPu0cKQjhxBso7UkoxijRwsbq6QM4kFmVYSlZJzpcY/k2NsFGFKyHN9Hx9pMmCdAce72TK7SkfatcvRdHPxk+YN0BruCENR+3BgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAsuuu6Iwn1QSzfnYrRIa2f/i5FCzjnwDv32SN1j4fahTyq0V7Xk41kUInl1rH0tI9eWEbcpDkAbgz89/ZTaRNLMxJDpKM0uOHO7ND/JXaMxecpg9Nv0bCw26RKZ1V1Oa5AQYHAgEABwQDBRiE8aphpOG6LgEAAAAAAAAAAMqaOwAAAAA=" {
						"4JuCzP32wMRsiP9Lb3nb8YQXTjy1o5XaU86g7gD6WUsRuQ4U4W9dmzpb4uTgXcNKUhLRundzwJC2jpcZMBfcy5QC"
					} else {
						""
					};

					let response = json!({
						"jsonrpc": "2.0",
						"result": transaction_id,
						"id": id
					});

					build_success_response(response)
				},
				"getLatestBlockhash" => {
					// Example request body: {"id": Number(0), "jsonrpc": String("2.0"), "method": String("getLatestBlockhash"), "params": Array [Object {"commitment": String("confirmed")}]}
					let response = json!({
						"jsonrpc": "2.0",
						"result": {
							"context": {
								"apiVersion": "1.10.34",
								"slot": 123456789
							},
							"value": {
								"blockhash": "EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N",
								"lastValidBlockHeight": 123456789
							}
						},
						"id": id
					});

					build_success_response(response)
				},
				"getSignatureStatuses" => {
					// Example request body: {"id": Number(2), "jsonrpc": String("2.0"), "method": String("getSignatureStatuses"), "params": Array [Array [String("4JuCzP32wMRsiP9Lb3nb8YQXTjy1o5XaU86g7gD6WUsRuQ4U4W9dmzpb4uTgXcNKUhLRundzwJC2jpcZMBfcy5QC")]]}
					let response = json!({
						"jsonrpc": "2.0",
						"result": {
							"context": {
							"slot": 82
							},
							"value": [
							{
								"slot": 48,
								"confirmations": null,
								"err": null,
								"status": {
									"Ok": null
								},
								"confirmationStatus": "finalized"
							},
							null
							]
						},
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
