use crate::metadata::{MetadataProvider, SubxtMetadataProvider};
use crate::rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use executor_core::event_handler::Error::RecoverableError;
use executor_core::key_store::KeyStore;
use log::error;
use parity_scale_codec::Decode;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use subxt_core::config::{DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use subxt_core::tx::payload::Payload;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};
use subxt_core::{tx, Config, Metadata};
use subxt_signer::sr25519::SecretKeyBytes;

pub struct TransactionSigner<
	KeyStoreT,
	RpcClient: SubstrateRpcClient,
	RpcClientFactory: SubstrateRpcClientFactory<RpcClient>,
	ChainConfig,
	MetadataT,
	MetadataProviderT: MetadataProvider<MetadataT>,
> {
	metadata_provider: Arc<MetadataProviderT>,
	rpc_client_factory: Arc<RpcClientFactory>,
	key_store: Arc<KeyStoreT>,
	nonce: RwLock<u64>,
	phantom_data: PhantomData<(RpcClient, ChainConfig, MetadataT)>,
}

impl<
		KeyStoreT: KeyStore<SecretKeyBytes>,
		RpcClient: SubstrateRpcClient,
		RpcClientFactory: SubstrateRpcClientFactory<RpcClient>,
		ChainConfig: Config<
			ExtrinsicParams = DefaultExtrinsicParams<ChainConfig>,
			AccountId = AccountId32,
			Address = MultiAddress<AccountId32, u32>,
			Signature = MultiSignature,
		>,
	>
	TransactionSigner<
		KeyStoreT,
		RpcClient,
		RpcClientFactory,
		ChainConfig,
		Metadata,
		SubxtMetadataProvider<ChainConfig>,
	>
{
	pub fn new(
		metadata_provider: Arc<SubxtMetadataProvider<ChainConfig>>,
		rpc_client_factory: Arc<RpcClientFactory>,
		key_store: Arc<KeyStoreT>,
	) -> Self {
		Self {
			metadata_provider,
			rpc_client_factory,
			key_store,
			//todo: read nonce from chain
			nonce: RwLock::new(0),
			phantom_data: PhantomData,
		}
	}

	pub async fn sign<Call: Payload>(&self, call: Call) -> Vec<u8> {
		let secret_key_bytes = self
			.key_store
			.read()
			.map_err(|e| {
				error!("Could not unseal key: {:?}", e);
			})
			.unwrap();

		let signer = subxt_signer::sr25519::Keypair::from_secret_key(secret_key_bytes)
			.map_err(|e| {
				error!("Could not create secret key: {:?}", e);
			})
			.unwrap();
		let mut client = self.rpc_client_factory.new_client().await.unwrap();
		let runtime_version = client.runtime_version().await.unwrap();

		let genesis_hash = client.get_genesis_hash().await.unwrap();

		let nonce = *self
			.nonce
			.read()
			.map_err(|e| {
				error!("Could not read nonce: {:?}", e);
				RecoverableError
			})
			.unwrap();

		*self
			.nonce
			.write()
			.map_err(|e| {
				error!("Could not write nonce: {:?}", e);
				RecoverableError
			})
			.unwrap() = nonce + 1;

		// we should get latest metadata
		let metadata = self.metadata_provider.get(None).await;

		let state = tx::ClientState::<ChainConfig> {
			metadata: { metadata },
			genesis_hash: ChainConfig::Hash::decode(&mut genesis_hash.as_slice()).unwrap(),
			runtime_version: tx::RuntimeVersion {
				spec_version: runtime_version.spec_version,
				transaction_version: runtime_version.transaction_version,
			},
		};
		let params = DefaultExtrinsicParamsBuilder::<ChainConfig>::new().nonce(nonce).build();
		let signed_call = tx::create_signed(&call, &state, &signer, params).unwrap();

		signed_call.encoded().to_vec()
	}
}
