# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This package is a refactor of [parachain-api](https://www.npmjs.com/package/@heima-network/parachain-api) and [sidechain-api](https://www.npmjs.com/package/@heima-network/sidechain-api).  

## [Unreleased]

## [0.0.3] - 2025-07-08
### Added
- Added Omni Executor interface definitions for task execution and authentication
- Added `NativeTask` enum with task types:
  - `RequestIntent` for intent-based operations
  - `PumpxRequestJwt`, `PumpxExportWallet`, `PumpxAddWallet`, `PumpxSignLimitOrder`, `PumpxTransferWidthdraw`, `PumpxNotifyLimitOrderResult` for PumpX integration
- Added `OmniAuth` enum with `Web3`, `Email`, `AuthToken`, and `OAuth2` authentication methods
- Added `TxStatus` enum with transaction states: `Future`, `Ready`, `Broadcast`, `InBlock`, `Retracted`, `FinalityTimeout`, `Finalized`, `Usurped`, `Dropped`, `Invalid`
- Added response types: `PumpxAddWalletResponse`, `PumpxCreateTransferTxResponse`, `PumpxRequestJwt`
- Added data types: `PumpxCreateTransferTxData`, `PumpxAddWalletData`
- Added error types: `PumpxApiError`, `PumpxSignerError`

## [0.0.2] - 2025-06-05
### Changed
- Change `CorePrimitivesIdentity` to `HeimaPrimitivesIdentity`

## [0.0.1] - 2025-05-22
### Changed
- Change `@heima-network/*-api` to `@heima-network/api-augment/*`


