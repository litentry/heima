pub mod aes256;
pub mod confidential;
pub mod jwt;
pub mod passkey;
pub mod privacy_pool;
pub mod secp256k1;
pub mod shielding_key;
pub mod traits;

// wasmer-vm (pulled in via ark-circom's singlepass backend) references the
// `__rust_probestack` symbol on x86_64 Linux. Since Rust 1.90 the compiler no
// longer emits this symbol by default, so linking the crate (notably its test
// binary, which uses the default linker) fails with
// `undefined symbol: __rust_probestack`. Provide the canonical stack-probe
// implementation (verbatim from rustc's compiler-builtins) so the symbol
// resolves. Scoped to x86_64 Linux only — other targets (incl. SGX builds)
// use wasmer-vm's own platform branch and don't need this.
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
core::arch::global_asm!(
	".weak __rust_probestack",
	".type __rust_probestack,@function",
	".hidden __rust_probestack",
	"__rust_probestack:",
	".cfi_startproc",
	"pushq %rbp",
	".cfi_adjust_cfa_offset 8",
	".cfi_offset %rbp, -16",
	"movq %rsp, %rbp",
	".cfi_def_cfa_register %rbp",
	"mov %rax,%r11",
	"cmp $0x1000,%r11",
	"jna 3f",
	"2:",
	"sub $0x1000,%rsp",
	"test %rsp,8(%rsp)",
	"sub $0x1000,%r11",
	"cmp $0x1000,%r11",
	"ja 2b",
	"3:",
	"sub %r11,%rsp",
	"test %rsp,8(%rsp)",
	"add %rax,%rsp",
	"leave",
	".cfi_def_cfa_register %rsp",
	".cfi_adjust_cfa_offset -8",
	"ret",
	".cfi_endproc",
	options(att_syntax)
);

pub mod rsa {
	pub use rsa::*;

	use ethers::types::Bytes;
	use serde::{Deserialize, Serialize};
	use std::vec::Vec;

	#[derive(Debug, Serialize, Deserialize)]
	pub struct Rsa3072PubKey {
		pub n: Vec<u8>,
		pub e: Vec<u8>,
	}

	#[derive(Clone, Serialize, Deserialize)]
	pub struct SerdeRsa3072PubKey {
		pub n: Bytes,
		pub e: Bytes,
	}

	impl From<Rsa3072PubKey> for SerdeRsa3072PubKey {
		fn from(k: Rsa3072PubKey) -> Self {
			Self { n: k.n.into(), e: k.e.into() }
		}
	}
}

pub use sp_core::{crypto::Pair as PairTrait, ecdsa, ed25519, hashing, sr25519, ByteArray};
