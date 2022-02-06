#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_utils::{Role, Status, TypeID, String};
	use scale_info::TypeInfo;
	use frame_support::inherent::Vec;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(bounds(), skip_type_params(T))]
	pub struct Certificate<T:Config> {
		cid: TypeID,
		org: T::AccountId,
		scrore: u32,
		metadata: String,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn certificate_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type CertificateId<T> = StorageValue<_, TypeID, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn certificate_by_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type CertificateById<T> = StorageMap<_, Twox64Concat, TypeID, Certificate<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		CertificateCreated(T::AccountId),
        CertificateRevoked(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000)]
		pub fn create_certificate(origin: OriginFor<T>, _meta_data: String) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			let cid = <CertificateId<T>>::get();
			// Update storage.
			<CertificateById<T>>::insert(cid, Certificate {
				cid: cid,
				org: who.clone(),
				metadata: _meta_data,
				scrore: 5,
			});
            <CertificateId<T>>::mutate(|n| {
				*n += 1;
			});
			// Emit an event.
			Self::deposit_event(Event::CertificateCreated(who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000)]
		pub fn revoke_certificate(origin: OriginFor<T>, _cid: TypeID) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			<CertificateById<T>>::remove(_cid);
			Self::deposit_event(Event::CertificateRevoked(_who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}
