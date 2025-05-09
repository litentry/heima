use executor_primitives::AccountId;
use parentchain_rpc_client::{
	metadata::{MetadataProvider, SubxtMetadataProvider},
	RpcClientHeader, SubstrateRpcClient, SubstrateRpcClientFactory, ToPrimitiveType,
};
use parity_scale_codec::Decode;
use std::sync::Arc;
use std::{
	marker::PhantomData,
	sync::atomic::{AtomicU64, Ordering},
};
use subxt_core::config::{DefaultExtrinsicParams, DefaultExtrinsicParamsBuilder};
use subxt_core::tx::payload::Payload;
use subxt_core::utils::{AccountId32, MultiAddress, MultiSignature};
use subxt_core::{tx, Config, Metadata};
use subxt_signer::sr25519::Keypair;

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
	nonce: AtomicU64,
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
	pub fn new(
		metadata_provider: Arc<SubxtMetadataProvider<ChainConfig>>,
		rpc_client_factory: Arc<RpcClientFactory>,
		signer: Keypair,
		nonce: u64,
	) -> Self {
		tracing::log::info!("Initializing TxSigner nonce with {}", nonce);
		let nonce = AtomicU64::new(nonce);
		Self { metadata_provider, rpc_client_factory, signer, nonce, phantom_data: PhantomData }
	}

	pub async fn sign<Call: Payload>(&self, call: Call) -> Vec<u8> {
		let mut client = self.rpc_client_factory.new_client().await.unwrap();

		let runtime_version = client.runtime_version().await.unwrap();
		let genesis_hash = client.get_genesis_hash().await.unwrap();

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
		let nonce = self.nonce.fetch_add(1, Ordering::SeqCst);
		if let Some(details) = call.validation_details() {
			tracing::log::info!(
				"Signing call {}::{} with nonce {}",
				details.pallet_name,
				details.call_name,
				nonce
			);
		} else {
			tracing::log::info!("Signing call with nonce {}", nonce);
		}
		let params = DefaultExtrinsicParamsBuilder::<ChainConfig>::new().nonce(nonce).build();
		let signed_call = tx::create_signed(&call, &state, &self.signer, params).unwrap();

		signed_call.encoded().to_vec()
	}

	fn get_signer_account_id(&self) -> AccountId {
		self.signer.public_key().to_account_id().to_primitive_type()
	}

	pub async fn update_nonce(&self) {
		let mut client =
			self.rpc_client_factory.new_client().await.expect("Failed to create rpc client");
		let account_id = self.get_signer_account_id();
		let nonce = client
			.get_account_nonce(&account_id)
			.await
			.expect("Failed to get account nonce");
		tracing::log::info!("Updating nonce to {}", nonce);
		self.nonce.store(nonce, Ordering::SeqCst);
	}
}
