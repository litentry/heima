// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.
use frame_support::traits::{Get, OnRuntimeUpgrade};
use pallet_democracy::{
	AccountVote, BoundedCallOf, Conviction, Delegations, DepositOf, PropIndex, ReferendumIndex,
	ReferendumInfo, ReferendumInfoOf, ReferendumStatus, Tally, VotingOf,
};
use sp_std::{marker::PhantomData, vec::Vec};

use frame_support::{
	migration::storage_key_iter, pallet_prelude::*, traits::Currency, Twox64Concat,
};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::EncodeLike;
use sp_runtime::Saturating;

type BalanceOf<T> = <<T as pallet_democracy::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

/// A "prior" lock, i.e. a lock for some now-forgotten reason.
#[derive(
	Encode,
	MaxEncodedLen,
	Decode,
	Default,
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	RuntimeDebug,
	TypeInfo,
)]
pub struct PriorLock<BlockNumber, Balance>(BlockNumber, Balance);

#[derive(Clone, Encode, Decode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
#[codec(mel_bound(skip_type_params(MaxVotes)))]
#[scale_info(skip_type_params(MaxVotes))]
pub enum Voting<Balance, AccountId, BlockNumber, MaxVotes: Get<u32>> {
	/// The account is voting directly. `delegations` is the total amount of post-conviction voting
	/// weight that it controls from those that have delegated to it.
	Direct {
		/// The current votes of the account.
		votes: BoundedVec<(ReferendumIndex, AccountVote<Balance>), MaxVotes>,
		/// The total amount of delegations that this account has received.
		delegations: Delegations<Balance>,
		/// Any pre-existing locks from past voting/delegating activity.
		prior: PriorLock<BlockNumber, Balance>,
	},
	/// The account is delegating `balance` of its balance to a `target` account with `conviction`.
	Delegating {
		balance: Balance,
		target: AccountId,
		conviction: Conviction,
		/// The total amount of delegations that this account has received.
		delegations: Delegations<Balance>,
		/// Any pre-existing locks from past voting/delegating activity.
		prior: PriorLock<BlockNumber, Balance>,
	},
}

/// Const<u32> doesn't implement partialeq trait, so we have to check manually
#[cfg(feature = "try-runtime")]
fn are_voting_instances_equal<Balance, AccountId, BlockNumber, MaxVotes: Get<u32>>(
	a: &Voting<Balance, AccountId, BlockNumber, MaxVotes>,
	b: &Voting<Balance, AccountId, BlockNumber, MaxVotes>,
) -> bool
where
	Balance: PartialEq,
	AccountId: PartialEq,
	BlockNumber: PartialEq,
{
	match (a, b) {
		(
			Voting::Direct { votes: votes_a, delegations: delegations_a, prior: prior_a },
			Voting::Direct { votes: votes_b, delegations: delegations_b, prior: prior_b },
		) => votes_a == votes_b && delegations_a == delegations_b && prior_a == prior_b,

		(
			Voting::Delegating {
				balance: balance_a,
				target: target_a,
				conviction: conviction_a,
				delegations: delegations_a,
				prior: prior_a,
			},
			Voting::Delegating {
				balance: balance_b,
				target: target_b,
				conviction: conviction_b,
				delegations: delegations_b,
				prior: prior_b,
			},
		) =>
			balance_a == balance_b &&
				target_a == target_b &&
				conviction_a == conviction_b &&
				delegations_a == delegations_b &&
				prior_a == prior_b,

		_ => false,
	}
}

// This is important when we want to insert into the storage item
impl<AccountId, Balance, BlockNumber, MaxVotes>
	EncodeLike<pallet_democracy::Voting<AccountId, Balance, BlockNumber, MaxVotes>>
	for Voting<AccountId, Balance, BlockNumber, MaxVotes>
where
	AccountId: EncodeLike<AccountId>,
	Balance: EncodeLike<Balance>,
	BlockNumber: EncodeLike<BlockNumber>,
	MaxVotes: Get<u32>,
{
}

pub struct ReplaceDemocracyStorage<T>(PhantomData<T>);
impl<T: pallet_democracy::Config> ReplaceDemocracyStorage<T>
where
	BalanceOf<T>: From<u128>,
{
	fn replace_deposit_of_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceDemocracyStorage",
			"running migration to Democracy DepositOf Storage Item"
		);
		let pallet_prefix: &[u8] = b"Democracy";
		let storage_item_prefix: &[u8] = b"DepositOf";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (prop_index, mut value) in storage_key_iter::<
			PropIndex,
			(BoundedVec<T::AccountId, T::MaxDeposits>, BalanceOf<T>),
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			value.1 = value.1.saturating_mul(DECIMAL_CONVERTOR.into());

			<DepositOf<T>>::insert(prop_index, value);

			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}

	fn replace_referendum_info_of_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceDemocracyStorage",
			"running migration to Democracy ReferendumInfoOf Storage Item"
		);
		let pallet_prefix: &[u8] = b"Democracy";
		let storage_item_prefix: &[u8] = b"ReferenceInfoOf";

		let mut weight: Weight = frame_support::weights::Weight::zero();
		for (ref_index, ref_info) in storage_key_iter::<
			ReferendumIndex,
			ReferendumInfo<T::BlockNumber, BoundedCallOf<T>, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			let new_ref_info = match ref_info {
				ReferendumInfo::Finished { approved, end } =>
					ReferendumInfo::Finished { approved, end },
				ReferendumInfo::Ongoing(ref_status) => ReferendumInfo::Ongoing(ReferendumStatus {
					end: ref_status.end,
					proposal: ref_status.proposal,
					threshold: ref_status.threshold,
					delay: ref_status.delay,
					tally: Tally {
						ayes: ref_status.tally.ayes.saturating_mul(DECIMAL_CONVERTOR.into()),
						nays: ref_status.tally.nays.saturating_mul(DECIMAL_CONVERTOR.into()),
						turnout: ref_status.tally.turnout.saturating_mul(DECIMAL_CONVERTOR.into()),
					},
				}),
			};

			<ReferendumInfoOf<T>>::insert(ref_index, new_ref_info);

			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}

	fn replace_voting_of_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceDemocracyStorage",
			"running migration to Democracy VotingOf Storage Item"
		);
		let pallet_prefix: &[u8] = b"Democracy";
		let storage_item_prefix: &[u8] = b"VotingOf";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (who, voting) in storage_key_iter::<
			T::AccountId,
			Voting<BalanceOf<T>, T::AccountId, BlockNumberFor<T>, T::MaxVotes>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			let new_voting = match voting {
				Voting::Delegating { balance, target, conviction, delegations, prior } => {
					let new_balance = balance.saturating_mul(DECIMAL_CONVERTOR.into());
					let new_delegation = Delegations {
						votes: delegations.votes.saturating_mul(DECIMAL_CONVERTOR.into()),
						capital: delegations.capital.saturating_mul(DECIMAL_CONVERTOR.into()),
					};
					let new_prior_locks =
						PriorLock(prior.0, prior.1.saturating_mul(DECIMAL_CONVERTOR.into()));
					Voting::Delegating {
						balance: new_balance,
						target,
						conviction,
						delegations: new_delegation,
						prior: new_prior_locks,
					}
				},
				Voting::Direct { votes, delegations, prior } => {
					let new_votes: Vec<_> = votes
						.into_iter()
						.map(|(id, vote)| {
							let new_vote = match vote {
								AccountVote::Split { aye, nay } => AccountVote::Split {
									aye: aye.saturating_mul(DECIMAL_CONVERTOR.into()),
									nay: nay.saturating_mul(DECIMAL_CONVERTOR.into()),
								},
								AccountVote::Standard { vote, balance } => AccountVote::Standard {
									vote,
									balance: balance.saturating_mul(DECIMAL_CONVERTOR.into()),
								},
							};
							(id, new_vote)
						})
						.collect();

					// This unwrap cannot fail since it is the same BoundedVec
					let bounded_new_votes: BoundedVec<
						(u32, AccountVote<BalanceOf<T>>),
						T::MaxVotes,
					> = new_votes.try_into().unwrap();

					let new_delegation = Delegations {
						votes: delegations.votes.saturating_mul(DECIMAL_CONVERTOR.into()),
						capital: delegations.capital.saturating_mul(DECIMAL_CONVERTOR.into()),
					};
					let new_prior_locks =
						PriorLock(prior.0, prior.1.saturating_mul(DECIMAL_CONVERTOR.into()));

					Voting::Direct {
						votes: bounded_new_votes,
						delegations: new_delegation,
						prior: new_prior_locks,
					}
				},
			};
			<VotingOf<T>>::insert(who, new_voting);

			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}
}

#[cfg(feature = "try-runtime")]
impl<T: pallet_democracy::Config> ReplaceDemocracyStorage<T>
where
	BalanceOf<T>: From<u128>,
{
	fn pre_upgrade_deposit_of_storage() -> Result<Vec<u8>, &'static str> {
		let result: Vec<_> = <DepositOf<T>>::iter()
			.map(|(prop_index, value)| {
				let mut new_value = value;
				new_value.1 = new_value.1.saturating_mul(DECIMAL_CONVERTOR.into());

				(prop_index, new_value)
			})
			.collect();
		Ok(result.encode())
	}

	fn post_upgrade_deposit_of_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_result = Vec::<(
			PropIndex,
			(BoundedVec<T::AccountId, T::MaxDeposits>, BalanceOf<T>),
		)>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode Bounties")?;

		let actual_result: Vec<_> =
			<DepositOf<T>>::iter().map(|(prop_index, value)| (prop_index, value)).collect();

		for x in 0..actual_result.len() {
			assert_eq!(actual_result[x], expected_result[x])
		}

		Ok(())
	}

	fn pre_upgrade_referendum_info_of_storage() -> Result<Vec<u8>, &'static str> {
		let result: Vec<_> = <ReferendumInfoOf<T>>::iter()
			.map(|(ref_index, ref_info)| {
				let new_ref_info = match ref_info {
					ReferendumInfo::Finished { approved, end } =>
						ReferendumInfo::Finished { approved, end },
					ReferendumInfo::Ongoing(ref_status) =>
						ReferendumInfo::Ongoing(ReferendumStatus {
							end: ref_status.end,
							proposal: ref_status.proposal,
							threshold: ref_status.threshold,
							delay: ref_status.delay,
							tally: Tally {
								ayes: ref_status
									.tally
									.ayes
									.saturating_mul(DECIMAL_CONVERTOR.into()),
								nays: ref_status
									.tally
									.nays
									.saturating_mul(DECIMAL_CONVERTOR.into()),
								turnout: ref_status
									.tally
									.turnout
									.saturating_mul(DECIMAL_CONVERTOR.into()),
							},
						}),
				};

				(ref_index, new_ref_info)
			})
			.collect();
		Ok(result.encode())
	}

	fn post_upgrade_referendum_info_of_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_result = Vec::<(
			ReferendumIndex,
			ReferendumInfo<T::BlockNumber, BoundedCallOf<T>, BalanceOf<T>>,
		)>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode Bounties")?;

		let actual_result: Vec<_> = <ReferendumInfoOf<T>>::iter()
			.map(|(ref_index, ref_info)| (ref_index, ref_info))
			.collect();
		for x in 0..actual_result.len() {
			assert_eq!(actual_result[x], expected_result[x])
		}
		Ok(())
	}

	fn pre_upgrade_voting_of_storage() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"Democracy";
		let storage_item_prefix: &[u8] = b"VotingOf";
		let stored_data: Vec<_> = storage_key_iter::<
			T::AccountId,
			Voting<BalanceOf<T>, T::AccountId, BlockNumberFor<T>, T::MaxVotes>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let result: Vec<_> = stored_data
			.into_iter()
			.map(|(who, voting)| {
				let new_voting = match voting {
					Voting::Delegating { balance, target, conviction, delegations, prior } => {
						let new_balance = balance.saturating_mul(DECIMAL_CONVERTOR.into());
						let new_delegation = Delegations {
							votes: delegations.votes.saturating_mul(DECIMAL_CONVERTOR.into()),
							capital: delegations.capital.saturating_mul(DECIMAL_CONVERTOR.into()),
						};
						let new_prior_locks =
							PriorLock(prior.0, prior.1.saturating_mul(DECIMAL_CONVERTOR.into()));
						Voting::Delegating {
							balance: new_balance,
							target,
							conviction,
							delegations: new_delegation,
							prior: new_prior_locks,
						}
					},
					Voting::Direct { votes, delegations, prior } => {
						let new_votes: Vec<_> = votes
							.into_iter()
							.map(|(id, vote)| {
								let new_vote = match vote {
									AccountVote::Split { aye, nay } => AccountVote::Split {
										aye: aye.saturating_mul(DECIMAL_CONVERTOR.into()),
										nay: nay.saturating_mul(DECIMAL_CONVERTOR.into()),
									},
									AccountVote::Standard { vote, balance } =>
										AccountVote::Standard {
											vote,
											balance: balance
												.saturating_mul(DECIMAL_CONVERTOR.into()),
										},
								};
								(id, new_vote)
							})
							.collect();

						let bounded_new_votes: BoundedVec<
							(u32, AccountVote<BalanceOf<T>>),
							T::MaxVotes,
						> = new_votes.try_into().unwrap();

						let new_delegation = Delegations {
							votes: delegations.votes.saturating_mul(DECIMAL_CONVERTOR.into()),
							capital: delegations.capital.saturating_mul(DECIMAL_CONVERTOR.into()),
						};
						let new_prior_locks =
							PriorLock(prior.0, prior.1.saturating_mul(DECIMAL_CONVERTOR.into()));

						Voting::Direct {
							votes: bounded_new_votes,
							delegations: new_delegation,
							prior: new_prior_locks,
						}
					},
				};
				(who, new_voting)
			})
			.collect();

		Ok(result.encode())
	}

	fn post_upgrade_voting_of_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_result = Vec::<(
			T::AccountId,
			Voting<BalanceOf<T>, T::AccountId, BlockNumberFor<T>, T::MaxVotes>,
		)>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode Bounties")?;

		let pallet_prefix: &[u8] = b"Democracy";
		let storage_item_prefix: &[u8] = b"VotingOf";
		let actual_result: Vec<_> = storage_key_iter::<
			T::AccountId,
			Voting<BalanceOf<T>, T::AccountId, BlockNumberFor<T>, T::MaxVotes>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		for x in 0..actual_result.len() {
			assert_eq!(actual_result[x].0, expected_result[x].0);
			let result = are_voting_instances_equal::<
				BalanceOf<T>,
				T::AccountId,
				BlockNumberFor<T>,
				T::MaxVotes,
			>(&actual_result[x].1, &expected_result[x].1);
			assert!(result);
		}

		Ok(())
	}
}

impl<T: pallet_democracy::Config> OnRuntimeUpgrade for ReplaceDemocracyStorage<T>
where
	BalanceOf<T>: From<u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let deposit_of_state_vec = Self::pre_upgrade_deposit_of_storage()?;
		let referendum_info_of_state_vec = Self::pre_upgrade_referendum_info_of_storage()?;
		let voting_of_state_vec = Self::pre_upgrade_voting_of_storage()?;

		log::info!(target: "ReplaceDemocracyStorage", "Finished performing pre upgrade checks");
		Ok((deposit_of_state_vec, referendum_info_of_state_vec, voting_of_state_vec).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);
		weight += Self::replace_deposit_of_storage();
		weight += Self::replace_referendum_info_of_storage();
		weight += Self::replace_voting_of_storage();

		log::info!(target: "ReplaceDemocracyStorage", "Finished performing storage migrations");
		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>, Vec<u8>, Vec<u8>) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;
		Self::post_upgrade_deposit_of_storage(pre_vec.0)?;
		Self::post_upgrade_referendum_info_of_storage(pre_vec.1)?;
		Self::post_upgrade_voting_of_storage(pre_vec.2)?;
		log::info!(target: "ReplaceDemocracyStorage", "Finished performing post upgrade checks");
		Ok(())
	}
}
