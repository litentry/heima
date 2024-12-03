SHELL=/bin/bash
all:
	@make help

# variant declaration

NODE_BIN=litentry-collator

.PHONY: help ## Display help commands
help:
	@printf 'Usage:\n'
	@printf '  make <tagert>\n'
	@printf '\n'
	@printf 'Targets:\n'
	@IFS=$$'\n' ; \
    help_lines=(`fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//'`); \
    for help_line in $${help_lines[@]}; do \
        IFS=$$'#' ; \
        help_split=($$help_line) ; \
        help_info=`echo $${help_split[2]} | sed -e 's/^ *//' -e 's/ *$$//'` ; \
		IFS=$$':' ; \
		phony_command=($$help_split); \
        help_command=`echo $${phony_command[1]} | sed -e 's/^ *//' -e 's/ *$$//'` ; \
		printf "  %-50s %s\n" $$help_command $$help_info ; \
    done

# build release

.PHONY: build-node ## Build release node
build-node:
	cd parachain && cargo build --locked -p litentry-collator --release

.PHONY: build-runtime-litentry ## Build litentry release runtime
build-runtime-litentry:
	cd parachain && cargo build --locked -p litentry-parachain-runtime --release

.PHONY: build-runtime-paseo ## Build paseo release runtime
build-runtime-paseo:
	cd parachain && cargo build --locked -p paseo-parachain-runtime --release

.PHONY: build-docker-release ## Build docker image using cargo profile `release`
build-docker-release:
	@cd parachain && ./scripts/build-docker.sh release latest

.PHONY: build-docker-production ## Build docker image using cargo profile `production`
build-docker-production:
	@cd parachain && ./scripts/build-docker.sh production

.PHONY: build-node-benchmarks ## Build release node with `runtime-benchmarks` feature
build-node-benchmarks:
	cd parachain && cargo build --locked --features runtime-benchmarks --release

.PHONY: build-node-tryruntime ## Build release node with `try-runtime` feature
build-node-tryruntime:
	cd parachain && cargo build --locked --features try-runtime --release

# launch a local network

.PHONY: launch-standalone ## Launch a local standalone node without relaychain network
launch-standalone:
	@cd parachain && ./scripts/launch-standalone.sh

.PHONY: launch-network-litentry ## Launch a local litentry network with relaychain network
launch-network-litentry:
	@cd parachain && ./scripts/launch-network.sh litentry

.PHONY: launch-network-paseo ## Launch a local litentry network with relaychain network
launch-network-paseo:
	@cd parachain && ./scripts/launch-network.sh paseo

# run tests

.PHONY: test-cargo-all ## cargo test --all
test-cargo-all:
	@cd parachain && cargo test --release --all

.PHONY: test-cargo-all-benchmarks ## cargo test --all --features runtime-benchmarks
test-cargo-all-benchmarks:
	@cd parachain && cargo test --release --all --features runtime-benchmarks

.PHONY: test-ts-litentry ## Run litentry ts tests without clean-up
test-ts-litentry: launch-network-litentry
	@cd parachain && ./scripts/run-ts-test.sh litentry

.PHONY: test-ts-paseo ## Run paseo ts tests without clean-up
test-ts-paseo: launch-network-paseo
	@cd parachain && ./scripts/run-ts-test.sh paseo

# clean up
.PHONY: clean-network ## Clean up the network launched by 'launch-network'
clean-network:
	@cd parachain && ./scripts/clean-network.sh

# update dependencies
.PHONY: update-ts-dep ## update ts-tests dependencies
update-ts-dep:
	@cd parachain/ts-tests && pnpm dlx npm-check-updates -u && pnpm install

# format
.PHONY: fmt ## (cargo, taplo, ts) fmt
fmt: fmt-cargo fmt-taplo fmt-ts

.PHONY: fmt-cargo ## cargo fmt
fmt-cargo:
	@cd parachain && cargo fmt --all
	@cd tee-worker && cargo fmt --all
	@cd tee-worker/identity/enclave-runtime && cargo fmt --all
	@cd tee-worker/bitacross/enclave-runtime && cargo fmt --all

.PHONY: fmt-taplo ## taplo fmt
fmt-taplo:
	@cd parachain && RUST_LOG=error taplo fmt
	@cd tee-worker && RUST_LOG=error taplo fmt

.PHONY: fmt-ts ## ts fmt
fmt-ts:
	@cd parachain/ts-tests && pnpm install && pnpm run format
	@cd tee-worker/identity/ts-tests && pnpm install && pnpm run format

.PHONY: githooks ## install the githooks
githooks:
	git config core.hooksPath .githooks

# clippy
.PHONY: clippy ## cargo clippy
clippy:
	cd parachain && SKIP_WASM_BUILD=1 cargo clippy --locked --workspace --all -- -D warnings

.PHONY: clippyfix ## cargo clippy --fix
clippyfix:
	cd parachain && SKIP_WASM_BUILD=1 cargo clippy --locked --allow-dirty --allow-staged --fix --workspace --all -- -D warnings

.PHONY: cargofix ## cargo fix
cargofix:
	cd parachain && cargo fix --locked --allow-dirty --allow-staged --workspace --all
