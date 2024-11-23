#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[cfg(test)]
use crate::Pallet as BenchmarkTemplate;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_value(x: Linear<1, 1_000>) {
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		set_value(RawOrigin::Signed(caller), x);
	}

	impl_benchmark_test_suite!(BenchmarkTemplate, crate::mock::new_test_ext(), crate::mock::Test);
}
