#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
	traits::Randomness, StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use sp_runtime::traits::Hash;
use sp_std::vec::Vec;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<Hash, Balance> {
	id: Hash,
	dna: Hash,
	price: Balance,
	gen: u64,
}

pub trait Trait: frame_system::Trait + pallet_balances::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// Errors =================================================
decl_error! {
	pub enum Error for Module<T: Trait> {
		KittyAlreadyExists
	}
}

// Events =================================================
decl_event! {
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId,
		Hash = <T as frame_system::Trait>::Hash
	{
		KittyCreated(AccountId, Hash),
	}
}

// Storage ================================================
decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		Kitties get(fn kitty): map hasher(blake2_128_concat) T::Hash => Kitty<T::Hash, T::Balance>;
		KittyOwner get(fn owner_of): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
		OwnedKitty get(fn kitty_of_owner): map hasher(blake2_128_concat) T::AccountId => T::Hash;
		Nonce: u64;
	}
}

// Functions ==============================================
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

			// Events must be initialized if they are used by the pallet.
			fn deposit_event() = default;

		#[weight = 10_000]
		fn create_kitty(origin) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let random_hash = <pallet_randomness_collective_flip::Module<T>>::random(&b"my context"[..]).using_encoded(<T as frame_system::Trait>::Hashing::hash);

			ensure!(!KittyOwner::<T>::contains_key(random_hash), Error::<T>::KittyAlreadyExists);

			let new_kitty = Kitty {
				id: random_hash,
				dna: random_hash,
				price: Into::<T::Balance>::into(5),
				gen:0,
			};

			Kitties::<T>::insert(random_hash, new_kitty);
			KittyOwner::<T>::insert(random_hash, &sender);
			OwnedKitty::<T>::insert(&sender, random_hash);

			Nonce::mutate(|x| *x += 1);

			Self::deposit_event(RawEvent::KittyCreated(sender, random_hash));
			return Ok(());
		}
	}
}
