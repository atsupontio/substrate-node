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

	use codec::alloc::string::ToString;
	use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use pallet_utils::{Role, Status};
	use scale_info::TypeInfo;
	use serde::{Deserialize, Serialize};
	use serde_json::{json, Value};
	use sp_std::{str, vec, vec::Vec};

	// pub type String = Vec<u8>;

	pub enum OperationType {
		SYS,
		ORG,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[derive(Decode, Encode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(bounds(), skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct SysManAccount<T: Config> {
		// id: T::AccountId,
		pub role: Role,
		pub status: Status,
		pub level: Option<u8>,
		pub parent: Option<T::AccountId>,
		pub children: Option<Vec<T::AccountId>>,
		pub metadata: Vec<u8>,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sys_man)]
	pub type SysMan<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SysManAccount<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sys_man_revoked)]
	pub type SysManRevoked<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SysManAccount<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn org)]
	pub type Org<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SysManAccount<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn org_revoked)]
	pub type OrgRevoked<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, SysManAccount<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sys_man_cnt)]
	/// Keeps track of the number of system managers in existence.
	pub(super) type SysManCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn org_cnt)]
	/// Keeps track of the number of system managers in existence.
	pub(super) type OrgCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub sys_man: Vec<(T::AccountId, SysManAccount<T>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			Self { sys_man: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (account_id, sys_man_account) in &self.sys_man {
				SysMan::<T>::insert(account_id, sys_man_account);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Approved { target_id: T::AccountId, metadata: Vec<u8>, approver: T::AccountId },
		Revoked { target_id: T::AccountId, revoker: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		NoValidAuthorization,
		CreateFailed,
		AlreadyRegistered,
		AlreadyRevoked,
		ConvertMetadataFailed,
		RetrieveAuthorityFailed,
		SysManNotExist,
		RevokedSysManNotExist,
		RevokedOrgNotExist,
		OperationTypeInvalid,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn approve_sys_man(
			origin: OriginFor<T>,
			sys_man_id: T::AccountId,
			metadata: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// ensure extrinsics caller has right permission
			let authority = Self::get_account(&sender, OperationType::SYS)?;

			// check whether sys man has been approved
			ensure!(!SysMan::<T>::contains_key(&sys_man_id), Error::<T>::AlreadyRegistered);

			// check whether sys man has been revoked
			ensure!(!SysManRevoked::<T>::contains_key(&sys_man_id), Error::<T>::AlreadyRevoked);

			// create system manager account
			let sys_man = Self::create_account(
				Role::SysMan,
				Status::Active,
				Some(authority.level.unwrap() + 1),
				Some(vec![]),
				Some(sender.clone()),
				metadata.clone(),
			)
			.unwrap();

			SysMan::<T>::insert(&sys_man_id, sys_man);

			Self::deposit_event(Event::<T>::Approved {
				target_id: sys_man_id,
				metadata,
				approver: sender,
			});

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn approve_org(
			origin: OriginFor<T>,
			org_id: T::AccountId,
			metadata: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// check permisisno of authority
			let _ = Self::get_account(&sender, OperationType::SYS)?;

			// check whether org has been approved
			ensure!(!Org::<T>::contains_key(&org_id), Error::<T>::AlreadyRegistered);

			// check whether org has been revoked
			ensure!(!OrgRevoked::<T>::contains_key(&org_id), Error::<T>::AlreadyRevoked);

			// create system manager account
			let org_account = Self::create_account(
				Role::Organization,
				Status::Active,
				None,
				None,
				None,
				metadata.clone(),
			)
			.unwrap();

			Org::<T>::insert(&org_id, org_account);

			Self::deposit_event(Event::<T>::Approved {
				target_id: org_id,
				metadata,
				approver: sender,
			});

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_org(
			origin: OriginFor<T>,
			revoke_org_id: T::AccountId,
			description: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// check permission of revoker
			let _ = Self::get_account(&sender, OperationType::SYS)?;

			let mut revoke_org = Self::get_account(&revoke_org_id, OperationType::ORG)?;

			// ensure revoked org has not been revoked yet
			ensure!(!OrgRevoked::<T>::contains_key(&revoke_org_id), Error::<T>::AlreadyRevoked);

			// add revoked description to metadata string
			let mut metadata = json!(revoke_org.metadata);

			metadata = Self::add_json_field(
				&metadata,
				"revoke_description",
				&str::from_utf8(&description).unwrap(),
			);

			revoke_org.metadata = metadata.to_string().as_bytes().to_vec();

			// remove revoked sys man from Org Storage
			Org::<T>::remove(&revoke_org_id);

			// add revoked sys man to OrgRevoked Storage
			OrgRevoked::<T>::insert(&revoke_org_id, revoke_org);

			// emit revoked event with information of revoked sys man
			Self::deposit_event(Event::<T>::Revoked { target_id: revoke_org_id, revoker: sender });

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn revoke_sys_man(
			origin: OriginFor<T>,
			revoke_id: T::AccountId,
			description: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let authority = Self::get_account(&sender, OperationType::SYS)?;

			let mut revoke_sys_man = Self::get_account(&revoke_id, OperationType::SYS)?;

			// Ensure authority has higher hierarchical level than system manager to be revoked
			ensure!(
				authority.level.unwrap_or(0) < revoke_sys_man.level.unwrap_or(0),
				Error::<T>::NoValidAuthorization
			);

			// ensure revoked sys man has not been revoked yet
			ensure!(!SysManRevoked::<T>::contains_key(&revoke_id), Error::<T>::AlreadyRevoked);

			// add revoked description to metadata string

			let mut metadata = json!(revoke_sys_man.metadata);

			metadata = Self::add_json_field(
				&metadata,
				"revoke_description",
				&str::from_utf8(&description).unwrap(),
			);

			// metadata["revoke_description"] = Value::String(String::from_utf8(description).unwrap());

			revoke_sys_man.metadata = metadata.to_string().as_bytes().to_vec();

			// remove revoked sys man from SysMan Storage
			SysMan::<T>::remove(&revoke_id);

			// add revoked sys man to SysManRevoked Storage
			SysManRevoked::<T>::insert(&revoke_id, revoke_sys_man);

			// emit revoked event with information of revoked sys man
			Self::deposit_event(Event::<T>::Revoked { target_id: revoke_id, revoker: sender });

			Ok(().into())
		}
	}

	// private functions
	impl<T: Config> Pallet<T> {
		pub fn create_account(
			role: Role,
			status: Status,
			level: Option<u8>,
			children: Option<Vec<T::AccountId>>,
			parent: Option<T::AccountId>,
			metadata: Vec<u8>,
		) -> Result<SysManAccount<T>, Error<T>> {
			// TODO: validate metadata to be a valid JSON string

			let sys_man = SysManAccount::<T> { role, status, level, children, parent, metadata };

			Ok(sys_man)
		}

		pub fn get_account(
			id: &T::AccountId,
			op_type: OperationType,
		) -> Result<SysManAccount<T>, Error<T>> {
			let authority = match op_type {
				OperationType::SYS => match SysMan::<T>::try_get(id) {
					Ok(val) => match val.role {
						Role::SysMan => val,
						_ => Err(Error::<T>::NoValidAuthorization)?,
					},
					Err(_) => Err(Error::<T>::SysManNotExist)?,
				},

				OperationType::ORG => match Org::<T>::try_get(id) {
					Ok(val) => match val.role {
						Role::Organization => val,
						_ => Err(Error::<T>::RevokedOrgNotExist)?,
					},
					Err(_) => Err(Error::<T>::RevokedOrgNotExist)?,
				},
			};

			Ok(authority)
		}

		pub fn str2vec(s: &str) -> Vec<u8> {
			s.as_bytes().to_vec()
		}

		fn add_json_field(v: &Value, field_key: &str, field_val: &str) -> Value {
			match v {
				Value::Object(map) => {
					let mut map = map.clone();

					map.insert(field_key.to_string(), Value::String(field_val.to_string()));

					Value::Object(map)
				},
				v => v.clone(),
			}
		}
	}
}
