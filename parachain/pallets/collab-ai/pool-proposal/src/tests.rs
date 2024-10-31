use crate::mock::*;
use frame_support::assert_ok;
use sp_runtime::AccountId32;

#[test]
fn test_propose_investing_pool_ok() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let info_hash: [u8; 32] = [1; 32];
		let curator_index = PublicCuratorCount::<Test>::get();

		// Register curator
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		// Check if curator is stored correctly
		assert_eq!(PublicCuratorToIndex::<Test>::get(&curator), Some(curator_index));
		assert_eq!(PublicCuratorCount::<Test>::get(), curator_index + 1);
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(info_hash), 1, curator.clone(), CandidateStatus::Unverified))
		);

		System::assert_last_event(RuntimeEvent::Curator(crate::Event::CuratorRegisted {
			curator,
			curator_index,
			info_hash: sp_core::H256(info_hash),
		}));
	})
}