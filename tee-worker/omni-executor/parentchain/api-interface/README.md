# Parentchain API Interface

This crate contains auto-generated Substrate runtime API definitions for the OmniAccount and Teebag pallets.

## Overview

The crate exports the generated interface from a scale encoded metadata file. The interface is used to interact with the parentchain's runtime API.

## Generation Process

The API interface can be generated using the following commands:

1. First, get the pallet metadata (requires a running node):
```bash
make get-pallet-metadata
```
This will fetch the metadata from a local node (http://localhost:9944) and save it as a scale file.

2. Then generate the interface:
```bash
make generate-api-interface
```
This will generate the Rust interface code using `subxt` and save it to `src/interface.rs`.

## Dependencies

- subxt: For runtime API interaction
- A running local node (for metadata generation)
- `subxt-cli` tools installed

## Usage

Add this crate as a dependency in your `Cargo.toml`:

```toml
[dependencies]
parentchain-api-interface = { path = "path/to/parentchain-api-interface" }
```

Then import the API types in your code:
```rust
use parentchain_api_interface::*;
```
