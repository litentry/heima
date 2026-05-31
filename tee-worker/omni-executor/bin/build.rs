// Provide __rust_probestack for wasmer cranelift JIT.
// Cranelift emits calls to this symbol for stack probing on x86_64 Linux,
// but it's not exported as a C symbol from libcompiler_builtins.rlib.
// A no-op stub is safe here: the stack sizes used by the witness calculator
// are well within the default thread stack limit.

fn main() {
	if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("x86_64")
		&& std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux")
	{
		let out = std::env::var("OUT_DIR").unwrap();
		let src = format!("{out}/probestack.c");
		let obj = format!("{out}/probestack.o");

		std::fs::write(
			&src,
			r#"
void __rust_probestack(void) __attribute__((naked));
void __rust_probestack(void) { __asm__ volatile("ret"); }
"#,
		)
		.unwrap();

		let status = std::process::Command::new("cc")
			.args(["-c", &src, "-o", &obj])
			.status()
			.expect("cc not found");
		assert!(status.success(), "failed to compile probestack stub");

		println!("cargo:rustc-link-arg={obj}");
	}
}
