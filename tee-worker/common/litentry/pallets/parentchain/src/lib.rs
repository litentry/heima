#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::weights::WeightInfo;
	use frame_support::{pallet_prelude::*, sp_runtime::traits::Header};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	/// Configuration trait.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		ShardVaultAlreadyInitialized,
	}

	/// The current block number being processed. Set by `set_block`.
	#[pallet::storage]
	#[pallet::getter(fn block_number)]
	pub(super) type Number<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Hash of the previous block. Set by `set_block`.
	#[pallet::storage]
	#[pallet::getter(fn parent_hash)]
	pub(super) type ParentHash<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::Hash, ValueQuery>;

	/// Hash of the last block. Set by `set_block`.
	#[pallet::storage]
	#[pallet::getter(fn block_hash)]
	pub(super) type BlockHash<T: Config<I>, I: 'static = ()> = StorageValue<_, T::Hash, ValueQuery>;

	/// The current block number being processed. Set by `init_shard_vault`.
	#[pallet::storage]
	#[pallet::getter(fn shard_vault)]
	pub(super) type ShardVault<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn creation_block_hash)]
	pub(super) type CreationBlockHash<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::Hash, OptionQuery>;

	/// The creation block number. Set by `set_creation_block`.
	#[pallet::storage]
	#[pallet::getter(fn creation_block_number)]
	pub(super) type CreationBlockNumber<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::BlockNumber, OptionQuery>;

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight({195_000})]
		pub fn set_block(origin: OriginFor<T>, header: T::Header) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			<Number<T, I>>::put(header.number());
			<ParentHash<T, I>>::put(header.parent_hash());
			<BlockHash<T, I>>::put(header.hash());
			Ok(Pays::No.into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({195_000})]
		pub fn init_shard_vault(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			ensure!(Self::shard_vault().is_none(), Error::<T, I>::ShardVaultAlreadyInitialized);
			<ShardVault<T, I>>::put(account.clone());
			Ok(Pays::No.into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({195_000})]
		pub fn set_creation_block(
			origin: OriginFor<T>,
			header: T::Header,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			<CreationBlockNumber<T, I>>::put(header.number());
			<CreationBlockHash<T, I>>::put(header.hash());
			Ok(Pays::No.into())
		}
	}
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;
