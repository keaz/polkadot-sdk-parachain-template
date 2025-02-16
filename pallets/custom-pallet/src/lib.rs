#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::BoundedVec;
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // Configuration trait for the pallet.
    // Configuration trait for the pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Defines the event type for the pallet.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Add these type constants
        #[pallet::constant]
        type MaxNameLength: Get<u32> + scale_info::TypeInfo;

        #[pallet::constant]
        type MaxEmailLength: Get<u32> + scale_info::TypeInfo;

        #[pallet::constant]
        type MaxDocHashLength: Get<u32> + scale_info::TypeInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn identities)]
    pub type Identities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        IdentityInfo<T::MaxNameLength, T::MaxEmailLength, T::MaxDocHashLength>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn verifications)]
    pub type Verifications<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId, // Validator
        Blake2_128Concat,
        T::AccountId, // Identity owner
        bool,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The counter value has been set to a new value by Root.
        CounterValueSet {
            /// The new value set.
            counter_value: u32,
        },
        /// A user has successfully incremented the counter.
        CounterIncremented {
            /// The new value set.
            counter_value: u32,
            /// The account who incremented the counter.
            who: T::AccountId,
            /// The amount by which the counter was incremented.
            incremented_amount: u32,
        },
        /// A user has successfully decremented the counter.
        CounterDecremented {
            /// The new value set.
            counter_value: u32,
            /// The account who decremented the counter.
            who: T::AccountId,
            /// The amount by which the counter was decremented.
            decremented_amount: u32,
        },
        IdentityCreated(T::AccountId),
        IdentityVerified(T::AccountId, T::AccountId),
        IdentityRevoked(T::AccountId),
    }

    /// Storage for the current value of the counter.
    #[pallet::storage]
    pub type CounterValue<T> = StorageValue<_, u32>;

    /// Storage map to track the number of interactions performed by each account.
    #[pallet::storage]
    pub type UserInteractions<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u32>;

    #[pallet::error]
    pub enum Error<T> {
        /// The counter value exceeds the maximum allowed value.
        CounterValueExceedsMax,
        /// The counter value cannot be decremented below zero.
        CounterValueBelowZero,
        /// Overflow occurred in the counter.
        CounterOverflow,
        /// Overflow occurred in user interactions.
        UserInteractionOverflow,
        IdentityNotFound,
        AlreadyVerified,
        NotAuthorized,
        NameTooLong,
        EmailTooLong,
        DocHashTooLong,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_or_update_identity(
            origin: OriginFor<T>,
            name: Vec<u8>,
            email: Vec<u8>,
            document_hash: Vec<u8>,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;

            // Convert to bounded vectors
            let bounded_name = BoundedVec::<u8, T::MaxNameLength>::try_from(name)
                .map_err(|_| Error::<T>::NameTooLong)?;
            let bounded_email = BoundedVec::<u8, T::MaxEmailLength>::try_from(email)
                .map_err(|_| Error::<T>::EmailTooLong)?;
            let bounded_doc_hash = BoundedVec::<u8, T::MaxDocHashLength>::try_from(document_hash)
                .map_err(|_| Error::<T>::DocHashTooLong)?;

            let identity = IdentityInfo {
                name: bounded_name,
                email: bounded_email,
                document_hash: bounded_doc_hash,
                revoked: false,
            };

            Identities::<T>::insert(&user, identity);
            Self::deposit_event(Event::IdentityCreated(user));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn verify_identity(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            let validator = ensure_signed(origin)?;

            ensure!(
                Identities::<T>::contains_key(&target),
                Error::<T>::IdentityNotFound
            );
            ensure!(
                !Verifications::<T>::contains_key(&validator, &target),
                Error::<T>::AlreadyVerified
            );

            Verifications::<T>::insert(&validator, &target, true);
            Self::deposit_event(Event::IdentityVerified(validator, target));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn revoke_identity(origin: OriginFor<T>) -> DispatchResult {
            let user = ensure_signed(origin)?;
            Identities::<T>::mutate(&user, |identity| {
                if let Some(id) = identity {
                    id.revoked = true;
                }
            });
            Self::deposit_event(Event::IdentityRevoked(user));
            Ok(())
        }
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct IdentityInfo<NameLimit, EmailLimit, DocHashLimit>
    where
        NameLimit: Get<u32> + TypeInfo,
        EmailLimit: Get<u32> + TypeInfo,
        DocHashLimit: Get<u32> + TypeInfo,
    {
        pub name: BoundedVec<u8, NameLimit>,
        pub email: BoundedVec<u8, EmailLimit>,
        pub document_hash: BoundedVec<u8, DocHashLimit>,
        pub revoked: bool,
    }
}
