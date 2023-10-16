use super::*;
use base58::FromBase58;
use codec::Decode;
use itp_top_pool_author::{error::Result, mocks::GLOBAL_MOCK_AUTHOR_API};
use lc_stf_task_sender::AssertionBuildRequest;
use litentry_primitives::Assertion;
use sp_core::{blake2_256, sr25519, Pair};
use std::{sync::mpsc::Receiver, vec::Vec};

pub const COMMON_SEED: &[u8] =
	b"crouch whisper apple ladder skull blouse ridge oven despair cloth pony";

pub fn init_global_mock_author_api() -> Result<Receiver<Vec<u8>>> {
	let (sender, receiver) = std::sync::mpsc::channel();
	let mut stf_task_storage = GLOBAL_MOCK_AUTHOR_API.lock().unwrap();
	*stf_task_storage = Some(sender);
	Ok(receiver)
}

pub fn construct_assertion_request(assertion: Assertion) -> RequestType {
	let s: String = String::from("751h9re4VmXYTEyFtsVPDm7H8PHgbz9D3guUSd1vKyUf");
	let s = s.from_base58().unwrap();
	let shard: ShardIdentifier = ShardIdentifier::decode(&mut &s[..]).unwrap();

	let seed = blake2_256(COMMON_SEED).to_vec();
	let pair = sr25519::Pair::from_seed_slice(&seed)
		.expect("Failed to create a key pair from the provided seed");
	let public_id = pair.public();

	let request: RequestType = AssertionBuildRequest {
		shard,
		signer: public_id.into(),
		enclave_account: public_id.into(),
		who: public_id.into(),
		assertion,
		identities: vec![],
		top_hash: H256::zero(),
		req_ext_hash: H256::zero(),
	}
	.into();
	request
}
