# Migrate to remove old bridge pallets storage
p9151.rs
This migration safely removes all on-chain storage for the legacy bridge implementation (ChainBridge, BridgeTransfer, AssetsHandler) after migration to omni-bridge.
The pallets are completely independent with no shared storage.

# A set of migrations
P9223.rs
https://github.com/litentry/heima/releases/tag/v0.9.22-03
This migration is for updating storage version for Paseo