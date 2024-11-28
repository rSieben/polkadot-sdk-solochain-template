#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[cfg(test)]
use crate::Pallet as BenchmarkTemplate;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;
    use scale_info::prelude::vec;

	#[benchmark]
	fn set_value(x: Linear<1, 1_000>) {
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		set_value(RawOrigin::Signed(caller), x);
	}

    #[benchmark]
	fn mint(x: Linear<1, 10000>) {
		let caller: T::AccountId = whitelisted_caller();
		let recipient: T::AccountId = whitelisted_caller();
        let collection = vec![1u8; x as usize];
        let asset = vec![2u8; 32];
		#[extrinsic_call]
		mint(RawOrigin::Signed(caller), collection.try_into().unwrap(), asset.try_into().unwrap(), recipient, 10);
	}

	impl_benchmark_test_suite!(BenchmarkTemplate, crate::mock::new_test_ext(), crate::mock::Test);
}
