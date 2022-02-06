#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::inherent::Vec;

pub type TypeID = u32;
pub type UnixEpoch = u64;
pub type String = Vec<u8>;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Currency;
	use frame_system as system;

	use scale_info::TypeInfo;
	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};
	use sp_runtime::RuntimeDebug;
	use sp_std::prelude::*;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(bounds(), skip_type_params(T))]
	pub struct WhoAndWhen<T: Config> {
		pub account: T::AccountId,
		pub block: T::BlockNumber,
		pub time: T::Moment,
	}

	impl<T: Config> WhoAndWhen<T> {
		pub fn new(account: T::AccountId) -> Self {
			WhoAndWhen {
				account,
				block: <system::Pallet<T>>::block_number(),
				time: <pallet_timestamp::Pallet<T>>::now(),
			}
		}
	}

	#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum User<AccountId> {
		Account(AccountId),
	}

	#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Role {
		Organization,
		SysMan,
		User,
	}

	#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Status {
		Active,
		Revoked,
		Deactivated,
	}

	impl Default for Status {
		fn default() -> Self {
			Self::Active
		}
	}

	impl<AccountId> User<AccountId> {
		pub fn maybe_account(self) -> Option<AccountId> {
			if let User::Account(account_id) = self {
				Some(account_id)
			} else {
				None
			}
		}
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Deserialize))]
	#[cfg_attr(feature = "std", serde(tag = "contentType", content = "contentId"))]
	pub enum Content {
		/// No content.
		None,
		/// A raw vector of bytes.
		Raw(Vec<u8>),
		/// IPFS CID v0 of content.
		#[allow(clippy::upper_case_acronyms)]
		IPFS(Vec<u8>),
		/// Hypercore protocol (former DAT) id of content.
		Hyper(Vec<u8>),
	}

	impl From<Content> for Vec<u8> {
		fn from(content: Content) -> Vec<u8> {
			match content {
				Content::None => Vec::new(),
				Content::Raw(vec_u8) => vec_u8,
				Content::IPFS(vec_u8) => vec_u8,
				Content::Hyper(vec_u8) => vec_u8,
			}
		}
	}

	impl Default for Content {
		fn default() -> Self {
			Self::None
		}
	}

	impl Content {
		pub fn is_none(&self) -> bool {
			self == &Self::None
		}

		pub fn is_some(&self) -> bool {
			!self.is_none()
		}

		pub fn is_ipfs(&self) -> bool {
			matches!(self, Self::IPFS(_))
		}
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Account is blocked in a given space.
		AccountIsBlocked,
		/// Content is blocked in a given space.
		ContentIsBlocked,
		/// Post is blocked in a given space.
		PostIsBlocked,
		/// IPFS CID is invalid.
		InvalidIpfsCid,
		/// `Raw` content type is not yet supported.
		RawContentTypeNotSupported,
		/// `Hyper` content type is not yet supported.
		HypercoreContentTypeNotSupported,
		/// Space handle is too short.
		HandleIsTooShort,
		/// Space handle is too long.
		HandleIsTooLong,
		/// Space handle contains invalid characters.
		HandleContainsInvalidChars,
		/// Content type is `None`.
		ContentIsEmpty,
	}
}
