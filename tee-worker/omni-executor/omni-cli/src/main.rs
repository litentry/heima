use aa_contracts_client::{
	calculate_omni_account_address, create_paymaster_and_data, prepare_factory_init_code, OwnerType,
};
use alloy::primitives::{Address, FixedBytes, TxKind, U256};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use alloy::sol;
use alloy::sol_types::SolCall;
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use ethereum_rpc::{AlloyRpcProvider, RpcProvider};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{ChainId, Web2IdentityType};
use heima_primitives::Identity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

sol! {
	// Owner type enum from aa-contracts-client
	enum OwnerTypeEnum {
		Pumpx,
		Email,
		Twitter,
		Discord,
		Github,
		Substrate,
		Evm,
		Bitcoin,
		Solana,
		Google,
		Passkey
	}

	// OmniAccountFactory interface
	function getAddress(bytes32 oa, OwnerTypeEnum oaType, bytes memory clientId, address root) public view returns (address);
	function accountImplementation() public view returns (address);
}

#[derive(Debug, Clone, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
enum CliChainType {
	Evm,
	Solana,
	Tron,
}

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

#[derive(Debug, Serialize)]
struct SubmitUserOpTestParams {
	user_operations: Vec<SerializablePackedUserOperation>,
	chain_id: ChainId,
	wallet_index: u32,
	omni_account: String,
	client_id: String,
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

#[derive(Debug, Serialize)]
struct GetSmartWalletRootSignerParams {
	omni_account: String,
	chain_type: CliChainType,
	wallet_index: u32,
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

	/// Submit UserOperation test (with omni_account and client_id)
	SubmitUserOpTest {
		#[arg(long)]
		chain_id: Option<u64>,

		#[arg(long, default_value = "0")]
		wallet_index: u32,

		#[arg(long)]
		omni_account: String,

		#[arg(long)]
		client_id: String,

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

	/// Get smart wallet root signer address
	GetSmartWalletRootSigner {
		#[arg(long)]
		omni_account: String,

		#[arg(long, value_enum)]
		chain_type: CliChainType,

		#[arg(long, default_value = "0")]
		wallet_index: u32,
	},

	/// Generate OmniAccount (AA) address from omni-account bytes and owner type
	GenerateAAAddress {
		#[arg(long, help = "32-byte omni-account identifier in hex format (0x-prefixed)")]
		oa_bytes: String,

		#[arg(
			long,
			help = "Owner type: Email, Evm, Substrate, Bitcoin, Solana, Twitter, Discord, Github, Google, Pumpx, Passkey"
		)]
		oa_type: String,

		#[arg(long)]
		client_id: String,

		#[arg(long)]
		factory_address: String,

		#[arg(long)]
		root_address: String,

		#[arg(long, default_value = "http://localhost:8545")]
		rpc_url: String,

		#[arg(
			long,
			help = "Account implementation address (if not provided, will try to get from factory)"
		)]
		account_implementation: Option<String>,
	},

	/// Generate user operation init code from omni-account bytes and owner type
	GenerateInitCode {
		#[arg(long, help = "32-byte omni-account identifier in hex format (0x-prefixed)")]
		oa_bytes: String,

		#[arg(
			long,
			help = "Owner type: Email, Evm, Substrate, Bitcoin, Solana, Twitter, Discord, Github, Google, Pumpx, Passkey"
		)]
		oa_type: String,

		#[arg(long)]
		client_id: String,

		#[arg(long)]
		factory_address: String,

		#[arg(long)]
		root_address: String,
	},

	/// Generate paymaster and data for user operations
	GeneratePaymasterData {
		#[arg(long)]
		paymaster_address: String,

		#[arg(long, default_value = "50000")]
		verification_gas_limit: u64,

		#[arg(long, default_value = "50000")]
		post_op_gas_limit: u64,
	},

	/// Get OmniAccount identifier from email/identity and client_id
	GetOmniAccount {
		#[arg(long)]
		email: String,

		#[arg(long)]
		client_id: String,
	},

	/// Pack two gas limits into accountGasLimits format for PackedUserOperation
	PackGasLimits {
		#[arg(long, help = "Gas limit for account verification (decimal)")]
		verification_gas: u64,

		#[arg(long, help = "Gas limit for call execution (decimal)")]
		call_gas: u64,
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

fn parse_owner_type(oa_type_str: &str) -> Result<OwnerType> {
	match oa_type_str.to_lowercase().as_str() {
		"email" => Ok(OwnerType::Email),
		"evm" => Ok(OwnerType::Evm),
		"substrate" => Ok(OwnerType::Substrate),
		"bitcoin" => Ok(OwnerType::Bitcoin),
		"solana" => Ok(OwnerType::Solana),
		"twitter" => Ok(OwnerType::Twitter),
		"discord" => Ok(OwnerType::Discord),
		"github" => Ok(OwnerType::Github),
		"google" => Ok(OwnerType::Google),
		"pumpx" => Ok(OwnerType::Pumpx),
		"passkey" => Ok(OwnerType::Passkey),
		_ => anyhow::bail!("Invalid owner type '{}'. Valid types are: Email, Evm, Substrate, Bitcoin, Solana, Twitter, Discord, Github, Google, Pumpx, Passkey", oa_type_str),
	}
}

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
async fn handle_submit_user_op_test(
	client: &RpcClient,
	chain_id: Option<u64>,
	wallet_index: u32,
	omni_account: String,
	client_id: String,
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

	let params = SubmitUserOpTestParams {
		user_operations: vec![user_operation],
		chain_id: chain,
		wallet_index,
		omni_account,
		client_id,
	};

	info!("Submitting UserOperation (test) with params: {:?}", params);

	let json_params = serde_json::to_value(&params)?;
	debug!("JSON serialized params: {}", serde_json::to_string_pretty(&json_params)?);

	let response: SubmitUserOpResponse = client.call("omni_submitUserOpTest", json_params).await?;

	info!("Response: {:?}", response);

	if let Some(tx_hash) = response.transaction_hash {
		println!("Transaction Hash: {}", tx_hash);
	} else {
		println!("No transaction hash returned");
	}

	Ok(())
}

async fn handle_get_smart_wallet_root_signer(
	client: &RpcClient,
	omni_account: String,
	chain_type: CliChainType,
	wallet_index: u32,
) -> Result<()> {
	let params = GetSmartWalletRootSignerParams { omni_account, chain_type, wallet_index };

	info!("Getting smart wallet root signer with params: {:?}", params);

	let response: String = client
		.call("omni_getSmartWalletRootSigner", serde_json::to_value(&params)?)
		.await?;

	info!("Response: {:?}", response);

	println!("Root Signer Address: {}", response);

	Ok(())
}

async fn handle_generate_aa_address(
	oa_bytes_hex: String,
	oa_type_str: String,
	client_id: String,
	factory_address: String,
	root_address: String,
	rpc_url: String,
	account_implementation: Option<String>,
) -> Result<()> {
	// Parse OmniAccount bytes from hex
	let oa_bytes = parse_hex_to_array::<32>(&oa_bytes_hex)?;

	// Parse owner type
	let oa_type = parse_owner_type(&oa_type_str)?;

	// Parse addresses
	let factory_addr = factory_address
		.parse::<Address>()
		.map_err(|e| anyhow::anyhow!("Invalid factory address '{}': {}", factory_address, e))?;

	let root_addr = root_address
		.parse::<Address>()
		.map_err(|e| anyhow::anyhow!("Invalid root address '{}': {}", root_address, e))?;

	info!("Debug info:");
	info!("  OA bytes: 0x{}", hex::encode(oa_bytes));
	info!("  OA type: {}", oa_type_str);
	info!("  Client ID: {}", client_id);
	info!("  Factory address: {}", factory_address);
	info!("  Root address: {}", root_address);
	info!("  RPC URL: {}", rpc_url);

	// Create RPC provider
	let provider = AlloyRpcProvider::new(&rpc_url);

	// Check if the contract exists
	info!("Checking if contract exists at {}", factory_address);
	match provider.get_code_at(factory_addr).await {
		Ok(code) => {
			if code.is_empty() {
				return Err(anyhow::anyhow!("No contract deployed at address {}", factory_address));
			}
			info!("Contract exists, code length: {} bytes", code.len());
		},
		Err(e) => {
			return Err(anyhow::anyhow!("Failed to check contract code: {}", e));
		},
	}

	// Get account implementation address - either from parameter or from factory contract
	let account_impl_addr = if let Some(impl_addr_str) = account_implementation {
		info!("Using provided account implementation address: {}", impl_addr_str);
		impl_addr_str.parse::<Address>().map_err(|e| {
			anyhow::anyhow!("Invalid account implementation address '{}': {}", impl_addr_str, e)
		})?
	} else {
		info!("Getting account implementation address from factory...");
		let impl_call = accountImplementationCall {};
		let impl_tx = TransactionRequest {
			to: Some(TxKind::Call(factory_addr)),
			input: TransactionInput {
				data: Some(impl_call.abi_encode().into()),
				..Default::default()
			},
			..Default::default()
		};

		let impl_result = provider.call(impl_tx).await
			.map_err(|e| anyhow::anyhow!("Failed to get account implementation from factory (you can provide --account-implementation manually): {}", e))?;

		let addr = if impl_result.len() >= 32 {
			Address::from_slice(&impl_result[12..32])
		} else {
			return Err(anyhow::anyhow!("Invalid implementation address response length: expected at least 32 bytes, got {}", impl_result.len()));
		};

		info!("Retrieved account implementation address: 0x{}", hex::encode(addr.as_slice()));
		addr
	};

	// Now calculate the address locally using the same logic as the contract
	let aa_address = calculate_omni_account_address(
		factory_addr,
		account_impl_addr,
		FixedBytes::from_slice(&oa_bytes),
		oa_type,
		client_id.as_bytes(),
		root_addr,
	);

	info!(
		"Generated AA address for oa_bytes '0x{}' with oa_type '{}', client_id '{}', factory '{}', root '{}': 0x{}",
		hex::encode(oa_bytes),
		oa_type_str,
		client_id,
		factory_address,
		root_address,
		hex::encode(aa_address.as_slice())
	);

	println!("AA Address: 0x{}", hex::encode(aa_address.as_slice()));

	Ok(())
}

fn handle_generate_init_code(
	oa_bytes_hex: String,
	oa_type_str: String,
	client_id: String,
	factory_address: String,
	root_address: String,
) -> Result<()> {
	// Parse OmniAccount bytes from hex
	let oa_bytes = parse_hex_to_array::<32>(&oa_bytes_hex)?;

	// Parse owner type
	let oa_type = parse_owner_type(&oa_type_str)?;

	// Parse addresses
	let factory_addr = factory_address
		.parse::<Address>()
		.map_err(|e| anyhow::anyhow!("Invalid factory address '{}': {}", factory_address, e))?;

	let root_addr = root_address
		.parse::<Address>()
		.map_err(|e| anyhow::anyhow!("Invalid root address '{}': {}", root_address, e))?;

	// Generate init code using the specified owner type
	let init_code =
		prepare_factory_init_code(factory_addr, oa_bytes, oa_type, client_id.as_bytes(), root_addr);

	// Convert to hex string
	let init_code_hex = hex::encode(&init_code);

	info!(
		"Generated init code for oa_bytes '0x{}' with oa_type '{}', client_id '{}', factory '{}', root '{}': 0x{}",
		hex::encode(oa_bytes), oa_type_str, client_id, factory_address, root_address, init_code_hex
	);

	println!("Init Code: 0x{}", init_code_hex);

	Ok(())
}

fn handle_generate_paymaster_data(
	paymaster_address: String,
	verification_gas_limit: u64,
	post_op_gas_limit: u64,
) -> Result<()> {
	// Parse paymaster address
	let paymaster_addr = paymaster_address
		.parse::<Address>()
		.map_err(|e| anyhow::anyhow!("Invalid paymaster address '{}': {}", paymaster_address, e))?;

	// Generate paymaster and data
	let paymaster_and_data = create_paymaster_and_data(
		paymaster_addr,
		U256::from(verification_gas_limit),
		U256::from(post_op_gas_limit),
	);

	// Convert to hex string
	let paymaster_data_hex = hex::encode(&paymaster_and_data);

	info!(
		"Generated paymaster data for address '{}', verification_gas: {}, post_op_gas: {}: 0x{}",
		paymaster_address, verification_gas_limit, post_op_gas_limit, paymaster_data_hex
	);

	println!("Paymaster Data: 0x{}", paymaster_data_hex);

	Ok(())
}

fn handle_get_omni_account(email: String, client_id: String) -> Result<()> {
	// Create Identity from email
	let identity = Identity::from_web2_account(&email, Web2IdentityType::Email);

	// Generate OmniAccount
	let omni_account = identity.to_omni_account(&client_id);
	let oa_bytes: [u8; 32] = omni_account.into();

	// Convert to hex string
	let omni_account_hex = hex::encode(oa_bytes);

	info!(
		"Generated OmniAccount for email '{}' with client_id '{}': 0x{}",
		email, client_id, omni_account_hex
	);

	println!("OmniAccount: 0x{}", omni_account_hex);

	Ok(())
}

fn handle_pack_gas_limits(verification_gas: u64, call_gas: u64) -> Result<()> {
	// Pack the gas limits into a 32-byte (256-bit) value
	// Higher 128 bits = verification_gas, lower 128 bits = call_gas

	// Create the packed value as two 128-bit parts
	let verification_part = verification_gas as u128;
	let call_part = call_gas as u128;

	// Since we need 256 bits total, we'll use a byte array approach
	let mut packed_bytes = [0u8; 32];

	// Put verification_gas in the higher 16 bytes (128 bits)
	packed_bytes[16..32].copy_from_slice(&verification_part.to_be_bytes());
	// Put call_gas in the lower 16 bytes (128 bits)
	packed_bytes[0..16].copy_from_slice(&call_part.to_be_bytes());

	// Convert to hex string
	let packed_hex = format!("0x{}", hex::encode(packed_bytes));

	info!(
		"Packed gas limits: verification={}, call={} -> {}",
		verification_gas, call_gas, packed_hex
	);

	println!("Packed Gas Limits: {}", packed_hex);

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
		Commands::SubmitUserOpTest {
			chain_id,
			wallet_index,
			omni_account,
			client_id,
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
			handle_submit_user_op_test(
				&client,
				chain_id,
				wallet_index,
				omni_account,
				client_id,
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
		Commands::GetSmartWalletRootSigner { omni_account, chain_type, wallet_index } => {
			handle_get_smart_wallet_root_signer(&client, omni_account, chain_type, wallet_index)
				.await?;
		},
		Commands::GenerateAAAddress {
			oa_bytes,
			oa_type,
			client_id,
			factory_address,
			root_address,
			rpc_url,
			account_implementation,
		} => {
			handle_generate_aa_address(
				oa_bytes,
				oa_type,
				client_id,
				factory_address,
				root_address,
				rpc_url,
				account_implementation,
			)
			.await?;
		},
		Commands::GenerateInitCode {
			oa_bytes,
			oa_type,
			client_id,
			factory_address,
			root_address,
		} => {
			handle_generate_init_code(oa_bytes, oa_type, client_id, factory_address, root_address)?;
		},
		Commands::GeneratePaymasterData {
			paymaster_address,
			verification_gas_limit,
			post_op_gas_limit,
		} => {
			handle_generate_paymaster_data(
				paymaster_address,
				verification_gas_limit,
				post_op_gas_limit,
			)?;
		},
		Commands::GetOmniAccount { email, client_id } => {
			handle_get_omni_account(email, client_id)?;
		},
		Commands::PackGasLimits { verification_gas, call_gas } => {
			handle_pack_gas_limits(verification_gas, call_gas)?;
		},
	}

	Ok(())
}
