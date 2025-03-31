use crate::get_signer;
use executor_core::key_store::KeyStore as KeyStoreTrait;
use executor_primitives::AccountId;
use parentchain_rpc_client::{
	metadata::{MetadataProvider, SubxtMetadataProvider},
	RpcClientHeader, SubstrateRpcClient, SubstrateRpcClientFactory, ToPrimitiveType,
};
use parity_scale_codec::Decode;
use std::marker::PhantomData;
use std::sync::Arc;
use subxt_core::config::{DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use subxt_core::tx::payload::Payload;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};
use subxt_core::{tx, Config, Metadata};
use subxt_signer::sr25519::{Keypair, SecretKeyBytes};

pub struct TxSigner<
	RpcClient: SubstrateRpcClient<ChainConfig::Header>,
	RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::Header, RpcClient>,
	ChainConfig: Config,
	MetadataT,
	MetadataProviderT: MetadataProvider<MetadataT>,
> {
	metadata_provider: Arc<MetadataProviderT>,
	rpc_client_factory: Arc<RpcClientFactory>,
	signer: Keypair,
	phantom_data: PhantomData<(RpcClient, ChainConfig, MetadataT)>,
}

impl<
		RpcClient: SubstrateRpcClient<ChainConfig::Header>,
		RpcClientFactory: SubstrateRpcClientFactory<ChainConfig::Header, RpcClient>,
		ChainConfig: Config<
			ExtrinsicParams = DefaultExtrinsicParams<ChainConfig>,
			AccountId = AccountId32,
			Address = MultiAddress<AccountId32, u32>,
			Signature = MultiSignature,
			Header = RpcClientHeader,
		>,
	>
	TxSigner<RpcClient, RpcClientFactory, ChainConfig, Metadata, SubxtMetadataProvider<ChainConfig>>
{
	pub fn new<KeyStore: KeyStoreTrait<SecretKeyBytes>>(
		metadata_provider: Arc<SubxtMetadataProvider<ChainConfig>>,
		rpc_client_factory: Arc<RpcClientFactory>,
		key_store: Arc<KeyStore>,
	) -> Self {
		let signer = get_signer(key_store.clone());
		Self { metadata_provider, rpc_client_factory, signer, phantom_data: PhantomData }
	}

	pub async fn sign<Call: Payload>(&self, call: Call, next_nonce: Option<u64>) -> Vec<u8> {
		let mut client = self.rpc_client_factory.new_client().await.unwrap();
		let runtime_version = client.runtime_version().await.unwrap();

		let genesis_hash = client.get_genesis_hash().await.unwrap();

		let account_id = self.signer.public_key().to_account_id().to_primitive_type();

		let nonce: u64;

		if let Some(n) = next_nonce {
			nonce = n;
		} else {
			nonce = client.get_account_nonce(&account_id).await.unwrap();
		}

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
		let signed_call = tx::create_signed(&call, &state, &self.signer, params).unwrap();

		signed_call.encoded().to_vec()
	}

	pub fn get_signer_account_id(&self) -> AccountId {
		self.signer.public_key().to_account_id().to_primitive_type()
	}
}
