#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

mod weights;
pub use weights::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(core::marker::PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Value<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ValueUpdated { owner: T::AccountId, old: u32, new: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T>::WeightInfo::set_value(*value))]
		pub fn set_value(origin: OriginFor<T>, value: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let old = Value::<T>::get().unwrap_or(0);
			Value::<T>::put(value);
			Self::deposit_event(Event::ValueUpdated { owner: who, old, new: value });

			Ok(())
		}
	}
}
