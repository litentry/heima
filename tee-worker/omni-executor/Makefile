# Copyright (C) 2023 Gramine contributors
# SPDX-License-Identifier: BSD-3-Clause

ARCH_LIBDIR ?= /lib/$(shell $(CC) -dumpmachine)
SGX_MODE ?= HW
WORKER_DEV ?= 0

SELF_EXE = omni-executor-worker
COMPILATION_FEATURES =

ifeq ($(WORKER_DEV), 1)
	COMPILATION_FEATURES := $(COMPILATION_FEATURES),development
endif

ROOTDIR := $(shell git rev-parse --show-toplevel)
OMNI_DIR := $(ROOTDIR)/tee-worker/omni-executor

.PHONY: all
all: $(SELF_EXE) omni-executor.manifest
ifeq ($(SGX_MODE), HW)
COMPILATION_FEATURES := $(COMPILATION_FEATURES),gramine-quote
all: omni-executor.manifest.sgx omni-executor.sig
endif

.PHONY: $(SELF_EXE)
$(SELF_EXE):
	cargo build --release --features=$(COMPILATION_FEATURES)
	cp target/release/executor-worker $(SELF_EXE)


ifeq ($(DEBUG),1)
GRAMINE_LOG_LEVEL = debug
else
GRAMINE_LOG_LEVEL = error
endif

omni-executor.manifest: omni-executor.manifest.template $(SELF_EXE)
	gramine-manifest \
		-Dlog_level=$(GRAMINE_LOG_LEVEL) \
		-Darch_libdir=$(ARCH_LIBDIR) \
		-Dself_exe=$(SELF_EXE) \
		$< $@

# Make on Ubuntu <= 20.04 doesn't support "Rules with Grouped Targets" (`&:`),
# see the helloworld example for details on this workaround.
omni-executor.manifest.sgx omni-executor.sig: sgx_sign
	@:

.INTERMEDIATE: sgx_sign
sgx_sign: omni-executor.manifest $(SELF_EXE)
	gramine-sgx-sign \
		--manifest $< \
		--output $<.sgx \
		--key enclave_key.pem

ifeq ($(SGX_MODE), HW)
GRAMINE = gramine-sgx
else
GRAMINE = gramine-direct
endif

.PHONY: start-gramine-server
start-gramine-server: all
	$(GRAMINE) omni-executor -- ${ARGS}

.PHONY: clean
clean:
	$(RM) -rf $(SELF_EXE) *.token *.sig *.manifest.sgx *.manifest result-* OUTPUT

.PHONY: distclean
distclean: clean
	cargo clean

# By default SGX_MODE=SW for docker image
.PHONY: build-docker
build-docker:
	docker build --target executor-worker -t litentry/omni-executor:latest -f $(OMNI_DIR)/Dockerfile $(OMNI_DIR)

.PHONY: start-local
start-local:
	docker compose -f $(OMNI_DIR)/docker/docker-compose.yml up

.PHONY: stop-local
stop-local:
	docker compose -f $(OMNI_DIR)/docker/docker-compose.yml down

.PHONY: get-metadata
get-metadata:
	subxt metadata --url http://localhost:9944 --allow-insecure --pallets OmniAccount,Teebag > parentchain/api-interface/artifacts/metadata.scale

.PHONY: generate-api-interface
generate-api-interface: get-metadata
	subxt codegen --file parentchain/api-interface/artifacts/metadata.scale | rustfmt > parentchain/api-interface/src/interface.rs
