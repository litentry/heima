use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::types::RpcResultExt;
use crate::utils::validation::parse_rpc_params;
use ethers::abi::{encode, Token};
use ethers::types::{Address, U256};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use num_bigint::BigUint;
use oe_core::intent::executor::IntentExecutor;
use oe_crypto::confidential::decrypt_amount;
use oe_crypto::privacy_pool::{
	generate_nullifier, generate_pool_commitment, generate_withdrawal_proof,
};
use oe_storage::confidential_invoice::InvoiceStatus;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, error, info};

/// Keccak256("withdraw(uint256,uint256,address,bytes,uint256[3])")[0..4]
const WITHDRAW_SELECTOR: [u8; 4] = [0x9f, 0xad, 0xd0, 0x25];

#[derive(Debug, Deserialize)]
pub struct WithdrawFromPoolParams {
	pub invoice_id: String,
	pub seller_address: String,
	pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WithdrawFromPoolResponse {
	pub calldata: String,
	pub pool_address: String,
	pub amount_raw: String,
	pub nullifier_hex: String,
}

pub fn register_withdraw_from_pool<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_withdrawFromPool", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<WithdrawFromPoolParams>(params)?;

			debug!(
				"Received omni_withdrawFromPool, invoice_id: {}, seller: {}, chain_id: {}",
				params.invoice_id, params.seller_address, params.chain_id
			);

			let invoice = ctx
				.confidential_invoice_storage
				.get_by_id(&params.invoice_id)
				.map_err_internal("Failed to get invoice")?
				.ok_or_else(|| {
					error!("Invoice not found: {}", params.invoice_id);
					DetailedError::invalid_params("invoice_id", "Invoice not found")
				})?;

			if invoice.status != InvoiceStatus::Pending {
				return Err(DetailedError::invalid_params(
					"invoice_id",
					"Invoice is not in pending status",
				)
				.to_rpc_error());
			}

			let seller_lower = params.seller_address.to_lowercase();
			let recipient = invoice
				.recipients
				.iter()
				.find(|r| r.address.to_lowercase() == seller_lower)
				.ok_or_else(|| {
					DetailedError::invalid_params(
						"seller_address",
						"Address is not a recipient of this invoice",
					)
					.to_rpc_error()
				})?;

			let pool_secret = recipient.pool_secret.ok_or_else(|| {
				DetailedError::invalid_params(
					"invoice_id",
					"No pool deposit found for this recipient — call omni_payInvoice first",
				)
				.to_rpc_error()
			})?;

			let amount = decrypt_amount(&recipient.encrypted_amount, &ctx.aes256_key)
				.map_err_internal("Failed to decrypt recipient amount")?;

			// If leaf_index was stored after deposit, use it; otherwise query on-chain events.
			let leaf_index = if let Some(idx) = recipient.leaf_index {
				idx
			} else {
				let commitment = generate_pool_commitment(&pool_secret, amount)
					.map_err_internal("Failed to compute commitment for leaf lookup")?;
				query_leaf_index_for_commitment(
					&ctx.eth_rpc_url,
					&ctx.privacy_pool_address,
					&commitment,
				)
				.await
				.map_err_internal("Failed to query leaf index for commitment")?
			};

			let nullifier = generate_nullifier(&pool_secret, leaf_index)
				.map_err_internal("Failed to compute nullifier")?;

			let recipient_addr = Address::from_str(&params.seller_address).map_err(|_| {
				DetailedError::invalid_params("seller_address", "Invalid Ethereum address")
					.to_rpc_error()
			})?;

			info!(
				"Querying Merkle state and generating ZK proof for invoice: {}, leaf: {}",
				params.invoice_id, leaf_index
			);

			let (filled_subtrees, _next_index, root) =
				query_merkle_state(&ctx.eth_rpc_url, &ctx.privacy_pool_address)
					.await
					.map_err_internal("Failed to query on-chain Merkle state")?;

			let (path_elements, _path_indices) =
				reconstruct_merkle_path(leaf_index, &filled_subtrees);

			let proof = tokio::task::spawn_blocking({
				let secret = pool_secret;
				let wasm = ctx.circuit_wasm_path.clone();
				let zkey = ctx.circuit_zkey_path.clone();
				move || {
					generate_withdrawal_proof(
						&secret,
						leaf_index,
						amount,
						&path_elements,
						&root,
						&wasm,
						&zkey,
					)
				}
			})
			.await
			.map_err(|e| {
				DetailedError::internal_error(&format!("proof task panic: {e}")).to_rpc_error()
			})?
			.map_err_internal("Failed to generate ZK proof")?;

			let calldata = build_withdraw_calldata(
				&proof.pub_signals[1], // nullifier
				&proof.pub_signals[0], // root
				&proof.pub_signals[2], // commitment
				amount,
				recipient_addr,
				&proof.proof_bytes,
			);

			info!(
				"Issued ZK withdrawal for invoice: {}, recipient: {}, nullifier: 0x{}",
				params.invoice_id,
				params.seller_address,
				hex::encode(nullifier)
			);

			Ok::<WithdrawFromPoolResponse, ErrorObjectOwned>(WithdrawFromPoolResponse {
				calldata,
				pool_address: ctx.privacy_pool_address.clone(),
				amount_raw: amount.to_string(),
				nullifier_hex: hex::encode(nullifier),
			})
		})
		.expect("Failed to register omni_withdrawFromPool");
}

// ─── On-chain Merkle state query ──────────────────────────────────────────────

async fn query_merkle_state(
	rpc_url: &str,
	pool_address: &str,
) -> Result<([[u8; 32]; 20], u32, [u8; 32]), String> {
	use serde_json::json;

	let client = reqwest::Client::new();

	let call = |data: String| {
		let body = json!({
			"jsonrpc": "2.0",
			"id": 1,
			"method": "eth_call",
			"params": [{"to": pool_address, "data": data}, "latest"]
		});
		let c = client.clone();
		let url = rpc_url.to_string();
		async move {
			let resp =
				c.post(&url).json(&body).send().await.map_err(|e| format!("RPC request: {e}"))?;
			let json: serde_json::Value =
				resp.json().await.map_err(|e| format!("RPC parse: {e}"))?;
			let hex = json["result"]
				.as_str()
				.ok_or_else(|| format!("No result: {:?}", json))?
				.strip_prefix("0x")
				.unwrap_or("")
				.to_string();
			Ok::<String, String>(hex)
		}
	};

	// filledSubtrees(uint256) — selector 0xf178e47c
	let mut filled_subtrees = [[0u8; 32]; 20];
	for i in 0..20usize {
		let data = format!("0xf178e47c{:064x}", i);
		let hex = call(data).await?;
		let bytes = hex::decode(&hex).map_err(|e| format!("decode subtree {i}: {e}"))?;
		if bytes.len() >= 32 {
			filled_subtrees[i].copy_from_slice(&bytes[bytes.len() - 32..]);
		}
	}

	// nextIndex() — selector 0xfc7e9c6f
	let next_hex = call("0xfc7e9c6f".to_string()).await?;
	let next_bytes = hex::decode(&next_hex).map_err(|e| format!("decode nextIndex: {e}"))?;
	let next_index = if next_bytes.len() >= 4 {
		u32::from_be_bytes(next_bytes[next_bytes.len() - 4..].try_into().unwrap())
	} else {
		0
	};

	// root() — selector 0xebf0c717
	let root_hex = call("0xebf0c717".to_string()).await?;
	let root_bytes = hex::decode(&root_hex).map_err(|e| format!("decode root: {e}"))?;
	let mut root = [0u8; 32];
	if root_bytes.len() >= 32 {
		root.copy_from_slice(&root_bytes[root_bytes.len() - 32..]);
	}

	Ok((filled_subtrees, next_index, root))
}

/// Query on-chain `Deposit` events to find the `leafIndex` for a given commitment.
///
/// The `Deposit(uint256 indexed commitment, uint32 indexed leafIndex, uint256 amount)` event
/// is emitted on every `deposit()` call. We filter by the commitment topic.
async fn query_leaf_index_for_commitment(
	rpc_url: &str,
	pool_address: &str,
	commitment: &[u8; 32],
) -> Result<u32, String> {
	use serde_json::json;

	// keccak256("Deposit(uint256,uint32,uint256)")
	const DEPOSIT_TOPIC: &str =
		"0x2813ca2762c14ad53880ef467c7448a9015904c20e064e6216ffb3f63390ec5d";

	let commitment_topic = format!("0x{}", hex::encode(commitment));

	let client = reqwest::Client::new();
	let body = json!({
		"jsonrpc": "2.0",
		"id": 1,
		"method": "eth_getLogs",
		"params": [{
			"address": pool_address,
			"topics": [DEPOSIT_TOPIC, commitment_topic],
			"fromBlock": "0x0",
			"toBlock": "latest"
		}]
	});

	let resp = client
		.post(rpc_url)
		.json(&body)
		.send()
		.await
		.map_err(|e| format!("eth_getLogs request: {e}"))?;
	let json: serde_json::Value =
		resp.json().await.map_err(|e| format!("eth_getLogs parse: {e}"))?;

	let logs = json["result"]
		.as_array()
		.ok_or_else(|| format!("eth_getLogs no result: {json}"))?;

	if logs.is_empty() {
		return Err(format!("No Deposit event found for commitment 0x{}", hex::encode(commitment)));
	}

	// Take the most recent deposit (last log)
	let log = &logs[logs.len() - 1];
	let topics = log["topics"].as_array().ok_or("no topics in log")?;
	// topics[1] = indexed leafIndex (padded to 32 bytes as uint256)
	let leaf_topic = topics.get(2).and_then(|t| t.as_str()).ok_or("no leafIndex topic")?;
	let leaf_bytes = hex::decode(leaf_topic.strip_prefix("0x").unwrap_or(leaf_topic))
		.map_err(|e| format!("decode leafIndex topic: {e}"))?;
	let leaf_index = if leaf_bytes.len() >= 4 {
		u32::from_be_bytes(leaf_bytes[leaf_bytes.len() - 4..].try_into().unwrap())
	} else {
		0
	};

	info!("Found Deposit event for commitment: leaf_index={}", leaf_index);
	Ok(leaf_index)
}

/// Reconstruct the Merkle authentication path for `leaf_index`.
fn reconstruct_merkle_path(
	leaf_index: u32,
	filled_subtrees: &[[u8; 32]; 20],
) -> ([[u8; 32]; 20], [bool; 20]) {
	let mut path_elements = [[0u8; 32]; 20];
	let mut path_indices = [false; 20];

	for i in 0..20 {
		let bit = (leaf_index >> i) & 1;
		path_indices[i] = bit == 1;
		if bit == 1 {
			path_elements[i] = filled_subtrees[i];
		} else {
			path_elements[i] = poseidon_zero_hash(i as u32);
		}
	}

	(path_elements, path_indices)
}

/// Poseidon zero hash at tree level. Level 0 = 0, level i = Poseidon(prev, prev).
fn poseidon_zero_hash(level: u32) -> [u8; 32] {
	use ark_ff::{BigInteger, PrimeField};
	use light_poseidon::{Poseidon, PoseidonHasher};

	let q: BigUint =
		"21888242871839275222246405745257275088548364400416034343698204186575808495617"
			.parse()
			.unwrap();
	let mut h = BigUint::from(0u32);

	for _ in 0..level {
		let f = ark_bn254::Fr::from(h.clone() % &q);
		let mut pos = Poseidon::<ark_bn254::Fr>::new_circom(2).expect("poseidon init");
		let hash = pos.hash(&[f, f]).expect("poseidon hash");
		let bytes = hash.into_bigint().to_bytes_be();
		let mut out = [0u8; 32];
		let len = bytes.len().min(32);
		out[32 - len..].copy_from_slice(&bytes[bytes.len() - len..]);
		h = BigUint::from_bytes_be(&out);
	}

	let b = h.to_bytes_be();
	let mut out = [0u8; 32];
	let len = b.len().min(32);
	out[32 - len..].copy_from_slice(&b[b.len() - len..]);
	out
}

// ─── Calldata builder ─────────────────────────────────────────────────────────

fn build_withdraw_calldata(
	nullifier: &[u8; 32],
	root: &[u8; 32],
	commitment: &[u8; 32],
	amount: u128,
	recipient: Address,
	proof_bytes: &[u8],
) -> String {
	let encoded = encode(&[
		Token::Uint(U256::from_big_endian(nullifier)),
		Token::Uint(U256::from(amount)),
		Token::Address(recipient),
		Token::Bytes(proof_bytes.to_vec()),
		Token::FixedArray(vec![
			Token::Uint(U256::from_big_endian(root)),
			Token::Uint(U256::from_big_endian(nullifier)),
			Token::Uint(U256::from_big_endian(commitment)),
		]),
	]);
	let mut calldata = WITHDRAW_SELECTOR.to_vec();
	calldata.extend_from_slice(&encoded);
	format!("0x{}", hex::encode(calldata))
}
