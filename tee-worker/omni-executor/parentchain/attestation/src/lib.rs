use executor_primitives::MrEnclave;
use log::{debug, info};
use parentchain_api_interface::{
	runtime_types::core_primitives::teebag::types::DcapProvider,
	teebag::calls::types::register_enclave::{AttestationType, WorkerMode, WorkerType},
};
use parentchain_rpc_client::{
	metadata::SubxtMetadataProvider, CustomConfig, SubstrateRpcClient, SubxtClient,
	SubxtClientFactory,
};
use parentchain_signer::TxSigner;
use std::sync::Arc;
use subxt_core::Metadata;
use subxt_signer::sr25519::Keypair;

type ParentchainTxSigner = TxSigner<
	SubxtClient<CustomConfig>,
	SubxtClientFactory<CustomConfig>,
	CustomConfig,
	Metadata,
	SubxtMetadataProvider<CustomConfig>,
>;

pub async fn generate_attestation_data(signer: Keypair) -> Result<(Vec<u8>, MrEnclave), ()> {
	#[cfg(feature = "gramine-quote")]
	let (mut quote, mut mrenclave) = (vec![], MrEnclave::default());
	#[cfg(not(feature = "gramine-quote"))]
	let (quote, mrenclave) = (vec![], MrEnclave::default());

	log::debug!("Using signer for attestation: {:?}", signer.public_key().0);

	#[cfg(feature = "gramine-quote")]
	{
		use executor_primitives::DcapQuote;
		use parity_scale_codec::Decode;
		use std::fs;
		use std::fs::File;
		use std::io::Write;

		let mut f = File::create("/dev/attestation/user_report_data").unwrap();
		let content = signer.public_key().0;
		f.write_all(&content).unwrap();

		let quote = fs::read("/dev/attestation/quote").unwrap();

		let dcap_quote: DcapQuote =
			DcapQuote::decode(&mut quote.as_slice()).expect("Failed to decode quote");
		debug!("Attestation dcap_quote {:?}", dcap_quote);

		mrenclave = dcap_quote.body.mr_enclave;
	}
	info!("MRENCLAVE in hex {:?}", hex::encode(mrenclave));

	Ok((quote, mrenclave))
}

#[allow(unused_assignments, unused_mut, unused_variables)]
pub async fn perform_attestation(
	client_factory: Arc<SubxtClientFactory<CustomConfig>>,
	signer: Keypair,
	transaction_signer: Arc<ParentchainTxSigner>,
	worker_url: &str,
	shielding_pubkey: Vec<u8>,
) -> Result<MrEnclave, ()> {
	let mut attestation_type = AttestationType::Dcap(DcapProvider::Intel);
	let (quote, mrenclave) = generate_attestation_data(signer.clone()).await?;

	#[cfg(not(feature = "gramine-quote"))]
	{
		attestation_type = AttestationType::Ignore;
	}

	let registration_call = parentchain_api_interface::tx().teebag().register_enclave(
		WorkerType::OmniExecutor,
		WorkerMode::OffChainWorker,
		quote,
		worker_url.as_bytes().to_vec(),
		Some(shielding_pubkey),
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
