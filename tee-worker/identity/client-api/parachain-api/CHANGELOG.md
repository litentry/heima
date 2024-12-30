# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.21-next.0] - 2024-12-17

-   Update for version [parachain-release v0.9.21-01](https://github.com/litentry/litentry-parachain/releases/tag/v0.9.21-01)

## Added

-   Type definitions for the new OmniAccount Pallet under the name of `omniAccount`.
-   Expose getter to get the derived `omniaccount` address from a given identity.
-   Expose the OmniAccount's trusted calls: `request_intent`, `create_account_store`, `add_account`, and `remove_accounts`.
-   Add `TrustedCallAuthenticated` and `TCAuthentication` trusted call structs.
-   Add `TrustedCallResult` to handle OmniAccount's call results.
-   Add `TrustedOperationAuthenticated` type definition.
-   Update `TrustedCallResult` type definition.
-   Add error variants to `RequestVcErrorDetail`.

## [0.9.20-4.1] - 2024-09-30

-   Update for version [parachain-release v0.9.20-04](https://github.com/litentry/litentry-parachain/releases/tag/v0.9.20-04)

### Added

-   `LinkedIdentities` entry in `Assertion` enum.
-   New `Email` identity. Consequent updates on `LitentryIdentity` and `Web2ValidationData`.

## [0.9.19-07] - 2024-08-26

Matching version for [parachain-release v0.9.19-07](https://github.com/litentry/litentry-parachain/releases/tag/v0.9.19-07)

## [0.9.18-11.2] - 2024-07-26

### Added

-   `TrustedCall`: expose `link_identity_callback`.

## [0.9.18-11] - 2024-07-19

### Added

-   (#2930) `PlatformUserType`: Add `DarenMarket`

### Changed

-   (#2930) `PlatformUserType`: rename `MagicCraftStakingUser` to `MagicCraftStaking`
-   (#2930) `PlatformUserType`: rename `KaratDaoUser` to `KaratDao`
-   (#2904) `AssertionSupportedNetwork`: update network list

## [0.9.18-10] - 2024-07-15

Matching version for [parachain-release v0.9.18-10](https://github.com/litentry/litentry-parachain/releases/tag/v0.9.18-10)

## [0.9.18-next.11] - 2024-07-12

### Added

-   (#2877, #2889) Add `MFan` and `Mvp` to `Web3NftType`
-   (#2884) Add `An` and `Tuna` to the list of `Web3TokenType`

## [0.9.18-next.8] - 2024-07-02

Routinely update

## [0.9.18-next.7] - 2024-06-19

### Added

-   `Web3Network`: add `Combo`.
-   Add `SubstrateNetwork`, `EvmNetwork`, `SolanaNetwork`, `BitcoinNetwork`, `Web2Network`, `PrimeIdentity` types.

## [0.9.18-next.6] - 2024-06-12

### Changed

-   `package.json`: add `module` entry and mark it as side-effects free.

## [0.9.18-next.5] - 2024-06-12

-   Routinary update
