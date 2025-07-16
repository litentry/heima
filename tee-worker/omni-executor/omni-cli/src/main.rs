use anyhow::Result;
use clap::{Parser, Subcommand};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::ChainId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
	jsonrpc: String,
	method: String,
	params: serde_json::Value,
	id: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
	jsonrpc: String,
	result: Option<T>,
	error: Option<JsonRpcError>,
	id: String,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
	code: i32,
	message: String,
	data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct SubmitUserOpParams {
	user_operations: Vec<SerializablePackedUserOperation>,
	chain_id: ChainId,
	wallet_index: u32,
}

#[derive(Debug, Deserialize)]
struct SubmitUserOpResponse {
	transaction_hash: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct RequestJwtParams {
	client_id: String,
	user_email: String,
	invite_code: Option<String>,
	google_code: String,
	language: Option<String>,
	email_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct UserConnectResponseData {
	user_id: Option<String>,
	google_auth_check: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct UserConnectResponse {
	code: i32,
	message: String,
	data: Option<UserConnectResponseData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RequestJwtResponse {
	access_token: String,
	id_token: String,
	backend_response: UserConnectResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
struct RequestEmailVerificationCodeParams {
	client_id: String,
	user_email: String,
}

#[derive(Debug, Parser)]
#[command(name = "omni-cli")]
#[command(about = "CLI for testing omni-executor RPC methods")]
struct Cli {
	#[command(subcommand)]
	command: Commands,

	#[arg(long, default_value = "http://localhost:2100/")]
	endpoint: String,

	#[arg(long)]
	token: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
	/// Submit UserOperation to omni-executor
	SubmitUserOp {
		#[arg(long)]
		chain_id: Option<u64>,

		#[arg(long, default_value = "0")]
		wallet_index: u32,

		#[arg(long)]
		sender: String,

		#[arg(long, default_value = "0")]
		nonce: String,

		#[arg(long, default_value = "")]
		init_code: String,

		#[arg(long, default_value = "")]
		call_data: String,

		#[arg(long, default_value = "0")]
		account_gas_limits: String,

		#[arg(long, default_value = "0")]
		pre_verification_gas: String,

		#[arg(long, default_value = "0")]
		gas_fees: String,

		#[arg(long, default_value = "")]
		paymaster_and_data: String,

		#[arg(long, default_value = "")]
		signature: String,
	},

	/// Request JWT token for authentication
	RequestJwt {
		#[arg(long)]
		client_id: String,

		#[arg(long)]
		user_email: String,

		#[arg(long)]
		invite_code: Option<String>,

		#[arg(long)]
		google_code: String,

		#[arg(long)]
		language: Option<String>,

		#[arg(long)]
		email_code: String,
	},

	/// Request email verification code
	RequestEmailVerificationCode {
		#[arg(long)]
		client_id: String,

		#[arg(long)]
		user_email: String,
	},
}

struct RpcClient {
	endpoint: String,
	token: Option<String>,
	client: reqwest::Client,
}

impl RpcClient {
	fn new(endpoint: String, token: Option<String>) -> Self {
		Self { endpoint, token, client: reqwest::Client::new() }
	}

	async fn call<T>(&self, method: &str, params: serde_json::Value) -> Result<T>
	where
		T: for<'de> Deserialize<'de> + std::fmt::Debug,
	{
		let request = JsonRpcRequest {
			jsonrpc: "2.0".to_string(),
			method: method.to_string(),
			params,
			id: uuid::Uuid::new_v4().to_string(),
		};

		let mut headers = HashMap::new();
		headers.insert("Content-Type".to_string(), "application/json".to_string());

		if let Some(token) = &self.token {
			headers.insert("Authorization".to_string(), format!("Bearer {}", token));
		}

		debug!("Sending request: {:?}", request);

		let mut request_builder = self.client.post(&self.endpoint).json(&request);

		for (key, value) in headers {
			request_builder = request_builder.header(&key, &value);
		}

		let response = request_builder.send().await?;

		if !response.status().is_success() {
			anyhow::bail!("HTTP error {}: {}", response.status(), response.text().await?);
		}

		let json_response: JsonRpcResponse<T> = response.json().await?;

		debug!("Received response: {:?}", json_response);

		if let Some(error) = json_response.error {
			anyhow::bail!("RPC error {}: {} - {:?}", error.code, error.message, error.data);
		}

		json_response.result.ok_or_else(|| anyhow::anyhow!("No result in response"))
	}
}

fn parse_hex_to_bytes(hex_str: &str, expected_len: usize) -> Result<Vec<u8>> {
	let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
	let mut bytes = hex::decode(hex_str)?;

	if bytes.len() < expected_len {
		// Pad at the beginning with zeros (for big-endian representation)
		let mut padded = vec![0; expected_len - bytes.len()];
		padded.extend(bytes);
		bytes = padded;
	} else if bytes.len() > expected_len {
		bytes.truncate(expected_len);
	}

	Ok(bytes)
}

fn parse_hex_to_array<const N: usize>(hex_str: &str) -> Result<[u8; N]> {
	let bytes = parse_hex_to_bytes(hex_str, N)?;
	let mut array = [0u8; N];
	array.copy_from_slice(&bytes);
	Ok(array)
}

async fn handle_submit_user_op(
	client: &RpcClient,
	chain_id: Option<u64>,
	wallet_index: u32,
	sender: String,
	nonce: String,
	init_code: String,
	call_data: String,
	account_gas_limits: String,
	pre_verification_gas: String,
	gas_fees: String,
	paymaster_and_data: String,
	signature: String,
) -> Result<()> {
	let chain = chain_id.unwrap_or(1); // Default to Ethereum mainnet if not specified

	// Convert hex strings to proper format for SerializablePackedUserOperation
	let nonce_val = if nonce.is_empty() || nonce == "0" {
		0u128
	} else {
		let nonce_bytes = parse_hex_to_array::<32>(&nonce)?;
		u128::from_be_bytes(nonce_bytes[16..32].try_into()?)
	};

	let pre_verification_gas_val = if pre_verification_gas.is_empty() || pre_verification_gas == "0"
	{
		0u128
	} else {
		let gas_bytes = parse_hex_to_array::<32>(&pre_verification_gas)?;
		let gas_val = u128::from_be_bytes(gas_bytes[16..32].try_into()?);
		debug!(
			"Parsed pre_verification_gas: input='{}', bytes={:?}, value={}",
			pre_verification_gas, gas_bytes, gas_val
		);
		gas_val
	};

	let user_operation = SerializablePackedUserOperation {
		sender,
		nonce: nonce_val,
		init_code,
		call_data,
		account_gas_limits,
		pre_verification_gas: pre_verification_gas_val,
		gas_fees,
		paymaster_and_data,
		signature: if signature.is_empty() { None } else { Some(signature) },
	};

	debug!(
		"Created UserOperation with pre_verification_gas: {}",
		user_operation.pre_verification_gas
	);

	let params =
		SubmitUserOpParams { user_operations: vec![user_operation], chain_id: chain, wallet_index };

	info!("Submitting UserOperation with params: {:?}", params);

	let json_params = serde_json::to_value(&params)?;
	debug!("JSON serialized params: {}", serde_json::to_string_pretty(&json_params)?);

	let response: SubmitUserOpResponse = client.call("omni_submitUserOp", json_params).await?;

	info!("Response: {:?}", response);

	if let Some(tx_hash) = response.transaction_hash {
		println!("Transaction Hash: {}", tx_hash);
	} else {
		println!("No transaction hash returned");
	}

	Ok(())
}

async fn handle_request_jwt(
	client: &RpcClient,
	client_id: String,
	user_email: String,
	invite_code: Option<String>,
	google_code: String,
	language: Option<String>,
	email_code: String,
) -> Result<()> {
	let params =
		RequestJwtParams { client_id, user_email, invite_code, google_code, language, email_code };

	info!("Requesting JWT with params: {:?}", params);

	let response: RequestJwtResponse =
		client.call("omni_requestJwt", serde_json::to_value(&params)?).await?;

	info!("Response: {:?}", response);

	println!("JWT Token Request Successful!");
	println!("Access Token: {}", response.access_token);
	println!("ID Token: {}", response.id_token);

	if let Some(data) = &response.backend_response.data {
		if let Some(user_id) = &data.user_id {
			println!("User ID: {}", user_id);
		}
		if let Some(google_auth_check) = &data.google_auth_check {
			println!("Google Auth Check: {}", google_auth_check);
		}
	}

	println!("\n💡 Tip: Use the access token with --token flag for protected methods");

	Ok(())
}

async fn handle_request_email_verification_code(
	client: &RpcClient,
	client_id: String,
	user_email: String,
) -> Result<()> {
	let params = RequestEmailVerificationCodeParams { client_id, user_email: user_email.clone() };

	info!("Requesting email verification code with params: {:?}", params);

	// The method returns void/null on success
	let _: () = client
		.call("omni_requestEmailVerificationCode", serde_json::to_value(&params)?)
		.await?;

	println!("Email Verification Code Sent Successfully!");
	println!("Please check your email ({}) for the verification code.", user_email);
	println!("\n💡 Next step: Use the verification code with the 'request-jwt' command");

	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();

	let cli = Cli::parse();
	let client = RpcClient::new(cli.endpoint, cli.token);

	match cli.command {
		Commands::SubmitUserOp {
			chain_id,
			wallet_index,
			sender,
			nonce,
			init_code,
			call_data,
			account_gas_limits,
			pre_verification_gas,
			gas_fees,
			paymaster_and_data,
			signature,
		} => {
			handle_submit_user_op(
				&client,
				chain_id,
				wallet_index,
				sender,
				nonce,
				init_code,
				call_data,
				account_gas_limits,
				pre_verification_gas,
				gas_fees,
				paymaster_and_data,
				signature,
			)
			.await?;
		},
		Commands::RequestJwt {
			client_id,
			user_email,
			invite_code,
			google_code,
			language,
			email_code,
		} => {
			handle_request_jwt(
				&client,
				client_id,
				user_email,
				invite_code,
				google_code,
				language,
				email_code,
			)
			.await?;
		},
		Commands::RequestEmailVerificationCode { client_id, user_email } => {
			handle_request_email_verification_code(&client, client_id, user_email).await?;
		},
	}

	Ok(())
}
