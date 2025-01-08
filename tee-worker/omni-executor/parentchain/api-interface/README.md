# Parentchain API Interface

This crate contains auto-generated Substrate runtime API definitions for the OmniAccount and Teebag pallets.

## Overview

The crate exports the generated interface from a scale encoded metadata file. The interface is used to interact with the parentchain's runtime API.

#### When to regenerate the interface?
Whenever the types of the relevant pallets (OmniAccount, Teebag) change, the interface should be regenerated. This can be done by following the steps in the Generation Process section.

Please note that the metadata fetched from the local node may not be the same as the metadata of the live chains. Therefore, when running the worker against the live chains, the metadata should be fetched from the live chain and the interface should be regenerated.

## Generation Process

The API interface can be generated using the following command (requires a running node):

```bash
make generate-api-interface
```
This will fetch the metadata from a local node (http://localhost:9944) and save it as a scale file,
then it will generate the Rust interface code using `subxt` and save it to `src/interface.rs`.

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
