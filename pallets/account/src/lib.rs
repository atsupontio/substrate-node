#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use pallet_utils::{Role, Status};
use scale_info::TypeInfo;
use generic_array::GenericArray;
use typenum::U12;
use aes_gcm::{AesGcm, Key, Nonce}; // Or `Aes128Gcm`
use aes_gcm::{NewAead};
use generic_array::ArrayLength;
//use aes::Aes256;

//#[cfg(feature = "aes")]
use aes_gcm::Aes256Gcm;
use aead::Buffer;
use sp_std::vec::Vec;
use aes_gcm::AeadInPlace;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	//use frame_support::inherent::Vec;


	//pub type Key256<U12> = GenericArray<u8, U12>;
	//pub type Aes256Gcm = AesGcm<Aes256, U12>;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(bounds(), skip_type_params(T))]
	pub struct Account<T: Config> {
		pub id: T::AccountId,
		pub role: Role,
		pub status: Status,
		pub enkey: Option<Vec<u8>>,
		pub metadata: Vec<u8>,
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
	#[pallet::getter(fn account_storage)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type AccountStorage<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Account<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account_role)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type AccountRole<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Role, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AccountRegisted,
		AccountUpdated(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Account is already Registered
		AlreadyRegistered,
		/// Account is not Registered
		AccountNotRegistered,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000)]
		pub fn register(origin: OriginFor<T>, role: Role, metadata: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			//let key = GenericArray::from([0u8; 16]);
			//let key = Key::from_slice(b"ok");
			// Initialize cipher
			let key = Key::from_slice(b"an example very very secret key.");
			let cipher = Aes256Gcm::new(key);
			
			let nonce = Nonce::from_slice(b"unique nonce");  // 96-bits; unique per message

			let mut buffer: Vec<u8> = Vec::new();
			buffer.extend_from_slice(b"plaintext message");
			let cipher_text = cipher.encrypt_in_place_detached(nonce, b"", &mut buffer).expect("encryption failure!");

			let cipher_text_convert = AsRef::<[u8;16]>::as_ref(&cipher_text).to_vec();
			match <AccountStorage<T>>::try_get(&who) {
				Err(_) => {
					<AccountStorage<T>>::insert(
						&who,
						Account {
							id: who.clone(),
							role: role.clone(),
							status: Status::Active,
							metadata,
							enkey: Some(cipher_text_convert),
						},
					);
					<AccountRole<T>>::insert(who, role.clone());
				},
				Ok(_) => Err(Error::<T>::AlreadyRegistered)?,
			}
			// Return a successful DispatchResultWithPostInfo
			Self::deposit_event(Event::AccountRegisted);
			Ok(())
		}
		// TODO
		#[pallet::weight(10_000)]
		pub fn update(origin: OriginFor<T>, _role: Role, _metadata: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			match <AccountStorage<T>>::try_get(&who) {
				Ok(_) => {
					Self::deposit_event(Event::AccountUpdated(who));
					// Return a successful DispatchResultWithPostInfo
					Ok(())
				},
				Err(_) => Err(Error::<T>::AccountNotRegistered)?,
			}
		}
	}
}
