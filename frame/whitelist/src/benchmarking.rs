// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Whitelist pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::benchmarks;
use frame_support::ensure;
use sp_runtime::{traits::Hash, Perbill};
use std::convert::TryInto;

#[cfg(test)]
use crate::Pallet as Whitelist;

benchmarks! {
	whitelist_call {
		let origin = T::WhitelistOrigin::successful_origin();
		let call_hash = Default::default();
	}: _<T::Origin>(origin, call_hash)
	verify {
		ensure!(
			WhitelistedCall::<T>::contains_key(call_hash),
			"call not whitelisted"
		);
		ensure!(
			T::PreimageProvider::preimage_requested(&call_hash) == true,
			"preimage not requested"
		);
	}

	remove_whitelisted_call {
		let origin = T::WhitelistOrigin::successful_origin();
		let call_hash = Default::default();
		Pallet::<T>::whitelist_call(origin.clone(), call_hash)
			.expect("whitelisting call must be successful");
	}: _<T::Origin>(origin, call_hash)
	verify {
		ensure!(
			!WhitelistedCall::<T>::contains_key(call_hash),
			"whitelist not removed"
		);
		ensure!(
			T::PreimageProvider::preimage_requested(&call_hash) == false,
			"preimage still requested"
		);
	}

	dispatch_whitelisted_call {
		let origin = T::DispatchWhitelistedOrigin::successful_origin();
		let call: <T as Config>::Call = frame_system::Call::fill_block {
			ratio: Perbill::zero(),
		}.into();
		let call_weight = call.get_dispatch_info().weight;
		let encoded_call = call.encode();
		let call_hash = T::Hashing::hash(&encoded_call[..]);
		Pallet::<T>::whitelist_call(origin.clone(), call_hash)
			.expect("whitelisting call must be successful");
		let encoded_call = encoded_call.try_into().expect("encoded_call must be small enough");
		T::PreimageProvider::note_preimage(encoded_call);
	}: _<T::Origin>(origin, call_hash, call_weight)
	verify {
		ensure!(
			!WhitelistedCall::<T>::contains_key(call_hash),
			"whitelist not removed"
		);
		ensure!(
			T::PreimageProvider::preimage_requested(&call_hash) == false,
			"preimage still requested"
		);
	}

	impl_benchmark_test_suite!(Whitelist, crate::tests::new_test_ext(), crate::tests::Test);
}
