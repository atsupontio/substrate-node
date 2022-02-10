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
	pub enum EntityId<AccountId> {
		Account(AccountId),
		Item(TypeId),
	}

	/// Entity status is used in two cases: when moderators suggest a moderation status
	/// for a reported entity; or when a space owner makes a final decision to either block
	/// or allow this entity within the space.
	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
	pub enum EntityStatus {
		Allowed,
		Blocked,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
	pub struct Report<T: Config> {
		id: TypeID,
		created: WhoAndWhen<T>,
		/// An id of reported entity: account,item.
		reported_entity: EntityId<T::AccountId>,
		/// A reason should describe why this entity should be blocked in this space.
		reason: String,
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
	pub struct SuggestedStatus<T: Config> {
		/// An account id of a moderator who suggested this status.
		suggested: WhoAndWhen<T>,
		/// `None` if a moderator wants to signal that they have reviewed the entity,
		/// but they are not sure about what status should be applied to it.
		status: Option<EntityStatus>,
		/// `None` if a suggested status is not based on any reports.
		report_id: Option<TypeID>,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn report_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type ReportId<T> = StorageValue<_, TypeID, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn report_by_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type ReportById<T> = StorageMap<_, Twox64Concat, TypeID, Report<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn report_id_by_account_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type ReportIdByAccountId<T> = StorageMap<_, Twox64Concat, (EntityId<T::AccountId>, T::AccountId), TypeID, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> where
        AccountId = <T as system::Config>::AccountId,
        EntityId = EntityId<<T as system::Config>::AccountId>
    {
        EntityReported(AccountId, EntityId, ReportId),
        EntityStatusSuggested(AccountId, EntityId, Option<EntityStatus>),
        EntityStatusDeleted(AccountId, EntityId),
    }

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The account has already reported this entity.
        AlreadyReportedEntity,
        /// The entity has no status in this space. Nothing to delete.
        EntityHasNoStatusInScope,
        /// Entity scope differs from the scope provided.
        EntityNotInScope,
        /// Entity was not found by its id.
        EntityNotFound,
        /// Entity status is already as suggested one.
        SuggestedSameEntityStatus,
        /// Provided entity scope does not exist.
        ScopeNotFound,
        /// Account does not have a permission to suggest a new entity status.
        NoPermissionToSuggestEntityStatus,
        /// Account does not have a permission to update an entity status.
        NoPermissionToUpdateEntityStatus,
        /// Account does not have a permission to update the moderation settings.
        NoPermissionToUpdateModerationSettings,
        /// No updates provided for the space settings.
        NoUpdatesForModerationSettings,
        /// Report reason should not be empty.
        ReasonIsEmpty,
        /// Report was not found by its id.
        ReportNotFound,
        /// Trying to suggest an entity status in a scope that is different from the scope
        /// the entity was reported in.
        SuggestedStatusInWrongScope,
        /// Entity status has already been suggested by this moderator account.
        AlreadySuggestedEntityStatus,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000)]
		pub fn create_report(origin: OriginFor<T>,
            entity: EntityId<T::AccountId>,
            scope: SpaceId,
            reason: Content
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO check this func, if looks strange
            // Utils::<T>::ensure_content_is_some(&reason).map_err(|_| Error::<T>::ReasonIsEmpty)?;

            // Utils::<T>::is_valid_content(reason.clone())?;

            let not_reported_yet = Self::report_id_by_account_id((&entity, &who)).is_none();
            ensure!(not_reported_yet, Error::<T>::AlreadyReportedEntity);

            let report_id = Self::report_id();
            let new_report = Report::<T>::new(report_id, who.clone(), entity.clone(), scope, reason);

            <ReportById<T>>::insert(report_id, new_report);
            <ReportIdByAccountId<T>>::<T>::insert((&entity, &who), report_id);
            <ReportId<T>>::mutate(|n| { *n += 1; });

            Self::deposit_event(RawEvent::EntityReported(who, scope, entity, report_id));
            Ok(())
        }

		/// Allows a space owner/admin to update the final moderation status of a reported entity.
        #[weight = 10_000 /* TODO + T::DbWeight::get().reads_writes(_, _) */]
        pub fn update_entity_status(
            origin,
            entity: EntityId<T::AccountId>,
            status_opt: Option<EntityStatus>
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: add `forbid_content` parameter and track entity Content blocking via OCW
            //  - `forbid_content` - whether to block `Content` provided with entity

            Self::deposit_event(RawEvent::EntityStatusUpdated(who, scope, entity, status_opt));
            Ok(())
        }
}
