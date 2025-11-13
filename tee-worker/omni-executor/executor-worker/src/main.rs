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

use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use clap::Parser;
use cli::{Cli, Commands, ExportBundlerKeyArgs};
use config_loader::ConfigLoader;
use cross_chain_intent_executor::{Chain, CrossChainIntentExecutor, RpcEndpointRegistry};
use executor_core::ecdsa_key_store::EcdsaKeyStore;
use executor_core::ed25519_key_store::Ed25519KeyStore;
use executor_core::key_store::KeyStore;
use executor_core::shielding_key_store::ShieldingKeyStore;
use executor_core::wallet_metrics::Wallet;
use executor_core::wallet_metrics::{
	start_wallet_metrics, WalletBalanceFetcher, WalletId, WalletMetrics, WalletNetworkType,
};
use executor_core::Aes256KeyStore;
use executor_crypto::{ecdsa, ed25519, PairTrait};
use executor_storage::init_storage;
use intent_asset_lock::precise::PreciseAssetsLock;
use intent_asset_lock::AccountAssetLocks;
use metrics_exporter_prometheus::PrometheusBuilder;
use oe_client_accounting::{
	solana::AccountingContractClient as SolanaAccountingContractClient,
	AccountingContractClient as EthereumAccountingContractClient,
};
use oe_client_binance::BinanceApiClient;
use oe_client_ethereum::client::EthereumRpcClient;
use oe_client_pumpx::{pubkey_to_evm_address, pubkey_to_solana_address};
use oe_client_pumpx::{PumpxApi, PumpxApiClient};
use oe_client_solana::SolanaRpcClient;
use rpc_server::{start_server as start_rpc_server, AuthTokenKeyStore};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::FmtSubscriber;
mod cli;

#[tokio::main]
async fn main() -> Result<(), ()> {
	let subscriber = FmtSubscriber::builder()
		.with_env_filter(
			EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
		)
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

	let cli = Cli::parse();

	match cli.cmd {
		Commands::ExportBundlerKey(args) => {
			export_bundler_key(args).await?;
		},
		Commands::Run(args) => {
			if args.enable_mock_server {
				#[cfg(feature = "mock-server")]
				{
					let mock_server_port = args.mock_server_port;
					std::thread::spawn(move || {
						mock_server::run(mock_server_port).expect("Mock server failed to start");
					});
				}
			}

			let config_loader = ConfigLoader::from_env();
			let builder = PrometheusBuilder::new();

			let address = SocketAddr::from_str(&format!("0.0.0.0:{}", args.metrics_port)).unwrap();
			builder
				.with_http_listener(address)
				.install()
				.expect("failed to install Prometheus recorder");

			let auth_token_key_store = AuthTokenKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/auth_token_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);
			let jwt_rsa_private_key = auth_token_key_store.read().expect("Could not read jwt key");

			let pumpx_auth_key_store = oe_client_pumpx::auth_key_store::AuthKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/pumpx_auth_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let pumpx_signer_key =
				pumpx_auth_key_store.read().expect("Could not read PumpX signer key");

			let pumpx_signer_pair = ecdsa::Pair::from_seed_slice(&pumpx_signer_key).unwrap();
			info!("PumpX auth public key: {:?}", pumpx_signer_pair.public());

			let evm_accounting_ecdsa_signer_key = EcdsaKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/accounting_ecdsa_signer_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let evm_accounting_ecdsa_signer_key = evm_accounting_ecdsa_signer_key
				.read()
				.expect("Could not read accounting ecsa key");
			let evm_accounting_ecdsa_signer_key_pair =
				ecdsa::Pair::from_seed_slice(&evm_accounting_ecdsa_signer_key).unwrap();

			let bsc_accounting_signer =
				pubkey_to_evm_address(evm_accounting_ecdsa_signer_key_pair.public().as_ref())
					.unwrap();

			info!("Accounting ecdsa signer address: {:?}", bsc_accounting_signer);

			let solana_accounting_ed25519_signer_key = Ed25519KeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/solana_accounting_ed25519_signer_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let solana_accounting_ed25519_signer_key = solana_accounting_ed25519_signer_key
				.read()
				.expect("Could not read solana accounting ed25519 key");
			let solana_accounting_ed25519_signer_key_pair =
				ed25519::Pair::from_seed_slice(&solana_accounting_ed25519_signer_key).unwrap();

			info!(
				"Solana accounting ed25519 signer address: {:?}",
				pubkey_to_solana_address(
					solana_accounting_ed25519_signer_key_pair.public().as_ref()
				)
				.unwrap()
			);

			let storage_db = init_storage().await.expect("Could not initialize storage");

			let aes256_key_store = Aes256KeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/aes_256_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);
			let aes256_key = aes256_key_store.read().expect("Could not read aes256 key");

			let pumpx_signer_client: Arc<Box<dyn oe_client_signer::SignerClient>> =
				Arc::new(Box::new(oe_client_pumpx::signer_client::PumpxSignerClient::new(
					config_loader.pumpx_signer_url.clone(),
					pumpx_signer_pair,
				)));

			let mut rpc_endpoint_registry = RpcEndpointRegistry::new();
			rpc_endpoint_registry.insert(Chain::Solana, config_loader.solana_url.to_string());
			rpc_endpoint_registry.insert(Chain::Ethereum(56), config_loader.bsc_url.to_string());

			if let Some(ref bsc_testnet_url) = config_loader.bsc_testnet_url {
				rpc_endpoint_registry.insert(Chain::Ethereum(97), bsc_testnet_url.to_owned());
			}

			let pumpx_api: Arc<Box<dyn PumpxApi>> = Arc::new(Box::new(PumpxApiClient::new(
				config_loader.pumpx_api_base_url.to_string(),
			)));

			let oe_client_binance: Arc<BinanceApiClient> = Arc::new(BinanceApiClient::new(
				config_loader.oe_client_binance_key.clone(),
				config_loader.oe_client_binance_secret.clone(),
				config_loader.oe_client_binance_base_url.clone(),
			));

			let solana_client: Arc<SolanaRpcClient> =
				Arc::new(SolanaRpcClient::new(&config_loader.solana_url));

			let accounting_contract_signer =
				PrivateKeySigner::from_slice(&evm_accounting_ecdsa_signer_key_pair.seed())
					.expect("Could not create accounting contract signer");
			let accounting_contract_wallet = EthereumWallet::from(accounting_contract_signer);

			let bsc_rpc_provider = oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
				&config_loader.bsc_url,
				accounting_contract_wallet.clone(),
			);
			let evm_oe_client_accounting = EthereumAccountingContractClient::new(
				bsc_rpc_provider,
				args.accounting_contract_address.parse().unwrap(),
			);
			let solana_oe_client_accounting = SolanaAccountingContractClient::new(
				solana_accounting_ed25519_signer_key_pair,
				config_loader.solana_url.clone(),
				args.solana_accounting_contract_address.parse().unwrap(),
			);

			let bsc_client: Arc<EthereumRpcClient> =
				Arc::new(EthereumRpcClient::new(&config_loader.bsc_url));

			// wallet monitoring setup start
			let bsc_wallet_balance_fetcher: Arc<Box<dyn WalletBalanceFetcher>> = Arc::new(
				Box::new(oe_client_ethereum::AlloyRpcProvider::new(&config_loader.bsc_url)),
			);

			let solana_wallet_balance_fetcher: Arc<Box<dyn WalletBalanceFetcher>> =
				Arc::new(Box::new(SolanaRpcClient::new(&config_loader.solana_url)));

			let mut balance_fetchers: HashMap<
				WalletNetworkType,
				Arc<Box<dyn WalletBalanceFetcher>>,
			> = HashMap::new();
			balance_fetchers.insert(WalletNetworkType::Solana, solana_wallet_balance_fetcher);
			balance_fetchers.insert(WalletNetworkType::Ethereum(56), bsc_wallet_balance_fetcher);

			let mut wallet_metrics = WalletMetrics::new(balance_fetchers);

			wallet_metrics.register(Wallet {
				id: WalletId {
					address: bsc_accounting_signer,
					network_type: WalletNetworkType::Ethereum(56),
				},
				name: "bsc_accounting_signer".to_string(),
			});

			let join = start_wallet_metrics(Handle::current(), wallet_metrics);
			// wallet monitoring setup end

			let account_assets_lock: Arc<AccountAssetLocks<PreciseAssetsLock>> =
				Arc::new(AccountAssetLocks::new(storage_db.clone()));

			// TODO: Make these configurable via CLI args or config file
			let omni_account_factory_address = Address::from_slice(&[0u8; 20]);
			let omni_account_implementation_address = Address::from_slice(&[1u8; 20]);

			let cross_chain_intent_executor = CrossChainIntentExecutor::new(
				account_assets_lock,
				rpc_endpoint_registry,
				pumpx_signer_client.clone(),
				pumpx_api.clone(),
				storage_db.clone(),
				oe_client_binance.clone(),
				bsc_client,
				solana_client,
				Arc::new(Box::new(evm_oe_client_accounting)),
				Arc::new(Box::new(solana_oe_client_accounting)),
				Decimal::from_str(&args.instant_payout_threshold).unwrap(),
				omni_account_factory_address,
				omni_account_implementation_address,
			)?;

			// Create EntryPoint clients registry
			// Create RPC providers first
			// Add BSC (BNB Chain)
			let bsc_rpc = Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
				&config_loader.bsc_url,
				accounting_contract_wallet.clone(),
			));

			// Add BSC Testnet if configured
			let bsc_testnet_rpc = if let Some(ref bsc_testnet_url) = config_loader.bsc_testnet_url {
				let bsc_testnet_rpc =
					Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
						bsc_testnet_url,
						accounting_contract_wallet.clone(),
					));
				Some(bsc_testnet_rpc)
			} else {
				None
			};

			// Add Ethereum Mainnet
			let oe_client_ethereum =
				Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
					&config_loader.ethereum_url,
					accounting_contract_wallet.clone(),
				));

			// Add local development chain
			let local_rpc = Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
				"http://ethereum-node:8545",
				accounting_contract_wallet.clone(),
			));

			// Add Arbitrum One
			let arbitrum_rpc = Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
				&config_loader.arbitrum_url,
				accounting_contract_wallet.clone(),
			));

			// Add Arbitrum Testnet if configured
			let arbitrum_testnet_rpc =
				if let Some(ref arbitrum_testnet_url) = config_loader.arbitrum_testnet_url {
					let arbitrum_testnet_rpc =
						Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
							arbitrum_testnet_url,
							accounting_contract_wallet.clone(),
						));
					Some(arbitrum_testnet_rpc)
				} else {
					None
				};

			// Add HyperEVM
			let hyperevm_rpc = Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
				&config_loader.hyperevm_url,
				accounting_contract_wallet.clone(),
			));

			// Add HyperEVM Testnet if configured
			let hyperevm_testnet_rpc =
				if let Some(ref hyperevm_testnet_url) = config_loader.hyperevm_testnet_url {
					let hyperevm_testnet_rpc =
						Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
							hyperevm_testnet_url,
							accounting_contract_wallet.clone(),
						));
					Some(hyperevm_testnet_rpc)
				} else {
					None
				};

			// Add Base
			let base_rpc = Arc::new(oe_client_ethereum::AlloyRpcProvider::new_with_wallet(
				&config_loader.base_url,
				accounting_contract_wallet.clone(),
			));

			// Create EntryPoint clients
			let mut entry_point_clients = HashMap::new();

			// Parse EntryPoint address from configuration
			let entry_point_address = config_loader
				.entry_point_address
				.parse::<alloy::primitives::Address>()
				.expect("Invalid entry point address in configuration");

			// Add BSC (BNB Chain)
			let bsc_entry_point = Arc::new(oe_client_aa::EntryPointClient::new_with_config(
				entry_point_address,
				bsc_rpc,
				oe_client_aa::GasPriceConfig::bsc(),
				oe_client_aa::RetryConfig::bsc(),
			));
			entry_point_clients.insert(56, bsc_entry_point);

			// Add BSC Testnet if configured
			if let Some(bsc_testnet_rpc) = bsc_testnet_rpc {
				let bsc_testnet_entry_point =
					Arc::new(oe_client_aa::EntryPointClient::new_with_config(
						entry_point_address,
						bsc_testnet_rpc,
						oe_client_aa::GasPriceConfig::bsc(),
						oe_client_aa::RetryConfig::bsc(),
					));
				entry_point_clients.insert(97, bsc_testnet_entry_point);
			}

			// Add Ethereum Mainnet
			let ethereum_entry_point = Arc::new(oe_client_aa::EntryPointClient::new_with_config(
				entry_point_address,
				oe_client_ethereum,
				oe_client_aa::GasPriceConfig::mainnet(),
				oe_client_aa::RetryConfig::mainnet(),
			));
			entry_point_clients.insert(1, ethereum_entry_point);

			// Add local development chain
			let local_entry_point = Arc::new(oe_client_aa::EntryPointClient::new_with_config(
				entry_point_address,
				local_rpc,
				oe_client_aa::GasPriceConfig::default(),
				oe_client_aa::RetryConfig::default(),
			));
			entry_point_clients.insert(31337, local_entry_point);

			// Add Arbitrum One (Chain ID: 42161)
			let arbitrum_entry_point = Arc::new(oe_client_aa::EntryPointClient::new_with_config(
				entry_point_address,
				arbitrum_rpc,
				oe_client_aa::GasPriceConfig::l2(),
				oe_client_aa::RetryConfig::l2(),
			));
			entry_point_clients.insert(42161, arbitrum_entry_point);

			// Add Arbitrum Testnet if configured (Chain ID: 421614)
			if let Some(arbitrum_testnet_rpc) = arbitrum_testnet_rpc {
				let arbitrum_testnet_entry_point =
					Arc::new(oe_client_aa::EntryPointClient::new_with_config(
						entry_point_address,
						arbitrum_testnet_rpc,
						oe_client_aa::GasPriceConfig::l2(),
						oe_client_aa::RetryConfig::l2(),
					));
				entry_point_clients.insert(421614, arbitrum_testnet_entry_point);
			}

			// Add HyperEVM (Chain ID: 999)
			let hyperevm_entry_point = Arc::new(oe_client_aa::EntryPointClient::new_with_config(
				entry_point_address,
				hyperevm_rpc,
				oe_client_aa::GasPriceConfig::hyperevm(),
				oe_client_aa::RetryConfig::hyperevm(),
			));
			entry_point_clients.insert(999, hyperevm_entry_point);

			// Add HyperEVM Testnet if configured (Chain ID: 998)
			if let Some(hyperevm_testnet_rpc) = hyperevm_testnet_rpc {
				let hyperevm_testnet_entry_point =
					Arc::new(oe_client_aa::EntryPointClient::new_with_config(
						entry_point_address,
						hyperevm_testnet_rpc,
						oe_client_aa::GasPriceConfig::hyperevm(),
						oe_client_aa::RetryConfig::hyperevm(),
					));
				entry_point_clients.insert(998, hyperevm_testnet_entry_point);
			}

			// Add Base (Chain ID: 8453)
			let base_entry_point = Arc::new(oe_client_aa::EntryPointClient::new_with_config(
				entry_point_address,
				base_rpc,
				oe_client_aa::GasPriceConfig::l2(),
				oe_client_aa::RetryConfig::l2(),
			));
			entry_point_clients.insert(8453, base_entry_point);

			let entry_point_clients = Arc::new(entry_point_clients);

			let worker_url =
				url::Url::parse(&config_loader.pumpx_worker_url).expect("Invalid worker url");

			let shielding_key_store = ShieldingKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/shielding_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let shielding_key = shielding_key_store.read().expect("Could not read shielding key");

			// Create wildmeta API client and timestamp storage
			let wildmeta_api: Arc<Box<dyn oe_client_wildmeta::WildmetaApi>> = Arc::new(Box::new(
				oe_client_wildmeta::WildmetaApiClient::new(config_loader.wildmeta_api_url.clone()),
			));
			let wildmeta_timestamp_storage =
				Arc::new(executor_storage::WildmetaTimestampStorage::new(storage_db.clone()));

			// Create loan record storage
			let loan_record_storage =
				Arc::new(executor_storage::LoanRecordStorage::new(storage_db.clone()));

			// Parse wildmeta backend ECDSA public key from hex
			let wildmeta_backend_ecdsa_pubkey = {
				use executor_primitives::utils::hex::decode_hex;
				let pubkey_hex = &config_loader.wildmeta_backend_ecdsa_pubkey;
				let pubkey_bytes = decode_hex(pubkey_hex).map_err(|e| {
					error!("Failed to decode wildmeta backend ECDSA public key: {:?}", e);
				})?;
				if pubkey_bytes.len() != 33 {
					error!(
						"Invalid wildmeta backend ECDSA public key length: expected 33 bytes, got {}",
						pubkey_bytes.len()
					);
					return Err(());
				}
				let mut pubkey_array = [0u8; 33];
				pubkey_array.copy_from_slice(&pubkey_bytes);
				pubkey_array
			};

			let bundler_key_export_authorized_pubkey =
				{
					use executor_primitives::utils::hex::decode_hex;
					let pubkey_hex = &config_loader.bundler_key_export_authorized_pubkey;
					let pubkey_bytes =
						decode_hex(pubkey_hex).map_err(|e| {
							error!("Failed to decode bundler key export authorized ECDSA public key: {:?}", e);
						})?;
					if pubkey_bytes.len() != 33 {
						error!(
						"Invalid bundler key export authorized ECDSA public key length: expected 33 bytes, got {}",
						pubkey_bytes.len()
					);
						return Err(());
					}
					let mut pubkey_array = [0u8; 33];
					pubkey_array.copy_from_slice(&pubkey_bytes);
					pubkey_array
				};

			start_rpc_server(
				worker_url.port().expect("Missing worker port"),
				shielding_key,
				pumpx_api,
				storage_db.clone(),
				jwt_rsa_private_key,
				&config_loader,
				pumpx_signer_client,
				oe_client_binance,
				wildmeta_api,
				wildmeta_timestamp_storage,
				loan_record_storage,
				wildmeta_backend_ecdsa_pubkey,
				evm_accounting_ecdsa_signer_key,
				bundler_key_export_authorized_pubkey,
				Arc::new(cross_chain_intent_executor),
				aes256_key,
				entry_point_clients,
			)
			.await
			.map_err(|e| {
				error!("Could not start server: {:?}", e);
			})?;

			if let Err(e) = join.await {
				error!("There was an error in associated task: {:?}", e);
			};

			match signal::ctrl_c().await {
				Ok(()) => {},
				Err(err) => {
					eprintln!("Unable to listen for shutdown signal: {}", err);
					// we also shut down in case of error
				},
			}
		},
	}

	Ok(())
}

async fn export_bundler_key(args: ExportBundlerKeyArgs) -> Result<(), ()> {
	use alloy::primitives::keccak256;
	use executor_crypto::{
		aes256::{aes_decrypt, Aes256Key, Aes256KeyNonce, AesOutput},
		ecdsa, PairTrait,
	};
	use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
	use rsa::{Oaep, RsaPublicKey};
	use sha2::Sha256;
	use std::time::{SystemTime, UNIX_EPOCH};

	info!("Loading authorized key from: {}", args.authorized_key_path);

	let authorized_key_bytes = std::fs::read(&args.authorized_key_path).map_err(|e| {
		error!("Failed to read authorized key file: {:?}", e);
		eprintln!("❌ Error: Failed to read authorized key file: {}", e);
	})?;

	if authorized_key_bytes.len() != 32 {
		error!(
			"Invalid authorized key length: expected 32 bytes, got {}",
			authorized_key_bytes.len()
		);
		eprintln!("❌ Error: Invalid authorized key file (expected 32 bytes)");
		return Err(());
	}

	let mut authorized_seed = [0u8; 32];
	authorized_seed.copy_from_slice(&authorized_key_bytes);

	let authorized_pair = ecdsa::Pair::from_seed_slice(&authorized_seed).map_err(|e| {
		error!("Failed to create keypair from seed: {:?}", e);
		eprintln!("❌ Error: Failed to create keypair from authorized key");
	})?;

	info!("Authorized public key: {:?}", authorized_pair.public());

	info!("Connecting to worker at: {}", args.worker_url);
	let client = WsClientBuilder::default().build(&args.worker_url).await.map_err(|e| {
		error!("Failed to connect to worker: {:?}", e);
		eprintln!("❌ Error: Failed to connect to worker at {}: {}", args.worker_url, e);
	})?;

	info!("Getting shielding key from worker...");
	#[derive(serde::Deserialize, Debug)]
	struct ShieldingKeyResponse {
		n: ethers::types::Bytes,
		e: ethers::types::Bytes,
	}

	let shielding_key: ShieldingKeyResponse =
		client.request("omni_getShieldingKey", rpc_params![]).await.map_err(|e| {
			error!("Failed to get shielding key: {:?}", e);
			eprintln!("❌ Error: Failed to get shielding key from worker: {}", e);
		})?;

	let mut n_bytes = shielding_key.n.to_vec();
	n_bytes.reverse();
	let mut e_bytes = shielding_key.e.to_vec();
	e_bytes.reverse();

	let rsa_public_key = RsaPublicKey::new(
		rsa::BigUint::from_bytes_be(&n_bytes),
		rsa::BigUint::from_bytes_be(&e_bytes),
	)
	.map_err(|e| {
		error!("Failed to create RSA public key: {:?}", e);
		eprintln!("❌ Error: Failed to create RSA public key: {}", e);
	})?;

	info!("Generating random AES key...");
	let aes_key: Aes256Key = rand::random();

	info!("RSA-encrypting AES key...");
	let encrypted_aes_key = rsa_public_key
		.encrypt(&mut rand::thread_rng(), Oaep::new::<Sha256>(), &aes_key)
		.map_err(|e| {
			error!("Failed to RSA-encrypt AES key: {:?}", e);
			eprintln!("❌ Error: Failed to RSA-encrypt AES key: {}", e);
		})?;

	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_err(|e| {
			error!("Failed to get current time: {:?}", e);
			eprintln!("❌ Error: System time error");
		})?
		.as_millis() as u64;

	info!("Signing timestamp: {}", timestamp);
	let timestamp_str = timestamp.to_string();
	let challenge_hash = keccak256(timestamp_str.as_bytes());
	let signature = authorized_pair.sign_prehashed(&challenge_hash.0);

	let signature_hex = format!("0x{}", hex::encode(signature.0));
	let encrypted_key_hex = format!("0x{}", hex::encode(&encrypted_aes_key));

	info!("Calling omni_exportBundlerPrivateKey RPC...");
	#[derive(serde::Deserialize, Debug)]
	struct SerdeAesOutput {
		ciphertext: ethers::types::Bytes,
		aad: ethers::types::Bytes,
		nonce: ethers::types::Bytes,
	}

	let encrypted_response: SerdeAesOutput = client
		.request(
			"omni_exportBundlerPrivateKey",
			rpc_params![timestamp, signature_hex, encrypted_key_hex],
		)
		.await
		.map_err(|e| {
			error!("RPC call failed: {:?}", e);
			eprintln!("❌ Error: RPC call failed: {}", e);
		})?;

	info!("Decrypting bundler private key...");
	let nonce: Aes256KeyNonce = encrypted_response.nonce.to_vec().try_into().map_err(|_| {
		error!("Invalid nonce length in response");
		eprintln!("❌ Error: Invalid response nonce length");
	})?;

	let mut aes_output = AesOutput {
		ciphertext: encrypted_response.ciphertext.to_vec(),
		aad: encrypted_response.aad.to_vec(),
		nonce,
	};

	let bundler_key = aes_decrypt(&aes_key, &mut aes_output).ok_or_else(|| {
		error!("Failed to decrypt bundler private key");
		eprintln!("❌ Error: Failed to decrypt bundler private key");
	})?;

	let bundler_key_hex = format!("0x{}", hex::encode(&bundler_key));

	println!("\n✅ Bundler Private Key: {}", bundler_key_hex);
	println!("⚠️  WARNING: This is a sensitive key. Store it securely and never share it.\n");

	Ok(())
}
