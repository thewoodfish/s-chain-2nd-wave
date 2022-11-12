//! Directory pallet
//! 
//! Files and directories are accessible using this pallet.
//! Each file and directory have different permissions used to manage access for the files.
//! 
//! 	permission: 000 - files or directories with this permission means no entity or user can access this file. 
//! 	permission: 400 - any permission equal to or greater than 4 means the user has read access.
//! 	permission: 700 - any permission equal to or greater than 7 means the user has read and write access.
//! 
//! Every time a file is created on the network, the permissions are checked to ensure specific groups have the authorization for creating, modifying or deleting files,

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
use scale_info::prelude::vec::Vec;
use scale_info::prelude::string::String;

#[frame_support::pallet]
pub mod pallet {
	// use core::default;

	use frame_support::{pallet_prelude::{*, DispatchResult}, BoundedVec};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use scale_info::prelude::vec::Vec;
	use scale_info::prelude::format;
	use frame_support::traits::UnixTime;

	// The Inode object  
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Inode {
		hash: H256, // TODO: remove this field
		metadata: H256,
		permission: u32,
		is_dir: bool,
		parent: Option<H256>,
		index: u64,
		created: u64,
		last_access: u64
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type TimeProvider: UnixTime;

		#[pallet::constant]
		type MaxInodeCount: Get<u32>;

		#[pallet::constant]
		type MaxDIDLength: Get<u32>;
	}


	/// trait to help manage the Samaritan filesystem from external pallets
	pub trait FileSystem {
		fn create_root_dir(did_str: Vec<u8>, hash: H256, metadata: H256) -> DispatchResult;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Storage map for tracking files and directories
	// This maps the DID with the Inode tree 
	#[pallet::storage]
	#[pallet::getter(fn file_reg)]
	pub(super) type InodeRegistry<T: Config> = 
		StorageMap<_, 
		Twox64Concat, 
		BoundedVec<u8, T::MaxDIDLength>, 
		BoundedVec<Inode, T::MaxInodeCount>
		>;

	#[pallet::storage]
	#[pallet::getter(fn dir_reg)]
	pub(super) type DirRegistry<T: Config> = StorageMap<_, Twox64Concat, H256, BoundedVec<H256, T::MaxInodeCount>>;

	#[pallet::storage]
	#[pallet::getter(fn inode_count)]
	pub(super) type InodeCount<T: Config> = StorageValue<_, u64>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// a new file has been uploaded to the internet
		NewInodeEntryCreated { did: Vec<u8>, file_hash: H256, is_dir: bool, height: u64 },
		/// samaritan file system root directory has been created
		RootDirCreated { did: Vec<u8>, hash: H256 },
		/// file metadata fetched, also containing documents in folder, if folder
		FileMetaDataFetched { meta: H256, files: Vec<H256> },
		/// inode has been deleted,
		InodeEntryDeleted { hash: H256, is_dir: bool },
		/// inode access permissions has been modified
		InodePermissionModified { hash: H256, old_mode: u32, mode: u32 },
		/// inode ownership has been transferred
		InodeOwnerChanged { hash: H256, old_owner: Vec<u8>, owner: Vec<u8> }
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// DID length overflow
		DIDLengthOverflow,
		/// Hash Length overflow
		HashLengthOverflow,
		/// Directory doesn't exist
		InvalidDirectory,
		/// Too many files
		InodeCountOverflow,
		/// Inode or directory inode not found
		InvalidInodeEntry,
		/// Cannot perform operation on inode
		PermissionDenied,
		/// Cannot delete a dir because its not empty
		DirectoryNotEmpty,
		/// Buffer overflow
		BufferOverflow
	}

	impl<T: Config> FileSystem for Pallet<T> {
		/// create the root node of a samaritan file system
		fn create_root_dir(did_str: Vec<u8>, hash: H256, metadata: H256) -> DispatchResult {
			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			// the gist is that the root folder is always the first file
			let root = Inode {
				hash: hash.clone(),
				metadata,
				permission: 700,	// very private by default
				is_dir: true,
				parent: None,
				index: 0,
				created: T::TimeProvider::now().as_secs(),
				last_access: T::TimeProvider::now().as_secs()
			};

			// create new 
			let mut files: BoundedVec<Inode, T::MaxInodeCount> = Default::default();
			files.try_push(root).map_err(|()| Error::<T>::InodeCountOverflow)?;

			// save to storage
			InodeRegistry::<T>::insert(&did, files);

			// crete entry
			let dir_root: BoundedVec<H256, T::MaxInodeCount> = Default::default();
				DirRegistry::<T>::insert(hash.clone(), dir_root);
		
			// increase inode count
			if let Some(count) = InodeCount::<T>::get() {
				InodeCount::<T>::put(count + 1);
			}
			
			// emit event
			Self::deposit_event(Event::RootDirCreated { did: did_str, hash } );

			Ok(())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)] 
		/// create a new file node
		pub fn add_inode(
			origin: OriginFor<T>, 
			did_str: Vec<u8>, 
			metadata: H256, 
			hash: H256, 
			p_dir_hash: H256, 
			is_dir: bool
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut file = Inode {
				// TODO: this could be the concat(Twox(dir_name) + Twox(method))
				// this would be more like the path to a file
				hash: hash.clone(),
				metadata,
				permission: 700,	// very private by default
				is_dir,
				parent: Some(p_dir_hash),
				index: 0, 	// this might change when we get find the height
				created: T::TimeProvider::now().as_secs(),
				last_access: T::TimeProvider::now().as_secs()
			};

			// height of nodes owned by a Samaritan
			let mut height = 0;

			// save under directory
			if let Some(mut dir) = DirRegistry::<T>::get(&p_dir_hash) {

				// select current lib
				match InodeRegistry::<T>::get(&did) {
					Some(mut files) => {
						height = files.len() as u64;

						// set height, for easy retrieval
						file.index = dir.len() as u64;
						
						files.try_push(file).map_err(|()| Error::<T>::InodeCountOverflow)?;
						InodeRegistry::<T>::insert(&did, files);
					},
					None => {
						// create new 
						let mut files: BoundedVec<Inode, T::MaxInodeCount> = Default::default();

						files.try_push(file).map_err(|()| Error::<T>::InodeCountOverflow)?;

						// save to storage
						InodeRegistry::<T>::insert(&did, files);
					}
				}

				// increase file count
				if let Some(count) = InodeCount::<T>::get() {
					InodeCount::<T>::put(count + 1);
				}

				dir.try_push(hash.clone()).map_err(|()| Error::<T>::InodeCountOverflow)?;
				DirRegistry::<T>::insert(&p_dir_hash, dir);

				// if directory, create a new entry
				if is_dir {
					let dir_root: BoundedVec<H256, T::MaxInodeCount> = Default::default();
					DirRegistry::<T>::insert(hash.clone(), dir_root);
				}

			} else {
				// throw error 
				return Err(Error::<T>::InvalidDirectory.into());
			}

			// emit event
			Self::deposit_event(Event::NewInodeEntryCreated { did: did_str, file_hash: hash, is_dir, height });

			Ok(())
		}

		#[pallet::weight(0)]
		/// fetch inode metadata
		pub fn fetch_metadata(
			origin: OriginFor<T>, 
			did_str: Vec<u8>, 
			owner_did: Vec<u8>, 
			index: u64, 
			hash: H256
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;   

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let o_did: BoundedVec<_, T::MaxDIDLength> = 
				owner_did.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut _metadata: H256;

			let mut files: Vec<H256> = Vec::new();

			// first select the file
			if let Some(inodes) = InodeRegistry::<T>::get(&o_did) {
				// find the specific entry
				let inode = &inodes[index as usize];

				// confirm inode is valid
				if inode.hash != hash {
					// throw error
					return Err(Error::<T>::InvalidInodeEntry.into());
				}

				// now check access permissions
				let perm = format!("{}", inode.permission);
				
				if did != o_did && u64::from(Self::str_to_vec(perm)[2]) < 4 {
					// throw error
					return Err(Error::<T>::PermissionDenied.into());
				} else {
					// get metadata
					_metadata = inode.metadata.clone();

					// if inode is for a directory, return all files/dirs it contains, 
					// along with metadata
					if inode.is_dir {
						if let Some(contained) = DirRegistry::<T>::get(&hash) {
							for f in contained {
								files.push(f.clone());
							}

						}
					}
				}
			} else {
				// throw error
				return Err(Error::<T>::InvalidInodeEntry.into());
			}

			// emit event
			Self::deposit_event(Event::FileMetaDataFetched { meta: _metadata, files } );

			Ok(())
		}

		#[pallet::weight(0)]
		/// delete a file or a directory
		pub fn unlink_inode(origin: OriginFor<T>, did_str: Vec<u8>, owner_did: Vec<u8>, index: u64, hash: H256) -> DispatchResult {
			let _who = ensure_signed(origin)?;   

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let o_did: BoundedVec<_, T::MaxDIDLength> = 
				owner_did.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut _is_dir = false;

			// first select the inode
			if let Some(mut inodes) = InodeRegistry::<T>::get(&o_did) {
				// find the specific entry
				let inode = &inodes[index as usize];

				// confirm inode is valid
				if inode.hash != hash {
					// throw error
					return Err(Error::<T>::InvalidInodeEntry.into());
				}

				// now check access permissions
				let perm = format!("{}", inode.permission);
				let bytes: [u8; 32] = [0; 32];
				let default: H256 = H256(bytes);
				
				if did != o_did && u64::from(Self::str_to_vec(perm)[2]) != 7 {	
					// throw error
					return Err(Error::<T>::PermissionDenied.into());
				} else {
					// if its a directory
					if inode.is_dir {
						// , make sure its empty
						if let Some(dir) = DirRegistry::<T>::get(&inode.hash) {
							if dir.len() != 0 {
								// throw error
								return Err(Error::<T>::DirectoryNotEmpty.into());
							} else {
								// dir is empty, deletion can proceed
								DirRegistry::<T>::remove(inode.hash);
							}
						}

						_is_dir = true;
					}

					// first unlink it from its parent
					if let Some(mut parent) = DirRegistry::<T>::get(&inode.parent.unwrap_or(default)) {
						// get the entry in the parents root
						parent.remove(inode.index as usize);	// panics
					}

					// delete inode entry
					inodes.remove(index as usize);
				}
			} else {
				// throw error
				return Err(Error::<T>::InvalidInodeEntry.into());
			}

			// emit event
			Self::deposit_event(Event::InodeEntryDeleted { hash, is_dir: _is_dir } );

			Ok(())
		}

		#[pallet::weight(0)]
		/// change the permission mode of an inode
		pub fn change_permission(origin: OriginFor<T>, did_str: Vec<u8>, owner_did: Vec<u8>, index: u64, hash: H256, mode: u32) -> DispatchResult {
			let _who = ensure_signed(origin)?;   

			let did: BoundedVec<_, T::MaxDIDLength> = 
				did_str.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let o_did: BoundedVec<_, T::MaxDIDLength> = 
				owner_did.clone().try_into().map_err(|()| Error::<T>::DIDLengthOverflow)?;

			let mut _is_dir = false;
			let mut _old_mode = 007;	// james bond, cool

			// first select the inode
			if let Some(mut inodes) = InodeRegistry::<T>::get(&o_did) {
				// find the specific entry
				let inode = &mut inodes[index as usize];

				// confirm inode is valid
				if inode.hash != hash {
					// throw error
					return Err(Error::<T>::InvalidInodeEntry.into());
				}

				// now check access permissions
				let perm = format!("{}", inode.permission);
				
				if did != o_did && u64::from(Self::str_to_vec(perm)[2]) != 7 {	
					// throw error
					return Err(Error::<T>::PermissionDenied.into());
				} else {
					// change permission
					_old_mode = inode.permission;

					let mut new_inode = inode.clone();
					new_inode.permission = mode;
					*inode = new_inode;

					InodeRegistry::<T>::insert(&o_did, inodes);
				}
			} else {
				// throw error
				return Err(Error::<T>::InvalidInodeEntry.into());
			}

			// emit event
			Self::deposit_event(Event::InodePermissionModified { hash, old_mode: _old_mode, mode } );

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

