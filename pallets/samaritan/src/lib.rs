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
	// use core::str::FromStr;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use scale_info::prelude::vec::Vec;
	use scale_info::prelude::string::String;
	use scale_info::prelude::format;

	// important structs
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Samaritan<T: Config> {
		pub did: BoundedVec<u8, T::MaxDIDLength>,
		pub doc_cid: BoundedVec<u8, T::MaxDocCIDLength>,
		pub account_id: T::AccountId
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxDIDLength: Get<u32>;

		#[pallet::constant]
		type MaxSamNameLength: Get<u32>;

		#[pallet::constant]
		type MaxDocCIDLength: Get<u32>;

		#[pallet::constant]
		type MaxNames: Get<u128>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sign_ins)]
	pub(super) type SamaritanPool<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxSamNameLength>, Samaritan<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// status of name search
		NameSearchConcluded(Vec<u8>, bool),
		/// creation of a Samaritan
		SamaritanCreated(Vec<u8>, Vec<u8>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// SamaritanName does not exist
		NameNotFound,
		/// SamaritanName overflow
		SamaritanNameOverflow,
		/// DID length overflow
		DIDLengthOverflow,
		/// DID Doceument CID overflow
		DocumentCIDOverflow
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// check for existence of a DID
		#[pallet::weight(100)]
		pub fn check_existence(origin: OriginFor<T>, sam_name: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// first check sam_name length
			let pn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let mut exist = false;

			if let Some(_x) = SamaritanPool::<T>::get(&pn) {
				exist = true;
			}

			// deposit event
			Self::deposit_event(Event::NameSearchConcluded(sam_name, exist));
			
			Ok(())
		}

		#[pallet::weight(100)]
		/// function to create a new Samaritan 
		pub fn create_samaritan(origin: OriginFor<T>, sam_name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let did = Self::create_did(&who)?;
			let doc = Self::str_to_vec(String::from("coming soon"));

			let rdoc: BoundedVec<_, T::MaxDocCIDLength> =
				doc.clone().try_into().map_err(|()| Error::<T>::DocumentCIDOverflow)?;

			let sam: Samaritan<T> = Samaritan {
				did: did.clone(),
				doc_cid: rdoc,
				account_id: who
			};

			// insert Samaritan into pool
			SamaritanPool::<T>::insert(sn, sam);

			// emit event
			Self::deposit_event(Event::SamaritanCreated(sam_name, did.to_vec()));

			Ok(())
		}

	}

	/// helper functions
	impl<T: Config> Pallet<T> {
		/// create did of the form did:sam:root:<accountId>
		/// The accountId is passed to the function and concatenated to the DID method scheme
		pub fn create_did(
			id: &T::AccountId
		) -> Result<BoundedVec<u8, T::MaxDIDLength>, Error<T>> {
			let did_str = format!("did:sam:root:{}", Self::accid_to_str((*id).clone()));
			let did_vec = Self::str_to_vec(did_str);

			// to bounded vec
			let did: BoundedVec<_, T::MaxDIDLength> =
				did_vec.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			Ok(did)
		}

		/// convert account id to string
		pub fn accid_to_str(
			id: T::AccountId
		) -> String {
			match String::from_utf8(id.encode()) {
				Ok(s) => s,
				Err(_e) => String::from("00000000000000000000000000000000000000"),
			}
		}

		/// convert a string to a vector
		pub fn str_to_vec(
			val: String
		) -> Vec<u8> {
			let s: &str = &val[..];
			let bytes: Vec<u8> = s.as_bytes().to_vec();

			bytes
		}
	}
}
