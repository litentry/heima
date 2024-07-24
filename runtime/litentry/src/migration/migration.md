# MigrateCollatorSelectionIntoParachainStaking
P9100.rs
https://github.com/litentry/litentry-parachain/releases/tag/v0.9.10
This migration is for the replacement of CollatorSelection with ParachainStaking

The main purpose of runtime upgrade is for make up the missing genesis build of ParachainStaking and clean the old CollatorSelection storage.

# MigrateAtStakeAutoCompound
P9135.rs
https://github.com/litentry/litentry-parachain/releases/tag/v0.9.13-1
This migration is for the update of AtStaked with ParachainStaking

The main purpose of runtime upgrade is for adding the autocompound staking function of ParachainStaking and need to update storage to the latest struct.


# RemoveSudoAndStorage
P9175.rs
https://github.com/litentry/litentry-parachain/releases/tag/p0.9.17-9175-w0.0.2-105
This migration is for the sudo remove on Litentry Parachain

The main purpose of runtime upgrade is for killing sudo storage