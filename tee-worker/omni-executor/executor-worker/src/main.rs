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

use accounting_contract_client::{
	solana::AccountingContractClient as SolanaAccountingContractClient,
	AccountingContractClient as EthereumAccountingContractClient,
};
use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use binance_api::BinanceApiClient;
use clap::Parser;
use cli::{Cli, Commands, RunArgs};
use config_loader::ConfigLoader;
use cross_chain_intent_executor::{Chain, CrossChainIntentExecutor, RpcEndpointRegistry};
use ethereum_intent_executor::EthereumIntentExecutor;
use ethereum_rpc::client::EthereumRpcClient;
use executor_core::ecdsa_key_store::EcdsaKeyStore;
use executor_core::ed25519_key_store::Ed25519KeyStore;
use executor_core::key_store::KeyStore;
use executor_core::shielding_key_store::ShieldingKeyStore;
use executor_core::wallet_metrics::Wallet;
use executor_core::wallet_metrics::{
	start_wallet_metrics, WalletBalanceFetcher, WalletId, WalletMetrics, WalletNetworkType,
};
use executor_crypto::rsa::{traits::PublicKeyParts, Rsa3072PubKey};
use executor_crypto::{ecdsa, ed25519, PairTrait};
use executor_primitives::AccountId;
use executor_storage::{init_storage, StorageDB};
use intent_asset_lock::precise::PreciseAssetsLock;
use intent_asset_lock::AccountAssetLocks;
use metrics_exporter_prometheus::PrometheusBuilder;
use native_task_handler::Aes256KeyStore;
use parentchain_attestation::perform_attestation;
use parentchain_rpc_client::metadata::SubxtMetadataProvider;
use parentchain_rpc_client::{
	CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClientFactory,
	ToPrimitiveType,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TxSigner};
use pumpx::{pubkey_to_evm_address, pubkey_to_solana_address};
use pumpx::{PumpxApi, PumpxApiClient};
use rpc_server::{start_server as start_rpc_server, AuthTokenKeyStore};
use rust_decimal::Decimal;
use solana::SolanaRpcClient;
use solana_intent_executor::SolanaIntentExecutor;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tokio::runtime::Handle;
use tokio::signal;
use tokio::sync::oneshot;
use tracing::info;
use tracing::log::error;
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
					thread::spawn(move || {
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

			let pumpx_auth_key_store = pumpx::auth_key_store::AuthKeyStore::new(
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

			let storage_db = init_storage(&config_loader.parentchain_url)
				.await
				.expect("Could not initialize storage");

			let client_factory =
				SubxtClientFactory::<CustomConfig>::new(&config_loader.parentchain_url);
			let metadata_provider = Arc::new(SubxtMetadataProvider::new(client_factory.clone()));
			let parentchain_rpc_client_factory = Arc::new(client_factory);

			let substrate_key_store = Arc::new(SubstrateKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/substrate_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			));
			let parentchain_signer = parentchain_signer::get_signer(substrate_key_store.clone());
			let signer_account_id: AccountId =
				parentchain_signer.public_key().to_account_id().to_primitive_type();
			let mut parentchain_rpc_client = parentchain_rpc_client_factory
				.new_client()
				.await
				.expect("Could not create RPC client");
			let signer_account_nonce = parentchain_rpc_client
				.get_account_nonce(&signer_account_id)
				.await
				.expect("Could not get signer account nonce");

			let tx_signer = Arc::new(TxSigner::new(
				metadata_provider,
				parentchain_rpc_client_factory.clone(),
				parentchain_signer.clone(),
				signer_account_nonce,
			));
			let aes256_key_store = Aes256KeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/aes_256_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			);
			let aes256_key = aes256_key_store.read().expect("Could not read aes256 key");

			let pumpx_signer_client: Arc<Box<dyn signer_client::SignerClient>> =
				Arc::new(Box::new(pumpx::signer_client::PumpxSignerClient::new(
					config_loader.pumpx_signer_url.clone(),
					pumpx_signer_pair,
				)));

			let ethereum_intent_executor = EthereumIntentExecutor::new(
				&config_loader.ethereum_url,
				&args.delegation_contract_address,
			)?;
			let solana_intent_executor = SolanaIntentExecutor::new(&config_loader.solana_url)?;

			let mut rpc_endpoint_registry = RpcEndpointRegistry::new();
			rpc_endpoint_registry.insert(Chain::Solana, config_loader.solana_url.to_string());
			rpc_endpoint_registry.insert(Chain::Ethereum(56), config_loader.bsc_url.to_string());

			if let Some(ref bsc_testnet_url) = config_loader.bsc_testnet_url {
				rpc_endpoint_registry.insert(Chain::Ethereum(97), bsc_testnet_url.to_owned());
			}

			let pumpx_api: Arc<Box<dyn PumpxApi>> = Arc::new(Box::new(PumpxApiClient::new(
				config_loader.pumpx_api_base_url.to_string(),
			)));

			let binance_api = Arc::new(BinanceApiClient::new(
				config_loader.binance_api_key.clone(),
				config_loader.binance_api_secret.clone(),
				config_loader.binance_api_base_url.clone(),
			));

			let solana_client: Arc<SolanaRpcClient> =
				Arc::new(SolanaRpcClient::new(&config_loader.solana_url));

			let accounting_contract_signer =
				PrivateKeySigner::from_slice(&evm_accounting_ecdsa_signer_key_pair.seed())
					.expect("Could not create accounting contract signer");
			let accounting_contract_wallet = EthereumWallet::from(accounting_contract_signer);

			let bsc_rpc_provider = ethereum_rpc::AlloyRpcProvider::new_with_wallet(
				&config_loader.bsc_url,
				accounting_contract_wallet.clone(),
			);
			let evm_accounting_contract_client = EthereumAccountingContractClient::new(
				bsc_rpc_provider,
				args.accounting_contract_address.parse().unwrap(),
			);
			let solana_accounting_contract_client = SolanaAccountingContractClient::new(
				solana_accounting_ed25519_signer_key_pair,
				config_loader.solana_url.clone(),
				args.solana_accounting_contract_address.parse().unwrap(),
			);

			let bsc_client: Arc<EthereumRpcClient> =
				Arc::new(EthereumRpcClient::new(&config_loader.bsc_url));

			// wallet monitoring setup start
			let bsc_wallet_balance_fetcher: Arc<Box<dyn WalletBalanceFetcher>> =
				Arc::new(Box::new(ethereum_rpc::AlloyRpcProvider::new(&config_loader.bsc_url)));

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
				binance_api,
				bsc_client,
				solana_client,
				Arc::new(Box::new(evm_accounting_contract_client)),
				Arc::new(Box::new(solana_accounting_contract_client)),
				Decimal::from_str(&args.instant_payout_threshold).unwrap(),
				omni_account_factory_address,
				omni_account_implementation_address,
			)?;

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
			let shielding_pubkey = shielding_key.public_key();
			let shielding_pubkey_vec = serde_json::to_vec(&Rsa3072PubKey {
				n: shielding_pubkey.n().to_bytes_le(),
				e: shielding_pubkey.e().to_bytes_le(),
			})
			.expect("Could not serialize shielding public key");

			let _ = perform_attestation(
				parentchain_rpc_client_factory.clone(),
				parentchain_signer,
				tx_signer.clone(),
				worker_url.as_str(),
				shielding_pubkey_vec,
			)
			.await
			.map_err(|_| {
				error!("Could not perform attestation");
			})?;

			// Create wildmeta API client and timestamp storage
			let wildmeta_api: Arc<Box<dyn wildmeta_api::WildmetaApi>> = Arc::new(Box::new(
				wildmeta_api::WildmetaApiClient::new(config_loader.wildmeta_api_url.clone()),
			));
			let wildmeta_timestamp_storage =
				Arc::new(executor_storage::WildmetaTimestampStorage::new(storage_db.clone()));

			start_rpc_server(
				worker_url.port().expect("Missing worker port"),
				shielding_key,
				pumpx_api,
				storage_db.clone(),
				jwt_rsa_private_key,
				&config_loader,
				pumpx_signer_client,
				wildmeta_api,
				wildmeta_timestamp_storage,
				Arc::new(ethereum_intent_executor),
				Arc::new(solana_intent_executor),
				Arc::new(cross_chain_intent_executor),
				parentchain_rpc_client_factory.clone(),
				aes256_key,
				tx_signer,
				entry_point_clients,
			)
			.await
			.map_err(|e| {
				error!("Could not start server: {:?}", e);
			})?;

			if args.parentchain_sync {
				listen_to_parentchain(*args, storage_db, &config_loader.parentchain_url)
					.await
					.unwrap();
			}

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
		Commands::GenKey(args) => {
			let key_store = Arc::new(SubstrateKeyStore::new(
				Path::new(&args.local_directory_path)
					.join("keystore/substrate_key.bin")
					.into_os_string()
					.into_string()
					.unwrap(),
			));
			let _ = parentchain_signer::get_signer(key_store);
		},
	}

	Ok(())
}

async fn listen_to_parentchain(
	args: RunArgs,
	storage_db: Arc<StorageDB>,
	ws_rpc_endpoint: &str,
) -> Result<JoinHandle<()>, ()> {
	let (_sub_stop_sender, sub_stop_receiver) = oneshot::channel();
	let mut parentchain_listener = parentchain_listener::create_listener(
		"heima",
		Handle::current(),
		ws_rpc_endpoint,
		sub_stop_receiver,
		storage_db,
		&Path::new(&args.local_directory_path)
			.join("log/parentchain_last_log.bin")
			.into_os_string()
			.into_string()
			.unwrap(),
	)
	.await?;

	Ok(thread::Builder::new()
		.name("heima_sync".to_string())
		.spawn(move || parentchain_listener.sync(args.start_block))
		.unwrap())
}
