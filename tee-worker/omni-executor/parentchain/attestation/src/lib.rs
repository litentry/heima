use executor_primitives::MrEnclave;
use parentchain_api_interface::{
	runtime_types::core_primitives::teebag::types::DcapProvider,
	teebag::calls::types::register_enclave::{AttestationType, WorkerMode, WorkerType},
};
use parentchain_rpc_client::{
	metadata::SubxtMetadataProvider, CustomConfig, SubstrateRpcClient, SubxtClient,
	SubxtClientFactory,
};
use parentchain_signer::{key_store::SubstrateKeyStore, TransactionSigner};
use std::sync::Arc;
use subxt_core::Metadata;
use subxt_signer::sr25519::Keypair;

type TxSigner = TransactionSigner<
	SubstrateKeyStore,
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

#[allow(unused_assignments, unused_mut, unused_variables)]
pub async fn perform_attestation(
	client_factory: Arc<SubxtClientFactory<CustomConfig>>,
	signer: Keypair,
	transaction_signer: Arc<TxSigner>,
) -> Result<MrEnclave, ()> {
	let mut quote = vec![];
	let mut attestation_type = AttestationType::Dcap(DcapProvider::Intel);
	let mut mrenclave = MrEnclave::default();

	#[cfg(feature = "gramine-quote")]
	{
		use executor_primitives::DcapQuote;
		use log::info;
		use parity_scale_codec::Decode;
		use std::fs;
		use std::fs::File;
		use std::io::Write;
		let mut f = File::create("/dev/attestation/user_report_data").unwrap();
		let content = signer.public_key().0;
		f.write_all(&content).unwrap();

		quote = fs::read("/dev/attestation/quote").unwrap();
		info!("Attestation quote {:?}", quote);

		let dcap_quote: DcapQuote =
			DcapQuote::decode(&mut quote.as_slice()).expect("Failed to decode quote");

		mrenclave = dcap_quote.body.mr_enclave;
		info!("MRENCLAVE {:?}", mrenclave);
	}
	#[cfg(not(feature = "gramine-quote"))]
	{
		attestation_type = AttestationType::Ignore;
	}

	let registration_call = parentchain_api_interface::tx().teebag().register_enclave(
		WorkerType::OmniExecutor,
		WorkerMode::OffChainWorker,
		quote,
		vec![],
		None,
		None,
		attestation_type,
	);

	let mut client = client_factory.new_client_until_connected().await;
	let signed_call = transaction_signer.sign(registration_call).await;
	client.submit_tx(&signed_call).await.map_err(|e| {
		log::error!("Error while submitting tx: {:?}", e);
	})?;

	Ok(mrenclave)
}
