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

	use scale_info::prelude::vec::Vec;
	// use sp_core::H256;

	use frame_support::traits::UnixTime;

	// important structs
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Samaritan<T: Config> {
		pub did: BoundedVec<u8, T::MaxDIDLength>,   
		pub name: BoundedVec<u8, T::MaxNameLength>
    }

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DocMetadata<T: Config>{
		pub version: u64,
		pub hl: BoundedVec<u8, T::MaxHashLength>,
		pub created: u64,
		pub active: bool
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TimeProvider: UnixTime;

		#[pallet::constant]
		type MaxDIDLength: Get<u32>;

		#[pallet::constant]
		type MaxNameLength: Get<u32>;
		
		#[pallet::constant]
		type MaxHashLength: Get<u32>;

		#[pallet::constant]
		type MaxCacheLength: Get<u32>;

		#[pallet::constant]
		type MaxQuorumMembersCount: Get<u32>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sam_reg)]
	pub(super) type SamaritanRegistry<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Samaritan<T>>;

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
		SamaritanCreated { name: Vec<u8>, did: Vec<u8> },
		/// changed the name of a Samaritan
		SamaritanNameChanged { name: Vec<u8> },
		/// DID document updated
		DIDDocumentUpdated { did: Vec<u8> },
		/// changed the visibility scope of a Samaritan
		SamaritanScopeChanged { did: Vec<u8>, state: bool },
		/// quorum updated
		TrustQuorumUpdated { did: Vec<u8>, trust_did: Vec<u8> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// DID length overflow
		DIDLengthOverflow,
		/// Hash Length overflow
		HashLengthOverflow,
		/// Cache Oveflow
		CacheOverflow,
		/// Samaritan too long
		NameOverflow,
		/// Hash didn't match any DID
		DIDNotFound,
		/// Samaritan not found
		SamaritanNotFound,
		/// DID metadata not found
		DIDMetaNotFound,
		/// Quorum filled up
		QuorumOverflow,
		/// Duplicate member
		DuplicateQuorumMember,
		/// Quorum not set up
		QuorumUninitialized
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// function to create a new Samaritan 
		pub fn create_samaritan(origin: OriginFor<T>, name: Vec<u8>, did_str: Vec<u8>, meta_hash: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxNameLength> =
				name.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let hash: BoundedVec<_, T::MaxHashLength> = 
				meta_hash.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let sam: Samaritan<T> = Samaritan {
				did: did.clone(),
				name: sn
			};

			// register Samaritan
			SamaritanRegistry::<T>::insert(&who, sam);

			// register Document
			let doc: DocMetadata<T> = DocMetadata {
				version: 0,
				hl: hash,
				created: T::TimeProvider::now().as_secs(),
				active: true
			};

			let mut cache: BoundedVec<DocMetadata<T>, T::MaxCacheLength> = Default::default();
			cache.try_push(doc).map_err(|()| Error::<T>::CacheOverflow)?;

			// insert into storage 
			DocMetaRegistry::<T>::insert(&did, cache);

			// emit event
			Self::deposit_event(Event::SamaritanCreated { name, did: did_str } );

			Ok(())
		}

		#[pallet::weight(0)] 
		/// rename a Samaritan
		pub fn rename_samaritan(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxNameLength> =
				name.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

			match SamaritanRegistry::<T>::get(&who) {
				Some(mut sam) => {
					sam.name = sn.clone();
					SamaritanRegistry::<T>::insert(&who, sam);
				},
				None => {
					// throw error
					return Err(Error::<T>::SamaritanNotFound.into());
				}
			}

			// emit event
			Self::deposit_event(Event::SamaritanNameChanged { name: sn.to_vec() } );

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
			Self::deposit_event(Event::SamaritanScopeChanged { did: did_str, state: state });

			Ok(())
		}

		#[pallet::weight(0)] 
		/// update DID document
		pub fn update_document(origin: OriginFor<T>, did_str: Vec<u8>, doc: Vec<u8>,) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;
			
			let hl: BoundedVec<_, T::MaxHashLength> =
				doc.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			// create metadata
			let ndoc: DocMetadata<T> = DocMetadata {
				version: DocMetaRegistry::<T>::get(&did).unwrap_or_default().len() as u64,
				hl,
				created: T::TimeProvider::now().as_secs(),
				active: true
			};

			// select the latest DID document 
			match DocMetaRegistry::<T>::get(&did) {
				Some(doc) => {
					let mut index = 0;
					for mut _d in &doc {
						index += 1;
					}

					// disable the current active DID doc, there can be only one
					let mut d_vec = doc.into_inner();
					d_vec[index - 1].active = false;

					let mut meta: BoundedVec<DocMetadata<T>, T::MaxCacheLength> = Default::default();

					for i in d_vec {
						meta.try_push(i).map_err(|()| Error::<T>::CacheOverflow)?;
					}

					// insert the new doc
					meta.try_push(ndoc).map_err(|()| Error::<T>::CacheOverflow)?;

					// save to storage
					DocMetaRegistry::<T>::insert(&did, meta);
				},

				None => { }
			}

			// emit event
			Self::deposit_event(Event::DIDDocumentUpdated { did: did_str });

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
			Self::deposit_event(Event::TrustQuorumUpdated { did: did_str, trust_did });

			Ok(())
		}

		#[pallet::weight(0)] 
		/// remove samaritan from quorum
		pub fn filter_quorum(origin: OriginFor<T>, did_str: Vec<u8>, trust_did: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			match TrustQuorum::<T>::get(&did) {
				Some(quorum) => {
					let mut nq: BoundedVec<BoundedVec<u8, T::MaxDIDLength>, T::MaxQuorumMembersCount> = Default::default();
					for i in &quorum {
						let t_did: BoundedVec<_, T::MaxDIDLength> = 
							trust_did.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

						if *i != t_did {
							nq.try_push(i.clone()).map_err(|()| Error::<T>::QuorumOverflow)?;
						}
					}

					// save the new quorum
					TrustQuorum::<T>::insert(&did, nq);
				},

				None => {
					// throw error, quorum full
					return Err(Error::<T>::QuorumUninitialized.into());
				}
			}

			// emit event
			Self::deposit_event(Event::TrustQuorumUpdated { did: did_str, trust_did });

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