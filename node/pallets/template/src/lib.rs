#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::{DispatchResult, *},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Event emitted when a claim has been created.
		ClaimCreated { who: T::AccountId, claim: T::Hash },
		// Event emitted when a claim has been revoked.
		ClaimRevoked { who: T::AccountId, claim: T::Hash },
	}

	#[pallet::error]
	pub enum Error<T> {
		// The claim already exists.
		AlreadyClaimed,
		// The claim does not exist, so it cannot be revoked.
		NoSuchClaim,
		// The claim is owned by another account, so the caller can't revoke it.
		NotClaimOwner,
	}

	#[pallet::storage]
	pub(super) type Claims<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, T::BlockNumber)>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer
			// Thsi function will return an error if the extrinsic is not signed.
			let sender = ensure_signed(origin)?;

			// Verify that the specified claim has not already been stored
			ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

			// Get the block number from the FRAME system pallet
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Store the claim with the sender and block number
			Claims::<T>::insert(&claim, (&sender, current_block));

			// Emit an event that the claim was created
			Self::deposit_event(Event::ClaimCreated { who: sender, claim });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer
			// This function will return an error if the extrinsic is not signed
			let sender = ensure_signed(origin)?;

			// Get owner of the claim, if none return an error
			let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

			// Verify that sender of the current call is the claim owner
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			// Remove claim from the stroage
			Claims::<T>::remove(&claim);

			// Emit an event that the claim was erased
			Self::deposit_event(Event::ClaimRevoked { who: sender, claim });
			Ok(())
		}
	}
}
