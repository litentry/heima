use alloy::primitives::{Address, Signature};
use alloy::{consensus::SignableTransaction, network::TxSigner};
use signer_client::{ChainType, SignerClient};
use sp_core::keccak_256;
use std::sync::Arc;
use tracing::error;

/// A remote signer implementation for EVM-compatible chains
/// that delegates signing operations to a remote signing service
#[derive(Clone)]
pub struct RemoteSigner {
	/// The client used to communicate with the remote signing service
	signer_client: Arc<Box<dyn SignerClient>>,
	/// The wallet index used to identify the specific wallet
	wallet_index: u32,
	/// The omni account identifier
	omni_account: [u8; 32],
	/// The Ethereum address associated with this signer
	address: Address,
}

impl RemoteSigner {
	/// Creates a new RemoteSigner instance
	///
	/// # Arguments
	/// * `signer_client` - The client for remote signing service
	/// * `wallet_index` - Index of the wallet to use
	/// * `omni_account` - The omni account identifier
	///
	/// # Returns
	/// * `Result<Self, String>` - The created signer or an error message
	pub async fn new(
		signer_client: Arc<Box<dyn SignerClient>>,
		wallet_index: u32,
		omni_account: [u8; 32],
	) -> Result<Self, String> {
		// Request wallet address from remote service
		let wallet_bytes = signer_client
			.request_wallet(ChainType::Evm, wallet_index, omni_account)
			.await
			.map_err(|_| "Failed to get wallet address".to_string())?;

		let wallet_bytes: [u8; 33] = wallet_bytes.clone().try_into().map_err(|_| {
			let error_msg = format!(
				"Invalid wallet bytes length: expected 33 bytes, got {}",
				wallet_bytes.len()
			);
			error!("{}", error_msg);
			error_msg
		})?;
		let uncompressed_pubkey = libsecp256k1::PublicKey::parse_slice(
			&wallet_bytes,
			Some(libsecp256k1::PublicKeyFormat::Compressed),
		)
		.map_err(|e| {
			let error_msg = format!("Failed to parse public key: {}", e);
			error!("{}", error_msg);
			error_msg
		})?
		.serialize();
		let pubkey: [u8; 20] = keccak_256(&uncompressed_pubkey[1..])[12..]
			.try_into()
			.map_err(|_| "Failed to extract address bytes from hash".to_string())?;

		Ok(Self {
			signer_client,
			wallet_index,
			omni_account,
			address: Address::from_slice(&pubkey),
		})
	}
}

#[async_trait::async_trait]
impl TxSigner<Signature> for RemoteSigner {
	/// Returns the Ethereum address associated with this signer
	fn address(&self) -> Address {
		self.address
	}

	/// Signs a transaction message using the remote signing service
	///
	/// # Arguments
	/// * `tx` - The transaction to sign
	///
	/// # Returns
	/// * `Result<Signature, alloy_signer::Error>` - The signature or an error
	async fn sign_transaction(
		&self,
		tx: &mut dyn SignableTransaction<Signature>,
	) -> alloy::signers::Result<Signature> {
		// Get the message to sign from the transaction
		let message = tx.signature_hash();

		// Request signature from remote service
		let signature_bytes = self
			.signer_client
			.request_signature(
				ChainType::Evm,
				self.wallet_index,
				self.omni_account,
				message.to_vec(),
			)
			.await
			.map_err(|_| alloy::signers::Error::Other("Failed to request signature".into()))?;

		// Validate signature length (65 bytes for Ethereum signature)
		if signature_bytes.len() != 65 {
			return Err(alloy::signers::Error::Other("Invalid signature bytes length".into()));
		}

		let signature = Signature::try_from(signature_bytes.as_slice())?;

		Ok(signature)
	}
}
