use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use crate::utils::validation::parse_rpc_params;
use executor_core::intent_executor::IntentExecutor;
use executor_storage::LoanRecord;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryLoanTestParams {
	pub omni_account: String,
	pub nonce: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QueryLoanTestResponse {
	pub records: HashMap<String, LoanRecord>,
}

pub fn register_query_loan_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_method("omni_queryLoanTest", |params, ctx, _ext| {
			let params = parse_rpc_params::<QueryLoanTestParams>(params)?;

			debug!("Received omni_queryLoanTest, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account)?;

			let record_list = ctx.loan_record_storage.query_records(&omni_account, params.nonce);

			let mut records = HashMap::new();
			for (nonce, record) in record_list {
				records.insert(nonce.to_string(), record);
			}

			Ok::<QueryLoanTestResponse, ErrorObjectOwned>(QueryLoanTestResponse { records })
		})
		.expect("Failed to register omni_queryLoanTest method");
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{start_server, ShieldingKey};
	use config_loader::ConfigLoader;
	use executor_core::intent_executor::MockedIntentExecutor;
	use executor_primitives::utils::hex::hex_encode;
	use executor_primitives::AccountId;
	use executor_storage::{
		loan_record::LoanState, LoanRecordStorage, Storage, StorageDB, WildmetaTimestampStorage,
	};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use oe_client_binance::mocks::MockBinanceApiClient;
	use oe_client_pumpx::PumpxApiClient;
	use oe_client_signer::{mocks::MockSignerClient, SignerClient};
	use oe_client_wildmeta::{MockWildmetaApi, WildmetaApi};
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use std::collections::HashMap;
	use std::sync::Arc;
	use tempfile::tempdir;

	#[tokio::test]
	pub async fn test_query_loan_single_record() {
		let tmp_dir = tempdir().unwrap();
		let port = 3001;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		// Create a test omni account
		let omni_account = AccountId::from([1u8; 32]);
		let nonce: u64 = 123;

		// Store a test loan record
		let loan_record_storage = Arc::new(LoanRecordStorage::new(db.clone()));
		let test_record = LoanRecord {
			collateral_ticker: "BTC".to_string(),
			collateral_size: "0.5".to_string(),
			usdc_sold: "25000.00".to_string(),
			usdc_loaned: "20000.00".to_string(),
			usdc_for_perp: "20000.00".to_string(),
			txs: Vec::new(),
			cloids: vec![
				("spot_sell".to_string(), "spot123".to_string()),
				("hedge_open".to_string(), "hedge456".to_string()),
			],
			position_size: "0.5".to_string(),
			state: LoanState::HedgeOpened,
		};

		let storage_key =
			executor_storage::loan_record::Key { account_id: omni_account.clone(), nonce };
		loan_record_storage.insert(&storage_key, test_record.clone()).unwrap();

		// Start server
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
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		let (cross_chain_intent_executor, _cross_chain_mock_recv) = MockedIntentExecutor::new();
		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			oe_client_binance_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			loan_record_storage,
			[0u8; 33],
			[0u8; 32],
			[0u8; 33],
			Arc::new(cross_chain_intent_executor),
			aes_key,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();

		// Query the specific record
		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let response: QueryLoanTestResponse = client
			.request(
				"omni_queryLoanTest",
				rpc_params![hex_encode(omni_account.as_ref()), Some(nonce)],
			)
			.await
			.unwrap();

		// Verify response
		assert_eq!(response.records.len(), 1);
		let returned_record = response.records.get(&nonce.to_string()).unwrap();
		assert_eq!(returned_record.collateral_ticker, "BTC");
		assert_eq!(returned_record.collateral_size, "0.5");
		assert_eq!(returned_record.usdc_sold, "25000.00");
		assert_eq!(returned_record.usdc_loaned, "20000.00");
		let spot_sell_cloid = returned_record
			.cloids
			.iter()
			.find(|(name, _)| name == "spot_sell")
			.map(|(_, cloid)| cloid.as_str());
		assert_eq!(spot_sell_cloid, Some("spot123"));
		let hedge_open_cloid = returned_record
			.cloids
			.iter()
			.find(|(name, _)| name == "hedge_open")
			.map(|(_, cloid)| cloid.as_str());
		assert_eq!(hedge_open_cloid, Some("hedge456"));
	}

	#[tokio::test]
	pub async fn test_query_loan_all_records() {
		let tmp_dir = tempdir().unwrap();
		let port = 3002;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		// Create a test omni account
		let omni_account = AccountId::from([2u8; 32]);

		// Store multiple test loan records
		let loan_record_storage = Arc::new(LoanRecordStorage::new(db.clone()));

		let test_records = vec![
			(
				100u64,
				LoanRecord {
					collateral_ticker: "BTC".to_string(),
					collateral_size: "1.0".to_string(),
					usdc_sold: "50000.00".to_string(),
					usdc_loaned: "40000.00".to_string(),
					usdc_for_perp: "40000.00".to_string(),
					txs: Vec::new(),
					cloids: vec![
						("spot_sell".to_string(), "spot100".to_string()),
						("hedge_open".to_string(), "hedge100".to_string()),
					],
					position_size: "1.0".to_string(),
					state: LoanState::HedgeOpened,
				},
			),
			(
				200u64,
				LoanRecord {
					collateral_ticker: "ETH".to_string(),
					collateral_size: "10.0".to_string(),
					usdc_sold: "25000.00".to_string(),
					usdc_loaned: "20000.00".to_string(),
					usdc_for_perp: "20000.00".to_string(),
					txs: Vec::new(),
					cloids: vec![
						("spot_sell".to_string(), "spot200".to_string()),
						("hedge_open".to_string(), "hedge200".to_string()),
					],
					position_size: "10.0".to_string(),
					state: LoanState::HedgeOpened,
				},
			),
			(
				300u64,
				LoanRecord {
					collateral_ticker: "SOL".to_string(),
					collateral_size: "100.0".to_string(),
					usdc_sold: "15000.00".to_string(),
					usdc_loaned: "12000.00".to_string(),
					usdc_for_perp: "12000.00".to_string(),
					txs: Vec::new(),
					cloids: vec![
						("spot_sell".to_string(), "spot300".to_string()),
						("hedge_open".to_string(), "hedge300".to_string()),
					],
					position_size: "100.0".to_string(),
					state: LoanState::HedgeOpened,
				},
			),
		];

		for (nonce, record) in &test_records {
			let storage_key = executor_storage::loan_record::Key {
				account_id: omni_account.clone(),
				nonce: *nonce,
			};
			loan_record_storage.insert(&storage_key, record.clone()).unwrap();
		}

		// Start server
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
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		let (cross_chain_intent_executor, _cross_chain_mock_recv) = MockedIntentExecutor::new();
		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			oe_client_binance_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			loan_record_storage,
			[0u8; 33],
			[0u8; 32],
			[0u8; 33],
			Arc::new(cross_chain_intent_executor),
			aes_key,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();

		// Query all records (no nonce specified)
		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let response: QueryLoanTestResponse = client
			.request(
				"omni_queryLoanTest",
				rpc_params![hex_encode(omni_account.as_ref()), None::<u64>],
			)
			.await
			.unwrap();

		// Verify response contains all 3 records
		assert_eq!(response.records.len(), 3);

		for (nonce, expected_record) in &test_records {
			let returned_record = response.records.get(&nonce.to_string()).unwrap();
			assert_eq!(returned_record.collateral_ticker, expected_record.collateral_ticker);
			assert_eq!(returned_record.collateral_size, expected_record.collateral_size);
			assert_eq!(returned_record.usdc_sold, expected_record.usdc_sold);
			assert_eq!(returned_record.usdc_loaned, expected_record.usdc_loaned);
			assert_eq!(returned_record.cloids, expected_record.cloids);
		}
	}

	#[tokio::test]
	pub async fn test_query_loan_nonexistent_nonce() {
		let tmp_dir = tempdir().unwrap();
		let port = 3003;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let omni_account = AccountId::from([3u8; 32]);
		let loan_record_storage = Arc::new(LoanRecordStorage::new(db.clone()));

		// Store a record with nonce 100
		let storage_key =
			executor_storage::loan_record::Key { account_id: omni_account.clone(), nonce: 100 };
		let test_record = LoanRecord {
			collateral_ticker: "BTC".to_string(),
			collateral_size: "1.0".to_string(),
			usdc_sold: "50000.00".to_string(),
			usdc_loaned: "40000.00".to_string(),
			usdc_for_perp: "40000.00".to_string(),
			txs: Vec::new(),
			cloids: vec![
				("spot_sell".to_string(), "spot100".to_string()),
				("hedge_open".to_string(), "hedge100".to_string()),
			],
			position_size: "1.0".to_string(),
			state: LoanState::HedgeOpened,
		};
		loan_record_storage.insert(&storage_key, test_record).unwrap();

		// Start server
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
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		let (cross_chain_intent_executor, _cross_chain_mock_recv) = MockedIntentExecutor::new();
		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			oe_client_binance_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			loan_record_storage,
			[0u8; 33],
			[0u8; 32],
			[0u8; 33],
			Arc::new(cross_chain_intent_executor),
			aes_key,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();

		// Query with a non-existent nonce (999)
		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let response: QueryLoanTestResponse = client
			.request(
				"omni_queryLoanTest",
				rpc_params![hex_encode(omni_account.as_ref()), Some(999u64)],
			)
			.await
			.unwrap();

		// Verify response is empty
		assert_eq!(response.records.len(), 0);
	}

	#[tokio::test]
	pub async fn test_query_loan_nonexistent_account() {
		let tmp_dir = tempdir().unwrap();
		let port = 3004;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let loan_record_storage = Arc::new(LoanRecordStorage::new(db.clone()));

		// Store a record for a different account
		let existing_account = AccountId::from([4u8; 32]);
		let storage_key =
			executor_storage::loan_record::Key { account_id: existing_account.clone(), nonce: 100 };
		let test_record = LoanRecord {
			collateral_ticker: "BTC".to_string(),
			collateral_size: "1.0".to_string(),
			usdc_sold: "50000.00".to_string(),
			usdc_loaned: "40000.00".to_string(),
			usdc_for_perp: "40000.00".to_string(),
			txs: Vec::new(),
			cloids: vec![
				("spot_sell".to_string(), "spot100".to_string()),
				("hedge_open".to_string(), "hedge100".to_string()),
			],
			position_size: "1.0".to_string(),
			state: LoanState::HedgeOpened,
		};
		loan_record_storage.insert(&storage_key, test_record).unwrap();

		// Start server
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
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		let (cross_chain_intent_executor, _cross_chain_mock_recv) = MockedIntentExecutor::new();
		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			oe_client_binance_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			loan_record_storage,
			[0u8; 33],
			[0u8; 32],
			[0u8; 33],
			Arc::new(cross_chain_intent_executor),
			aes_key,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();

		// Query with a different account that has no records
		let query_account = AccountId::from([99u8; 32]);
		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		let response: QueryLoanTestResponse = client
			.request(
				"omni_queryLoanTest",
				rpc_params![hex_encode(query_account.as_ref()), None::<u64>],
			)
			.await
			.unwrap();

		// Verify response is empty
		assert_eq!(response.records.len(), 0);
	}
}
