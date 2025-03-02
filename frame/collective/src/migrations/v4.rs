// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

use sp_io::hashing::twox_128;

use frame_support::{
	traits::{
		Get, GetStorageVersion, PalletInfoAccess, StorageVersion,
		STORAGE_VERSION_STORAGE_KEY_POSTFIX,
	},
	weights::Weight,
};

/// Migrate the entire storage of this pallet to a new prefix.
///
/// This new prefix must be the same as the one set in construct_runtime. For safety, use
/// `PalletInfo` to get it, as:
/// `<Runtime as frame_system::Config>::PalletInfo::name::<CollectivePallet>`.
///
/// The migration will look into the storage version in order not to trigger a migration on an up
/// to date storage. Thus the on chain storage version must be less than 4 in order to trigger the
/// migration.
pub fn migrate<T: frame_system::Config, P: GetStorageVersion + PalletInfoAccess, N: AsRef<str>>(
	old_pallet_name: N,
) -> Weight {
	let old_pallet_name = old_pallet_name.as_ref();
	let new_pallet_name = <P as PalletInfoAccess>::name();

	if new_pallet_name == old_pallet_name {
		log::info!(
			target: "runtime::collective",
			"New pallet name is equal to the old pallet name. No migration needs to be done.",
		);
		return 0;
	}

	let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();
	log::info!(
		target: "runtime::collective",
		"Running migration to v4 for collective with storage version {:?}",
		on_chain_storage_version,
	);

	if on_chain_storage_version < 4 {
		frame_support::storage::migration::move_pallet(
			old_pallet_name.as_bytes(),
			new_pallet_name.as_bytes(),
		);
		log_migration("migration", old_pallet_name, new_pallet_name);

		StorageVersion::new(4).put::<P>();
		<T as frame_system::Config>::BlockWeights::get().max_block
	} else {
		log::warn!(
			target: "runtime::collective",
			"Attempted to apply migration to v4 but failed because storage version is {:?}",
			on_chain_storage_version,
		);
		0
	}
}

/// Some checks prior to migration. This can be linked to
/// [`frame_support::traits::OnRuntimeUpgrade::pre_upgrade`] for further testing.
///
/// Panics if anything goes wrong.
pub fn pre_migrate<P: GetStorageVersion + PalletInfoAccess, N: AsRef<str>>(old_pallet_name: N) {
	let old_pallet_name = old_pallet_name.as_ref();
	let new_pallet_name = <P as PalletInfoAccess>::name();
	log_migration("pre-migration", old_pallet_name, new_pallet_name);

	if new_pallet_name == old_pallet_name {
		return;
	}

	let new_pallet_prefix = twox_128(new_pallet_name.as_bytes());
	let storage_version_key = twox_128(STORAGE_VERSION_STORAGE_KEY_POSTFIX);

	let mut new_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
		new_pallet_prefix.to_vec(),
		new_pallet_prefix.to_vec(),
		|key| Ok(key.to_vec()),
	);

	// Ensure nothing except the storage_version_key is stored in the new prefix.
	assert!(new_pallet_prefix_iter.all(|key| key == storage_version_key));

	assert!(<P as GetStorageVersion>::on_chain_storage_version() < 4);
}

/// Some checks for after migration. This can be linked to
/// [`frame_support::traits::OnRuntimeUpgrade::post_upgrade`] for further testing.
///
/// Panics if anything goes wrong.
pub fn post_migrate<P: GetStorageVersion + PalletInfoAccess, N: AsRef<str>>(old_pallet_name: N) {
	let old_pallet_name = old_pallet_name.as_ref();
	let new_pallet_name = <P as PalletInfoAccess>::name();
	log_migration("post-migration", old_pallet_name, new_pallet_name);

	if new_pallet_name == old_pallet_name {
		return;
	}

	// Assert that nothing remains at the old prefix.
	let old_pallet_prefix = twox_128(old_pallet_name.as_bytes());
	let old_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
		old_pallet_prefix.to_vec(),
		old_pallet_prefix.to_vec(),
		|_| Ok(()),
	);
	assert_eq!(old_pallet_prefix_iter.count(), 0);

	// NOTE: storage_version_key is already in the new prefix.
	let new_pallet_prefix = twox_128(new_pallet_name.as_bytes());
	let new_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
		new_pallet_prefix.to_vec(),
		new_pallet_prefix.to_vec(),
		|_| Ok(()),
	);
	assert!(new_pallet_prefix_iter.count() >= 1);

	assert_eq!(<P as GetStorageVersion>::on_chain_storage_version(), 4);
}

fn log_migration(stage: &str, old_pallet_name: &str, new_pallet_name: &str) {
	log::info!(
		target: "runtime::collective",
		"{}, prefix: '{}' ==> '{}'",
		stage,
		old_pallet_name,
		new_pallet_name,
	);
}
