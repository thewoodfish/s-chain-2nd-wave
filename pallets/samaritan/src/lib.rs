#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// use frame_support::BoundedVec;

use scale_info::prelude::vec::Vec;
use scale_info::prelude::string::String;

#[frame_support::pallet]
pub mod pallet {
	// use core::str::FromStr;

	// use core::ops::Bound;
	// use parity_scale_codec::alloc::string::ToString;
	// use scale_info::prelude::format;

	use frame_support::{pallet_prelude::{*, DispatchResult}, BoundedVec};
	use frame_system::pallet_prelude::*;


	use scale_info::prelude::vec::Vec;
	// use scale_info::prelude::string::String;

	use frame_support::traits::UnixTime;

	// important structs
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Samaritan<T: Config> {
		pub name: BoundedVec<u8, T::MaxNameLength>,
		pub account_id: T::AccountId
    }

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DocMetadata<T: Config> {
		version: u64,
		hl: BoundedVec<u8, T::MaxHashLength>,
		cid: BoundedVec<u8, T::MaxCIDLength>,
		created: u64,
		active: bool
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type TimeProvider: UnixTime;

		#[pallet::constant]
		type MaxDIDLength: Get<u32>;

		#[pallet::constant]
		type MaxNameLength: Get<u32>;

		#[pallet::constant]
		type MaxHashLength: Get<u32>;

		#[pallet::constant]
		type MaxCIDLength: Get<u32>;

		#[pallet::constant]
		type MaxCacheLength: Get<u32>;

		#[pallet::constant]
		type MaxQuorumMembersCount: Get<u32>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sampool)]
	pub(super) type SamaritanPool<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, Samaritan<T>>;

	#[pallet::storage]
	#[pallet::getter(fn authsigs)]
	pub(super) type AuthSigs<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxHashLength>, BoundedVec<u8, T::MaxDIDLength>>;

	#[pallet::storage]
	#[pallet::getter(fn doc_metareg)]
	pub(super) type DocMetaRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<DocMetadata<T>, T::MaxCacheLength>>;

	#[pallet::storage]
	#[pallet::getter(fn trust_quorum)]
	pub(super) type TrustQuorum<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<BoundedVec<u8, T::MaxDIDLength>, T::MaxQuorumMembersCount>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// creation of a Samaritan
		SamaritanCreated(Vec<u8>, Vec<u8>),
		/// creation of DID document
		DIDDocumentCreated(Vec<u8>, Vec<u8>),
		/// fetch did address
		DIDAddrFetched(Vec<u8>),
		/// changed the name of a Samaritan
		SamaritanNameChanged(Vec<u8>, Vec<u8>),
		/// changed the visibility scope of a Samaritan
		SamaritanScopeChanged(Vec<u8>, bool),
		/// quorum updated
		TrustQuorumUpdated(Vec<u8>, Vec<u8>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Name overflow
		NameOverflow,
		/// DID length overflow
		DIDLengthOverflow,
		/// CID overflowed
		IpfsCIDOverflow,
		/// Hash Length overflow
		HashLengthOverflow,
		/// Cache Oveflow
		CacheOverflow,
		/// Hash didn't match any DID
		DIDNotFound,
		/// Samaritan not found
		SamaritanNotFound,
		/// DID metadata not found
		DIDMetaNotFound,
		/// Quorum filled up
		QuorumOverflow,
		/// Duplicate member
		DuplicateQuorumMember
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// function to create a new Samaritan 
		pub fn create_samaritan(origin: OriginFor<T>, name: Vec<u8>, did_str: Vec<u8>, hash: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxNameLength> =
				name.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let sig: BoundedVec<_, T::MaxHashLength> = 
				hash.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;
			
			let sam: Samaritan<T> = Samaritan {
				name: sn.clone(),
				account_id: who
			};

			// insert Samaritan into pool
			SamaritanPool::<T>::insert(&did, sam);

			// insert into signature registry
			AuthSigs::<T>::insert(&sig, did.clone());

			// emit event
			Self::deposit_event(Event::SamaritanCreated(sn.to_vec(), did_str));

			Ok(())
		}

		#[pallet::weight(0)] 
		/// DID document has been created on the server, now record it onchain
		pub fn acknowledge_doc(origin: OriginFor<T>, did_str: Vec<u8>, doc_cid: Vec<u8>, hl: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;
			
			let dc: BoundedVec<_, T::MaxCIDLength> =
				doc_cid.clone().try_into().map_err(|()| Error::<T>::IpfsCIDOverflow)?;

			let hash: BoundedVec<_, T::MaxHashLength> =
				hl.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			// create metadata
			let doc: DocMetadata<T> = DocMetadata {
				version: 1,
				hl: hash,
				cid: dc,
				created: T::TimeProvider::now().as_secs(),
				active: true
			};

			let mut cache: BoundedVec<DocMetadata<T>, T::MaxCacheLength> = Default::default();

			cache.try_push(doc).map_err(|()| Error::<T>::CacheOverflow)?;

			// insert into storage 
			DocMetaRegistry::<T>::insert(&did, cache);

			// emit event
			Self::deposit_event(Event::DIDDocumentCreated(did.to_vec(), doc_cid));

			Ok(())
		}

		#[pallet::weight(0)] 
		/// for auth, get DID with signature
		pub fn fetch_address(origin: OriginFor<T>, hash: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let sig: BoundedVec<_, T::MaxHashLength> = 
				hash.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let mut _did: Vec<u8> = Vec::new();
			
			match AuthSigs::<T>::get(&sig) {
				Some(addr) => _did = addr.to_vec(),
				None => {
					// throw error
					return Err(Error::<T>::DIDNotFound.into());
				}
			}

			// emit event
			Self::deposit_event(Event::DIDAddrFetched(_did));

			Ok(())
		}

		#[pallet::weight(0)] 
		/// rename a Samaritan
		pub fn rename_samaritan(origin: OriginFor<T>, name: Vec<u8>, did_str: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let sn: BoundedVec<_, T::MaxNameLength> =
				name.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

			match SamaritanPool::<T>::get(&did) {
				Some(mut sam) => {
					sam.name = sn.clone();
					SamaritanPool::<T>::insert(&did, sam);
				},
				None => {
					// throw error
					return Err(Error::<T>::SamaritanNotFound.into());
				}
			}

			// emit event
			Self::deposit_event(Event::SamaritanNameChanged(sn.to_vec(), did_str));

			Ok(())
		}
		
		#[pallet::weight(0)] 
		/// enable/disable Samaritan
		pub fn alter_state(origin: OriginFor<T>, did_str: Vec<u8>, state: bool) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			// select the latest DID document 
			match DocMetaRegistry::<T>::get(&did) {
				Some(doc) => {
					let mut index = 0;
					for mut _d in &doc {
						index += 1;
					}

					let mut d_vec = doc.into_inner();
					d_vec[index - 1].active = state;

					let mut meta: BoundedVec<DocMetadata<T>, T::MaxCacheLength> = Default::default();

					for i in d_vec {
						meta.try_push(i).map_err(|()| Error::<T>::CacheOverflow)?;
					}

					// save to storage
					DocMetaRegistry::<T>::insert(&did, meta);

				},

				None => {
					// throw error
					return Err(Error::<T>::DIDMetaNotFound.into());
				}
			}

			// emit event
			Self::deposit_event(Event::SamaritanScopeChanged(did_str, state));

			Ok(())
		}

		#[pallet::weight(0)] 
		/// update Samaritan trust quorum
		pub fn update_quorum(origin: OriginFor<T>, did_str: Vec<u8>, trust_did: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let t_did: BoundedVec<_, T::MaxDIDLength> = 
				trust_did.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			match TrustQuorum::<T>::get(&did) {
				Some(mut quorum) => {
					// first check the length of the quorum
					if quorum.len() < T::MaxQuorumMembersCount::get() as usize {
						// check for duplicate 
						if quorum.contains(&t_did) {
							// throw error, quorum full
							return Err(Error::<T>::DuplicateQuorumMember.into());
						}

						// insert DID
						quorum.try_push(t_did).map_err(|()| Error::<T>::QuorumOverflow)?;

						// commit
						TrustQuorum::<T>::insert(&did, quorum);
					} else {
						// throw error, quorum full
						return Err(Error::<T>::QuorumOverflow.into());
					}
				},

				None => {
					// create new quorum instance
					let mut quorum: BoundedVec<_, T::MaxQuorumMembersCount> = Default::default();
					
					// insert DID
					quorum.try_push(t_did).map_err(|()| Error::<T>::QuorumOverflow)?;

					// commit
					TrustQuorum::<T>::insert(&did, quorum);
				}
			}

			// emit event
			Self::deposit_event(Event::TrustQuorumUpdated(did_str, trust_did));

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

