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

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(core::marker::PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MaxAttributeKeySize: Get<u32>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Value<T> = StorageValue<_, u32>;

    /// ERC1155
    /// Collection -> ASSET -> quantidade
    /// Spring           -> NFT#1 -> 1 
    /// LohannCollection -> NFT#1 -> 1
    /// LohannCollection -> NFT#2 -> 1
    /// LohannCollection -> NFT#3 -> 1
    /// LohannFungible   -> REAL -> 100
    #[pallet::storage]
	pub type TotalSupply<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, <T>::MaxAttributeKeySize>, // Collection
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<32>>, // Asset > Fungible ou Non-Fungible
        u32
    >;

    #[pallet::storage]
    pub type Balance<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, BoundedVec<u8, <T>::MaxAttributeKeySize>>, // Collection
            NMapKey<Blake2_128Concat, T::AccountId>, // Conta do usuario
            NMapKey<Twox64Concat, BoundedVec<u8, ConstU32<32>>> // Asset
        ),
        u32
    >;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ValueUpdated { owner: T::AccountId, old: u32, new: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {
        /// Collection empty
        CollectionIsEmpty,
        /// Asset empty
        AssetIsEmpty,
    }

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

        #[pallet::call_index(1)]
		#[pallet::weight(<T>::WeightInfo::mint(collection.len() as u32))]
		pub fn mint(
                origin: OriginFor<T>,
                collection: BoundedVec<u8, T::MaxAttributeKeySize>,
                asset: BoundedVec<u8, ConstU32<32>>,
                recipient: T::AccountId,
                amount: u32,
        ) -> DispatchResult {
			let _who = ensure_signed(origin)?;

            if collection.is_empty() {
                return Err(Error::<T>::CollectionIsEmpty.into());
            }
            if asset.is_empty() {
                return Err(Error::<T>::AssetIsEmpty.into());
            }
            
            // Incrementou o total supply
			TotalSupply::<T>::mutate_exists(&collection, &asset, |maybe_total_supply| {
                if let Some(total_supply) = maybe_total_supply {
                    *total_supply += amount;
                } else {
                    *maybe_total_supply = Some(amount);
                }
            });

            // Incrementar o saldo do usuario
            Balance::<T>::mutate_exists((&collection, &recipient, &asset), |maybe_balance| {
                if let Some(balance) = maybe_balance {
                    *balance += amount;
                } else {
                    *maybe_balance = Some(amount);
                }
            });

			Ok(())
		}

        #[pallet::call_index(2)]
		// #[pallet::weight(<T>::WeightInfo::set_value(0))]
		pub fn destroy_collection(
                origin: OriginFor<T>,
                collection: BoundedVec<u8, T::MaxAttributeKeySize>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // let _ = Balance::<T>::clear_prefix(&collection, 10, None);
            let _ = TotalSupply::<T>::clear_prefix(&collection, 10, None);

            Ok(())
        }

	}
}
