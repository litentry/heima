use super::*;

use crate::Runtime;
use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};

// This is just an example of how to write a custom migration
// It has no effect on tee-worker running
// For more details, see: https://docs.substrate.io/maintain/runtime-upgrades/#storage-migration
pub struct Upgrade;
impl OnRuntimeUpgrade for Upgrade {
	fn on_runtime_upgrade() -> Weight {
		pallet_identity_management_tee::migrations::migrate_to_v1::<Runtime, IdentityManagement>()
	}
}
