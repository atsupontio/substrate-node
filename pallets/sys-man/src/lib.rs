#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[frame_support::pallet]
pub mod pallet {

	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type SysMan<T> = StorageMap<_, BlakeTwo256, _, _>;

	#[pallet::storage]
	pub type SysMapRevoked<T> = StorageMap<_, BlakeTwo256, _, _>;

	#[pallet::storage]
	pub type Org<T> = StorageMap<_, BlakeTwo256, _, _>;

	#[pallet::storage]
	pub type OrgRevoked<T> = StorageMap<_, BlakeTwo256, _, _>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created,
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		CreateFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_sys_man(
			origin: OriginFor<T>,
			id: T::AccountId,
			des: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			Ok(().into())
		}
	}
}
