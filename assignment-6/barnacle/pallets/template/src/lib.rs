#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`
    use scale_info::TypeInfo;

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct Cert {
        pub signature: Vec<u8>,
        pub expired_timestamp: u128,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when a proof has been claimed. [who, claim]
        CertCreated(T::AccountId, Cert),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        CertRevoked(T::AccountId, Cert),
        CertTransfered(T::AccountId, T::AccountId, Cert),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The proof has already been claimed.
        ProofAlreadyClaimed,
        /// The proof does not exist, so it cannot be revoked.
        NoSuchProof,
        /// The proof is claimed by another account, so caller can't revoke it.
        NotProofOwner,

        // Transfer to the owner itself
        SelfTransferUnacceptable,
    }

    

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    // #[pallet::generate_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Cert, (T::AccountId, T::BlockNumber), ValueQuery>;
    // pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, (Vec<u8>, u128), (T::AccountId, T::BlockNumber), ValueQuery>;
    // pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber), ValueQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    // Dispatchable functions allow users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(1_000)]
        pub fn create_cert(
            origin: OriginFor<T>,
            signature: Vec<u8>,
            expired_timestamp: u128,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;
            let cert = Cert {
                signature: signature,
                expired_timestamp: expired_timestamp,
            };

            // Verify that the specified proof has not already been claimed.
            ensure!(!Proofs::<T>::contains_key(&cert), Error::<T>::ProofAlreadyClaimed);

            // Get the block number from the FRAME System pallet.
            let current_block = <frame_system::Pallet<T>>::block_number();

            // Store the proof with the sender and block number.
            Proofs::<T>::insert(&cert, (&sender, current_block));

            // Emit an event that the claim was created.
            Self::deposit_event(Event::CertCreated(sender, cert));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn revoke_cert(
            origin: OriginFor<T>,
            signature: Vec<u8>,
            expired_timestamp: u128,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;
            let cert = Cert {
                signature: signature,
                expired_timestamp: expired_timestamp,
            };

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&cert), Error::<T>::NoSuchProof);

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&cert);

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&cert);

            // Emit an event that the claim was erased.
            Self::deposit_event(Event::CertRevoked(sender, cert));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_cert(
            origin: OriginFor<T>,
            receiver: T::AccountId,
            signature: Vec<u8>,
            expired_timestamp: u128,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let sender = ensure_signed(origin)?;
            let cert = Cert {
                signature: signature,
                expired_timestamp: expired_timestamp,
            };

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&cert), Error::<T>::NoSuchProof);

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&cert);

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotProofOwner);
            ensure!(sender != receiver, Error::<T>::SelfTransferUnacceptable);

            // Remove claim from storage.
            // Proofs::<T>::remove(&cert);
            // Send cert to receiver
            let current_block = <frame_system::Pallet<T>>::block_number();
            Proofs::<T>::remove(&cert);
            Proofs::<T>::insert(&cert, (&receiver, current_block));

            // Emit an event that the claim was erased.
            Self::deposit_event(Event::CertTransfered(sender, receiver, cert));
            Ok(())
        }
    }
}