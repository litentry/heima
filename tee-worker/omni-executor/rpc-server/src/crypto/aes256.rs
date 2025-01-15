use parity_scale_codec::{Decode, Encode};
use rand::Rng;
use ring::{
	aead::{
		Aad, BoundKey, LessSafeKey, Nonce, NonceSequence, SealingKey, UnboundKey, AES_256_GCM,
		NONCE_LEN,
	},
	error::Unspecified,
};
use std::vec::Vec;

// we use 256-bit AES-GCM as request enc/dec key
pub const AES_KEY_LEN: usize = 32;

pub type Aes256Key = [u8; AES_KEY_LEN];
pub type Aes256KeyNonce = [u8; NONCE_LEN];

// all-in-one struct containing the encrypted ciphertext with other
// metadata that is required for decryption
//
// by default a postfix tag is used => last 16 bytes of ciphertext is MAC tag
#[derive(Debug, Default, Clone, Eq, PartialEq, Encode, Decode)]
pub struct AesOutput {
	pub ciphertext: Vec<u8>,
	pub aad: Vec<u8>,
	pub nonce: Aes256KeyNonce, // IV
}

// Returns the default if any error happens
// We don't propagate the error to upper level as this function is used in too many places,
// it's too verbose to handle them all and pass back to the parentchain as events.
// We rely on the parentchain event consumers to handle them correctly (and they kind of
// have to, because they'll find all fields are 0)
pub fn aes_encrypt_default(key: &Aes256Key, data: &[u8]) -> AesOutput {
	let mut in_out = data.to_vec();

	let mut nonce = RingAeadNonceSequence::new();
	if nonce.advance().is_ok() {
		let aad = b"";
		if let Ok(unbound_key) = UnboundKey::new(&AES_256_GCM, key.as_slice()) {
			let mut sealing_key = SealingKey::new(unbound_key, nonce.clone());
			if sealing_key.seal_in_place_append_tag(Aad::from(aad), &mut in_out).is_ok() {
				return AesOutput {
					ciphertext: in_out.to_vec(),
					aad: aad.to_vec(),
					nonce: nonce.nonce,
				};
			}
		}
	}

	AesOutput::default()
}

// use LessSafeKey::seal_in_place_append_tag to encrypt the data using the given nonce
// don't be scared by the name, it's similar to `SealingKey::seal_in_place_append_tag`,
// except that it accepts an arbitrary nonce.
// It's only used by the one-off verification message calculation.
pub fn aes_encrypt_nonce(key: &Aes256Key, data: &[u8], nonce: Aes256KeyNonce) -> AesOutput {
	let mut in_out = data.to_vec();
	let aad = b"";
	if let Ok(unbound_key) = UnboundKey::new(&AES_256_GCM, key.as_slice()) {
		let less_safe_key = LessSafeKey::new(unbound_key);
		if less_safe_key
			.seal_in_place_append_tag(
				Nonce::assume_unique_for_key(nonce),
				Aad::from(aad),
				&mut in_out,
			)
			.is_ok()
		{
			return AesOutput { ciphertext: in_out.to_vec(), aad: aad.to_vec(), nonce };
		}
	}

	AesOutput::default()
}

pub fn aes_decrypt(key: &Aes256Key, data: &mut AesOutput) -> Option<Vec<u8>> {
	let in_out = data.ciphertext.as_mut();
	if let Ok(unbound_key) = UnboundKey::new(&AES_256_GCM, key.as_slice()) {
		let less_safe_key = LessSafeKey::new(unbound_key);
		return less_safe_key
			.open_in_place(
				Nonce::assume_unique_for_key(data.nonce),
				Aad::from(data.aad.clone()),
				in_out,
			)
			.ok()
			.map(|data| data.to_vec());
	}
	None
}

#[derive(Clone)]
struct RingAeadNonceSequence {
	pub nonce: Aes256KeyNonce,
}

impl RingAeadNonceSequence {
	fn new() -> RingAeadNonceSequence {
		RingAeadNonceSequence { nonce: [0u8; NONCE_LEN] }
	}
}

impl NonceSequence for RingAeadNonceSequence {
	fn advance(&mut self) -> core::result::Result<Nonce, Unspecified> {
		let nonce = Nonce::assume_unique_for_key(self.nonce);
		let nonce_vec = rand::thread_rng().gen::<Aes256KeyNonce>();
		self.nonce.copy_from_slice(&nonce_vec[0..NONCE_LEN]);
		Ok(nonce)
	}
}
