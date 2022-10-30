#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use scale_info::prelude::vec::Vec;
use scale_info::prelude::string::String;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{pallet_prelude::{*, DispatchResult}, BoundedVec};
	use frame_support::{traits::UnixTime};
	use frame_system::pallet_prelude::*;

	use scale_info::prelude::vec::Vec;
use sp_core::H256;

	// important structs

	// Used to track the name and account of a samaritan
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Samaritan<T: Config> {
		pub name: BoundedVec<u8, T::MaxNameLength>,
		pub account_id: T::AccountId
    }

	// Metadata for a document that a samaritan can create
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DocMetadata<T: Config> {
		version: u64,
		hl: H256,
		cid: BoundedVec<u8, T::MaxCIDLength>,
		created: u64,
		active: bool
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct VCredential<T: Config> {
		hl: BoundedVec<u8, T::MaxHashLength>,
		cid: BoundedVec<u8, T::MaxCIDLength>,
		created: u64,
		active: bool,
		desc: BoundedVec<u8, T::MaxHashLength>,
		public: bool
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		// Some way to timestamp document creation 
		type TimeProvider: UnixTime;

		// DID length must be bounded.
		#[pallet::constant]
		type MaxDIDLength: Get<u32>;

		// User name length must be bounded.
		#[pallet::constant]
		type MaxNameLength: Get<u32>;

		// CID length must be bounded.
		#[pallet::constant]
		type MaxCIDLength: Get<u32>;

		// Cache length must be bounded.
		#[pallet::constant]
		type MaxCacheLength: Get<u32>;

		// Quorum size must be bounded.
		#[pallet::constant]
		type MaxQuorumMembersCount: Get<u32>;

		#[pallet::constant]
		type MaxCredentialsCount: Get<u32>;

		#[pallet::constant]
		type MaxResourceAddressLength: Get<u32>;

		#[pallet::constant]
		type MaxSigListHeight: Get<u32>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sampool)]
	pub(super) type SamaritanPool<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, Samaritan<T>>;

	#[pallet::storage]
	#[pallet::getter(fn authsigs)]
	pub(super) type AuthSigs<T: Config> = StorageMap<_, Twox64Concat, H256, BoundedVec<u8, T::MaxDIDLength>>;

	#[pallet::storage]
	#[pallet::getter(fn doc_metareg)]
	pub(super) type DocMetaRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<DocMetadata<T>, T::MaxCacheLength>>;

	#[pallet::storage]
	#[pallet::getter(fn vcred_reg)]
	pub(super) type VCredRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<VCredential<T>, T::MaxCredentialsCount>>;

	#[pallet::storage]
	#[pallet::getter(fn trust_quorum)]
	pub(super) type TrustQuorum<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<BoundedVec<u8, T::MaxDIDLength>, T::MaxQuorumMembersCount>>;

	#[pallet::storage]
	#[pallet::getter(fn vc_issuelist)]
	pub(super) type VCSigList<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<BoundedVec<u8, T::MaxResourceAddressLength>, T::MaxSigListHeight>>;

	#[pallet::storage]
	#[pallet::getter(fn vc_nonce)]
	pub(super) type VCNonce<T: Config> = StorageValue<_, u64>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// creation of a Samaritan
		SamaritanCreated { name: Vec<u8>, did: Vec<u8> },
		/// creation of DID document
		DIDDocumentCreated { did: Vec<u8>, cid: Vec<u8> },
		/// fetch did address
		DIDAddrFetched { did: Vec<u8> },
		/// changed the name of a Samaritan
		SamaritanNameChanged { name: Vec<u8>, did: Vec<u8> },
		/// changed the visibility scope of a Samaritan
		SamaritanScopeChanged { did: Vec<u8>, state: bool },
		/// quorum updated
		TrustQuorumUpdated { did: Vec<u8>, trust_did: Vec<u8> },
		/// get members of a quorum
		RetrieveQuorumMembers { did: Vec<u8>, names: Vec<Vec<u8>> },
		/// changed a samaritans auth signature
		AuthSigModified { hash: H256, key: H256 }
		/// fetch important figures for VC construction
		FetchVCIndexes(u64, u64, u64),
		/// verifiable credential has been created
		VCredentialCreated(Vec<u8>, Vec<u8>)
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
		DuplicateQuorumMember,
		/// Credential list overflow
		CredListOverflow,
		/// Maximum signature count on a credential attanined
		VCSigListOverflow
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// function to create a new Samaritan 
		pub fn create_samaritan(origin: OriginFor<T>, name: Vec<u8>, did_str: Vec<u8>, hash: H256) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxNameLength> =
				name.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			// TODO: handle whether did already exists?
			
			let sam: Samaritan<T> = Samaritan {
				name: sn.clone(),
				account_id: who
			};

			// insert Samaritan into pool
			SamaritanPool::<T>::insert(&did, sam);

			// insert into signature registry
			AuthSigs::<T>::insert(&hash, did.clone());

			// emit event
			Self::deposit_event(Event::SamaritanCreated { name: sn.to_vec(), did: did_str } );

			Ok(())
		}

		#[pallet::weight(0)] 
		/// DID document has been created on the server, now record it onchain
		pub fn acknowledge_doc(origin: OriginFor<T>, did_str: Vec<u8>, doc_cid: Vec<u8>, hl: H256) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;
			
			let dc: BoundedVec<_, T::MaxCIDLength> =
				doc_cid.clone().try_into().map_err(|()| Error::<T>::IpfsCIDOverflow)?;

			// create metadata
			let ndoc: DocMetadata<T> = DocMetadata {
				version: 1, // TODO: this should probably be incremented overtime
				hl,
				cid: dc,
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

				None => {
					let mut cache: BoundedVec<DocMetadata<T>, T::MaxCacheLength> = Default::default();

					cache.try_push(ndoc).map_err(|()| Error::<T>::CacheOverflow)?;

					// insert into storage 
					DocMetaRegistry::<T>::insert(&did, cache);
				}
			}

			// emit event
			Self::deposit_event(Event::DIDDocumentCreated { did: did.to_vec(), cid: doc_cid });

			Ok(())
		}

		#[pallet::weight(0)] 
		/// for auth, get DID with signature
		pub fn fetch_address(origin: OriginFor<T>, hash: H256) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let mut _did: Vec<u8> = Vec::new();
			
			match AuthSigs::<T>::get(&hash) {
				Some(addr) => _did = addr.to_vec(),
				None => {
					// throw error
					return Err(Error::<T>::DIDNotFound.into());
				}
			}

			// emit event
			Self::deposit_event(Event::DIDAddrFetched { did: _did } );

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
			Self::deposit_event(Event::SamaritanNameChanged { name: sn.to_vec(), did: did_str } );

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
		/// list members of a Samaritans trust quorum
		pub fn enum_quorum(origin: OriginFor<T>, did_str: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut list: Vec<Vec<u8>> = Vec::new();


			if let Some(quorum) = TrustQuorum::<T>::get(&did) {
				// loop through to get them
				for d in quorum {
					list.push(d.clone().to_vec());

					// select name of Samaritan
					if let Some(sam) = SamaritanPool::<T>::get(&d) {
						list.push(sam.name.to_vec());
					}
				}
			}


			// emit event
			Self::deposit_event(Event::RetrieveQuorumMembers { did: did_str, names: list });

			Ok(())
		}

		#[pallet::weight(0)] 
		/// change the auth sig of a samaritan
		pub fn change_sig(origin: OriginFor<T>, hk: H256, hash_key: H256) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// swap signature
			AuthSigs::<T>::swap(hk, hash_key);

			// emit event
			Self::deposit_event(Event::AuthSigModified {hash: hk, key: hash_key });

			Ok(())
		}

		#[pallet::weight(0)] 
		/// get important indexes to contruct a credential
		pub fn get_indexes(origin: OriginFor<T>, did_str: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut nonce = 1;
			let mut height = 0;
			let mut _version = 0;

			// select current vc registry nonce
			match VCNonce::<T>::get() {
				Some(n) => {
					nonce = n;
				},
				None => {
					// initialize nonce
					VCNonce::<T>::put(1);
				}
			}

			// select a samaritans credential height
			match VCredRegistry::<T>::get(&did) {
				Some(creds) => {
					height = creds.len() as u64;
				},
				None => {
					// do nothing
				}
			}

			// select version of latest DID document
			match DocMetaRegistry::<T>::get(&did) {
				Some(doc) => {
					let mut index = 0;
					for _d in &doc {
						index += 1;
					}

					let d_vec = doc.into_inner();
					_version = d_vec[index - 1].version;
				},

				None => {
					// throw error
					return Err(Error::<T>::DIDMetaNotFound.into());
				}
			}


			// emit event
			Self::deposit_event(Event::FetchVCIndexes(nonce, _version, height));

			Ok(())
		}

		// const tx = api.tx.samaritan.recordCredential(auth.did, rcred.id, cid, hash, rcred.scope, purpose);
		#[pallet::weight(0)] 
		/// record credential onchain
		pub fn record_credential(origin: OriginFor<T>, issuer: Vec<u8>, did_str: Vec<u8>, cred_cid: Vec<u8>, hash: Vec<u8>, public: bool, descr: Vec<u8>, vc_addr: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let iss_did: BoundedVec<_, T::MaxDIDLength> =
				issuer.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let hl: BoundedVec<_, T::MaxHashLength> = 
				hash.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let cid: BoundedVec<_, T::MaxCIDLength> =
				cred_cid.clone().try_into().map_err(|()| Error::<T>::IpfsCIDOverflow)?;

			let desc: BoundedVec<_, T::MaxHashLength> = 
				descr.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let addr: BoundedVec<u8, T::MaxResourceAddressLength> = 
				vc_addr.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let cred: VCredential<T> = VCredential { 
				hl, cid, 
				created: T::TimeProvider::now().as_secs(),
				active: true,
				desc,
				public
			};

			// its the samaritan the credential is about that would be recorded
			let mut clist: BoundedVec<VCredential<T>, T::MaxCredentialsCount> = Default::default();
			clist.try_push(cred).map_err(|()| Error::<T>::CredListOverflow)?;

			// save to storage
			VCredRegistry::<T>::insert(&did, clist);

			// record signature on the credential
			let mut addr_list: BoundedVec<BoundedVec<u8, T::MaxResourceAddressLength>, T::MaxSigListHeight> = Default::default();
			addr_list.try_push(addr).map_err(|()| Error::<T>::VCSigListOverflow)?;

			VCSigList::<T>::insert(&iss_did, addr_list);

			// emit event
			Self::deposit_event(Event::VCredentialCreated(did_str, vc_addr));

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

