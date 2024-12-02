#![cfg_attr(not(feature = "std"), no_std)]

use frame::prelude::*;
use frame::primitives::BlakeTwo256;
use frame::traits::Hash;
pub use pallet::*;

#[cfg(test)]
mod mock;

mod impls;
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(core::marker::PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
	}

	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Game<T: Config> {
		pub key_data: [u8; 32],
		pub owner: T::AccountId,
		pub value: Option<T::Balance>,
	}

	#[pallet::storage]
	pub type GlobalValue<T> = StorageValue<_, u32>;

	#[pallet::storage]
	pub(super) type GameCount<T: Config> = StorageValue<Value = u32, QueryKind = ValueQuery>;

	#[pallet::storage]
	pub(super) type Games<T: Config> = StorageMap<Key = [u8; 32], Value = Game<T>>;

	#[pallet::storage]
	pub(super) type GamesOwnedBy<T: Config> = StorageMap<
		Key = T::AccountId,
		Value = BoundedVec<[u8; 32], ConstU32<100>>,
		QueryKind = ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ValueUpdated { value: u32, account: T::AccountId },
		GameCreated { owner: T::AccountId, id: [u8; 32] },
		GameTransferred { from: T::AccountId, to: T::AccountId, id: [u8; 32] },
		PriceUpdated { owner: T::AccountId, id: [u8; 32], price: Option<T::Balance> },
		GameSold { buyer: T::AccountId, id: [u8; 32], price: T::Balance },
	}

	#[pallet::error]
	pub enum Error<T> {
		NoValueStored,
		Overflow,
		TooManyGames,
		DuplicatedGame,
		GameNotFound,
		TooManyGamesOwned,
		NotAuthorized,
		TransferToSelf,
		NotForSale,
		PriceTooLow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn create_game(origin: OriginFor<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let key_data = Self::generate_key_data();
			Self::mint_new_game(caller, key_data)?;
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn transfer_game(
			origin: OriginFor<T>,
			to: T::AccountId,
			id: [u8; 32],
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			Self::perform_transfer(from, to, id)?;
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn set_game_price(
			origin: OriginFor<T>,
			id: [u8; 32],
			price: Option<T::Balance>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::update_price(owner, id, price)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn buy_game(
			origin: OriginFor<T>,
			id: [u8; 32],
			max_price: T::Balance,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			Self::purchase_game(buyer, id, max_price)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn generate_key_data() -> [u8; 32] {
			let unique_payload = (
				frame_system::Pallet::<T>::parent_hash(),
				frame_system::Pallet::<T>::block_number(),
				frame_system::Pallet::<T>::extrinsic_index(),
				GameCount::<T>::get(),
			);
			let hash: [u8; 32] = BlakeTwo256::hash_of(&unique_payload).into();
			hash
		}

		fn mint_new_game(owner: T::AccountId, key_data: [u8; 32]) -> DispatchResult {
			let current_count = GameCount::<T>::get();
			let new_count = current_count.checked_add(1).ok_or(Error::<T>::TooManyGames)?;
			GameCount::<T>::set(new_count);

			ensure!(!Games::<T>::contains_key(key_data), Error::<T>::DuplicatedGame);

			let new_game = Game { key_data, owner: owner.clone(), value: None };
			Games::<T>::insert(key_data, new_game);
			GamesOwnedBy::<T>::try_mutate(&owner, |owned_games| {
				owned_games.try_push(key_data).map_err(|_| Error::<T>::TooManyGamesOwned)
			})?;
			Self::deposit_event(Event::GameCreated { owner, id: key_data });
			Ok(())
		}

		fn perform_transfer(from: T::AccountId, to: T::AccountId, id: [u8; 32]) -> DispatchResult {
			ensure!(!from.eq(&to), Error::<T>::TransferToSelf);
			let game = Games::<T>::get(id).ok_or(Error::<T>::GameNotFound)?;
			ensure!(game.owner == from, Error::<T>::NotAuthorized);

			Games::<T>::try_mutate(id, |game| -> DispatchResult {
				let g = game.as_mut().ok_or(Error::<T>::GameNotFound)?;
				g.owner = to.clone();
				Ok(())
			})?;

			GamesOwnedBy::<T>::mutate(&from, |owned_games| {
				if let Some(index) = owned_games.iter().position(|x| *x == id) {
					owned_games.remove(index);
				}
			});

			GamesOwnedBy::<T>::try_mutate(&to, |owned_games| {
				owned_games.try_push(id).map_err(|_| Error::<T>::TooManyGamesOwned)
			})?;

			Self::deposit_event(Event::GameTransferred { from, to, id });
			Ok(())
		}

		fn update_price(
			owner: T::AccountId,
			id: [u8; 32],
			price: Option<T::Balance>,
		) -> DispatchResult {
			let mut game = Games::<T>::get(id).ok_or(Error::<T>::GameNotFound)?;
			ensure!(game.owner == owner, Error::<T>::NotAuthorized);

			game.value = price;
			Games::<T>::insert(id, game);

			Self::deposit_event(Event::PriceUpdated { owner, id, price });
			Ok(())
		}

		fn purchase_game(
			buyer: T::AccountId,
			id: [u8; 32],
			max_price: T::Balance,
		) -> DispatchResult {
			let game = Games::<T>::get(id).ok_or(Error::<T>::GameNotFound)?;
			ensure!(game.value.unwrap_or_default() <= max_price, Error::<T>::PriceTooLow);

			let seller = game.owner.clone();
			let price = game.value.unwrap_or_default();

			ensure!(seller != buyer, Error::<T>::TransferToSelf);

			Self::perform_transfer(seller.clone(), buyer.clone(), id)?;
			Self::deposit_event(Event::GameSold { buyer, id, price });
			Ok(())
		}
	}
}
