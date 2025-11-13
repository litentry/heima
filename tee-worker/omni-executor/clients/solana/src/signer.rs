use oe_client_signer::{ChainType, SignerClient};
use solana_sdk::{
	pubkey::Pubkey,
	signature::Signature,
	signer::{Signer, SignerError},
};
use std::sync::Arc;
use tokio::runtime::Handle;
use tracing::error;

pub struct RemoteSigner {
	signer_client: Arc<Box<dyn SignerClient>>,
	wallet_index: u32,
	omni_account: [u8; 32],
	handle: Handle,
}

impl RemoteSigner {
	pub fn new(
		signer_client: Arc<Box<dyn SignerClient>>,
		wallet_index: u32,
		omni_account: [u8; 32],
		handle: Handle,
	) -> Self {
		Self { signer_client, wallet_index, omni_account, handle }
	}
}

impl Signer for RemoteSigner {
	fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
		let wallet = tokio::task::block_in_place(|| {
			self.handle.block_on(async {
				self.signer_client
					.request_wallet(ChainType::Solana, self.wallet_index, self.omni_account)
					.await
					.map_err(|e| {
						error!("Error requesting wallet: {:?}", e);
						SignerError::Custom(format!("Error requesting wallet: {:?}", e))
					})
			})
		})?;
		let wallet_pubkey: [u8; 32] = wallet.try_into().map_err(|e| {
			error!("Error converting wallet to Pubkey: {:?}", e);
			SignerError::Custom(format!("Error converting wallet to Pubkey: {:?}", e))
		})?;

		Ok(Pubkey::new_from_array(wallet_pubkey))
	}

	fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
		let signed_message = tokio::task::block_in_place(|| {
			self.handle.block_on(async {
				self.signer_client
					.request_signature(
						ChainType::Solana,
						self.wallet_index,
						self.omni_account,
						message.to_vec(),
					)
					.await
					.map_err(|e| {
						error!("Error requesting signature: {:?}", e);
						SignerError::Custom(format!("Error requesting signature: {:?}", e))
					})
			})
		})?;
		let signed_message: [u8; 64] = signed_message.try_into().map_err(|e| {
			error!("Error converting signed message to Signature: {:?}", e);
			SignerError::Custom(format!("Error converting signed message to Signature: {:?}", e))
		})?;

		Ok(Signature::from(signed_message))
	}

	fn is_interactive(&self) -> bool {
		false
	}
}
