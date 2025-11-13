use crate::{
	detailed_error::DetailedError, error_code::AUTH_VERIFICATION_FAILED_CODE, server::RpcContext,
	utils::types::RpcResultExt, utils::validation::parse_rpc_params, Deserialize, RpcResult,
};
use alloy::primitives::keccak256;
use jsonrpsee::{types::ErrorObjectOwned, RpcModule};
use oe_core::intent::executor::IntentExecutor;
use oe_crypto::{
	aes256::{aes_encrypt_default, Aes256Key, SerdeAesOutput},
	ecdsa,
};
use oe_primitives::utils::hex::decode_hex;
use oe_storage::{Storage, WildmetaTimestampStorage};
use rsa::Oaep;
use sha2::Sha256;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct ExportBundlerPrivateKeyParams {
	pub timestamp: u64,
	pub signature: String, // ECDSA signature over the timestamp, in 0x-hex-string
	pub key: String, // RSA-encrypted AES key to encrypt the bundler private key, in 0x-hex-string
}

const BUNDLER_KEY_EXPORT_STORAGE_KEY: &str = "bundler_key_export";
const TIMESTAMP_VALIDITY_WINDOW_MS: u64 = 5 * 60 * 1000; // 5 minutes
const TIMESTAMP_FUTURE_TOLERANCE_MS: u64 = 60 * 1000; // 1 minute

fn verify_signature(
	timestamp: u64,
	signature: &str,
	expected_pubkey: &[u8; 33],
	storage: &Arc<WildmetaTimestampStorage>,
) -> RpcResult<()> {
	let current_time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(|e| {
			error!("Failed to get current time: {:?}", e);
			DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication verification failed")
				.with_reason("System time error")
				.to_rpc_error()
		})?
		.as_millis() as u64;

	if timestamp < current_time.saturating_sub(TIMESTAMP_VALIDITY_WINDOW_MS) {
		error!("Timestamp too old: {} vs current {}", timestamp, current_time);
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Authentication verification failed",
		)
		.with_field("timestamp")
		.with_reason(format!(
			"Timestamp is too old (must be within last {} minutes)",
			TIMESTAMP_VALIDITY_WINDOW_MS / 60000
		))
		.with_suggestion("Use a recent timestamp")
		.to_rpc_error());
	}

	if timestamp > current_time + TIMESTAMP_FUTURE_TOLERANCE_MS {
		error!("Timestamp too far in future: {} vs current {}", timestamp, current_time);
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Authentication verification failed",
		)
		.with_field("timestamp")
		.with_reason("Timestamp is too far in the future")
		.with_suggestion("Ensure system clock is synchronized")
		.to_rpc_error());
	}

	let last_timestamp = storage
		.get(&BUNDLER_KEY_EXPORT_STORAGE_KEY.to_string())
		.map_err(|e| {
			error!("Failed to get last timestamp from storage: {:?}", e);
			DetailedError::storage_service_error("get timestamp").to_rpc_error()
		})?
		.unwrap_or(0);

	if timestamp <= last_timestamp {
		error!("Timestamp not greater than last used: {} <= {}", timestamp, last_timestamp);
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Authentication verification failed",
		)
		.with_field("timestamp")
		.with_reason("Timestamp must be greater than previously used timestamp")
		.with_suggestion("This may be a replay attack. Use a fresh timestamp.")
		.to_rpc_error());
	}
	let signature_bytes = decode_hex(signature).map_err_parse("Failed to decode signature")?;
	if signature_bytes.len() != 65 {
		let msg = format!("Invalid signature length, expected 65, got {}", signature_bytes.len());
		error!(msg);
		return Err(DetailedError::parse_error(&msg).to_rpc_error());
	}

	let signature_array: [u8; 65] = signature_bytes
		.try_into()
		.map_err_parse("Failed to convert signature bytes to array")?;

	let timestamp_bytes = timestamp.to_string();
	let challenge_hash = keccak256(timestamp_bytes.as_bytes());
	let challenge_hash_array: [u8; 32] = challenge_hash.0;

	let public_key = ecdsa::Public::from_raw(*expected_pubkey);
	let signature = ecdsa::Signature::from_raw(signature_array);

	if !ecdsa::Pair::verify_prehashed(&signature, &challenge_hash_array, &public_key) {
		error!("Signature verification failed for bundler key export");
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Authentication verification failed",
		)
		.with_field("signature")
		.with_reason("The signature does not match the challenge")
		.with_suggestion("Ensure you are using the correct private key")
		.to_rpc_error());
	}

	storage
		.insert(&BUNDLER_KEY_EXPORT_STORAGE_KEY.to_string(), timestamp)
		.map_err(|e| {
			error!("Failed to store new timestamp: {:?}", e);
			DetailedError::storage_service_error("insert timestamp").to_rpc_error()
		})?;

	debug!("Timestamp {} validated and stored successfully", timestamp);

	Ok(())
}

pub fn register_export_bundler_private_key<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_method("omni_exportBundlerPrivateKey", |params, ctx, _ext| {
			let params = parse_rpc_params::<ExportBundlerPrivateKeyParams>(params)?;

			debug!(
				"Received omni_exportBundlerPrivateKey request with timestamp: {}",
				params.timestamp
			);

			let key_bytes = decode_hex(&params.key).map_err_parse("Failed to decode key hex")?;

			let aes_key = ctx
				.shielding_key
				.private_key()
				.decrypt(Oaep::new::<Sha256>(), &key_bytes)
				.map_err_internal("Failed to decrypt shielded value")?;

			let aes_key: Aes256Key =
				aes_key.try_into().map_err_internal("Failed to convert AesKey")?;

			verify_signature(
				params.timestamp,
				&params.signature,
				&ctx.bundler_key_export_authorized_pubkey,
				&ctx.wildmeta_timestamp_storage,
			)?;

			debug!("Signature verified successfully, returning bundler private key");

			let encrypted_key: SerdeAesOutput =
				aes_encrypt_default(&aes_key, &ctx.bundler_private_key).into();
			Ok::<SerdeAesOutput, ErrorObjectOwned>(encrypted_key)
		})
		.expect("Failed to register omni_exportBundlerPrivateKey method");
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{start_server, ShieldingKey};
	use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
	use oe_client_binance::mocks::MockBinanceApiClient;
	use oe_client_pumpx::PumpxApiClient;
	use oe_client_signer::{mocks::MockSignerClient, SignerClient};
	use oe_client_wildmeta::{MockWildmetaApi, WildmetaApi};
	use oe_core::config::ConfigLoader;
	use oe_core::intent::executor::MockedIntentExecutor;
	use oe_crypto::{ecdsa, PairTrait};
	use oe_primitives::utils::hex::hex_encode;
	use oe_storage::{StorageDB, WildmetaTimestampStorage};
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use std::{collections::HashMap, sync::Arc};
	use tempfile::tempdir;

	const TEST_AES_KEY: Aes256Key = [42u8; 32];

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

	fn generate_and_encrypt_aes_key(shielding_key: &ShieldingKey) -> (Aes256Key, String) {
		use rsa::Oaep;
		use sha2::Sha256;
		let aes_key = TEST_AES_KEY;
		let encrypted_key = shielding_key
			.public_key()
			.encrypt(&mut rand::thread_rng(), Oaep::new::<Sha256>(), &aes_key)
			.expect("Failed to encrypt AES key");
		(aes_key, hex_encode(&encrypted_key))
	}

	fn decrypt_response(encrypted: &SerdeAesOutput, aes_key: &Aes256Key) -> Vec<u8> {
		use oe_crypto::aes256::{aes_decrypt, Aes256KeyNonce, AesOutput};
		let nonce: Aes256KeyNonce =
			encrypted.nonce.to_vec().try_into().expect("Invalid nonce length");
		let mut aes_output = AesOutput {
			ciphertext: encrypted.ciphertext.to_vec(),
			aad: encrypted.aad.to_vec(),
			nonce,
		};
		aes_decrypt(aes_key, &mut aes_output).expect("Failed to decrypt response")
	}

	async fn setup_test_server(
		port: u16,
		bundler_key: [u8; 32],
		authorized_pubkey: [u8; 33],
		storage_db: Arc<StorageDB>,
		shielding_key: ShieldingKey,
	) {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new("https://api.pumpx.ai".to_string());
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));
		let oe_client_binance_client: Arc<dyn oe_client_binance::BinancePaymasterApi> =
			Arc::new(MockBinanceApiClient::new());

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage =
			Arc::new(WildmetaTimestampStorage::new(storage_db.clone()));
		let loan_record_storage = Arc::new(oe_storage::LoanRecordStorage::new(storage_db.clone()));

		let (cross_chain_intent_executor, _) = MockedIntentExecutor::new();

		let aes_key = TEST_AES_KEY;
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key,
			Arc::new(Box::new(pumpx_api)),
			storage_db.clone(),
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			oe_client_binance_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			loan_record_storage,
			[0u8; 33],
			bundler_key,
			authorized_pubkey,
			Arc::new(cross_chain_intent_executor),
			aes_key,
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

		let shielding_key = ShieldingKey::new();
		let (aes_key, encrypted_key) = generate_and_encrypt_aes_key(&shielding_key);

		setup_test_server(port, bundler_key, authorized_pubkey, db, shielding_key).await;

		let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

		let signature = sign_timestamp(current_time, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let response: SerdeAesOutput = client
			.request(
				"omni_exportBundlerPrivateKey",
				rpc_params![current_time, signature, encrypted_key],
			)
			.await
			.unwrap();

		let decrypted = decrypt_response(&response, &aes_key);
		assert_eq!(decrypted, bundler_key.to_vec());
	}

	#[tokio::test]
	async fn test_export_bundler_key_invalid_signature() {
		let tmp_dir = tempdir().unwrap();
		let port = 2011;
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let (seed_a, _) = create_test_keypair();
		let (_, authorized_pubkey_b) = create_test_keypair();
		let bundler_key = [42u8; 32];

		let shielding_key = ShieldingKey::new();
		let (_, encrypted_key) = generate_and_encrypt_aes_key(&shielding_key);

		setup_test_server(port, bundler_key, authorized_pubkey_b, db, shielding_key).await;

		let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

		let signature = sign_timestamp(current_time, &seed_a);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let result: Result<SerdeAesOutput, _> = client
			.request(
				"omni_exportBundlerPrivateKey",
				rpc_params![current_time, signature, encrypted_key],
			)
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

		let shielding_key = ShieldingKey::new();
		let (_, encrypted_key) = generate_and_encrypt_aes_key(&shielding_key);

		setup_test_server(port, bundler_key, authorized_pubkey, db, shielding_key).await;

		let old_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
			as u64 - (6 * 60 * 1000);

		let signature = sign_timestamp(old_timestamp, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let result: Result<SerdeAesOutput, _> = client
			.request(
				"omni_exportBundlerPrivateKey",
				rpc_params![old_timestamp, signature, encrypted_key],
			)
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

		let shielding_key = ShieldingKey::new();
		let (_, encrypted_key) = generate_and_encrypt_aes_key(&shielding_key);

		setup_test_server(port, bundler_key, authorized_pubkey, db, shielding_key).await;

		let future_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
			as u64 + (2 * 60 * 1000);

		let signature = sign_timestamp(future_timestamp, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let result: Result<SerdeAesOutput, _> = client
			.request(
				"omni_exportBundlerPrivateKey",
				rpc_params![future_timestamp, signature, encrypted_key],
			)
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

		let shielding_key = ShieldingKey::new();
		let (_, encrypted_key) = generate_and_encrypt_aes_key(&shielding_key);

		setup_test_server(port, bundler_key, authorized_pubkey, db, shielding_key).await;

		let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

		let signature = sign_timestamp(current_time, &seed);

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let first_result: Result<SerdeAesOutput, _> = client
			.request(
				"omni_exportBundlerPrivateKey",
				rpc_params![current_time, signature.clone(), encrypted_key.clone()],
			)
			.await;
		assert!(first_result.is_ok());

		let second_result: Result<SerdeAesOutput, _> = client
			.request(
				"omni_exportBundlerPrivateKey",
				rpc_params![current_time, signature, encrypted_key],
			)
			.await;

		assert!(second_result.is_err());
	}
}
