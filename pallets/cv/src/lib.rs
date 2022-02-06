#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_utils::{String, TypeID, UnixEpoch, WhoAndWhen};
	use scale_info::TypeInfo;
	use frame_support::inherent::Vec;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(bounds(), skip_type_params(T))]
	pub struct Item<T: Config> {
		item_id: TypeID,
		user_id: T::AccountId,
		created: WhoAndWhen<T>,
		org_date: Option<UnixEpoch>,
		exp_date: Option<UnixEpoch>,
		certificate_id: Option<TypeID>,
		score: u32,
		metadata: String,
	}

	impl<T: Config> Item<T> {
		pub fn new(
			id: TypeID,
			user_id: T::AccountId,
			created_by: T::AccountId,
			org_date: Option<UnixEpoch>,
			exp_date: Option<UnixEpoch>,
			certificate_id: Option<TypeID>,
			score: u32,
			metadata: String,
		) -> Self {
			Item {
				item_id: id,
				user_id,
				created: WhoAndWhen::<T>::new(created_by.clone()),
				org_date,
				exp_date,
				certificate_id,
				score,
				metadata,
			}
		}

		// pub fn ensure_owner(&self, account: &T::AccountId) -> DispatchResult {
		// 	ensure!(self.is_owner(account), Error::<T>::NotAPostOwner);
		// 	Ok(())
		// }

		// pub fn is_owner(&self, account: &T::AccountId) -> bool {
		// 	self.owner == *account
		// }
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum Status {
		Pending,
		Allow,
		Deny,
	}
	impl Default for Status {
		fn default() -> Self {
            Self::Pending
        }
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_utils::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn item_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type ItemId<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn item_by_id)]
	pub type ItemById<T> = StorageMap<_, Twox64Concat, TypeID, Item<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn item_status_by_item_id)]
	pub type ItemStatusByItemId<T> = StorageMap<_, Twox64Concat, TypeID, Status, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn items_by_accountid)]
	pub type ItemsByAccountId<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<TypeID>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		RevokeSucceed(TypeID),
		CreateSucceed(TypeID),
		SetStatusSucceed(TypeID),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		ItemNotFound,
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
		pub fn create_item(
			origin: OriginFor<T>,
			_account_id: T::AccountId,
			_metadata: String,
			_org_date: Option<UnixEpoch>,
			_exp_date: Option<UnixEpoch>,
			_certificated_id: Option<TypeID>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			let item_id = Self::item_id();
			let new_item: Item<T> = Item::new(
				item_id,
				_account_id.clone(),
				who.clone(),
				_org_date,
				_exp_date,
				_certificated_id,
				0,
				_metadata,
			);
			<ItemById<T>>::insert(item_id, new_item);
			<ItemsByAccountId<T>>::mutate(who, |x| x.push(item_id));
			<ItemId<T>>::mutate(|n| {
				*n += 1;
			});
			// Emit an event.
			Self::deposit_event(Event::CreateSucceed(item_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_item(origin: OriginFor<T>, _item_id: TypeID) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			let item_idx = Self::items_by_accountid(&who).iter().position(|x| *x == _item_id);
			ensure!(item_idx != None, Error::<T>::ItemNotFound);
			if let Some(iid) = item_idx {
				<ItemsByAccountId<T>>::mutate(&who, |x| x.swap_remove(iid));
			}
			<ItemById<T>>::remove(_item_id);
			// Emit an event.
			Self::deposit_event(Event::RevokeSucceed(_item_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn set_status_item(origin: OriginFor<T>, _item_id: TypeID, status: Status) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let item_idx = Self::items_by_accountid(&who).iter()
			.position(|x| { *x == _item_id });
			ensure!(item_idx != None, Error::<T>::ItemNotFound);
			match <ItemStatusByItemId<T>>::contains_key(_item_id) {
				true => {
					<ItemStatusByItemId<T>>::mutate(_item_id, |x| *x = status);
				},
				_ => {
					<ItemStatusByItemId<T>>::insert(_item_id, status);
				}
			}
			Self::deposit_event(Event::SetStatusSucceed(_item_id));
			Ok(())
		}
	}
}