#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
use scale_info::prelude::vec::Vec;
use scale_info::prelude::string::String;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::{*, DispatchResult}, BoundedVec};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use scale_info::prelude::vec::Vec;

	use frame_support::traits::UnixTime;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct File {
		hash: H256,
		metadata: H256,
		permission: u32,
		is_dir: bool,
		created: u64,
		last_access: u64
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type TimeProvider: UnixTime;

		#[pallet::constant]
		type MaxFileCount: Get<u32>;

		#[pallet::constant]
		type MaxDIDLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn file_reg)]
	pub(super) type FileRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<File, T::MaxFileCount>>;

	#[pallet::storage]
	#[pallet::getter(fn dir_reg)]
	pub(super) type DirRegistry<T: Config> = StorageMap<_, Twox64Concat, H256, BoundedVec<H256, T::MaxFileCount>>;

	#[pallet::storage]
	#[pallet::getter(fn inode_count)]
	pub(super) type FileCount<T: Config> = StorageValue<_, u64>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// a new file has been uploaded to the internet
		NewFileAdded { did: Vec<u8>, file_hash: H256 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// DID length overflow
		DIDLengthOverflow,
		/// Hash Length overflow
		HashLengthOverflow,
		// Directory doesn't exist
		InvalidDirectory,
		// Too many files
		FileCountOverflow
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)] 
		pub fn add_file(origin: OriginFor<T>, did_str: Vec<u8>, metadata: H256, hash: H256, p_dir_hash: H256) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let file = File {
				hash: hash.clone(),
				metadata,
				permission: 700,	// very private by default
				is_dir: false,
				created: T::TimeProvider::now().as_secs(),
				last_access: T::TimeProvider::now().as_secs()
			};

			// save under directory
			if let Some(mut dir) = DirRegistry::<T>::get(&p_dir_hash) {

				// select current lib
				match FileRegistry::<T>::get(&did) {
					Some(mut files) => {
						files.try_push(file).map_err(|()| Error::<T>::FileCountOverflow)?;

						FileRegistry::<T>::insert(&did, files);
					},
					None => {
						// create new 
						let mut files: BoundedVec<File, T::MaxFileCount> = Default::default();

						files.try_push(file).map_err(|()| Error::<T>::FileCountOverflow)?;

						// save to storage
						FileRegistry::<T>::insert(&did, files);
					}
				}

				// increase file count
				if let Some(count) = FileCount::<T>::get() {
					FileCount::<T>::put(count + 1);
				}

				dir.try_push(hash.clone()).map_err(|()| Error::<T>::FileCountOverflow)?;
				DirRegistry::<T>::insert(&p_dir_hash, dir);

			} else {
				// throw error 
				return Err(Error::<T>::InvalidDirectory.into());
			}

			// emit event
			Self::deposit_event(Event::NewFileAdded { did: did_str, file_hash: hash });

			Ok(())
		}
	}
}

/// helper functions
impl<T: Config> Pallet<T> {
	/// convert account id to string
	pub fn vec_to_str(
		vector: &Vec<u8>
	) -> String {
		match String::from_utf8(vector.clone()) {
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

