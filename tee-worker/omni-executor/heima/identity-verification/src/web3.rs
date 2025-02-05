use executor_primitives::{utils::hex::hex_encode, Identity, Web3ValidationData};

// This function validates the signature with both the raw message and its prettified format. Any of them is valid.
// The prettified version was introduced to extend the support for utf-8 signatures for browser wallets that
// do not play well with raw bytes signing, like Solana's Phantom and OKX wallets.
//
// The prettified format is the raw message with a prefix "Token: ".
pub fn verify_identity(
	identity: &Identity,
	expected_raw_msg: &[u8],
	validation_data: &Web3ValidationData,
) -> Result<(), ()> {
	let mut expected_prettified_msg = hex_encode(expected_raw_msg);
	expected_prettified_msg.insert_str(0, "Token: ");

	let expected_prettified_msg = expected_prettified_msg.as_bytes();

	let received_message = validation_data.message().as_slice();

	if expected_raw_msg != received_message && expected_prettified_msg != received_message {
		return Err(());
	}

	let signature = validation_data.signature();

	if !signature.verify(expected_raw_msg, identity)
		&& !signature.verify(expected_prettified_msg, identity)
	{
		return Err(());
	}

	Ok(())
}
