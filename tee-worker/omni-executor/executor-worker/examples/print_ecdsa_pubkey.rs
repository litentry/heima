use executor_crypto::{ecdsa, PairTrait};
use std::env;
use std::fs;
use std::process;

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() != 2 {
		eprintln!("Usage: {} <path_to_32_byte_seed_file>", args[0]);
		eprintln!("\nExample:");
		eprintln!("  {} /tmp/test-keys/authorized_key.bin", args[0]);
		process::exit(1);
	}

	let key_path = &args[1];

	let seed_bytes = fs::read(key_path).unwrap_or_else(|e| {
		eprintln!("❌ Error: Failed to read key file '{}': {}", key_path, e);
		process::exit(1);
	});

	if seed_bytes.len() != 32 {
		eprintln!(
			"❌ Error: Invalid key file length. Expected 32 bytes, got {}",
			seed_bytes.len()
		);
		process::exit(1);
	}

	let mut seed = [0u8; 32];
	seed.copy_from_slice(&seed_bytes);

	let pair = ecdsa::Pair::from_seed_slice(&seed).unwrap_or_else(|e| {
		eprintln!("❌ Error: Failed to create keypair from seed: {:?}", e);
		process::exit(1);
	});

	let pubkey = pair.public();
	let pubkey_hex = hex::encode(pubkey.0);

	println!("Compressed ECDSA Public Key (33 bytes):");
	println!("{}", pubkey_hex);
	println!("\nEnvironment Variable:");
	println!("OE_BUNDLER_KEY_EXPORT_AUTHORIZED_PUBKEY={}", pubkey_hex);
}
