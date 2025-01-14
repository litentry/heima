use crate::metadata::{MetadataProvider, SubxtMetadataProvider};
use executor_core::key_store::KeyStore;
use log::error;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::Decode;
use std::marker::PhantomData;
use std::sync::Arc;
use subxt_core::config::{DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use subxt_core::tx::payload::Payload;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};
use subxt_core::{tx, Config, Metadata};
use subxt_signer::sr25519::SecretKeyBytes;

pub struct TransactionSigner<
	KeyStoreT,
	RpcClient: SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header>,
	RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, ChainConfig::Header, RpcClient>,
	ChainConfig: Config,
	MetadataT,
	MetadataProviderT: MetadataProvider<MetadataT>,
> {
	metadata_provider: Arc<MetadataProviderT>,
	rpc_client_factory: Arc<RpcClientFactory>,
	key_store: Arc<KeyStoreT>,
	phantom_data: PhantomData<(RpcClient, ChainConfig, MetadataT)>,
}

impl<
		KeyStoreT: KeyStore<SecretKeyBytes>,
		RpcClient: SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header>,
		RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::AccountId, ChainConfig::Header, RpcClient>,
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
		Self { metadata_provider, rpc_client_factory, key_store, phantom_data: PhantomData }
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

		let account_id = AccountId32::from(signer.public_key());

		let nonce = client
			.get_account_nonce(&account_id)
			.await
			.map_err(|e| error!("Could not read nonce: {:?}", e))
			.unwrap();

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
