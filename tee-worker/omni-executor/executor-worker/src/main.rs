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
use alloy::signers::local::PrivateKeySigner;
use binance_api::BinanceApiClient;
use clap::Parser;
use cli::{Cli, Commands};
use config_loader::ConfigLoader;
use ethereum_intent_executor::EthereumIntentExecutor;
use executor_core::ecdsa_key_store::EcdsaKeyStore;
use executor_core::ed25519_key_store::Ed25519KeyStore;
use executor_core::key_store::KeyStore;
use executor_core::shielding_key_store::ShieldingKeyStore;
use executor_crypto::{ecdsa, ed25519, PairTrait};
use executor_storage::init_storage;
use executor_utils::{pubkey_to_evm_address, pubkey_to_solana_address};
use metrics_exporter_prometheus::PrometheusBuilder;
use native_task_handler::Aes256KeyStore;
use rpc_server::{start_server as start_rpc_server, AuthTokenKeyStore};
use solana_intent_executor::SolanaIntentExecutor;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
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

			let signer_auth_key_store = EcdsaKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/pumpx_auth_key.bin") // keeping the old name for now
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let signer_auth_key =
				signer_auth_key_store.read().expect("Could not read signer auth key");

			let signer_auth_pair = ecdsa::Pair::from_seed_slice(&signer_auth_key).unwrap();
			info!("Signer auth public key: {:?}", signer_auth_pair.public());

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

			let signer_client: Arc<Box<dyn signer_client::SignerClient>> =
				Arc::new(Box::new(signer_client::http_client::HttpSignerClient::new(
					config_loader.signer_url.clone(),
					signer_auth_pair,
				)));

			let ethereum_intent_executor = EthereumIntentExecutor::new(
				&config_loader.ethereum_url,
				&args.delegation_contract_address,
			)?;
			let solana_intent_executor = SolanaIntentExecutor::new(&config_loader.solana_url)?;

			let binance_api: Arc<BinanceApiClient> = Arc::new(BinanceApiClient::new(
				config_loader.binance_api_key.clone(),
				config_loader.binance_api_secret.clone(),
				config_loader.binance_api_base_url.clone(),
			));

			let accounting_contract_signer =
				PrivateKeySigner::from_slice(&evm_accounting_ecdsa_signer_key_pair.seed())
					.expect("Could not create accounting contract signer");
			let accounting_contract_wallet = EthereumWallet::from(accounting_contract_signer);

			// Create EntryPoint clients registry
			// Create RPC providers first
			// Add BSC (BNB Chain)
			let bsc_rpc = Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
				&config_loader.bsc_url,
				accounting_contract_wallet.clone(),
			));

			// Add BSC Testnet if configured
			let bsc_testnet_rpc = if let Some(ref bsc_testnet_url) = config_loader.bsc_testnet_url {
				let bsc_testnet_rpc = Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
					bsc_testnet_url,
					accounting_contract_wallet.clone(),
				));
				Some(bsc_testnet_rpc)
			} else {
				None
			};

			// Add Ethereum Mainnet
			let ethereum_rpc = Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
				&config_loader.ethereum_url,
				accounting_contract_wallet.clone(),
			));

			// Add local development chain
			let local_rpc = Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
				"http://ethereum-node:8545",
				accounting_contract_wallet.clone(),
			));

			// Add Arbitrum One
			let arbitrum_rpc = Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
				&config_loader.arbitrum_url,
				accounting_contract_wallet.clone(),
			));

			// Add Arbitrum Testnet if configured
			let arbitrum_testnet_rpc =
				if let Some(ref arbitrum_testnet_url) = config_loader.arbitrum_testnet_url {
					let arbitrum_testnet_rpc =
						Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
							arbitrum_testnet_url,
							accounting_contract_wallet.clone(),
						));
					Some(arbitrum_testnet_rpc)
				} else {
					None
				};

			// Add HyperEVM
			let hyperevm_rpc = Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
				&config_loader.hyperevm_url,
				accounting_contract_wallet.clone(),
			));

			// Add HyperEVM Testnet if configured
			let hyperevm_testnet_rpc =
				if let Some(ref hyperevm_testnet_url) = config_loader.hyperevm_testnet_url {
					let hyperevm_testnet_rpc =
						Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
							hyperevm_testnet_url,
							accounting_contract_wallet.clone(),
						));
					Some(hyperevm_testnet_rpc)
				} else {
					None
				};

			// Add Base
			let base_rpc = Arc::new(ethereum_rpc::AlloyRpcProvider::new_with_wallet(
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
			let bsc_entry_point = Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
				entry_point_address,
				bsc_rpc,
				aa_contracts_client::GasPriceConfig::bsc(),
				aa_contracts_client::RetryConfig::bsc(),
			));
			entry_point_clients.insert(56, bsc_entry_point);

			// Add BSC Testnet if configured
			if let Some(bsc_testnet_rpc) = bsc_testnet_rpc {
				let bsc_testnet_entry_point =
					Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
						entry_point_address,
						bsc_testnet_rpc,
						aa_contracts_client::GasPriceConfig::bsc(),
						aa_contracts_client::RetryConfig::bsc(),
					));
				entry_point_clients.insert(97, bsc_testnet_entry_point);
			}

			// Add Ethereum Mainnet
			let ethereum_entry_point =
				Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
					entry_point_address,
					ethereum_rpc,
					aa_contracts_client::GasPriceConfig::mainnet(),
					aa_contracts_client::RetryConfig::mainnet(),
				));
			entry_point_clients.insert(1, ethereum_entry_point);

			// Add local development chain
			let local_entry_point =
				Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
					entry_point_address,
					local_rpc,
					aa_contracts_client::GasPriceConfig::default(),
					aa_contracts_client::RetryConfig::default(),
				));
			entry_point_clients.insert(31337, local_entry_point);

			// Add Arbitrum One (Chain ID: 42161)
			let arbitrum_entry_point =
				Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
					entry_point_address,
					arbitrum_rpc,
					aa_contracts_client::GasPriceConfig::l2(),
					aa_contracts_client::RetryConfig::l2(),
				));
			entry_point_clients.insert(42161, arbitrum_entry_point);

			// Add Arbitrum Testnet if configured (Chain ID: 421614)
			if let Some(arbitrum_testnet_rpc) = arbitrum_testnet_rpc {
				let arbitrum_testnet_entry_point =
					Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
						entry_point_address,
						arbitrum_testnet_rpc,
						aa_contracts_client::GasPriceConfig::l2(),
						aa_contracts_client::RetryConfig::l2(),
					));
				entry_point_clients.insert(421614, arbitrum_testnet_entry_point);
			}

			// Add HyperEVM (Chain ID: 999)
			let hyperevm_entry_point =
				Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
					entry_point_address,
					hyperevm_rpc,
					aa_contracts_client::GasPriceConfig::hyperevm(),
					aa_contracts_client::RetryConfig::hyperevm(),
				));
			entry_point_clients.insert(999, hyperevm_entry_point);

			// Add HyperEVM Testnet if configured (Chain ID: 998)
			if let Some(hyperevm_testnet_rpc) = hyperevm_testnet_rpc {
				let hyperevm_testnet_entry_point =
					Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
						entry_point_address,
						hyperevm_testnet_rpc,
						aa_contracts_client::GasPriceConfig::hyperevm(),
						aa_contracts_client::RetryConfig::hyperevm(),
					));
				entry_point_clients.insert(998, hyperevm_testnet_entry_point);
			}

			// Add Base (Chain ID: 8453)
			let base_entry_point =
				Arc::new(aa_contracts_client::EntryPointClient::new_with_config(
					entry_point_address,
					base_rpc,
					aa_contracts_client::GasPriceConfig::l2(),
					aa_contracts_client::RetryConfig::l2(),
				));
			entry_point_clients.insert(8453, base_entry_point);

			let entry_point_clients = Arc::new(entry_point_clients);

			let worker_url =
				url::Url::parse(&config_loader.worker_url).expect("Invalid worker url");

			let shielding_key_store = ShieldingKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/shielding_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);

			let shielding_key = shielding_key_store.read().expect("Could not read shielding key");

			// Create wildmeta API client and timestamp storage
			let wildmeta_api: Arc<Box<dyn wildmeta_api::WildmetaApi>> = Arc::new(Box::new(
				wildmeta_api::WildmetaApiClient::new(config_loader.wildmeta_api_url.clone()),
			));
			let wildmeta_timestamp_storage =
				Arc::new(executor_storage::WildmetaTimestampStorage::new(storage_db.clone()));

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
				storage_db.clone(),
				jwt_rsa_private_key,
				&config_loader,
				signer_client,
				binance_api,
				wildmeta_api,
				wildmeta_timestamp_storage,
				wildmeta_backend_ecdsa_pubkey,
				evm_accounting_ecdsa_signer_key,
				bundler_key_export_authorized_pubkey,
				Arc::new(ethereum_intent_executor),
				Arc::new(solana_intent_executor),
				aes256_key,
				entry_point_clients,
			)
			.await
			.map_err(|e| {
				error!("Could not start server: {:?}", e);
			})?;

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
