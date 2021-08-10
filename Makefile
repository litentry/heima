all:
	@echo "Make All"

build:
	cargo build
node:
	cargo build --package $(call pkgid, litentry-collator)

test-node:
	cargo test --package $(call pkgid, litentry-collator)

test:
	cargo test

# benchmark build
build-benchmark:
	cargo build --features runtime-benchmarks --release

benchmark-frame-system:
	target/release/litentry-collator benchmark \
	--chain=./source/local.json \
	--execution=wasm  \
	--db-cache=20 \
	--wasm-execution=compiled \
	--pallet=frame_system \
	--extrinsic=* \
	--heap-pages=4096 \
	--steps=20 \
	--repeat=50 \
	--output=weights.rs \
	--template=./.maintain/frame-weight-template.hbs

fmt:
	cargo fmt
define pkgid
	$(shell cargo pkgid $1)
endef
