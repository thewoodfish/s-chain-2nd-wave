#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::BoundedVec;

use scale_info::prelude::vec::Vec;
use scale_info::prelude::string::String;

#[frame_support::pallet]
pub mod pallet {
	// use core::str::FromStr;

	// use core::ops::Bound;
	// use parity_scale_codec::alloc::string::ToString;
	use scale_info::prelude::format;

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
		pub account_id: T::AccountId,
    }

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct DocMetadata<T: Config> {
		cid: BoundedVec<u8, T::MaxCIDLength>,
		created: u64
	}


	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct VCredential<T: Config> {
		index: u64,
		cid: BoundedVec<u8, T::MaxCIDLength>,
		subject: BoundedVec<u8, T::MaxDIDLength>,
		created: u64,
		public: bool
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
		type MaxCIDLength: Get<u32>;

		#[pallet::constant]
		type MaxVCLength: Get<u32>;

		#[pallet::constant]
		type MaxAssertions: Get<u32>;
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
	pub(super) type DocMetaRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, DocMetadata<T>>;

	#[pallet::storage]
	#[pallet::getter(fn doc_vcreg)]
	pub(super) type VCRegistry<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxDIDLength>, BoundedVec<VCredential<T>, T::MaxVCLength>>;

	#[pallet::storage]
	#[pallet::getter(fn assertions)]
	pub(super) type AssertionsList<T: Config> = StorageMap<_, Twox64Concat, BoundedVec<u8, T::MaxCIDLength>, BoundedVec<BoundedVec<u8, T::MaxDIDLength>, T::MaxAssertions>>;

	#[pallet::storage]
	#[pallet::getter(fn vc_nonce)]
	pub(super) type VCNonce<T: Config> = StorageValue<_, u64>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// status of name search
		NameSearchConcluded(Vec<u8>, bool),
		/// creation of a Samaritan
		SamaritanCreated(Vec<u8>, Vec<u8>),
		/// creation of DID document
		DIDDocumentCreated(Vec<u8>, Vec<u8>),
		/// retrieve DID of a Samaritan
		ConvertNameToDID(Vec<u8>, Vec<u8>),
		/// get the nonce for constructing a credential
		RetreiveVCredentialNonce(u64),
		/// a verifiable credential has been created
		VCredentialCreated(Vec<u8>, Vec<u8>, Vec<u8>),
		/// retrieve a Samaritans credential list
		RetrieveCredentialsList(Vec<u8>, Vec<u8>),
		/// retrieve the credential of a Samaritan
		RetrieveCredential(u64, Vec<u8>),
		/// assert a credential
		CredentialAsserted(Vec<u8>, Vec<u8>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// SamaritanName overflow
		SamaritanNameOverflow,
		/// DID length overflow
		DIDLengthOverflow,
		/// CID overflowed!,
		IpfsCIDOverflow,
		/// DID of a Samaritan could not be retrieved
		NameToDIDFailed,
		/// Verifiable Credential Overflow
		VCOverflow,
		/// Credential not found
		VCNotFound,
		/// Assertion List overflow
		AssertionsListOverflow
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// check for existence of a DID
		#[pallet::weight(0)]
		pub fn check_existence(origin: OriginFor<T>, value: Vec<u8>, is_did: bool) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let exists: bool;
			let did: BoundedVec<_, T::MaxDIDLength>;
			let s_name: BoundedVec<_, T::MaxSamNameLength>;

			// if vallue is a DID instead of a Samaritan name
			if is_did {
				did = value.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;
				(_, exists) = Self::get_sam_name(&did);
			} else {
				s_name = value.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;
				(_, exists) = Self::get_did(&s_name);
			}

			// deposit event
			Self::deposit_event(Event::NameSearchConcluded(value, exists));
			
			Ok(())
		}

		#[pallet::weight(0)]
		/// function to create a new Samaritan 
		pub fn create_samaritan(origin: OriginFor<T>, sam_name: Vec<u8>, did_str: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;
			
			let sam: Samaritan<T> = Samaritan {
				did: did.clone(),
				account_id: who
			};

			// register DID with its name
			DIDRegistry::<T>::insert(&did, sn.clone());

			// insert Samaritan into pool
			SamaritanPool::<T>::insert(&sn, sam);

			// emit event
			Self::deposit_event(Event::SamaritanCreated(sn.to_vec(), did_str));

			Ok(())
		}

		#[pallet::weight(0)]
		/// DID document has been created on the server, now record it onchain
		pub fn acknowledge_doc(origin: OriginFor<T>, sam_name: Vec<u8>, doc_cid: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let dc: BoundedVec<_, T::MaxCIDLength> =
				doc_cid.clone().try_into().map_err(|()| Error::<T>::IpfsCIDOverflow)?;

			// create metadata
			let doc: DocMetadata<T> = DocMetadata {
				cid: dc.clone(),
				created: T::TimeProvider::now().as_secs()
			};

			let did = Self::get_did(&sn).0;

			// insert into storage 
			DocMetaRegistry::<T>::insert(&did, doc);

			// emit event
			Self::deposit_event(Event::DIDDocumentCreated(did.to_vec(), doc_cid));

			Ok(())
		}

		#[pallet::weight(0)]
		/// retrieve the DID url of a Samaritan
		pub fn retrieve_did(origin: OriginFor<T>, sam_name: Vec<u8>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let sn: BoundedVec<_, T::MaxSamNameLength> =
				sam_name.clone().try_into().map_err(|()| Error::<T>::SamaritanNameOverflow)?;

			let res = Self::get_did(&sn);

			if res.1 {
				// emit event
				Self::deposit_event(Event::ConvertNameToDID(sn.to_vec(), res.0.to_vec()));
			} else {
				// throw error
				return Err(Error::<T>::NameToDIDFailed.into());
			}

			Ok(())
		}

		#[pallet::weight(0)]
		/// retrieve the nonce for constructing a verifiable credential for a Samaritan
		pub fn get_vc_nonce(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let mut c = 1;

			if VCNonce::<T>::exists() {
				if let Some(count) = VCNonce::<T>::get() {
					c = count;
				} 
			} else {
				// initialize
				VCNonce::<T>::put(1);
			}


			Self::deposit_event(Event::RetreiveVCredentialNonce(c));

			Ok(())
		}

		#[pallet::weight(0)]
		/// record the creation of a verifiable credential onchain
		pub fn record_credential(origin: OriginFor<T>, did_str: Vec<u8>, sbjct: Vec<u8>, cid_str: Vec<u8>, public: bool) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;
			
			let subject: BoundedVec<_, T::MaxDIDLength> = 
				sbjct.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let cid: BoundedVec<_, T::MaxCIDLength> =
				cid_str.clone().try_into().map_err(|()| Error::<T>::IpfsCIDOverflow)?;

			let mut index = 0;
			// get nonce
			if let Some(nonce) = VCNonce::<T>::get() {
				index = nonce;
			}

			// create record
			let vc = VCredential {
				index,
				cid: cid.clone(),
				subject: subject.clone(),
				created: T::TimeProvider::now().as_secs(),
				public
			};

			// get existing credentials
			match VCRegistry::<T>::get(&did) {
				Some(mut creds) => {
					creds.try_push(vc).map_err(|()| Error::<T>::VCOverflow)?;

					VCRegistry::<T>::insert(&did, creds);

					// increase nonce
					if let Some(n) = VCNonce::<T>::get() {
						VCNonce::<T>::put(n + 1);
					}
				},
				None => {
					// create new entry
					let mut creds: BoundedVec<VCredential<T>, T::MaxVCLength> = Default::default();

					creds.try_push(vc).map_err(|()| Error::<T>::VCOverflow)?;
						
					// insert into storage
					VCRegistry::<T>::insert(&did, creds);

					// increase nonce
					if let Some(n) = VCNonce::<T>::get() {
						VCNonce::<T>::put(n + 1);
					}
				}
			}

			Self::deposit_event(Event::VCredentialCreated(did.to_vec(), subject.to_vec(), cid.to_vec()));

			Ok(())
		}

		#[pallet::weight(0)]
		/// retrieve a list of credentials owned by a Samaritan
		pub fn list_credentials(origin: OriginFor<T>, did_str: Vec<u8>, is_auth: bool) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut istr: String = String::new();
		 
			match VCRegistry::<T>::get(&did) {
				Some(creds) =>  {
					for c in &creds {
						if is_auth {
							istr.push_str(format!("{}", c.index).as_str()); 
							istr.push_str("-");
						} else { // select only public ones
							if c.public {
								istr.push_str(format!("{}", c.index).as_str()); 
								istr.push_str("-");
							} 
						}
					}
				},
				None => {
					// do nothing
				}
			}

			Self::deposit_event(Event::RetrieveCredentialsList(did_str, Self::str_to_vec(istr)));

			Ok(())
		}

		#[pallet::weight(0)]
		/// retrieve a credential owned by a Samaritan
		pub fn get_credential(origin: OriginFor<T>, did_str: Vec<u8>, nonce: u64, is_same: bool) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut cid: BoundedVec<_, T::MaxCIDLength> = Default::default();
			let mut error = true;
			
			// make sure the nonce is greater than what was supplied
			if VCNonce::<T>::get() > Some(nonce) {
				match VCRegistry::<T>::get(&did) {
					Some(creds) =>  {
						for c in &creds {
							if c.index == nonce {
								if is_same || c.public {
									cid = c.cid.clone();
									error = false;
								}

								break;
							}
						}
					},
					None => {
						// do nothing
					}
				}
			} 

			if error {
				// throw error
				return Err(Error::<T>::VCNotFound.into());
			}

			Self::deposit_event(Event::RetrieveCredential(nonce, cid.to_vec()));

			Ok(())
		}

		// const tx = api.tx.samaritan.assertCredential(did, req.did, cid, nonce);
		#[pallet::weight(0)]
		/// retrieve a credential owned by a Samaritan
		pub fn assert_credential(origin: OriginFor<T>, did_str: Vec<u8>, asserter: Vec<u8>, cid_str: Vec<u8>, nonce: u64) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let as_did: BoundedVec<_, T::MaxDIDLength> = 
				asserter.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let cid: BoundedVec<_, T::MaxCIDLength> =
				cid_str.clone().try_into().map_err(|()| Error::<T>::IpfsCIDOverflow)?;

			match AssertionsList::<T>::get(&cid) {
				Some(mut list) => {
					list.try_push(as_did).map_err(|()| Error::<T>::AssertionsListOverflow)?;
					AssertionsList::<T>::insert(&cid, list);
				},
				None => {
					// create new record
					let list: BoundedVec<BoundedVec<u8, T::MaxDIDLength>, T::MaxAssertions> = Default::default();
					AssertionsList::<T>::insert(&cid, list);
				}
			}

			// update CID
			match VCRegistry::<T>::get(&did) {
				Some(mut creds) =>  {
					for mut c in &creds {
						if c.index == nonce {
							c.cid = cid.clone();

							break;
						}
					}
				},
				None => {
					// do nothing
				}
			}


			Self::deposit_event(Event::CredentialAsserted(cid.to_vec(), asserter));

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

		/// retrieve DID from storage
		pub fn get_did(
			sam_name: &BoundedVec<u8, T::MaxSamNameLength>
		) -> (BoundedVec<u8, T::MaxDIDLength>, bool) {
			// get DID from storage
			match SamaritanPool::<T>::get(sam_name) {
				Some(sam) => (sam.did, true) ,
				None => {
					let vec: BoundedVec<u8, T::MaxDIDLength> =  Default::default();
					(vec, false)
				}
			}
		}

		/// retrieve Samaritan name from DID
		pub fn get_sam_name(
			did: &BoundedVec<u8, T::MaxDIDLength>
		) -> (Vec<u8>, bool) {
			// get from registry
			match DIDRegistry::<T>::get(did) {
				Some(name) => (name.to_vec(), true) ,
				None => {
					let vec: Vec<u8> = Vec::new();
					(vec, false)
				}
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

