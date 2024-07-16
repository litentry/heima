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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;
#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use musig2_sgx as musig2;
use std::{format, string::String, sync::Arc};

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(feature = "std")]
use std::sync::RwLock;

#[cfg(feature = "sgx")]
use std::sync::SgxRwLock as RwLock;

use crate::CeremonyEvent::CeremonyEnded;
use codec::{Decode, Encode};
use itp_sgx_crypto::{key_repository::AccessKey, schnorr::Pair as SchnorrPair};
use k256::SecretKey;
pub use k256::{elliptic_curve::sec1::FromEncodedPoint, PublicKey};
use litentry_primitives::RequestAesKey;
use log::*;
use musig2::{
	secp::{Point, Scalar},
	verify_single, BinaryEncoding, CompactSignature, KeyAggContext, LiftedSignature,
	SecNonceSpices,
};
pub use musig2::{PartialSignature, PubNonce};
use std::{
	collections::HashMap,
	fmt::Display,
	time::{SystemTime, UNIX_EPOCH},
	vec,
	vec::Vec,
};

pub type CeremonyId = SignBitcoinPayload;
pub type SignaturePayload = Vec<u8>;
pub type Signers = Vec<SignerId>;
pub type CeremonyRegistry<AK> = HashMap<CeremonyId, (Arc<RwLock<MuSig2Ceremony<AK>>>, u64)>;
pub type CeremonyCommandTmp = HashMap<CeremonyId, (Arc<RwLock<Vec<CeremonyCommand>>>, u64)>;
// enclave public key is used as signer identifier
pub type SignerId = [u8; 32];
pub type SignersWithKeys = Vec<(SignerId, PublicKey)>;

#[derive(Debug, Eq, PartialEq, Encode)]
pub enum CeremonyError {
	CeremonyInitError(CeremonyErrorReason),
	NonceReceivingError(CeremonyErrorReason),
	PartialSignatureReceivingError(CeremonyErrorReason),
}

#[derive(Debug, Eq, PartialEq, Encode)]
pub enum CeremonyErrorReason {
	SignerNotFound,
	ContributionError,
	IncorrectRound,
	RoundFinalizationError,
}

// events come from outside and are consumed by ceremony in tick fn
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CeremonyCommand {
	Init,
	SaveNonce(SignerId, PubNonce),
	SavePartialSignature(SignerId, PartialSignature),
}

impl Display for CeremonyCommand {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			CeremonyCommand::Init => write!(f, "Init"),
			CeremonyCommand::SaveNonce(_, _) => write!(f, "SaveNonce"),
			CeremonyCommand::SavePartialSignature(_, _) => write!(f, "SavePartialSignature"),
		}
	}
}

// commands are created by ceremony and executed by runner
#[derive(Debug, Eq, PartialEq)]
pub enum CeremonyEvent {
	FirstRoundStarted(Signers, CeremonyId, PubNonce),
	SecondRoundStarted(Signers, CeremonyId, PartialSignature),
	CeremonyEnded([u8; 64], RequestAesKey),
	CeremonyError(Signers, CeremonyError, RequestAesKey),
}

impl Display for CeremonyEvent {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			CeremonyEvent::FirstRoundStarted(_, _, _) => write!(f, "FirstRoundStarted"),
			CeremonyEvent::SecondRoundStarted(_, _, _) => write!(f, "SecondRoundStarted"),
			CeremonyEvent::CeremonyEnded(_, _) => write!(f, "CeremonyEnded"),
			CeremonyEvent::CeremonyError(_, _, _) => write!(f, "CeremonyError"),
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SignBitcoinPayload {
	Derived(SignaturePayload),
	TaprootUnspendable(SignaturePayload),
	TaprootSpendable(SignaturePayload, [u8; 32]),
	WithTweaks(SignaturePayload, Vec<([u8; 32], bool)>),
}

pub fn generate_aggregated_public_key(mut public_keys: Vec<PublicKey>) -> PublicKey {
	public_keys.sort();
	KeyAggContext::new(public_keys).unwrap().aggregated_pubkey()
}

pub struct MuSig2CeremonyData<AK: AccessKey<KeyType = SchnorrPair>> {
	payload: SignBitcoinPayload,
	// P-713: move to layer above, ceremony should be communication agnostic
	aes_key: RequestAesKey,
	me: SignerId,
	signers: SignersWithKeys,
	signing_key_access: Arc<AK>,
	agg_key: PublicKey,
}

pub struct MuSig2CeremonyState {
	first_round: Option<musig2::FirstRound>,
	second_round: Option<musig2::SecondRound<SignaturePayload>>,
}

pub struct MuSig2Ceremony<AK: AccessKey<KeyType = SchnorrPair>> {
	ceremony_data: MuSig2CeremonyData<AK>,
	ceremony_state: MuSig2CeremonyState,
}

impl<AK: AccessKey<KeyType = SchnorrPair>> MuSig2Ceremony<AK> {
	// Creates new ceremony
	pub fn new(
		me: SignerId,
		aes_key: RequestAesKey,
		mut signers: SignersWithKeys,
		payload: SignBitcoinPayload,
		signing_key_access: Arc<AK>,
	) -> Result<Self, String> {
		info!("Creating new ceremony {:?}", payload);
		if signers.len() < 3 {
			return Err(format!("Not enough signers, minimum: {:?}, actual {:?}", 3, signers.len()))
		}

		signers.sort_by_key(|k| k.1);
		// we are always the first key in the vector
		let my_index = signers.iter().position(|r| r.0 == me).ok_or("Could not determine index")?;
		let all_keys = signers.iter().map(|p| p.1).collect::<Vec<PublicKey>>();
		let key_context = match &payload {
			SignBitcoinPayload::TaprootSpendable(_, root_hash) =>
				KeyAggContext::new(all_keys.iter().map(|p| Point::from(*p)))
					.map_err(|e| format!("Key context creation error: {:?}", e))?
					.with_taproot_tweak(root_hash)
					.map_err(|e| format!("Key context creation error: {:?}", e))?,
			SignBitcoinPayload::TaprootUnspendable(_) =>
				KeyAggContext::new(all_keys.iter().map(|p| Point::from(*p)))
					.map_err(|e| format!("Key context creation error: {:?}", e))?
					.with_unspendable_taproot_tweak()
					.map_err(|e| format!("Key context creation error: {:?}", e))?,
			SignBitcoinPayload::Derived(_) =>
				KeyAggContext::new(all_keys.iter().map(|p| Point::from(*p)))
					.map_err(|e| format!("Key context creation error: {:?}", e))?,
			SignBitcoinPayload::WithTweaks(_, tweaks) => {
				let mut prepared_tweaks = vec![];
				for (tweak_bytes, is_x_only) in tweaks.iter() {
					let scalar: Scalar = tweak_bytes.try_into().map_err(|e| {
						format!("Key context creation error, could not parse scalar: {:?}", e)
					})?;
					prepared_tweaks.push((scalar, *is_x_only));
				}
				KeyAggContext::new(all_keys.iter().map(|p| Point::from(*p)))
					.map_err(|e| format!("Key context creation error: {:?}", e))?
					.with_tweaks(prepared_tweaks)
					.map_err(|e| format!("Key context creation error: {:?}", e))?
			},
		};

		info!(
			"Ceremony aggregated public key: {:?}",
			key_context.aggregated_pubkey::<PublicKey>().to_sec1_bytes().to_vec()
		);
		let agg_key = key_context.aggregated_pubkey::<PublicKey>();
		let nonce_seed = random_seed();
		let first_round =
			musig2::FirstRound::new(key_context, nonce_seed, my_index, SecNonceSpices::new())
				.map_err(|e| format!("First round creation error: {:?}", e))?;

		Ok(Self {
			ceremony_data: MuSig2CeremonyData {
				payload,
				aes_key,
				me,
				signers,
				signing_key_access,
				agg_key,
			},
			ceremony_state: MuSig2CeremonyState {
				first_round: Some(first_round),
				second_round: None,
			},
		})
	}

	pub fn process_command(&mut self, command: CeremonyCommand) -> Option<CeremonyEvent> {
		let event_ret = match command {
			CeremonyCommand::Init => self
				.ceremony_state
				.first_round
				.as_ref()
				.map(|f| {
					Some(CeremonyEvent::FirstRoundStarted(
						self.ceremony_data
							.signers
							.iter()
							.filter(|e| e.0 != self.ceremony_data.me)
							.map(|s| s.0)
							.collect(),
						self.ceremony_data.payload.clone(),
						f.our_public_nonce(),
					))
				})
				.ok_or(CeremonyError::CeremonyInitError(CeremonyErrorReason::IncorrectRound)),
			CeremonyCommand::SaveNonce(signer, nonce) => self.receive_nonce(signer, nonce),
			CeremonyCommand::SavePartialSignature(signer, partial_signature) =>
				self.receive_partial_sign(signer, partial_signature),
		};

		match event_ret {
			Ok(event) => event,
			Err(e) => Some(CeremonyEvent::CeremonyError(
				self.ceremony_data
					.signers
					.iter()
					.filter(|e| e.0 != self.ceremony_data.me)
					.map(|s| s.0)
					.collect(),
				e,
				self.ceremony_data.aes_key,
			)),
		}
	}

	// Saves signer's nonce
	fn receive_nonce(
		&mut self,
		signer: SignerId,
		nonce: PubNonce,
	) -> Result<Option<CeremonyEvent>, CeremonyError> {
		info!("Saving nonce from signer: {:?}", signer);
		let peer_index = self
			.ceremony_data
			.signers
			.iter()
			.position(|p| p.0 == signer)
			.ok_or(CeremonyError::NonceReceivingError(CeremonyErrorReason::SignerNotFound))?;

		if let Some(ref mut r) = self.ceremony_state.first_round {
			r.receive_nonce(peer_index, nonce).map_err(|e| {
				error!("Nonce receiving error: {:?}", e);
				CeremonyError::NonceReceivingError(CeremonyErrorReason::ContributionError)
			})?;
			if r.is_complete() {
				let secret_key = SecretKey::from_slice(
					&self
						.ceremony_data
						.signing_key_access
						.retrieve_key()
						.map_err(|e| {
							error!("Nonce receiving error: {:?}", e);
							CeremonyError::NonceReceivingError(
								CeremonyErrorReason::RoundFinalizationError,
							)
						})?
						.private_bytes(),
				)
				.map_err(|e| {
					error!("Nonce receiving error: {:?}", e);
					CeremonyError::NonceReceivingError(CeremonyErrorReason::RoundFinalizationError)
				})?;
				self.start_second_round(secret_key).map(|e| Some(e))
			} else {
				Ok(None)
			}
		} else {
			Err(CeremonyError::NonceReceivingError(CeremonyErrorReason::IncorrectRound))
		}
	}

	// Starts the second round
	fn start_second_round(
		&mut self,
		private_key: SecretKey,
	) -> Result<CeremonyEvent, CeremonyError> {
		let first_round = self
			.ceremony_state
			.first_round
			.take()
			.ok_or(CeremonyError::NonceReceivingError(CeremonyErrorReason::IncorrectRound))?;

		let message = match &self.ceremony_data.payload {
			SignBitcoinPayload::TaprootSpendable(message, _) => message.clone(),
			SignBitcoinPayload::Derived(message) => message.clone(),
			SignBitcoinPayload::TaprootUnspendable(message) => message.clone(),
			SignBitcoinPayload::WithTweaks(message, _) => message.clone(),
		};
		let second_round = first_round.finalize(private_key, message).map_err(|e| {
			error!("Could not start second round: {:?}", e);
			CeremonyError::NonceReceivingError(CeremonyErrorReason::RoundFinalizationError)
		})?;

		let partial_signature: PartialSignature = second_round.our_signature();

		self.ceremony_state.second_round = Some(second_round);

		Ok(CeremonyEvent::SecondRoundStarted(
			self.ceremony_data
				.signers
				.iter()
				.filter(|e| e.0 != self.ceremony_data.me)
				.map(|s| s.0)
				.collect(),
			self.get_id_ref().clone(),
			partial_signature,
		))
	}

	// Saves signer's partial signature
	fn receive_partial_sign(
		&mut self,
		signer: SignerId,
		partial_signature: impl Into<PartialSignature>,
	) -> Result<Option<CeremonyEvent>, CeremonyError> {
		info!("Saving partial signature from signer: {:?}", signer);
		let peer_index = self.ceremony_data.signers.iter().position(|p| p.0 == signer).ok_or(
			CeremonyError::PartialSignatureReceivingError(CeremonyErrorReason::SignerNotFound),
		)?;

		if let Some(ref mut r) = self.ceremony_state.second_round {
			r.receive_signature(peer_index, partial_signature).map_err(|e| {
				error!("Signature receiving error: {:?}", e);
				CeremonyError::PartialSignatureReceivingError(
					CeremonyErrorReason::ContributionError,
				)
			})?;
			if r.is_complete() {
				if let Some(r) = self.ceremony_state.second_round.take() {
					let signature: CompactSignature = r
						.finalize::<LiftedSignature>()
						.map_err(|e| {
							error!("Could not finish second round: {:?}", e);
							CeremonyError::PartialSignatureReceivingError(
								CeremonyErrorReason::RoundFinalizationError,
							)
						})?
						.compact();

					info!("Ceremony {:?} `has ended`", self.get_id_ref());
					info!("Aggregated public key {:?}", self.ceremony_data.agg_key.to_sec1_bytes());
					info!("Signature {:?}", signature.to_bytes());

					let message = match &self.ceremony_data.payload {
						SignBitcoinPayload::Derived(p) => p,
						SignBitcoinPayload::TaprootUnspendable(p) => p,
						SignBitcoinPayload::TaprootSpendable(p, _) => p,
						SignBitcoinPayload::WithTweaks(p, _) => p,
					};

					info!("Message {:?}", message);

					info!("Verification result: ");
					match verify_single(self.ceremony_data.agg_key, signature, message) {
						Ok(_) => info!("OK!"),
						Err(_) => info!("NOK!"),
					};
					Ok(Some(CeremonyEnded(signature.to_bytes(), self.ceremony_data.aes_key)))
				} else {
					Err(CeremonyError::PartialSignatureReceivingError(
						CeremonyErrorReason::IncorrectRound,
					))
				}
			} else {
				Ok(None)
			}
		} else {
			Err(CeremonyError::PartialSignatureReceivingError(CeremonyErrorReason::IncorrectRound))
		}
	}

	pub fn get_id_ref(&self) -> &CeremonyId {
		&self.ceremony_data.payload
	}

	pub fn get_aes_key(&self) -> &RequestAesKey {
		&self.ceremony_data.aes_key
	}

	pub fn is_first_round(&self) -> bool {
		self.ceremony_state.first_round.is_some()
	}
}

pub fn get_current_timestamp() -> u64 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[cfg(feature = "std")]
fn random_seed() -> [u8; 32] {
	use rand::{thread_rng, RngCore};

	let mut seed = [0u8; 32];
	let mut rand = thread_rng();
	rand.fill_bytes(&mut seed);
	seed
}

#[cfg(feature = "sgx")]
fn random_seed() -> [u8; 32] {
	use sgx_rand::{Rng, StdRng};
	let mut seed = [0u8; 32];
	let mut rand = StdRng::new().unwrap();
	rand.fill_bytes(&mut seed);
	seed
}

#[cfg(test)]
pub mod test {
	use crate::{
		CeremonyCommand, CeremonyError, CeremonyErrorReason, CeremonyEvent, MuSig2Ceremony,
		SignBitcoinPayload, SignerId, SignersWithKeys,
	};
	use alloc::sync::Arc;
	use itp_sgx_crypto::{key_repository::AccessKey, schnorr::Pair as SchnorrPair};
	use k256::{elliptic_curve::PublicKey, schnorr::SigningKey};
	use litentry_primitives::RequestAesKey;
	use musig2::SecNonce;

	pub const MY_SIGNER_ID: SignerId = [0u8; 32];

	fn my_priv_key() -> SigningKey {
		SigningKey::from_bytes(&[
			252, 240, 35, 85, 243, 83, 129, 54, 7, 155, 24, 114, 254, 0, 134, 251, 207, 83, 177, 9,
			92, 118, 222, 5, 202, 239, 188, 215, 132, 113, 127, 94,
		])
		.unwrap()
	}

	fn signer1_priv_key() -> SigningKey {
		SigningKey::from_bytes(&[
			42, 82, 57, 169, 208, 130, 125, 141, 62, 185, 167, 41, 142, 217, 252, 135, 158, 128,
			44, 129, 222, 71, 55, 86, 230, 183, 54, 111, 152, 83, 85, 155,
		])
		.unwrap()
	}

	pub const SIGNER_1_ID: SignerId = [1u8; 32];
	pub const SIGNER_1_SEC_NONCE: [u8; 64] = [
		57, 232, 181, 133, 43, 97, 251, 79, 229, 110, 26, 121, 197, 2, 249, 237, 222, 207, 129,
		232, 8, 227, 120, 202, 127, 61, 209, 41, 92, 54, 8, 91, 80, 31, 9, 126, 14, 137, 126, 143,
		98, 223, 254, 134, 9, 190, 5, 157, 133, 254, 18, 119, 117, 25, 65, 179, 35, 130, 156, 109,
		233, 51, 18, 32,
	];

	pub const SIGNER_2_ID: SignerId = [2u8; 32];

	fn signer2_priv_key() -> SigningKey {
		SigningKey::from_bytes(&[
			117, 130, 176, 36, 185, 53, 187, 61, 123, 86, 24, 38, 174, 143, 129, 73, 245, 210, 127,
			148, 115, 136, 32, 98, 62, 47, 26, 196, 57, 211, 171, 185,
		])
		.unwrap()
	}

	pub const SIGNER_2_SEC_NONCE: [u8; 64] = [
		78, 229, 109, 189, 246, 169, 247, 85, 184, 199, 144, 135, 45, 60, 71, 109, 214, 121, 165,
		206, 185, 246, 120, 52, 228, 49, 155, 9, 160, 129, 171, 252, 69, 160, 122, 66, 151, 147,
		141, 118, 226, 189, 100, 94, 74, 163, 158, 245, 111, 99, 108, 202, 224, 110, 71, 106, 178,
		255, 89, 34, 16, 10, 195, 107,
	];

	fn signers_with_keys() -> SignersWithKeys {
		vec![
			(MY_SIGNER_ID, PublicKey::from(my_priv_key().verifying_key())),
			(SIGNER_1_ID, PublicKey::from(signer1_priv_key().verifying_key())),
			(SIGNER_2_ID, PublicKey::from(signer2_priv_key().verifying_key())),
		]
	}

	fn save_signer1_nonce_cmd() -> CeremonyCommand {
		CeremonyCommand::SaveNonce(
			SIGNER_1_ID,
			SecNonce::from_bytes(&SIGNER_1_SEC_NONCE).unwrap().public_nonce(),
		)
	}

	fn save_signer2_nonce_cmd() -> CeremonyCommand {
		CeremonyCommand::SaveNonce(
			SIGNER_2_ID,
			SecNonce::from_bytes(&SIGNER_2_SEC_NONCE).unwrap().public_nonce(),
		)
	}

	fn unknown_signer_nonce_cmd() -> CeremonyCommand {
		CeremonyCommand::SaveNonce(
			[10u8; 32],
			SecNonce::from_bytes(&SIGNER_2_SEC_NONCE).unwrap().public_nonce(),
		)
	}

	pub const SAMPLE_REQUEST_AES_KEY: RequestAesKey = [0u8; 32];
	pub const SAMPLE_SIGNATURE_PAYLOAD: [u8; 32] = [0u8; 32];

	struct MockedSigningKeyAccess {
		signing_key: SigningKey,
	}

	impl AccessKey for MockedSigningKeyAccess {
		type KeyType = SchnorrPair;

		fn retrieve_key(&self) -> itp_sgx_crypto::Result<Self::KeyType> {
			Ok(SchnorrPair::new(self.signing_key.clone()))
		}
	}

	#[test]
	fn it_should_create_ceremony_in_firstround() {
		// given
		let signing_key_access = MockedSigningKeyAccess { signing_key: my_priv_key() };

		// when
		let result = MuSig2Ceremony::new(
			MY_SIGNER_ID,
			SAMPLE_REQUEST_AES_KEY.clone(),
			signers_with_keys(),
			SignBitcoinPayload::Derived(SAMPLE_SIGNATURE_PAYLOAD.to_vec()),
			Arc::new(signing_key_access),
		);

		// then
		assert!(result.is_ok());
		assert!(result.unwrap().is_first_round())
	}

	#[test]
	fn it_should_prevent_from_creating_ceremony_without_sufficient_signers() {
		// given
		let signing_key_access = MockedSigningKeyAccess { signing_key: my_priv_key() };

		// when
		let result = MuSig2Ceremony::new(
			MY_SIGNER_ID,
			SAMPLE_REQUEST_AES_KEY.clone(),
			signers_with_keys()[0..1].to_vec(),
			SignBitcoinPayload::Derived(SAMPLE_SIGNATURE_PAYLOAD.to_vec()),
			Arc::new(signing_key_access),
		);

		// then
		assert!(result.is_err());
	}

	#[test]
	fn it_should_produce_error_due_to_nonce_from_unknown_signer() {
		// given
		let signing_key_access = MockedSigningKeyAccess { signing_key: my_priv_key() };
		let mut ceremony = MuSig2Ceremony::new(
			MY_SIGNER_ID,
			SAMPLE_REQUEST_AES_KEY.clone(),
			signers_with_keys(),
			SignBitcoinPayload::Derived(SAMPLE_SIGNATURE_PAYLOAD.to_vec()),
			Arc::new(signing_key_access),
		)
		.unwrap();

		let event = ceremony.process_command(CeremonyCommand::Init);
		assert!(ceremony.ceremony_state.first_round.is_some());
		assert!(ceremony.ceremony_state.second_round.is_none());
		assert!(event.is_some());
		assert_eq!(
			event.unwrap(),
			CeremonyEvent::FirstRoundStarted(
				vec![SIGNER_1_ID, SIGNER_2_ID],
				SignBitcoinPayload::Derived(SAMPLE_SIGNATURE_PAYLOAD.to_vec()),
				ceremony.ceremony_state.first_round.as_ref().unwrap().our_public_nonce(),
			)
		);

		let event = ceremony.process_command(unknown_signer_nonce_cmd());
		assert!(ceremony.ceremony_state.first_round.is_some());
		assert!(ceremony.ceremony_state.second_round.is_none());
		assert!(event.is_some());
		assert!(matches!(
			event.unwrap(),
			CeremonyEvent::CeremonyError(
				_,
				CeremonyError::NonceReceivingError(CeremonyErrorReason::SignerNotFound),
				_,
			)
		));
	}

	#[test]
	fn it_should_complete_successfully() {
		// given
		let signing_key_access = MockedSigningKeyAccess { signing_key: my_priv_key() };
		let mut ceremony = MuSig2Ceremony::new(
			MY_SIGNER_ID,
			SAMPLE_REQUEST_AES_KEY.clone(),
			signers_with_keys(),
			SignBitcoinPayload::Derived(SAMPLE_SIGNATURE_PAYLOAD.to_vec()),
			Arc::new(signing_key_access),
		)
		.unwrap();

		let event = ceremony.process_command(CeremonyCommand::Init);
		assert!(ceremony.ceremony_state.first_round.is_some());
		assert!(ceremony.ceremony_state.second_round.is_none());
		assert!(event.is_some());
		assert_eq!(
			event.unwrap(),
			CeremonyEvent::FirstRoundStarted(
				vec![SIGNER_1_ID, SIGNER_2_ID],
				SignBitcoinPayload::Derived(SAMPLE_SIGNATURE_PAYLOAD.to_vec()),
				ceremony.ceremony_state.first_round.as_ref().unwrap().our_public_nonce(),
			)
		);

		let event = ceremony.process_command(save_signer1_nonce_cmd());
		assert!(ceremony.ceremony_state.first_round.is_some());
		assert!(ceremony.ceremony_state.second_round.is_none());
		assert!(event.is_none());

		let event = ceremony.process_command(save_signer2_nonce_cmd());
		assert!(ceremony.ceremony_state.first_round.is_none());
		assert!(ceremony.ceremony_state.second_round.is_some());
		assert!(event.is_some());
		assert_eq!(
			event.unwrap(),
			CeremonyEvent::SecondRoundStarted(
				vec![SIGNER_1_ID, SIGNER_2_ID],
				SignBitcoinPayload::Derived(SAMPLE_SIGNATURE_PAYLOAD.to_vec()),
				ceremony.ceremony_state.second_round.as_ref().unwrap().our_signature(),
			)
		);
	}
}
