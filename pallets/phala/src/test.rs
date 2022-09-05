use crate::basepool;
use crate::mining;
use crate::pawnshop;
use crate::poolproxy::*;
use crate::stakepoolv2;
use crate::vault;
use assert_matches::assert_matches;
use fixed_macro::types::U64F64 as fp;
use frame_support::traits::Currency;
use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::Get,
	traits::tokens::fungibles::{Create, Inspect},
};
use hex_literal::hex;
use sp_runtime::AccountId32;

use crate::mock::{
	ecdh_pubkey, elapse_cool_down, elapse_seconds, new_test_ext, set_block_1, setup_workers,
	setup_workers_linked_operators, take_events, teleport_to_block, worker_pubkey, Balance,
	BlockNumber, Event as TestEvent, Origin, Test, DOLLARS,
};
// Pallets
use crate::mock::{
	Balances, PhalaBasePool, PhalaMining, PhalaRegistry, PhalaStakePool, System, Timestamp,
};

use phala_types::WorkerPublicKey;

#[test]
fn test_pool_subaccount() {
	let sub_account: AccountId32 =
		stakepoolv2::pool_sub_account(1, &WorkerPublicKey::from_raw([0u8; 32]));
	let expected = AccountId32::new(hex!(
		"73706d2f02ab4d74c86ec3b3997a4fadf33e55e8279650c8539ea67e053c02dc"
	));
	assert_eq!(sub_account, expected, "Incorrect sub account");
}

#[test]
fn test_pawn() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		let free = <Test as mining::Config>::Currency::free_balance(
			&<Test as pawnshop::Config>::PawnShopAccountId::get(),
		);
		assert_eq!(free, 0);
		let free = <Test as mining::Config>::Currency::free_balance(1);
		assert_eq!(free, 1000 * DOLLARS);
		assert_ok!(pawnshop::pallet::Pallet::<Test>::pawn(
			Origin::signed(1),
			100 * DOLLARS
		));
		let free = <Test as mining::Config>::Currency::free_balance(
			&<Test as pawnshop::Config>::PawnShopAccountId::get(),
		);
		assert_eq!(free, 100 * DOLLARS);
		let free = <Test as mining::Config>::Currency::free_balance(1);
		assert_eq!(free, 900 * DOLLARS);
		let ppha_free = get_balance(1);
		assert_eq!(ppha_free, 100 * DOLLARS);
	});
}

#[test]
fn test_redeem() {
	new_test_ext().execute_with(|| {
		mock_asset_id();
		assert_ok!(pawnshop::pallet::Pallet::<Test>::pawn(
			Origin::signed(1),
			100 * DOLLARS
		));
		assert_ok!(pawnshop::pallet::Pallet::<Test>::redeem(
			Origin::signed(1),
			50 * DOLLARS,
			false
		));
		let free = <Test as mining::Config>::Currency::free_balance(1);
		assert_eq!(free, 950 * DOLLARS);
		let free = <Test as mining::Config>::Currency::free_balance(
			&<Test as pawnshop::Config>::PawnShopAccountId::get(),
		);
		assert_eq!(free, 50 * DOLLARS);
		let ppha_free = get_balance(1);
		assert_eq!(ppha_free, 50 * DOLLARS);
		pawnshop::pallet::StakerAccounts::<Test>::insert(
			1,
			pawnshop::FinanceAccount::<u128> {
				invest_pools: vec![],
				locked: 20 * DOLLARS,
			},
		);
		assert_noop!(
			pawnshop::pallet::Pallet::<Test>::redeem(Origin::signed(1), 50 * DOLLARS, false),
			pawnshop::Error::<Test>::RedeemAmountExceedsAvaliableStake
		);
		assert_ok!(pawnshop::pallet::Pallet::<Test>::redeem(
			Origin::signed(1),
			50 * DOLLARS,
			true
		));
		let free = <Test as mining::Config>::Currency::free_balance(
			&<Test as pawnshop::Config>::PawnShopAccountId::get(),
		);
		assert_eq!(free, 20 * DOLLARS);
		let ppha_free = get_balance(1);
		assert_eq!(ppha_free, 20 * DOLLARS);
	});
}

fn mock_asset_id() {
	<pallet_assets::pallet::Pallet<Test> as Create<u64>>::create(
		<Test as pawnshop::Config>::PPhaAssetId::get(),
		1,
		true,
		1,
	);
}

fn get_balance(account_id: u64) -> u128 {
	<pallet_assets::pallet::Pallet<Test> as Inspect<u64>>::balance(
		<Test as pawnshop::Config>::PPhaAssetId::get(),
		&account_id,
	)
}

/*#[test]
fn test_create() {
	// Check this fixed: <https://github.com/Phala-Network/phala-blockchain/issues/285>
	new_test_ext().execute_with(|| {
		set_block_1();
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		PhalaStakePool::on_finalize(1);
		assert_matches!(
			take_events().as_slice(),
			[
				TestEvent::Uniques(pallet_uniques::Event::Created {
					collection: 0,
					creator: _,
					owner: _
				}),
				TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
					issuer: _,
					collection_id: 0
				}),
				TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 0 }),
				TestEvent::Uniques(pallet_uniques::Event::Created {
					collection: 1,
					creator: _,
					owner: _
				}),
				TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
					issuer: _,
					collection_id: 1
				}),
				TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 1 }),
			]
		);
		assert_eq!(
			basepool::Pools::<Test>::get(0),
			Some(PoolProxy::<u64, Balance>::StakePool(StakePool::<
				u64,
				Balance,
			> {
				basepool: basepool::BasePool {
					pid: 0,
					owner: 1,
					total_shares: 0,
					total_value: 0,
					free_stake: 0,
					withdraw_queue: VecDeque::new(),
					value_subscribers: VecDeque::new(),
					cid: 0,
				},
				payout_commission: None,
				owner_reward: 0,
				cap: None,
				workers: vec![],
				cd_workers: vec![],
			})),
		);
		assert_eq!(basepool::PoolCount::<Test>::get(), 2);
	});
}

#[test]
fn test_create_vault() {
	// Check this fixed: <https://github.com/Phala-Network/phala-blockchain/issues/285>
	new_test_ext().execute_with(|| {
		set_block_1();
		assert_ok!(PhalaStakePool::create_vault(Origin::signed(1)));
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		PhalaStakePool::on_finalize(1);
		assert_matches!(
			take_events().as_slice(),
			[
				TestEvent::Uniques(pallet_uniques::Event::Created {
					collection: 0,
					creator: _,
					owner: _
				}),
				TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
					issuer: _,
					collection_id: 0
				}),
				TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 0 }),
				TestEvent::Uniques(pallet_uniques::Event::Created {
					collection: 1,
					creator: _,
					owner: _
				}),
				TestEvent::RmrkCore(pallet_rmrk_core::Event::CollectionCreated {
					issuer: _,
					collection_id: 1
				}),
				TestEvent::PhalaStakePool(Event::PoolCreated { owner: 1, pid: 1 }),
			]
		);
		assert_eq!(
			basepool::Pools::<Test>::get(0),
			Some(PoolProxy::Vault(Vault::<u64, Balance> {
				basepool: basepool::BasePool {
					pid: 0,
					owner: 1,
					total_shares: 0,
					total_value: 0,
					free_stake: 0,
					withdraw_queue: VecDeque::new(),
					value_subscribers: VecDeque::new(),
					cid: 0,
				},
				pool_account_id: 3899606504431772022,
				last_share_price_checkpoint: 0,
				commission: None,
				owner_shares: 0,
				invest_pools: vec![],
			})),
		);
		assert_eq!(basepool::PoolCount::<Test>::get(), 2);
	});
}

#[test]
fn test_mint_nft() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		let pool_info = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			1000 * DOLLARS,
		));

		assert_ok!(PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0));
		let nft_attr = PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0)
			.unwrap()
			.attr;
		assert_eq!(nft_attr.shares, 1000 * DOLLARS);
	});
}

#[test]
fn test_merge_or_init_nft() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		let pool_info = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			1000 * DOLLARS,
		));
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			2000 * DOLLARS,
		));
		let nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		assert_eq!(nftid_arr.len(), 2);
		assert_ok!(PhalaBasePool::merge_or_init_nft_for_staker(
			pool_info.basepool.cid,
			1
		));
		let nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		assert_eq!(nftid_arr.len(), 1);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, nftid_arr[0])
					.unwrap()
					.attr;
			assert_eq!(nft_attr.shares, 3000 * DOLLARS);
		}
		assert_ok!(PhalaBasePool::merge_or_init_nft_for_staker(
			pool_info.basepool.cid,
			2
		));
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, nftid_arr[0])
					.unwrap()
					.attr;
			assert_eq!(nft_attr.shares, 0 * DOLLARS);
		}
	});
}

#[test]
fn test_set_nft_attr() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		let pool_info = ensure_stake_pool::<Test>(0).unwrap();
		assert_ok!(PhalaBasePool::mint_nft(
			pool_info.basepool.cid,
			1,
			1000 * DOLLARS,
		));
		{
			let mut nft_attr_guard =
				PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0).unwrap();
			let mut nft_attr = nft_attr_guard.attr;
			nft_attr.shares = 5000 * DOLLARS;
			nft_attr_guard.attr = nft_attr;
			assert_ok!(nft_attr_guard.save());
		}
		{
			let nft_attr = PhalaBasePool::get_nft_attr_guard(pool_info.basepool.cid, 0)
				.unwrap()
				.attr;
			assert_eq!(nft_attr.shares, 5000 * DOLLARS);
		}
	});
}

#[test]
fn test_remove_stake_from_nft() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			50 * DOLLARS
		));
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		let mut nft_attr =
			PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr;
		assert_eq!(pool.basepool.share_price().unwrap(), 1);
		match PhalaBasePool::remove_stake_from_nft(
			&mut pool.basepool,
			40 * DOLLARS,
			&mut nft_attr,
		) {
			Some((amout, removed_shares)) => return,
			_ => panic!(),
		}
	});
}

#[test]
fn test_contibute() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			30 * DOLLARS
		));

		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		let nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
			.unwrap()
			.attr;
		assert_eq!(nft_attr.shares, 80 * DOLLARS);
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		let nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
			.unwrap()
			.attr;
		assert_eq!(nft_attr.shares, 50 * DOLLARS);
	});
}

#[test]
fn test_contibute_to_vault() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_vault(1); // pid = 0
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(1),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(2),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(1),
			0,
			30 * DOLLARS
		));

		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(1)
		});
		assert_eq!(nftid_arr.len(), 1);
		let pool = ensure_vault::<Test>(0).unwrap();
		let nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
			.unwrap()
			.attr;
		assert_eq!(nft_attr.shares, 80 * DOLLARS);
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
		});
		assert_eq!(nftid_arr.len(), 1);
		let nft_attr = PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
			.unwrap()
			.attr;
		assert_eq!(nft_attr.shares, 50 * DOLLARS);
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 130 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 130 * DOLLARS);
		assert_eq!(vault_info.basepool.free_stake, 130 * DOLLARS);
	});
}

#[test]
fn test_vault_investment() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_vault(3); // pid = 0
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 1
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(2),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			30 * DOLLARS
		));

		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 130 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 130 * DOLLARS);
		assert_eq!(vault_info.basepool.free_stake, 130 * DOLLARS);

		assert_noop!(
			PhalaStakePool::vault_investment(Origin::signed(2), 0, 1, 10 * DOLLARS),
			Error::<Test>::UnauthorizedPoolOwner
		);
		assert_ok!(PhalaStakePool::vault_investment(
			Origin::signed(3),
			0,
			1,
			50 * DOLLARS
		));
		let stakepool_info = ensure_stake_pool::<Test>(1).unwrap();
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(stakepool_info.basepool.cid)
				.collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(stakepool_info.basepool.cid, x)
				.unwrap();
			nft.owner
				== rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(
					vault_info.pool_account_id,
				)
		});
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(nftid_arr.len(), 1);
		let nft_attr =
			PhalaBasePool::get_nft_attr_guard(stakepool_info.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr;
		assert_eq!(nft_attr.shares, 50 * DOLLARS);
		assert_eq!(vault_info.basepool.total_value, 130 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 130 * DOLLARS);
		assert_eq!(vault_info.basepool.free_stake, 80 * DOLLARS);
		assert_eq!(stakepool_info.basepool.total_value, 50 * DOLLARS);
		assert_eq!(stakepool_info.basepool.total_shares, 50 * DOLLARS);
		assert_eq!(stakepool_info.basepool.free_stake, 50 * DOLLARS);
	});
}

#[test]
fn test_withdraw_from_vault() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_vault(3); // pid = 0
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 1
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(2),
			0,
			50 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			30 * DOLLARS
		));
		assert_ok!(PhalaStakePool::vault_investment(
			Origin::signed(3),
			0,
			1,
			80 * DOLLARS
		));
		assert_ok!(PhalaStakePool::withdraw_from_vault(
			Origin::signed(3),
			0,
			80 * DOLLARS
		));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 80 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 80 * DOLLARS);
		assert_eq!(vault_info.basepool.free_stake, 0 * DOLLARS);
		assert_eq!(vault_info.basepool.withdraw_queue.len(), 1);
		let nft_attr = PhalaBasePool::get_nft_attr_guard(
			vault_info.basepool.cid,
			vault_info.basepool.withdraw_queue[0].nft_id,
		)
		.unwrap()
		.attr;
		assert_eq!(nft_attr.shares, 30 * DOLLARS);
	});
}

#[test]
fn test_vault_withdraw() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_vault(3); // pid = 0
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 1
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			500 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(2),
			0,
			500 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			300 * DOLLARS
		));
		assert_ok!(PhalaStakePool::vault_investment(
			Origin::signed(3),
			0,
			1,
			800 * DOLLARS
		));
		let stakepool_info = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(stakepool_info.basepool.total_value, 800 * DOLLARS);
		assert_eq!(stakepool_info.basepool.total_shares, 800 * DOLLARS);
		assert_eq!(stakepool_info.basepool.free_stake, 800 * DOLLARS);
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			1,
			worker_pubkey(1),
			400 * DOLLARS
		));
		assert_ok!(PhalaStakePool::vault_withdraw(
			Origin::signed(3),
			1,
			0,
			600 * DOLLARS
		));
		let stakepool_info = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(stakepool_info.basepool.total_value, 400 * DOLLARS);
		assert_eq!(stakepool_info.basepool.total_shares, 400 * DOLLARS);
		assert_eq!(stakepool_info.basepool.free_stake, 0 * DOLLARS);
		assert_eq!(stakepool_info.basepool.withdraw_queue.len(), 1);
		let nft_attr = PhalaBasePool::get_nft_attr_guard(
			stakepool_info.basepool.cid,
			stakepool_info.basepool.withdraw_queue[0].nft_id,
		)
		.unwrap()
		.attr;
		assert_eq!(nft_attr.shares, 200 * DOLLARS);
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 1300 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 1300 * DOLLARS);
		assert_eq!(vault_info.basepool.free_stake, 900 * DOLLARS);
	});
}

#[test]
fn test_on_reward_for_vault() {
	use crate::mining::pallet::OnReward;
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_vault(3); // pid = 0
		setup_stake_pool_with_workers(1, &[1]);
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			1000 * DOLLARS
		));
		assert_ok!(PhalaStakePool::vault_investment(
			Origin::signed(3),
			0,
			1,
			500 * DOLLARS
		));
		// Staker2 contribute 1000 PHA and start mining
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			1,
			500 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			1,
			worker_pubkey(1),
			500 * DOLLARS
		));
		PhalaStakePool::on_reward(&vec![SettleInfo {
			pubkey: worker_pubkey(1),
			v: FixedPoint::from_num(1u32).to_bits(),
			payout: FixedPoint::from_num(1000u32).to_bits(),
			treasury: 0,
		}]);
		let mut pool = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(pool.basepool.free_stake, 1500 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 2000 * DOLLARS);
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 1500 * DOLLARS);
		assert_eq!(vault_info.basepool.free_stake, 500 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 1000 * DOLLARS);
	});
}

#[test]
fn test_vault_owner_shares() {
	use crate::mining::pallet::OnReward;
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_vault(3); // pid = 0
		assert_ok!(PhalaStakePool::set_vault_payout_pref(
			Origin::signed(3),
			0,
			Permill::from_percent(50)
		));
		setup_stake_pool_with_workers(1, &[1]);
		assert_ok!(PhalaStakePool::contribute_to_vault(
			Origin::signed(3),
			0,
			1000 * DOLLARS
		));
		assert_ok!(PhalaStakePool::vault_investment(
			Origin::signed(3),
			0,
			1,
			500 * DOLLARS
		));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.commission.unwrap(), Permill::from_percent(50));
		assert_ok!(PhalaStakePool::maybe_gain_owner_shares(
			Origin::signed(3),
			0
		));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.owner_shares, 0);
		// Staker2 contribute 1000 PHA and start mining
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			1,
			500 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			1,
			worker_pubkey(1),
			500 * DOLLARS
		));
		PhalaStakePool::on_reward(&vec![SettleInfo {
			pubkey: worker_pubkey(1),
			v: FixedPoint::from_num(1u32).to_bits(),
			payout: FixedPoint::from_num(1000u32).to_bits(),
			treasury: 0,
		}]);
		let mut pool = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(pool.basepool.free_stake, 1500 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 2000 * DOLLARS);
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.basepool.total_value, 1500 * DOLLARS);
		assert_eq!(vault_info.basepool.free_stake, 500 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 1000 * DOLLARS);
		assert_ok!(PhalaStakePool::maybe_gain_owner_shares(
			Origin::signed(3),
			0
		));
		let vault_info = ensure_vault::<Test>(0).unwrap();
		assert_eq!(vault_info.owner_shares, 200 * DOLLARS);
		assert_eq!(vault_info.basepool.total_shares, 1200 * DOLLARS);
	});
}

#[test]
fn test_withdraw() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			1000 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			400 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(2),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(2),
			0,
			800 * DOLLARS
		));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		let mut item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == 2);
		{
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
					.unwrap()
					.attr;
			assert_eq!(nft_attr.shares, 300 * DOLLARS);
			let mut nftid_arr: Vec<NftId> =
				pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
			nftid_arr.retain(|x| {
				let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
				nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(2)
			});
			let user_nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
					.unwrap()
					.attr;
			assert_eq!(user_nft_attr.shares, 200 * DOLLARS);
			assert_ok!(PhalaStakePool::contribute(
				Origin::signed(3),
				0,
				1000 * DOLLARS
			));
			let mut pool = ensure_stake_pool::<Test>(0).unwrap();
			assert_eq!(pool.basepool.withdraw_queue.len(), 0);
			let mut nftid_arr: Vec<NftId> =
				pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
			nftid_arr.retain(|x| {
				let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
				nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(3)
			});
			assert_eq!(nftid_arr.len(), 1);
			let nft_attr =
				PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
					.unwrap()
					.attr;
			assert_eq!(nft_attr.shares, 1000 * DOLLARS);
			assert_eq!(pool.basepool.total_value, 1200 * DOLLARS);
		}
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(3),
			0,
			900 * DOLLARS
		));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		let mut item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == 3);
		let nft_attr =
			PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
				.unwrap()
				.attr;
		assert_eq!(nft_attr.shares, 200 * DOLLARS);
		assert_ok!(PhalaStakePool::withdraw(Origin::signed(3), 0, 50 * DOLLARS));
		let mut nftid_arr: Vec<NftId> =
			pallet_rmrk_core::Nfts::<Test>::iter_key_prefix(0).collect();
		nftid_arr.retain(|x| {
			let nft = pallet_rmrk_core::Nfts::<Test>::get(0, x).unwrap();
			nft.owner == rmrk_traits::AccountIdOrCollectionNftTuple::AccountId(3)
		});
		let user_nft_attr =
			PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, nftid_arr[0])
				.unwrap()
				.attr;
		assert_eq!(user_nft_attr.shares, 250 * DOLLARS);
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		let mut item = pool
			.basepool
			.withdraw_queue
			.clone()
			.into_iter()
			.find(|x| x.user == 3);
		let nft_attr =
			PhalaBasePool::get_nft_attr_guard(pool.basepool.cid, item.unwrap().nft_id)
				.unwrap()
				.attr;
		assert_eq!(nft_attr.shares, 50 * DOLLARS);
	});
}

#[test]
fn test_set_pool_description() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]);
		let str_hello: DescStr =
			("hello").as_bytes().to_vec().try_into().unwrap();
		assert_ok!(PhalaStakePool::set_pool_description(
			Origin::signed(1),
			0,
			str_hello.clone(),
		));
		let list = PhalaStakePool::pool_descriptions(0).unwrap();
		assert_eq!(list, str_hello);
		let str_bye: DescStr =
			("bye").as_bytes().to_vec().try_into().unwrap();
		assert_noop!(
			PhalaStakePool::set_pool_description(Origin::signed(2), 0, str_bye,),
			Error::<Test>::UnauthorizedPoolOwner
		);
	});
}

#[test]
fn test_add_worker() {
	new_test_ext().execute_with(|| {
		set_block_1();
		let worker1 = worker_pubkey(1);
		let worker2 = worker_pubkey(2);

		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker1.clone(),
			ecdh_pubkey(1),
			Some(1)
		));

		// Create a pool (pid = 0)
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		// Bad inputs
		assert_noop!(
			PhalaStakePool::add_worker(Origin::signed(1), 1, worker2.clone()),
			Error::<Test>::WorkerNotRegistered
		);
		assert_noop!(
			PhalaStakePool::add_worker(Origin::signed(2), 0, worker1.clone()),
			Error::<Test>::UnauthorizedOperator
		);
		assert_noop!(
			PhalaStakePool::add_worker(Origin::signed(1), 0, worker1.clone()),
			Error::<Test>::BenchmarkMissing
		);
		// Add benchmark and retry
		PhalaRegistry::internal_set_benchmark(&worker1, Some(1));
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(1),
			0,
			worker1.clone()
		));
		// Check binding
		let subaccount = pool_sub_account(0, &worker_pubkey(1));
		assert_eq!(
			PhalaMining::ensure_worker_bound(&worker_pubkey(1)).unwrap(),
			subaccount,
		);
		assert_eq!(
			PhalaMining::ensure_miner_bound(&subaccount).unwrap(),
			worker_pubkey(1),
		);
		// Check assignments
		assert_eq!(WorkerAssignments::<Test>::get(&worker_pubkey(1)), Some(0));
		// Other bad cases
		assert_noop!(
			PhalaStakePool::add_worker(Origin::signed(1), 100, worker1.clone()),
			basepool::Error::<Test>::PoolDoesNotExist
		);
		// Bind one worker to antoher pool (pid = 1)
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		assert_noop!(
			PhalaStakePool::add_worker(Origin::signed(1), 1, worker1.clone()),
			Error::<Test>::FailedToBindMinerAndWorker
		);
	});
}

#[test]
fn test_start_mining() {
	new_test_ext().execute_with(|| {
		set_block_1();
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		// Cannot start mining without a bound worker
		assert_noop!(
			PhalaStakePool::start_mining(Origin::signed(1), 0, worker_pubkey(1), 0),
			Error::<Test>::WorkerDoesNotExist
		);
		// Basic setup
		setup_workers(2);
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(1),
			0,
			worker_pubkey(1)
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));
		// No enough stake
		assert_noop!(
			PhalaStakePool::start_mining(Origin::signed(1), 0, worker_pubkey(1), 0),
			mining::Error::<Test>::InsufficientStake
		);
		// Too much stake
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(99),
			0,
			30000 * DOLLARS
		));
		assert_noop!(
			PhalaStakePool::start_mining(
				Origin::signed(1),
				0,
				worker_pubkey(1),
				30000 * DOLLARS
			),
			mining::Error::<Test>::TooMuchStake
		);
		// Can start mining normally
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_eq!(PhalaMining::online_miners(), 1);
	});
}

#[test]
fn test_force_unbind() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers_linked_operators(2);
		setup_stake_pool_with_workers(1, &[1]); // pid = 0
		setup_stake_pool_with_workers(2, &[2]); // pid = 1
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			1,
			100 * DOLLARS
		));

		// Pool0: Change the operator to account101 and force unbind (not mining)
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker_pubkey(1),
			ecdh_pubkey(1),
			Some(101)
		));
		let sub_account = pool_sub_account(0, &worker_pubkey(1));
		assert_ok!(PhalaMining::unbind(Origin::signed(101), sub_account));
		// Check worker assignments cleared, and the worker removed from the pool
		assert!(!WorkerAssignments::<Test>::contains_key(&worker_pubkey(1)));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.workers.contains(&worker_pubkey(1)), false);
		// Check the mining is ready
		let miner = PhalaMining::miners(&sub_account).unwrap();
		assert_eq!(miner.state, mining::MinerState::Ready);

		// Pool1: Change the operator to account102 and force unbind (mining)
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(2),
			1,
			worker_pubkey(2),
			100 * DOLLARS
		));
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker_pubkey(2),
			ecdh_pubkey(2),
			Some(102)
		));
		let sub_account = pool_sub_account(1, &worker_pubkey(2));
		assert_ok!(PhalaMining::unbind(Origin::signed(102), sub_account));
		// Check worker assignments cleared, and the worker removed from the pool
		assert!(!WorkerAssignments::<Test>::contains_key(&worker_pubkey(2)));
		let mut pool = ensure_stake_pool::<Test>(1).unwrap();
		assert_eq!(pool.workers.contains(&worker_pubkey(2)), false);
		// Check the mining is stopped
		let miner = PhalaMining::miners(&sub_account).unwrap();
		assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
	});
}

#[test]
fn test_stop_mining() {
	new_test_ext().execute_with(|| {
		set_block_1();
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		// Cannot start mining without a bound worker
		assert_noop!(
			PhalaStakePool::start_mining(Origin::signed(1), 0, worker_pubkey(1), 0),
			Error::<Test>::WorkerDoesNotExist
		);
		// Basic setup
		setup_workers(2);
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(1),
			0,
			worker_pubkey(1)
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::stop_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
		));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
	});
}

#[test]
fn test_for_cdworkers() {
	new_test_ext().execute_with(|| {
		set_block_1();
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		// Cannot start mining without a bound worker
		assert_noop!(
			PhalaStakePool::start_mining(Origin::signed(1), 0, worker_pubkey(1), 0),
			Error::<Test>::WorkerDoesNotExist
		);
		// Basic setup
		setup_workers(2);
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(1),
			0,
			worker_pubkey(1)
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::remove_worker(
			Origin::signed(1),
			0,
			worker_pubkey(1),
		));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
		elapse_cool_down();
		assert_ok!(PhalaStakePool::reclaim_pool_worker(
			Origin::signed(1),
			0,
			worker_pubkey(1),
		));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.cd_workers, []);
	});
}

#[test]
fn test_check_and_maybe_force_withdraw() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(2);
		setup_stake_pool_with_workers(1, &[1, 2]); // pid = 0
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			1000 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			400 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(2),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(2),
			0,
			800 * DOLLARS
		));
		elapse_seconds(864000);
		assert_ok!(PhalaStakePool::stop_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
		));
		assert_ok!(PhalaStakePool::check_and_maybe_force_withdraw(
			Origin::signed(3),
			0
		));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.basepool.free_stake, 0 * DOLLARS);
		assert_eq!(pool.cd_workers, [worker_pubkey(1)]);
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(2),
			0,
			500 * DOLLARS
		));
		elapse_seconds(864000);
		assert_ok!(PhalaStakePool::check_and_maybe_force_withdraw(
			Origin::signed(3),
			0
		));
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.cd_workers, [worker_pubkey(1), worker_pubkey(2)]);
	});
}

#[test]
fn test_on_reward() {
	use crate::mining::pallet::OnReward;
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]);

		assert_ok!(PhalaStakePool::set_payout_pref(
			Origin::signed(1),
			0,
			Permill::from_percent(50)
		));
		// Staker2 contribute 1000 PHA and start mining
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			2000 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			1000 * DOLLARS
		));
		PhalaStakePool::on_reward(&vec![SettleInfo {
			pubkey: worker_pubkey(1),
			v: FixedPoint::from_num(1u32).to_bits(),
			payout: FixedPoint::from_num(2000u32).to_bits(),
			treasury: 0,
		}]);
		let mut pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.owner_reward, 1000 * DOLLARS);
		assert_eq!(pool.basepool.free_stake, 2000 * DOLLARS);
		assert_eq!(pool.basepool.total_value, 3000 * DOLLARS);
	});
}

#[test]
fn test_pool_cap() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid = 0

		assert_eq!(ensure_stake_pool::<Test>(0).unwrap().cap, None);
		// Pool existence
		assert_noop!(
			PhalaStakePool::set_cap(Origin::signed(2), 100, 1),
			basepool::Error::<Test>::PoolDoesNotExist,
		);
		// Owner only
		assert_noop!(
			PhalaStakePool::set_cap(Origin::signed(2), 0, 1),
			Error::<Test>::UnauthorizedPoolOwner,
		);
		// Cap to 1000 PHA
		assert_ok!(PhalaStakePool::set_cap(
			Origin::signed(1),
			0,
			1000 * DOLLARS
		));
		assert_eq!(
			ensure_stake_pool::<Test>(0).unwrap().cap,
			Some(1000 * DOLLARS)
		);
		// Check cap shouldn't be less than the current stake
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));
		assert_noop!(
			PhalaStakePool::set_cap(Origin::signed(1), 0, 99 * DOLLARS),
			Error::<Test>::InadequateCapacity,
		);
		// Stake to the cap
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			900 * DOLLARS
		));
		// Exceed the cap
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(2), 0, 900 * DOLLARS),
			Error::<Test>::StakeExceedsCapacity,
		);

		// Can stake exceed the cap to swap the withdrawing stake out, as long as the cap
		// can be maintained after the contribution
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			1000 * DOLLARS
		));
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(1),
			0,
			1000 * DOLLARS
		));
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(2), 0, 1001 * DOLLARS),
			Error::<Test>::StakeExceedsCapacity
		);
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			1000 * DOLLARS
		));
	});
}

#[test]
fn test_stake() {
	new_test_ext().execute_with(|| {
		set_block_1();
		let worker1 = worker_pubkey(1);
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker1.clone(),
			ecdh_pubkey(1),
			Some(1)
		));

		assert_ok!(PhalaStakePool::create(Origin::signed(1))); // pid = 0
		assert_ok!(PhalaStakePool::create(Origin::signed(2))); // pid = 1

		// Stake normally
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			1 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			10 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			1,
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			1,
			1000 * DOLLARS
		));
		// Check total stake
		assert_eq!(
			ensure_stake_pool::<Test>(0).unwrap().basepool.total_value,
			11 * DOLLARS
		);
		assert_eq!(
			ensure_stake_pool::<Test>(1).unwrap().basepool.total_value,
			1100 * DOLLARS
		);

		// Pool existence
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(1), 100, 1 * DOLLARS),
			basepool::Error::<Test>::PoolDoesNotExist
		);
		// Dust contribution
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(1), 0, 1),
			Error::<Test>::InsufficientContribution
		);
		// Stake more than account1 has
		assert_noop!(
			PhalaStakePool::contribute(
				Origin::signed(1),
				0,
				Balances::usable_balance(1) + 1
			),
			Error::<Test>::InsufficientBalance,
		);
	});
}

#[test]
fn test_claim_owner_rewards() {
	use crate::mining::pallet::OnReward;
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid = 0
		assert_ok!(PhalaStakePool::set_payout_pref(
			Origin::signed(1),
			0,
			Permill::from_percent(50)
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			400 * DOLLARS
		));
		PhalaStakePool::on_reward(&vec![SettleInfo {
			pubkey: worker_pubkey(1),
			v: FixedPoint::from_num(1u32).to_bits(),
			payout: FixedPoint::from_num(1000u32).to_bits(),
			treasury: 0,
		}]);
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.owner_reward, 500 * DOLLARS);
		assert_ok!(PhalaStakePool::claim_owner_rewards(Origin::signed(1), 0, 1));
		let pool = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool.owner_reward, 0 * DOLLARS);
	});
}
#[test]
fn test_staker_whitelist() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]);

		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			40 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			40 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(3),
			0,
			40 * DOLLARS
		));
		assert_ok!(PhalaStakePool::add_staker_to_whitelist(
			Origin::signed(1),
			0,
			2,
		));
		let whitelist = PhalaStakePool::pool_whitelist(0).unwrap();
		assert_eq!(whitelist, [2]);
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			10 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			40 * DOLLARS
		));
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(3), 0, 40 * DOLLARS),
			Error::<Test>::NotInContributeWhitelist
		);
		assert_ok!(PhalaStakePool::add_staker_to_whitelist(
			Origin::signed(1),
			0,
			3,
		));
		let whitelist = PhalaStakePool::pool_whitelist(0).unwrap();
		assert_eq!(whitelist, [2, 3]);
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(3),
			0,
			20 * DOLLARS,
		));
		PhalaStakePool::remove_staker_from_whitelist(Origin::signed(1), 0, 2);
		let whitelist = PhalaStakePool::pool_whitelist(0).unwrap();
		assert_eq!(whitelist, [3]);
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(2), 0, 20 * DOLLARS,),
			Error::<Test>::NotInContributeWhitelist
		);
		PhalaStakePool::remove_staker_from_whitelist(Origin::signed(1), 0, 3);
		assert!(PhalaStakePool::pool_whitelist(0).is_none());
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(3),
			0,
			20 * DOLLARS,
		));
	});
}

#[test]
fn issue_388_double_stake() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]);

		let balance = Balances::usable_balance(&1);
		assert_ok!(PhalaStakePool::contribute(Origin::signed(1), 0, balance));
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(1), 0, balance),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn test_full_procedure() {
	new_test_ext().execute_with(|| {
		set_block_1();
		let worker1 = worker_pubkey(1);
		let worker2 = worker_pubkey(2);
		let worker3 = worker_pubkey(3);
		// Register workers
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker1.clone(),
			ecdh_pubkey(1),
			Some(1)
		));
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker2.clone(),
			ecdh_pubkey(2),
			Some(1)
		));
		assert_ok!(PhalaRegistry::force_register_worker(
			Origin::root(),
			worker3.clone(),
			ecdh_pubkey(3),
			Some(1)
		));
		PhalaRegistry::internal_set_benchmark(&worker1, Some(1));
		PhalaRegistry::internal_set_benchmark(&worker2, Some(1));
		PhalaRegistry::internal_set_benchmark(&worker3, Some(1));

		// Create a pool (pid = 0)
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		let _ = take_events();
		assert_ok!(PhalaStakePool::set_payout_pref(
			Origin::signed(1),
			0,
			Permill::from_percent(50)
		));
		assert_eq!(
			take_events().as_slice(),
			[TestEvent::PhalaStakePool(Event::PoolCommissionSet {
				pid: 0,
				commission: 1000_000u32 * 50 / 100
			})]
		);
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(1),
			0,
			worker1.clone()
		));
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(1),
			0,
			worker2.clone()
		));
		// Create a pool (pid = 1)
		assert_ok!(PhalaStakePool::create(Origin::signed(1)));
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(1),
			1,
			worker3.clone()
		));
		// Contribute 300 PHA to pool0, 300 to pool1
		assert_ok!(PhalaStakePool::set_cap(Origin::signed(1), 0, 300 * DOLLARS));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(1),
			1,
			300 * DOLLARS
		));
		assert_eq!(
			ensure_stake_pool::<Test>(0).unwrap().basepool.total_value,
			100 * DOLLARS
		);

		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			200 * DOLLARS
		));
		assert_eq!(
			ensure_stake_pool::<Test>(0).unwrap().basepool.total_value,
			300 * DOLLARS
		);
		// Shouldn't exceed the pool cap
		assert_noop!(
			PhalaStakePool::contribute(Origin::signed(1), 0, 100 * DOLLARS),
			Error::<Test>::StakeExceedsCapacity
		);
		// Start mining on pool0 (stake 100 for worker1, 100 for worker2)
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker1.clone(),
			100 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker2.clone(),
			100 * DOLLARS
		));
		assert_eq!(PhalaMining::online_miners(), 2);
		// Withdraw 100 free funds
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(1),
			0,
			100 * DOLLARS
		));

		// TODO: check queued withdraw
		//   - withdraw 100 PHA
		//   - stop a worker
		//   - wait CD, withdraw succeeded
		//   - withdraw another 100 PHA
		//   - wait 3d, force stop
		//   - wait 7d, withdraw succeeded

		let sub_account1: u64 = pool_sub_account(0, &worker1);
		let sub_account2: u64 = pool_sub_account(0, &worker2);

		// Slash pool 0 to 90%
		let miner0 = PhalaMining::miners(sub_account1).unwrap();
		let ve = FixedPoint::from_bits(miner0.ve);
		simulate_v_update(1, (ve * fp!(0.9)).to_bits());

		// Stop mining
		assert_ok!(PhalaStakePool::stop_mining(
			Origin::signed(1),
			0,
			worker1.clone()
		));
		assert_ok!(PhalaStakePool::stop_mining(
			Origin::signed(1),
			0,
			worker2.clone()
		));
		assert_eq!(PhalaMining::online_miners(), 0);
		let miner1 = PhalaMining::miners(&sub_account1).unwrap();
		let miner2 = PhalaMining::miners(&sub_account2).unwrap();
		assert_eq!(miner1.state, mining::MinerState::MiningCoolingDown);
		assert_eq!(miner2.state, mining::MinerState::MiningCoolingDown);
		// Wait the cool down period
		elapse_cool_down();
		assert_ok!(PhalaStakePool::reclaim_pool_worker(
			Origin::signed(1),
			0,
			worker1
		));
		assert_ok!(PhalaStakePool::reclaim_pool_worker(
			Origin::signed(1),
			0,
			worker2
		));
		// 90% stake get returned from pool 0
		let pool0 = ensure_stake_pool::<Test>(0).unwrap();
		// TODO(hangyin): enable when stake is not skipped
		// assert_eq!(pool0.free_stake, 189_999999999999);
		assert_eq!(pool0.basepool.free_stake, 200000000000000);
		// Withdraw the stakes
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(2),
			0,
			200 * DOLLARS
		));
		// Stop pool1 and withdraw stake as well
		assert_ok!(PhalaStakePool::withdraw(
			Origin::signed(1),
			1,
			300 * DOLLARS
		));
		// Settle everything
		assert!(Balances::locks(1).is_empty());
		assert!(Balances::locks(2).is_empty());
		// Remove worker from the pools
		assert_ok!(PhalaStakePool::remove_worker(
			Origin::signed(1),
			0,
			worker1.clone()
		));
		assert_ok!(PhalaStakePool::remove_worker(
			Origin::signed(1),
			0,
			worker2.clone()
		));
	});
}

#[test]
fn issue500_should_not_restart_worker_in_cool_down() {
	new_test_ext().execute_with(|| {
		set_block_1();
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid=0
								// Start a worker as usual
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			1500 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			1500 * DOLLARS
		));
		assert_ok!(PhalaStakePool::stop_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1)
		));
		let subaccount: u64 = pool_sub_account(0, &worker_pubkey(1));
		let miner = PhalaMining::miners(subaccount).unwrap();
		assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
		// Remove the worker
		assert_ok!(PhalaStakePool::remove_worker(
			Origin::signed(1),
			0,
			worker_pubkey(1)
		));
		let miner = PhalaMining::miners(subaccount).unwrap();
		assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
		// Now the stake is still in CD state. We cannot add it back.
		assert_noop!(
			PhalaStakePool::add_worker(Origin::signed(1), 0, worker_pubkey(1)),
			Error::<Test>::FailedToBindMinerAndWorker,
		);
		let miner = PhalaMining::miners(subaccount).unwrap();
		assert_eq!(miner.state, mining::MinerState::MiningCoolingDown);
	});
}

#[test]
fn subaccount_preimage() {
	new_test_ext().execute_with(|| {
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid=0

		let subaccount: u64 = pool_sub_account(0, &worker_pubkey(1));
		let preimage = SubAccountPreimages::<Test>::get(subaccount);
		assert_eq!(preimage, Some((0, worker_pubkey(1))));
	});
}

#[test]
fn restart_mining_should_work() {
	new_test_ext().execute_with(|| {
		setup_workers(1);
		setup_stake_pool_with_workers(1, &[1]); // pid=0
		assert_ok!(PhalaStakePool::contribute(
			Origin::signed(2),
			0,
			2000 * DOLLARS
		));
		assert_ok!(PhalaStakePool::start_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			1500 * DOLLARS
		));
		// Bad cases
		assert_noop!(
			PhalaStakePool::restart_mining(
				Origin::signed(1),
				0,
				worker_pubkey(1),
				500 * DOLLARS
			),
			Error::<Test>::CannotRestartWithLessStake
		);
		assert_noop!(
			PhalaStakePool::restart_mining(
				Origin::signed(1),
				0,
				worker_pubkey(1),
				1500 * DOLLARS
			),
			Error::<Test>::CannotRestartWithLessStake
		);
		// Happy path
		let pool0 = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool0.basepool.free_stake, 500 * DOLLARS);
		assert_ok!(PhalaStakePool::restart_mining(
			Origin::signed(1),
			0,
			worker_pubkey(1),
			1501 * DOLLARS
		));
		let pool0 = ensure_stake_pool::<Test>(0).unwrap();
		assert_eq!(pool0.basepool.free_stake, 499 * DOLLARS);
	});
}

fn setup_stake_pool_with_workers(owner: u64, workers: &[u8]) -> u64 {
	let pid = PhalaBasePool::pool_count();
	assert_ok!(PhalaStakePool::create(Origin::signed(owner)));
	for id in workers {
		assert_ok!(PhalaStakePool::add_worker(
			Origin::signed(owner),
			pid,
			worker_pubkey(*id),
		));
	}
	pid
}

fn setup_vault(owner: u64) -> u64 {
	let pid = PhalaBasePool::pool_count();
	assert_ok!(PhalaStakePool::create_vault(Origin::signed(owner)));
	pid
}

fn simulate_v_update(worker: u8, v_bits: u128) {
	use phala_types::messaging::{
		DecodedMessage, MessageOrigin, MiningInfoUpdateEvent, SettleInfo, Topic,
	};
	let block = System::block_number();
	let now = Timestamp::now();
	assert_ok!(PhalaMining::on_gk_message_received(DecodedMessage::<
		MiningInfoUpdateEvent<BlockNumber>,
	> {
		sender: MessageOrigin::Gatekeeper,
		destination: Topic::new(*b"^phala/mining/update"),
		payload: MiningInfoUpdateEvent::<BlockNumber> {
			block_number: block,
			timestamp_ms: now,
			offline: vec![],
			recovered_to_online: vec![],
			settle: vec![SettleInfo {
				pubkey: worker_pubkey(worker),
				v: v_bits,
				payout: 0,
				treasury: 0,
			}],
		},
	}));
}*/
