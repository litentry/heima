// Copied from ORML: https://github.com/open-web3-stack/open-runtime-module-library

use frame_support::{traits::Get, BoundedVec, CloneNoBound, DefaultNoBound, PartialEqNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_std::{fmt, prelude::*};

/// An ordered set backed by `BoundedVec`
#[derive(
	PartialEqNoBound, Eq, Encode, Decode, DefaultNoBound, CloneNoBound, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(S))]
pub struct OrderedSet<
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq,
	S: Get<u32>,
>(pub BoundedVec<T, S>);

impl<T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq, S: Get<u32>>
	OrderedSet<T, S>
{
	/// Create a new empty set
	pub fn new() -> Self {
		Self(BoundedVec::default())
	}

	/// Create a set from a `Vec`.
	/// `v` will be sorted and dedup first.
	pub fn from(bv: BoundedVec<T, S>) -> Self {
		let mut v = bv.into_inner();
		v.sort();
		v.dedup();

		Self::from_sorted_set(v.try_into().map_err(|_| ()).expect("Did not add any values"))
	}

	/// Create a set from a `Vec`.
	/// Assume `v` is sorted and contain unique elements.
	pub fn from_sorted_set(bv: BoundedVec<T, S>) -> Self {
		Self(bv)
	}

	/// Insert an element.
	/// Return true if insertion happened.
	pub fn insert(&mut self, value: T) -> bool {
		match self.0.binary_search(&value) {
			Ok(_) => false,
			Err(loc) => self.0.try_insert(loc, value).is_ok(),
		}
	}

	/// Remove an element.
	/// Return true if removal happened.
	pub fn remove(&mut self, value: &T) -> bool {
		match self.0.binary_search(value) {
			Ok(loc) => {
				self.0.remove(loc);
				true
			},
			Err(_) => false,
		}
	}

	/// Return if the set contains `value`
	pub fn contains(&self, value: &T) -> bool {
		self.0.binary_search(value).is_ok()
	}

	/// Clear the set
	pub fn clear(&mut self) {
		self.0 = BoundedVec::default();
	}
}

impl<T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq, S: Get<u32>>
	From<BoundedVec<T, S>> for OrderedSet<T, S>
{
	fn from(v: BoundedVec<T, S>) -> Self {
		Self::from(v)
	}
}

#[cfg(feature = "std")]
impl<T, S> fmt::Debug for OrderedSet<T, S>
where
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq + fmt::Debug,
	S: Get<u32>,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("OrderedSet").field(&self.0).finish()
	}
}
