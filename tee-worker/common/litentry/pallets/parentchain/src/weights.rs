pub use frame_support::weights::{constants::RocksDbWeight, Weight};

/// Weight functions needed for pallet_parentchain.
pub trait WeightInfo {
	fn set_block() -> Weight;
}

/// Weights for pallet_parentchain using the Integritee parachain node and recommended hardware.
impl WeightInfo for () {
	fn set_block() -> Weight {
		Weight::from_parts(10_000, 0)
	}
}
