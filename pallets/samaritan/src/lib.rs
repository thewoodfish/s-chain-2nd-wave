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
		provider: BoundedVec<u8, T::MaxStringLength>,
		pub account_id: T::AccountId
    }

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DocMetadata<T: Config> {
		version: u64,
		hl: BoundedVec<u8, T::MaxHashLength>,
		uri: BoundedVec<u8, T::MaxURILength>,
		created: u64,
		active: bool
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DataFile<T: Config> {
		name: Option<BoundedVec<u8, T::MaxHashLength>>,
		desc: Option<BoundedVec<u8, T::MaxHashLength>>,
		uri: BoundedVec<u8, T::MaxURILength>,
		created: u64,
		public: bool
	}

	// #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	// #[scale_info(skip_type_params(T))]
	// #[codec(mel_bound())]
	// pub struct VCredential<T: Config> {
	// 	hl: BoundedVec<u8, T::MaxHashLength>,
	// 	uri: BoundedVec<u8, T::MaxURILength>,
	// 	created: u64,
	// 	active: bool,
	// 	desc: BoundedVec<u8, T::MaxHashLength>,
	// 	public: bool
	// }

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
		type MaxURILength: Get<u32>;

		#[pallet::constant]
		type MaxCacheLength: Get<u32>;

		#[pallet::constant]
		type MaxQuorumMembersCount: Get<u32>;

		#[pallet::constant]
		type MaxHoldingsCount: Get<u32>;

		#[pallet::constant]
		type MaxResourceAddressLength: Get<u32>;

		#[pallet::constant]
		type MaxSigListHeight: Get<u32>;

		#[pallet::constant]
		type MaxStringLength: Get<u32>;

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


	// data resources

	#[pallet::storage]
	#[pallet::getter(fn data_reg)]
	pub(super) type DataRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<DataFile<T>, T::MaxHoldingsCount>>;

	#[pallet::storage]
	#[pallet::getter(fn df_nonce)]
	pub(super) type DFNonce<T: Config> = StorageValue<_, u64>;


	// verifiable credentials 

	// #[pallet::storage]
	// #[pallet::getter(fn vcred_reg)]
	// pub(super) type VCredRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<VCredential<T>, T::MaxHoldingsCount>>;

	// #[pallet::storage]
	// #[pallet::getter(fn vc_issuelist)]
	// pub(super) type VCSigList<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<BoundedVec<u8, T::MaxResourceAddressLength>, T::MaxSigListHeight>>;

	// #[pallet::storage]
	// #[pallet::getter(fn vc_nonce)]
	// pub(super) type VCNonce<T: Config> = StorageValue<_, u64>;

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
		TrustQuorumUpdated(Vec<u8>, Vec<u8>),
		/// get members of a quorum
		RetrieveQuorumMembers(Vec<u8>, Vec<Vec<u8>>),
		/// changed a samaritans auth signature
		AuthSigModified(Vec<u8>, Vec<u8>),
		/// fetch important figures for URL construction
		FetchDataIndexes(u64, u64, u64, Vec<u8>),
		/// verifiable credential has been created
		VCredentialCreated(Vec<u8>, Vec<u8>),
		/// data has been added to the network
		DataAddedToNetwork(Vec<u8>, Vec<u8>),
		/// uri of a resource returned
		FetchResourceURI(Vec<u8>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Name overflow
		NameOverflow,
		/// DID length overflow
		DIDLengthOverflow,
		/// URI overflowed
		IpfsURIOverflow,
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
		/// Resource not found
		ResourceNotFound,
		/// Quorum filled up
		QuorumOverflow,
		/// Duplicate member
		DuplicateQuorumMember,
		/// Holdings list overflow
		HoldingsListOverflow,
		/// Maximum signature count on a credential attanined
		VCSigListOverflow,
		/// String too long
		StringLengthOverflow,
		/// Private resource, cannot view
		ResourceIsPrivate
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		/// function to create a new Samaritan 
		pub fn create_samaritan(origin: OriginFor<T>, name: Vec<u8>, did_str: Vec<u8>, hash: Vec<u8>, prov: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxNameLength> =
				name.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let sig: BoundedVec<_, T::MaxHashLength> = 
				hash.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let provider: BoundedVec<_, T::MaxStringLength> = 
				prov.clone().try_into().map_err(|()| Error::<T>::StringLengthOverflow)?;
			
			let sam: Samaritan<T> = Samaritan {
				name: sn.clone(),
				provider,
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
		pub fn acknowledge_doc(origin: OriginFor<T>, did_str: Vec<u8>, doc_uri: Vec<u8>, hl: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;
			
			let dc: BoundedVec<_, T::MaxURILength> =
				doc_uri.clone().try_into().map_err(|()| Error::<T>::IpfsURIOverflow)?;

			let hash: BoundedVec<_, T::MaxHashLength> =
				hl.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			// create metadata
			let ndoc: DocMetadata<T> = DocMetadata {
				version: 1,
				hl: hash,
				uri: dc,
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
			Self::deposit_event(Event::DIDDocumentCreated(did.to_vec(), doc_uri));

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
			Self::deposit_event(Event::RetrieveQuorumMembers(did_str, list));

			Ok(())
		}

		#[pallet::weight(0)] 
		/// change the auth sig of a samaritan
		pub fn change_sig(origin: OriginFor<T>, hk: Vec<u8>, hash_key: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let hash: BoundedVec<_, T::MaxHashLength> =
				hk.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let new_hash: BoundedVec<_, T::MaxHashLength> =
				hash_key.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			// swap signature
			AuthSigs::<T>::swap(hash, new_hash);

			// emit event
			Self::deposit_event(Event::AuthSigModified(hk, hash_key));

			Ok(())
		}

		#[pallet::weight(0)] 
		/// get important indexes to contruct a resource URL
		pub fn get_indexes(origin: OriginFor<T>, rtype: u64, did_str: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut nonce = 1;
			let mut height = 0;
			let mut _version = 0;
			let mut _provider: Vec<u8>;

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


			// select the provider
			match SamaritanPool::<T>::get(&did) {
				Some(sam) => {
					_provider = sam.provider.to_vec()
				},

				None => {
					// throw error
					return Err(Error::<T>::SamaritanNotFound.into());
				}
			}

			// [
			// 	1 => "data",
			// 	2 => credentials
			// ]

			match rtype {
				1 => {	// we have a resource here e.g a PDF
					// select current data registry nonce
					match DFNonce::<T>::get() {
						Some(n) => {
							nonce = n;
						},
						None => {
							// initialize nonce
							DFNonce::<T>::put(1);
						}
					}

					// select a samaritans data height
					match DataRegistry::<T>::get(&did) {
						Some(creds) => {
							height = creds.len() as u64;
						},
						None => {
							// do nothing
						}
					}
				},
				2 => {	// verifiable credentials
					
				}
				_ => {}
			}

			// emit event
			Self::deposit_event(Event::FetchDataIndexes(nonce, _version, height, _provider));

			Ok(())
		}

		// #[pallet::weight(0)] 
		// /// record credential onchain
		// pub fn record_credential(origin: OriginFor<T>, issuer: Vec<u8>, did_str: Vec<u8>, cred_uri: Vec<u8>, hash: Vec<u8>, public: bool, descr: Vec<u8>, vc_addr: Vec<u8>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;

		// 	let iss_did: BoundedVec<_, T::MaxDIDLength> =
		// 		issuer.clone().try_into().map_err(|()| Error::<T>::NameOverflow)?;

		// 	let did: BoundedVec<_, T::MaxDIDLength> = 
		// 		did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

		// 	let hl: BoundedVec<_, T::MaxHashLength> = 
		// 		hash.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

		// 	let uri: BoundedVec<_, T::MaxURILength> =
		// 		cred_uri.clone().try_into().map_err(|()| Error::<T>::IpfsURIOverflow)?;

		// 	let desc: BoundedVec<_, T::MaxHashLength> = 
		// 		descr.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

		// 	let addr: BoundedVec<u8, T::MaxResourceAddressLength> = 
		// 		vc_addr.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

		// 	let cred: VCredential<T> = VCredential { 
		// 		hl, uri, 
		// 		created: T::TimeProvider::now().as_secs(),
		// 		active: true,
		// 		desc,
		// 		public
		// 	};

		// 	// its the samaritan the credential is about that would be recorded
		// 	let mut clist: BoundedVec<VCredential<T>, T::MaxHoldingsCount> = Default::default();
		// 	clist.try_push(cred).map_err(|()| Error::<T>::CredListOverflow)?;

		// 	// save to storage
		// 	VCredRegistry::<T>::insert(&did, clist);

		// 	// record signature on the credential
		// 	let mut addr_list: BoundedVec<BoundedVec<u8, T::MaxResourceAddressLength>, T::MaxSigListHeight> = Default::default();
		// 	addr_list.try_push(addr).map_err(|()| Error::<T>::VCSigListOverflow)?;

		// 	VCSigList::<T>::insert(&iss_did, addr_list);

		// 	// emit event
		// 	Self::deposit_event(Event::VCredentialCreated(did_str, vc_addr));

		// 	Ok(())
		// }

		#[pallet::weight(0)] 
		pub fn add_resource(origin: OriginFor<T>, did_str: Vec<u8>, addr_uri: Vec<u8>, public: bool, name_: Vec<u8>, descr: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let uri: BoundedVec<_, T::MaxURILength> =
				addr_uri.clone().try_into().map_err(|()| Error::<T>::IpfsURIOverflow)?;

			let name: BoundedVec<_, T::MaxHashLength> = 
				name_.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let desc: BoundedVec<_, T::MaxHashLength> = 
				descr.clone().try_into().map_err(|()| Error::<T>::HashLengthOverflow)?;

			let mut _nw_name;
			let mut _nw_desc;

			if name.len() > 0 {
				_nw_name = Some(name);
			} else {
				_nw_name = None;
			}

			if desc.len() > 0 {
				_nw_desc = Some(desc);
			} else {
				_nw_desc = None;
			}

			let data: DataFile<T> = DataFile { 
				name: _nw_name,
				desc: _nw_desc,
				uri,
				created: T::TimeProvider::now().as_secs(),
				public
			};

			// record holdings
			let mut dlist: BoundedVec<DataFile<T>, T::MaxHoldingsCount> = Default::default();
				dlist.try_push(data).map_err(|()| Error::<T>::HoldingsListOverflow)?;

			// save to storage
			DataRegistry::<T>::insert(&did, dlist);

			// emit event
			Self::deposit_event(Event::DataAddedToNetwork(did_str, addr_uri));

			Ok(())
		}

		// const transfer = api.tx.samaritan.getResource(did, auth.is_auth, frags[1], frags[4], frags[5]);
		#[pallet::weight(0)] 
		pub fn fetch_resource(origin: OriginFor<T>, did_str: Vec<u8>, is_owner: bool, height: u64) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut _uri: Vec<u8>;

			match DataRegistry::<T>::get(&did) {
				Some(datafiles) => {
					match datafiles.get(height as usize) {
						Some(file) => {
							// check for privacy clause
							if !file.public && !is_owner {
								// throw error
								return Err(Error::<T>::ResourceIsPrivate.into());
							}
							_uri = file.uri.to_vec().clone();
						},
						None => {
							// throw error
							return Err(Error::<T>::ResourceNotFound.into());
						}
					}
				},
				None => {
					// throw error
					return Err(Error::<T>::ResourceNotFound.into());
				}
			}

			// emit event
			Self::deposit_event(Event::FetchResourceURI(_uri));

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

