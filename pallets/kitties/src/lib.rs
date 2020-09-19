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

		AllKittiesArray get(fn kitty_by_index): map hasher(blake2_128_concat) u64 => T::Hash;
		AllKittiesCount get(fn all_kitties_count): u64;
		AllKittiesIndex: map hasher(blake2_128_concat) T::Hash => u64;

		OwnedKittiesArray get(fn kitty_of_owner_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => T::Hash;
		OwnedKittiesCount get(fn owned_kitty_count): map hasher(blake2_128_concat) T::AccountId => u64;
		OwnedKittiesIndex: map hasher(blake2_128_concat) T::Hash => u64;

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

			let owned_kitty_count = Self::owned_kitty_count(&sender);
			let new_owned_kitty_count = owned_kitty_count.checked_add(1)
			.ok_or("Overflow adding a new kitty to account balance")?;

			// Get Kitties count
			let all_kitties_count = Self::all_kitties_count();

			let new_all_kitties_count = all_kitties_count.checked_add(1)
			.ok_or("Overflow adding a new kitty to total supply")?;


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

			AllKittiesArray::<T>::insert(all_kitties_count,random_hash);
			AllKittiesCount::put(new_all_kitties_count);
			AllKittiesIndex::<T>::insert(random_hash, all_kitties_count);

			OwnedKittiesArray::<T>::insert((sender.clone(), owned_kitty_count),random_hash);
			OwnedKittiesCount::<T>::insert(&sender, new_owned_kitty_count);
			OwnedKittiesIndex::<T>::insert(random_hash, owned_kitty_count);

			Nonce::mutate(|x| *x += 1);

			Self::deposit_event(RawEvent::KittyCreated(sender, random_hash));
			return Ok(());
		}
	}
}
