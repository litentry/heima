# Copyright (C) 2023 Gramine contributors
# SPDX-License-Identifier: BSD-3-Clause

ARCH_LIBDIR ?= /lib/$(shell $(CC) -dumpmachine)
SGX ?= 1

BIN_NAME = omni-executor
COMPILATION_FEATURES =

ROOTDIR := $(shell git rev-parse --show-toplevel)
OMNI_DIR := $(ROOTDIR)/tee-worker/omni-executor

.PHONY: all
all: $(BIN_NAME) $(BIN_NAME).manifest
ifeq ($(SGX), 1)
COMPILATION_FEATURES := $(COMPILATION_FEATURES),gramine-quote
all: $(BIN_NAME).manifest.sgx $(BIN_NAME).sig
endif

.PHONY: $(BIN_NAME)
$(BIN_NAME):
	cargo build --release --locked --features=$(COMPILATION_FEATURES)
	cp target/release/executor-worker $(BIN_NAME)


ifeq ($(DEBUG),1)
GRAMINE_LOG_LEVEL = debug
else
GRAMINE_LOG_LEVEL = error
endif

$(BIN_NAME).manifest: $(BIN_NAME).manifest.template $(BIN_NAME)
	gramine-manifest \
		-Dlog_level=$(GRAMINE_LOG_LEVEL) \
		-Darch_libdir=$(ARCH_LIBDIR) \
		-Dself_exe=$(BIN_NAME) \
		$< $@

# Make on Ubuntu <= 20.04 doesn't support "Rules with Grouped Targets" (`&:`),
# see the helloworld example for details on this workaround.
$(BIN_NAME).manifest.sgx $(BIN_NAME).sig: sgx_sign
	@:

.INTERMEDIATE: sgx_sign
sgx_sign: $(BIN_NAME).manifest $(BIN_NAME)
	gramine-sgx-sign \
		--manifest $< \
		--output $<.sgx \
		--key enclave_key.pem

ifeq ($(SGX), 1)
GRAMINE = gramine-sgx
else
GRAMINE = gramine-direct
endif

.PHONY: start-gramine-server
start-gramine-server: all
	$(GRAMINE) $(BIN_NAME) -- ${ARGS}

.PHONY: clean
clean:
	$(RM) -rf $(BIN_NAME) *.token *.sig *.manifest.sgx *.manifest result-* OUTPUT

.PHONY: distclean
distclean: clean
	cargo clean

.PHONY: clean-data
clean-data:
	$(RM) -rf $(OMNI_DIR)/local/log $(OMNI_DIR)/storage_db

.PHONY: build-docker
build-docker:
	docker build --target executor-worker -t litentry/$(BIN_NAME):latest -f $(OMNI_DIR)/Dockerfile $(ROOTDIR)

.PHONY: start-local
start-local:
	# Create .env file if it doesn't exist
	@if [ ! -f "$(OMNI_DIR)/.env" ]; then \
		touch $(OMNI_DIR)/.env; \
		chown "$$SUDO_USER":"$$(id -gn $$SUDO_USER)" $(OMNI_DIR)/.env; \
	fi
	docker compose -f $(OMNI_DIR)/docker/docker-compose.yml up

.PHONY: stop-local
stop-local:
	docker compose -f $(OMNI_DIR)/docker/docker-compose.yml down

.PHONY: build-docker-test
build-docker-test:
	docker build --build-arg CARGO_FEATURES="mock-server" --target executor-worker -t litentry/$(BIN_NAME):latest -f $(OMNI_DIR)/Dockerfile $(ROOTDIR)

.PHONY: start-test
start-test:
	docker compose -f $(OMNI_DIR)/docker/docker-compose.yml -f $(OMNI_DIR)/docker/docker-compose.test.yml up

.PHONY: stop-test
stop-test:
	docker compose -f $(OMNI_DIR)/docker/docker-compose.yml -f $(OMNI_DIR)/docker/docker-compose.test.yml down

.PHONY: get-metadata
get-metadata:
	subxt metadata --url http://localhost:9944 --allow-insecure --pallets System,Balances,OmniAccount,Teebag > parentchain/api-interface/artifacts/metadata.scale

.PHONY: generate-api-interface
generate-api-interface: get-metadata
	subxt codegen --file parentchain/api-interface/artifacts/metadata.scale | rustfmt > parentchain/api-interface/src/interface.rs
