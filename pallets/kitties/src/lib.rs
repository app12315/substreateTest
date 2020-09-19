#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
	traits::Currency, traits::ExistenceRequirement, traits::Randomness, StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use sp_runtime::traits::Hash;

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
		KittyAlreadyExists,
		KittyDoesNotExist,
		OverflowAddingToAccountBalance,
		OverflowAddingToTotalSupply,
		NoOwner,
		YouAreNotTheOwner,
		AccountIsNotTheOwner,
		TransferOverflowOfKittyBalance,
		TransferUnderflowOfKiittyBalance,
		YouCantBuyYourOwnKitty,
		KittyIsNotForSale,
	}
}

// Events =================================================
decl_event! {
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId,
		Hash = <T as frame_system::Trait>::Hash,
		Balance = <T as pallet_balances::Trait>::Balance
	{
		KittyCreated(AccountId, Hash),
		PriceSet(AccountId, Hash, Balance),
		Transferred(AccountId, AccountId, Hash),
		Bought(AccountId, AccountId, Hash, Balance),
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

// Internal Functions =====================================
impl<T: Trait> Module<T> {
	fn mint(
		to: T::AccountId,
		kitty_id: T::Hash,
		new_kitty: Kitty<T::Hash, T::Balance>,
	) -> DispatchResult {
		// Check that there is no kitty with this id
		ensure!(
			!KittyOwner::<T>::contains_key(kitty_id),
			Error::<T>::KittyAlreadyExists
		);

		// Calculate the own kitties count +1
		let owned_kitty_count = Self::owned_kitty_count(&to);
		let new_owned_kitty_count = owned_kitty_count
			.checked_add(1)
			.ok_or(Error::<T>::OverflowAddingToAccountBalance)?;

		// Calculate the total kitties count +1
		let all_kitties_count = Self::all_kitties_count();
		let new_all_kitties_count = all_kitties_count
			.checked_add(1)
			.ok_or(Error::<T>::OverflowAddingToTotalSupply)?;

		// Add new kitty to owner and to total supply
		Kitties::<T>::insert(kitty_id, new_kitty);
		KittyOwner::<T>::insert(kitty_id, &to);

		// Adds the kitty to the total "list"(~EnumerableStorageMap)
		AllKittiesArray::<T>::insert(all_kitties_count, kitty_id);
		AllKittiesCount::put(new_all_kitties_count);
		AllKittiesIndex::<T>::insert(kitty_id, all_kitties_count);

		// Adds the kitty to the owener "list"(~EnumerableStorageMap)
		OwnedKittiesArray::<T>::insert((to.clone(), owned_kitty_count), kitty_id);
		OwnedKittiesCount::<T>::insert(&to, new_owned_kitty_count);
		OwnedKittiesIndex::<T>::insert(kitty_id, owned_kitty_count);

		// Dispatch event
		Self::deposit_event(RawEvent::KittyCreated(to, kitty_id));
		return Ok(());
	}

	fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> DispatchResult {
		let owner = Self::owner_of(kitty_id).ok_or(Error::<T>::NoOwner)?;
		ensure!(owner == from, Error::<T>::AccountIsNotTheOwner);

		let owned_kitty_count_from = Self::owned_kitty_count(&from);
		let owned_kitty_count_to = Self::owned_kitty_count(&to);

		let new_owned_kitty_count_to = owned_kitty_count_to
			.checked_add(1)
			.ok_or(Error::<T>::TransferOverflowOfKittyBalance)?;

		let new_owned_kitty_count_from = owned_kitty_count_from
			.checked_sub(1)
			.ok_or(Error::<T>::TransferUnderflowOfKiittyBalance)?;

		let kitty_index = OwnedKittiesIndex::<T>::get(kitty_id);
		if kitty_index != new_owned_kitty_count_from {
			let last_kitty_id =
				OwnedKittiesArray::<T>::get((from.clone(), new_owned_kitty_count_from));
			OwnedKittiesArray::<T>::insert((from.clone(), kitty_index), last_kitty_id);
			OwnedKittiesIndex::<T>::insert(last_kitty_id, kitty_index);
		}

		KittyOwner::<T>::insert(&kitty_id, &to);
		OwnedKittiesIndex::<T>::insert(kitty_id, owned_kitty_count_to);

		OwnedKittiesArray::<T>::remove((from.clone(), new_owned_kitty_count_from));
		OwnedKittiesArray::<T>::insert((to.clone(), owned_kitty_count_to), kitty_id);

		OwnedKittiesCount::<T>::insert(&from, new_owned_kitty_count_from);
		OwnedKittiesCount::<T>::insert(&to, new_owned_kitty_count_to);

		// Dispatch event
		Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));
		return Ok(());
	}
}

// Public Functions =======================================
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

			// Events must be initialized if they are used by the pallet.
			fn deposit_event() = default;

		#[weight = 10_000]
		fn create_kitty(origin) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let nonce = Nonce::get();

			let random_hash = <pallet_randomness_collective_flip::Module<T>>::random(
				nonce.
				using_encoded(<T as frame_system::Trait>::Hashing::hash)
				.as_ref()
			)
			.using_encoded(<T as frame_system::Trait>::Hashing::hash);

			let new_kitty = Kitty {
				id: random_hash,
				dna: random_hash,
				price: Into::<T::Balance>::into(0),
				gen:0,
			};

			Self::mint(sender, random_hash, new_kitty)?;

			Nonce::mutate(|x| *x += 1);

			return Ok(());
		}

		#[weight = 10_000]
		fn set_price(origin, kitty_id: T::Hash, new_price: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::KittyDoesNotExist);

			let owner = Self::owner_of(kitty_id).ok_or(Error::<T>::NoOwner)?;
			ensure!(owner == sender, Error::<T>::YouAreNotTheOwner);

			let mut kitty = Self::kitty(kitty_id);
			kitty.price = new_price;

			Kitties::<T>::insert(kitty_id, kitty);

			// Dispatch event
			Self::deposit_event(RawEvent::PriceSet(sender,kitty_id, new_price));
			return Ok(());
		}

		#[weight = 10_000]
		fn transfer(origin, to: T::AccountId, kitty_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let owner = Self::owner_of(kitty_id).ok_or(Error::<T>::NoOwner)?;
			ensure!(owner == sender, Error::<T>::YouAreNotTheOwner);

			Self::transfer_from(sender, to, kitty_id)?;

			return Ok(());
		}

		#[weight = 20_000]
		fn buy_kitty(origin, kitty_id: T::Hash, max_price: T::Balance) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::KittyDoesNotExist);

			let owner = Self::owner_of(kitty_id).ok_or(Error::<T>::NoOwner)?;
			ensure!(owner != sender, Error::<T>::YouCantBuyYourOwnKitty);

			let mut kitty = Self::kitty(kitty_id);
			let kitty_price = kitty.price;

			ensure!(kitty_price == Into::<T::Balance>::into(0), Error::<T>::KittyIsNotForSale);
			ensure!(kitty_price <= max_price, "The cat you want to buy costs more than your max price");

			<pallet_balances::Module<T> as Currency<_>>::transfer(&sender, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;

			Self::transfer_from(owner.clone(), sender.clone(), kitty_id)
				.expect("`owner` is shown to own the kitty; \
				`owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
				`all_kitty_count` shares the same type as `owned_kitty_count` \
				and minting ensure there won't ever be more than `max()` kitties, \
				which means transfer cannot cause an overflow; \
				qed");

			kitty.price =Into::<T::Balance>::into(0);
			Kitties::<T>::insert(kitty_id, kitty);

			// Dispatch event
			Self::deposit_event(RawEvent::Bought(sender, owner, kitty_id, kitty_price));
			return Ok(());
		}
	}
}
