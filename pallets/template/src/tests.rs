#![cfg(test)]

use crate::*;
use crate::{self as pallet_games};
use frame::deps::sp_io;
use frame::runtime::prelude::*;
use frame::testing_prelude::*;
use frame::traits::fungible::*;

type Balance = u64;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

const ALICE: u64 = 1;
const BOB: u64 = 2;

const DEFAULT_GAME: Game<TestRuntime> = Game { key_data: [0u8; 32], owner: 0, value: None };

construct_runtime! {
	pub struct TestRuntime {
		System: frame_system,
		PalletBalances: pallet_balances,
		PalletGames: pallet_games,
	}
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for TestRuntime {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for TestRuntime {
	type AccountStore = System;
	type Balance = Balance;
}

impl pallet_games::Config for TestRuntime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<TestRuntime>::default()
		.build_storage()
		.unwrap()
		.into()
}

#[test]
fn balances_functionality_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(PalletBalances::mint_into(&ALICE, 100));
		assert_ok!(PalletBalances::mint_into(&BOB, 100));
	});
}

#[test]
fn verify_signed_origin_for_game_creation() {
	new_test_ext().execute_with(|| {
		assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		assert_noop!(PalletGames::create_game(RuntimeOrigin::none()), DispatchError::BadOrigin);
	});
}

#[test]
fn event_emitted_on_game_creation() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		let last_event = System::events().pop().expect("Event expected").event;
		match last_event {
			RuntimeEvent::PalletGames(Event::GameCreated { owner, .. }) => {
				assert_eq!(owner, ALICE);
			},
			_ => panic!("unexpected event"),
		}
	});
}

#[test]
fn game_counter_increases_correctly() {
	new_test_ext().execute_with(|| {
		assert_eq!(GameCount::<TestRuntime>::get(), u32::default());
		GameCount::<TestRuntime>::set(1337u32);
		assert_eq!(GameCount::<TestRuntime>::get(), 1337u32);
		GameCount::<TestRuntime>::put(1336u32);
		assert_ne!(GameCount::<TestRuntime>::get(), 1337u32);
		assert_eq!(GameCount::<TestRuntime>::get(), 1336u32);
	});
}

#[test]
fn test_game_counter_increment_on_game_creation() {
	new_test_ext().execute_with(|| {
		assert_eq!(GameCount::<TestRuntime>::get(), u32::default());
		System::set_block_number(1);
		assert_ok!(PalletGames::mint(ALICE, [0u8; 32]));
		assert_eq!(GameCount::<TestRuntime>::get(), 1);
	});
}

#[test]
fn validate_game_added_to_map_on_creation() {
	new_test_ext().execute_with(|| {
		let zero_key = [0u8; 32];
		assert!(!Games::<TestRuntime>::contains_key(zero_key));
		Games::<TestRuntime>::insert(zero_key, DEFAULT_GAME);
		assert!(Games::<TestRuntime>::contains_key(zero_key));
	});
}

#[test]
fn game_map_increments_on_successful_creation() {
	new_test_ext().execute_with(|| {
		assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		assert_eq!(Games::<TestRuntime>::iter().count(), 1);
	});
}

#[test]
fn prevent_duplicate_game_creation() {
	new_test_ext().execute_with(|| {
		assert_ok!(PalletGames::mint(ALICE, [0u8; 32]));
		assert_eq!(Games::<TestRuntime>::iter().count(), 1);
		assert_noop!(PalletGames::mint(BOB, [0u8; 32]), Error::<TestRuntime>::DuplicatedGame);
		assert_eq!(Games::<TestRuntime>::iter().count(), 1);
	});
}

#[test]
fn test_game_struct_encoding_and_decoding() {
	new_test_ext().execute_with(|| {
		let game = DEFAULT_GAME;
		let bytes = game.encode();
		let _decoded_game = Game::<TestRuntime>::decode(&mut &bytes[..]).unwrap();
		assert!(Game::<TestRuntime>::max_encoded_len() > 0);
		let _info = Game::<TestRuntime>::type_info();
	});
}

#[test]
fn verify_owner_in_game_struct_after_creation() {
	new_test_ext().execute_with(|| {
		assert_ok!(PalletGames::mint(1337, [42u8; 32]));
		let game = Games::<TestRuntime>::get([42u8; 32]).unwrap();
		assert_eq!(game.owner, 1337);
		assert_eq!(game.key_data, [42u8; 32]);
	});
}

#[test]
fn validate_multiple_games_owned_by_user() {
	new_test_ext().execute_with(|| {
		assert_eq!(GamesOwnedBy::<TestRuntime>::get(ALICE).len(), 0);
		assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		assert_eq!(GamesOwnedBy::<TestRuntime>::get(ALICE).len(), 2);
	});
}

#[test]
fn prevent_user_from_owning_too_many_games() {
	new_test_ext().execute_with(|| {
		for _ in 0..100 {
			assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		}
		assert_noop!(
			PalletGames::create_game(RuntimeOrigin::signed(1)),
			Error::<TestRuntime>::TooManyGamesOwned
		);
	});
}

#[test]
fn transfer_game_emits_event_successfully() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		let game_id = Games::<TestRuntime>::iter_keys().next().unwrap();
		assert_ok!(PalletGames::transfer_game(RuntimeOrigin::signed(ALICE), BOB, game_id));
		System::assert_last_event(
			Event::<TestRuntime>::GameTransferred { from: ALICE, to: BOB, id: game_id }.into(),
		);
	});
}

#[test]
fn game_transfer_logic_verification() {
	new_test_ext().execute_with(|| {
		assert_ok!(PalletGames::create_game(RuntimeOrigin::signed(ALICE)));
		let game = Games::<TestRuntime>::iter_values().next().unwrap();
		let game_id = game.key_data;

		assert_eq!(game.owner, ALICE);
		assert_eq!(GamesOwnedBy::<TestRuntime>::get(ALICE), vec![game_id]);
		assert_eq!(GamesOwnedBy::<TestRuntime>::get(BOB), vec![]);

		assert_noop!(
			PalletGames::do_transfer(ALICE, ALICE, game_id),
			Error::<TestRuntime>::TransferToSelf
		);

		assert_noop!(
			PalletGames::do_transfer(ALICE, BOB, [1u8; 32]),
			Error::<TestRuntime>::GameNotFound
		);

		assert_noop!(
			PalletGames::do_transfer(BOB, ALICE, game_id),
			Error::<TestRuntime>::NotAuthorized
		);
	});
}
