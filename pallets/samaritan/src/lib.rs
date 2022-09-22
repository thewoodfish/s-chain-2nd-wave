#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::BoundedVec;

use scale_info::prelude::vec::Vec;
use scale_info::prelude::string::String;
use scale_info::prelude::format;


#[frame_support::pallet]
pub mod pallet {
	// use core::str::FromStr;

	// use core::ops::Bound;
	// use parity_scale_codec::alloc::string::ToString;

	use frame_support::{pallet_prelude::{*, DispatchResult}, BoundedVec};
	use frame_system::pallet_prelude::*;

	use scale_info::prelude::vec::Vec;
	use scale_info::prelude::string::String;

	use frame_support::traits::{ UnixTime };

	// important structs
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Samaritan<T: Config> {
		pub did: BoundedVec<u8, T::MaxDIDLength>,
		pub account_id: T::AccountId
	}

	// important structs
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct WebSite<T: Config> {
		pub url: BoundedVec<u8, T::MaxWebSiteURL>,
		pub cid: BoundedVec<u8, T::MaxDocCIDLength>,
		pub access_count: u64
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DocMetadata<T: Config> {
		pub cid: BoundedVec<u8, T::MaxDocCIDLength>,
		pub created: u64,
		pub read_count: u64,
		pub active: bool
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct WebPageAccessList<T: Config> {
		pub url: BoundedVec<u8, T::MaxWebSiteURL>,
		pub cid: BoundedVec<u8, T::MaxDocCIDLength>,
		pub details_count: u32
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type TimeProvider: UnixTime;

		#[pallet::constant]
		type MaxDIDLength: Get<u32>;

		#[pallet::constant]
		type MaxSamNameLength: Get<u32>;

		#[pallet::constant]
		type MaxDocCIDLength: Get<u32>;

		#[pallet::constant]
		type MaxNames: Get<u128>;

		#[pallet::constant]
		type MaxWebSiteURL: Get<u32>;

		#[pallet::constant]
		type MaxWebSiteCount: Get<u32>;

		#[pallet::constant]
		type MaxSiteAccessCount: Get<u32>;
		
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sampool)]
	pub(super) type SamaritanPool<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxSamNameLength>, Samaritan<T>>;

	#[pallet::storage]
	#[pallet::getter(fn did_reg)]
	pub(super) type DIDRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<u8, T::MaxSamNameLength>>;

	#[pallet::storage]
	#[pallet::getter(fn doc_metareg)]
	pub(super) type DocMetaRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxSamNameLength>, DocMetadata<T>>;

	#[pallet::storage]
	#[pallet::getter(fn website_list)]
	pub(super) type WebSiteRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<WebSite<T>, T::MaxWebSiteCount>>;

	#[pallet::storage]
	#[pallet::getter(fn sa_list)]
	pub(super) type SiteAccessList<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxWebSiteURL>, BoundedVec<WebPageAccessList<T>, T::MaxSiteAccessCount>>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// status of name search
		NameSearchConcluded(Vec<u8>, bool),
		/// creation of a Samaritan
		SamaritanCreated(Vec<u8>, Vec<u8>),
		/// creation of DID document
		DIDiDocumentCreated(Vec<u8>, Vec<u8>),
		/// retrieve document CID
		GetDocumentCID(Vec<u8>, Vec<u8>, Vec<u8>),
		/// activate or deactivate Samaritan
		ChangeSamaritanVisibilty(Vec<u8>, Vec<u8>),
		/// website registered
		SiteAddedToNetwork(Vec<u8>, Vec<u8>),
		/// access list updated
		SiteAccessListUpdated(Vec<u8>, u32),
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
		DocumentCIDOverflow,
		/// WebSite URL length overflow
		WebSiteURLOverflow,
		/// Website Count Overflow
		WebSiteCountOverflow,
		/// Website access count overflow
		SiteAccessCountOverflow,
		/// access list update failed
		AccessListUpdateFailed
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// check for existence of a DID
		#[pallet::weight(0)]
		pub fn check_existence(origin: OriginFor<T>, sam_name: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// first check sam_name length
			let pn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let mut exist = false;
			let mut did: Vec<u8> = Vec::new();

			if let Some(_x) = SamaritanPool::<T>::get(&pn) {
				exist = true;
				did = Self::get_did(pn);
			}			

			// deposit event
			Self::deposit_event(Event::NameSearchConcluded(did, exist));
			
			Ok(())
		}

		#[pallet::weight(0)]
		/// function to create a new Samaritan 
		pub fn create_samaritan(origin: OriginFor<T>, sam_name: Vec<u8>, address: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let did = Self::create_did(address)?;
			
			let sam: Samaritan<T> = Samaritan {
				did: did.clone(),
				// doc_cid: rdoc,not recognized onc. Please enter the 
				account_id: who
			};

			// register DID with its name
			DIDRegistry::<T>::insert(did.clone(), sn.clone());

			// insert Samaritan into pool
			SamaritanPool::<T>::insert(sn, sam);

			// emit event
			Self::deposit_event(Event::SamaritanCreated(sam_name, did.to_vec()));

			Ok(())
		}

		#[pallet::weight(0)]
		/// DID document has been created on the server, now commit it to the chain
		pub fn acknowledge_doc(origin: OriginFor<T>, sam_name: Vec<u8>, doc_cid: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let dc: BoundedVec<_, T::MaxDocCIDLength> =
				doc_cid.clone().try_into().map_err(|()| Error::<T>::DocumentCIDOverflow)?;

			// create metadata
			let doc: DocMetadata<T> = DocMetadata {
				cid: dc.clone(),
				created: T::TimeProvider::now().as_secs(),
				read_count: 0,
				active: true
			};

			// insert into storage 
			DocMetaRegistry::<T>::insert(sn.clone(), doc);

			// emit event
			Self::deposit_event(Event::DIDiDocumentCreated(Self::get_did(sn), dc.to_vec()));

			Ok(())
		}

		#[pallet::weight(0)]
		/// retrieve document CID to perform read
		pub fn get_cid(origin: OriginFor<T>, sam_name: Vec<u8>, op: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			// retrieve document metadata
			let cid = match DocMetaRegistry::<T>::get(&sn) {
				Some(mut m) => {
					if op.contains(&b'd') {
						m.read_count += 1;
					} else {
						// write
					}

					let mv = m.cid.to_vec();

					// save to storage
					DocMetaRegistry::<T>::insert(&sn, m);

					mv
				},

				None => {
					// this can never occur though
					Vec::new()
				} 
			};

			// emit event
			Self::deposit_event(Event::GetDocumentCID(Self::get_did(sn), cid, op));

			Ok(())
		}

		#[pallet::weight(0)]
		/// retrieve document CID to perform read
		pub fn change_visibility(origin: OriginFor<T>, sam_name: Vec<u8>, state: bool) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			// retrieve document metadata
			match DocMetaRegistry::<T>::get(&sn) {
				Some(mut m) => {
					m.active = state;

					// save to storage
					DocMetaRegistry::<T>::insert(&sn, m);
				},

				None => {}
			};

			// emit event
			Self::deposit_event(Event::ChangeSamaritanVisibilty(
				Self::get_did(sn), 
				if state { Self::str_to_vec(String::from("activated")) } else { Self::str_to_vec(String::from("deactivated")) }
			));

			Ok(())
		}

		#[pallet::weight(0)]
		/// add webpage access to network
		pub fn add_website(origin: OriginFor<T>, link: Vec<u8>, cid: Vec<u8>, did_str: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let url: BoundedVec<_, T::MaxWebSiteURL> =
				link.clone().try_into().map_err(|()| Error::<T>::WebSiteURLOverflow)?;

			let dc: BoundedVec<_, T::MaxDocCIDLength> =
				cid.clone().try_into().map_err(|()| Error::<T>::DocumentCIDOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> =
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;


			let ws: WebSite<T> = WebSite {
				url,
				cid: dc,
				access_count: 0
			};

			// check first for any record
			match WebSiteRegistry::<T>::get(&did) {
				Some(mut sites) => {
					// add website to registry
					sites.try_push(ws).map_err(|()| Error::<T>::WebSiteCountOverflow)?;

					// insert into storage
					WebSiteRegistry::<T>::insert(&did, sites); 
				},
				None => {
					// add website to Vec first
					let mut sites: BoundedVec<WebSite<T>, T::MaxWebSiteCount> = Default::default();

					sites.try_push(ws).map_err(|()| Error::<T>::WebSiteCountOverflow)?;

					// insert into storage
					WebSiteRegistry::<T>::insert(&did, sites); 	
				}
			}

			// emit event
			Self::deposit_event(Event::SiteAddedToNetwork(did_str, link));

			Ok(())
		}

		#[pallet::weight(0)]
		/// add website to the network
		pub fn add_web_access(origin: OriginFor<T>, link: Vec<u8>, cid: Vec<u8>, did_str: Vec<u8>, details_count: u32) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let nlink = Self::extract_root(&link);
			let mut error = true;	// will be true if only we add the access
			let mut i = 0;

			let dc: BoundedVec<_, T::MaxDocCIDLength> =
				cid.clone().try_into().map_err(|()| Error::<T>::DocumentCIDOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> =
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let url: BoundedVec<_, T::MaxWebSiteURL> =
				link.clone().try_into().map_err(|()| Error::<T>::WebSiteURLOverflow)?;
			
			// first make sure a website with the root has been registered
			match WebSiteRegistry::<T>::get(&did) {
				Some(sites) => {
					for mut w in sites {
						if Self::vec_to_str(&w.url.encode()).contains(Self::vec_to_str(&nlink).as_str()) {	// select website
							// add access
							let access = WebPageAccessList {
								url,
								cid: dc,
								details_count
							};

							// now use the URl to index storage
							match SiteAccessList::<T>::get(&w.url) {
								Some(mut list) => {
									// update access list
									list.try_push(access).map_err(|()| Error::<T>::SiteAccessCountOverflow)?;
		
									// insert into storage
									SiteAccessList::<T>::insert(&w.url, list); 
									
									// completed!
									error = false;
								},
								None => {
									// create record
									let mut acs: BoundedVec<WebPageAccessList<T>, T::MaxSiteAccessCount> = Default::default();

									acs.try_push(access).map_err(|()| Error::<T>::SiteAccessCountOverflow)?;
										
									// insert into storage
									SiteAccessList::<T>::insert(&w.url, acs); 	

									// completed!
									error = false;
								}
							}

							break;
						}

						i += 1;	// increase index
					}

					// now update the website 
					if let Some(sites) = WebSiteRegistry::<T>::get(&did) {
						if let Some(mut web) = sites.get(i).take() {
							
							// update access count
							web.access_count += 1;

							WebSiteRegistry::<T>::insert(&did, sites);
						}
					}

				},
				None => {
					// do nothing
				}
			}

			if error {
				// throw error
				return Err(Error::<T>::AccessListUpdateFailed.into());
			} else {
				// emit event
				Self::deposit_event(Event::SiteAccessListUpdated(link, details_count));
			}

			Ok(())
		}

	}

	}

	

	/// helper functions
	impl<T: Config> Pallet<T> {
		/// create did of the form did:sam:root:<accountId>
		/// The accountId is passed to the function and concatenated to the DID method scheme
		pub fn create_did(
			vect: Vec<u8>
		) -> Result<BoundedVec<u8, T::MaxDIDLength>, Error<T>> {
			let did_str = format!("did:sam:root:{}", Self::vec_to_str(&vect));
			let did_vec = Self::str_to_vec(did_str);

			// to bounded vec
			let did: BoundedVec<_, T::MaxDIDLength> =
				did_vec.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			Ok(did)
		}

		/// convert account id to string
		pub fn vec_to_str(
			vector: &Vec<u8>
		) -> String {
			match String::from_utf8(vector.clone()) {
				Ok(s) => s,
				Err(_e) => String::from("00000000000000000000000000000000000000"),
			}
		}

		/// retrieve DID from storage
		pub fn get_did(
			sam_name: BoundedVec<u8, T::MaxSamNameLength>
		) -> Vec<u8> {
			// get DID from storage
			match SamaritanPool::<T>::get(&sam_name) {
				Some(sam) => sam.did.to_vec(),
				None => {
					let vec: Vec<u8> = Vec::new();
					vec
				}
			}
		}

		/// extract website root from URL
		pub fn extract_root(
			link: &Vec<u8>
		) -> Vec<u8> {
		
			let mut nlink = Vec::new();
			let mut n: usize = 0;
			let mut fs = 0;
		
			for s in link {
				if *s == b':' {
					for i in &link[n..] {
						if *i != b'/' {
							if *i != b':' {
								nlink.push(*i);
							}
						} else {
							if fs == 2 {
								break;
							}
		
							fs += 1;
						}
					}
		
					break;
				}
				n += 1;
			}
			nlink
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

