use crate::{
	detailed_error::DetailedError,
	error_code::{AUTH_VERIFICATION_FAILED_CODE, PARSE_ERROR_CODE},
	methods::omni::PumpxRpcError,
	server::RpcContext,
	Deserialize,
};
use alloy::primitives::keccak256;
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::ecdsa;
use executor_primitives::utils::hex::{decode_hex, hex_encode};
use executor_storage::{Storage, WildmetaTimestampStorage};
use jsonrpsee::RpcModule;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct ExportBundlerPrivateKeyParams {
	pub timestamp: u64,
	pub signature: String, // ECDSA signature over the timestamp, in 0x-hex-string
}

const BUNDLER_KEY_EXPORT_STORAGE_KEY: &str = "bundler_key_export";
const TIMESTAMP_VALIDITY_WINDOW_MS: u64 = 5 * 60 * 1000; // 5 minutes
const TIMESTAMP_FUTURE_TOLERANCE_MS: u64 = 60 * 1000; // 1 minute

fn verify_signature(
	timestamp: u64,
	signature: &str,
	expected_pubkey: &[u8; 33],
	storage: &Arc<WildmetaTimestampStorage>,
) -> Result<(), PumpxRpcError> {
	let current_time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(|e| {
			error!("Failed to get current time: {:?}", e);
			PumpxRpcError::from(
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_reason("System time error"),
			)
		})?
		.as_millis() as u64;

	if timestamp < current_time.saturating_sub(TIMESTAMP_VALIDITY_WINDOW_MS) {
		error!("Timestamp too old: {} vs current {}", timestamp, current_time);
		return Err(PumpxRpcError::from(
			DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication verification failed")
				.with_field("timestamp")
				.with_reason(format!(
					"Timestamp is too old (must be within last {} minutes)",
					TIMESTAMP_VALIDITY_WINDOW_MS / 60000
				))
				.with_suggestion("Use a recent timestamp"),
		));
	}

	if timestamp > current_time + TIMESTAMP_FUTURE_TOLERANCE_MS {
		error!("Timestamp too far in future: {} vs current {}", timestamp, current_time);
		return Err(PumpxRpcError::from(
			DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication verification failed")
				.with_field("timestamp")
				.with_reason("Timestamp is too far in the future")
				.with_suggestion("Ensure system clock is synchronized"),
		));
	}

	let last_timestamp = storage
		.get(&BUNDLER_KEY_EXPORT_STORAGE_KEY.to_string())
		.map_err(|e| {
			error!("Failed to get last timestamp from storage: {:?}", e);
			PumpxRpcError::from(
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_reason("Failed to retrieve last timestamp from storage"),
			)
		})?
		.unwrap_or(0);

	if timestamp <= last_timestamp {
		error!("Timestamp not greater than last used: {} <= {}", timestamp, last_timestamp);
		return Err(PumpxRpcError::from(
			DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication verification failed")
				.with_field("timestamp")
				.with_reason("Timestamp must be greater than previously used timestamp")
				.with_suggestion("This may be a replay attack. Use a fresh timestamp."),
		));
	}
	let signature_bytes = decode_hex(signature).map_err(|e| {
		error!("Failed to decode signature: {:?}", e);
		PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Parse error")
				.with_field("signature")
				.with_reason("The signature could not be decoded from hex"),
		)
	})?;

	if signature_bytes.len() != 65 {
		error!("Invalid signature length: expected 65 bytes, got {}", signature_bytes.len());
		return Err(PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Parse error")
				.with_field("signature")
				.with_reason(format!(
					"Invalid signature length: expected 65 bytes, got {}",
					signature_bytes.len()
				)),
		));
	}

	let signature_array: [u8; 65] = signature_bytes.try_into().map_err(|_| {
		error!("Failed to convert signature bytes to array");
		PumpxRpcError::from(
			DetailedError::new(PARSE_ERROR_CODE, "Parse error").with_field("signature"),
		)
	})?;

	let timestamp_bytes = timestamp.to_string();
	let challenge_hash = keccak256(timestamp_bytes.as_bytes());
	let challenge_hash_array: [u8; 32] = challenge_hash.0;

	let public_key = ecdsa::Public::from_raw(*expected_pubkey);
	let signature = ecdsa::Signature::from_raw(signature_array);

	if !ecdsa::Pair::verify_prehashed(&signature, &challenge_hash_array, &public_key) {
		error!("Signature verification failed for bundler key export");
		return Err(PumpxRpcError::from(
			DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication verification failed")
				.with_field("signature")
				.with_reason("The signature does not match the challenge")
				.with_suggestion("Ensure you are using the correct private key"),
		));
	}

	storage
		.insert(&BUNDLER_KEY_EXPORT_STORAGE_KEY.to_string(), timestamp)
		.map_err(|e| {
			error!("Failed to store new timestamp: {:?}", e);
			PumpxRpcError::from(
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_reason("Failed to store timestamp in storage"),
			)
		})?;

	debug!("Timestamp {} validated and stored successfully", timestamp);

	Ok(())
}

pub fn register_export_bundler_private_key<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
	module
		.register_method("omni_exportBundlerPrivateKey", |params, ctx, _ext| {
			let params = params.parse::<ExportBundlerPrivateKeyParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!(
				"Received omni_exportBundlerPrivateKey request with timestamp: {}",
				params.timestamp
			);

			verify_signature(
				params.timestamp,
				&params.signature,
				&ctx.bundler_key_export_authorized_pubkey,
				&ctx.wildmeta_timestamp_storage,
			)?;

			debug!("Signature verified successfully, returning bundler private key");

			let private_key_hex = hex_encode(&ctx.bundler_private_key);
			Ok::<String, PumpxRpcError>(private_key_hex)
		})
		.expect("Failed to register omni_exportBundlerPrivateKey method");
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{start_server, ShieldingKey};
	use binance_api::mocks::MockBinanceApiClient;
	use config_loader::ConfigLoader;
	use executor_core::intent_executor::MockedIntentExecutor;
	use executor_crypto::{ecdsa, PairTrait};
	use executor_primitives::utils::hex::decode_hex;
	use executor_storage::{StorageDB, WildmetaTimestampStorage};
	use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
	use parentchain_rpc_client::{
		metadata::SubxtMetadataProvider, CustomConfig, SubxtClientFactory,
	};
	use parentchain_signer::{key_store::SubstrateKeyStore, TxSigner};
	use pumpx::PumpxApiClient;
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use signer_client::{mocks::MockSignerClient, SignerClient};
	use std::{collections::HashMap, path::Path, sync::Arc};
	use tempfile::tempdir;
	use wildmeta_api::{MockWildmetaApi, WildmetaApi};

	fn create_test_keypair() -> ([u8; 32], [u8; 33]) {
		let (pair, seed) = ecdsa::Pair::generate();
		let public = pair.public();
		let compressed_pubkey: [u8; 33] = public.0;
		(seed, compressed_pubkey)
	}

	fn sign_timestamp(timestamp: u64, seed: &[u8; 32]) -> String {
		let pair = ecdsa::Pair::from_seed_slice(seed).unwrap();
		let timestamp_str = timestamp.to_string();
		let hash = keccak256(timestamp_str.as_bytes());
		let signature = pair.sign_prehashed(&hash.0);
		hex_encode(&signature.0)
	}

	async fn setup_test_server(
		port: u16,
		bundler_key: [u8; 32],
		authorized_pubkey: [u8; 33],
		storage_db: Arc<StorageDB>,
	) {
		let shielding_key = ShieldingKey::new();
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new("https://api.pumpx.ai".to_string());
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));
		let binance_api_client: Arc<dyn binance_api::BinancePaymasterApi> =
			Arc::new(MockBinanceApiClient::new());

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage =
			Arc::new(WildmetaTimestampStorage::new(storage_db.clone()));

		let (solana_intent_executor, _) = MockedIntentExecutor::new();
		let (ethereum_intent_executor, _) = MockedIntentExecutor::new();
		let (cross_chain_intent_executor, _) = MockedIntentExecutor::new();

		let client_factory =
			SubxtClientFactory::<CustomConfig>::new(&config_loader.parentchain_url);
		let metadata_provider = Arc::new(SubxtMetadataProvider::new(client_factory.clone()));
		let parentchain_rpc_client_factory = Arc::new(client_factory);

		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		let substrate_key_store = Arc::new(SubstrateKeyStore::new(
			Path::new("./")
				.join("keystore/substrate_key.bin")
				.into_os_string()
				.into_string()
				.unwrap(),
		));

		let parentchain_signer = parentchain_signer::get_signer(substrate_key_store.clone());

		let tx_signer = Arc::new(TxSigner::new(
			metadata_provider,
			parentchain_rpc_client_factory.clone(),
			parentchain_signer,
			0,
		));

		start_server(
			port,
			shielding_key,
			Arc::new(Box::new(pumpx_api)),
			storage_db.clone(),
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			binance_api_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			[0u8; 33],
			bundler_key,
			authorized_pubkey,
			Arc::new(ethereum_intent_executor),
			Arc::new(solana_intent_executor),
			Arc::new(cross_chain_intent_executor),
			parentchain_rpc_client_factory,
			aes_key,
			tx_signer,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();
	}

	#[tokio::test]
	async fn test_export_bundler_key_success() {
		let tmp_dir = tempdir().unwrap();
		let port = 2010;
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let (seed, authorized_pubkey) = create_test_keypair();
		let bundler_key = [42u8; 32];

		setup_test_server(port, bundler_key, authorized_pubkey, db).await;

		let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

		let signature = sign_timestamp(current_time, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let response: String = client
			.request("omni_exportBundlerPrivateKey", rpc_params![current_time, signature])
			.await
			.unwrap();

		let expected_hex = hex_encode(&bundler_key);
		assert_eq!(response, expected_hex);
	}

	#[tokio::test]
	async fn test_export_bundler_key_invalid_signature() {
		let tmp_dir = tempdir().unwrap();
		let port = 2011;
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let (seed_a, _) = create_test_keypair();
		let (_, authorized_pubkey_b) = create_test_keypair();
		let bundler_key = [42u8; 32];

		setup_test_server(port, bundler_key, authorized_pubkey_b, db).await;

		let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

		let signature = sign_timestamp(current_time, &seed_a);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let result: Result<String, _> = client
			.request("omni_exportBundlerPrivateKey", rpc_params![current_time, signature])
			.await;

		assert!(result.is_err());
	}

	#[tokio::test]
	async fn test_export_bundler_key_timestamp_too_old() {
		let tmp_dir = tempdir().unwrap();
		let port = 2012;
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let (seed, authorized_pubkey) = create_test_keypair();
		let bundler_key = [42u8; 32];

		setup_test_server(port, bundler_key, authorized_pubkey, db).await;

		let old_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
			as u64 - (6 * 60 * 1000);

		let signature = sign_timestamp(old_timestamp, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let result: Result<String, _> = client
			.request("omni_exportBundlerPrivateKey", rpc_params![old_timestamp, signature])
			.await;

		assert!(result.is_err());
	}

	#[tokio::test]
	async fn test_export_bundler_key_timestamp_future() {
		let tmp_dir = tempdir().unwrap();
		let port = 2013;
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let (seed, authorized_pubkey) = create_test_keypair();
		let bundler_key = [42u8; 32];

		setup_test_server(port, bundler_key, authorized_pubkey, db).await;

		let future_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
			as u64 + (2 * 60 * 1000);

		let signature = sign_timestamp(future_timestamp, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let result: Result<String, _> = client
			.request("omni_exportBundlerPrivateKey", rpc_params![future_timestamp, signature])
			.await;

		assert!(result.is_err());
	}

	#[tokio::test]
	async fn test_export_bundler_key_replay_attack() {
		let tmp_dir = tempdir().unwrap();
		let port = 2014;
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let (seed, authorized_pubkey) = create_test_keypair();
		let bundler_key = [42u8; 32];

		setup_test_server(port, bundler_key, authorized_pubkey, db).await;

		let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

		let signature = sign_timestamp(current_time, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let first_result: Result<String, _> = client
			.request("omni_exportBundlerPrivateKey", rpc_params![current_time, signature.clone()])
			.await;
		assert!(first_result.is_ok());

		let second_result: Result<String, _> = client
			.request("omni_exportBundlerPrivateKey", rpc_params![current_time, signature])
			.await;

		assert!(second_result.is_err());
	}
}
