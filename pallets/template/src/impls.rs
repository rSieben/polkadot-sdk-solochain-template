use super::*;
use frame::primitives::BlakeTwo256;
use frame::traits::{Currency, ExistenceRequirement::KeepAlive, Hash};

impl<T: Config> Pallet<T> {
	pub fn gen_game_key() -> [u8; 32] {
		let unique_payload = (
			frame_system::Pallet::<T>::parent_hash(),
			frame_system::Pallet::<T>::block_number(),
			frame_system::Pallet::<T>::extrinsic_index(),
			GameCount::<T>::get(),
		);
		let hash: [u8; 32] = BlakeTwo256::hash_of(&unique_payload).into();
		hash
	}

	pub fn do_transfer(from: T::AccountId, to: T::AccountId, id: [u8; 32]) -> DispatchResult {
		ensure!(!from.eq(&to), Error::<T>::TransferToSelf);

		let mut game = Games::<T>::get(id).ok_or(Error::<T>::GameNotFound)?;

		ensure!(game.owner.eq(&from), Error::<T>::NotAuthorized);

		let mut to_owned = GamesOwnedBy::<T>::get(&to);
		to_owned.try_push(id).map_err(|_| Error::<T>::TooManyGamesOwned)?;

		let mut from_owned = GamesOwnedBy::<T>::get(&from);
		let index = from_owned.iter().position(|&x| x == id).ok_or(Error::<T>::NotAuthorized)?;
		from_owned.remove(index);

		game.owner = to.clone();
		Games::<T>::insert(id, game);
		GamesOwnedBy::<T>::insert(&to, to_owned);
		GamesOwnedBy::<T>::insert(&from, from_owned);

		Self::deposit_event(Event::<T>::GameTransferred { from, to, id });
		Ok(())
	}

	pub fn mint(owner: T::AccountId, id: [u8; 32]) -> DispatchResult {
		let game = Game { key_data: id, owner: owner.clone(), value: None };

		ensure!(!Games::<T>::contains_key(id), Error::<T>::DuplicatedGame);

		let current_count = GameCount::<T>::get();
		let updated_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyGames)?;
		GameCount::<T>::set(updated_count);
		Games::<T>::insert(id, game);
		GamesOwnedBy::<T>::try_mutate(&owner, |games| {
			games.try_push(id).map_err(|_| Error::<T>::TooManyGamesOwned)
		})?;

		Self::deposit_event(Event::<T>::GameCreated { owner, id });
		Ok(())
	}

	pub fn do_set_price(
		from: T::AccountId,
		id: [u8; 32],
		price: Option<T::Balance>,
	) -> DispatchResult {
		let mut game = Games::<T>::get(id).ok_or(Error::<T>::GameNotFound)?;
		ensure!(game.owner == from, Error::<T>::NotAuthorized);

		game.value = price;
		Games::<T>::insert(id, game);

		Self::deposit_event(Event::<T>::PriceUpdated { owner: from, id, price });
		Ok(())
	}

	pub fn do_buy_game(buyer: T::AccountId, id: [u8; 32], max_price: T::Balance) -> DispatchResult {
		let game = Games::<T>::get(id).ok_or(Error::<T>::GameNotFound)?;

		let price = game.value.ok_or(Error::<T>::NotForSale)?;
		ensure!(price <= max_price, Error::<T>::PriceTooLow);
		ensure!(game.owner != buyer, Error::<T>::TransferToSelf);

		pallet_balances::Pallet::<T>::transfer(&buyer, &game.owner, price, KeepAlive)?;
		Self::do_transfer(game.owner.clone(), buyer.clone(), id)?;
		Self::do_set_price(buyer.clone(), id, None)?;

		Self::deposit_event(Event::<T>::GameSold { buyer, id, price });
		Ok(())
	}
}
