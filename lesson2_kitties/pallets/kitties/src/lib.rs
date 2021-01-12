#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_module, decl_storage, decl_error, decl_event, ensure, StorageValue, StorageMap, Parameter,
	traits::{Randomness, Currency, ExistenceRequirement},
};
use sp_io::hashing::blake2_128;
use frame_system::{self as system, ensure_signed};
use sp_runtime::{DispatchError, traits::{AtLeast32Bit, Bounded, Member}};
use crate::linked_item::{LinkedList, LinkedItem};

mod linked_item;

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
	type Currency: Currency<Self::AccountId>;
	type Randomness: Randomness<Self::Hash>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type KittyLinkedItem<T> = LinkedItem<<T as Trait>::KittyIndex>;
type OwnedKittiesList<T> = LinkedList<OwnedKitties<T>, <T as system::Trait>::AccountId, <T as Trait>::KittyIndex>;

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// 存储所有的kitties
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
		/// 存储kitties 总数
		pub KittiesCount get(fn kitties_count): T::KittyIndex;

		/// 用链表的方式存储 ，每个用户当前拥有的kitties.
		pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat) (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
		/// 每个kitty 的 主人
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
        //  存储kitty 的价格。
		pub KittyPrices get(fn kitty_price): map hasher(blake2_128_concat) T::KittyIndex => Option<BalanceOf<T>>;


		// 父母，配偶，子女，兄弟姐妹等关系在 函数 do_breed() 中处理
        //  某一个kitty 的父母们
		pub KittyParents get(fn kitty_parents): map hasher(blake2_128_concat) (Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
		
		//某一个kitty 的配偶们
		pub KittyMates get(fn kitty_mates): map hasher(blake2_128_concat) (Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;

		//某一个kitty 的子女们
		pub KittyChildren get(fn kitty_children): map hasher(blake2_128_concat) (Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;

        //某一个kitty 的兄弟姐妹们
		pub KittyBrothers get(fn kitty_brothers): map hasher(blake2_128_concat) (Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverflow,
		InvalidKittyId,
		RequireDifferentParent,
		RequireOwner,
		NotForSale,
		PriceTooLow,
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::KittyIndex,
		Balance = BalanceOf<T>,
	{
		/// A kitty is created. (owner, kitty_id)
		Created(AccountId, KittyIndex),
		/// A kitty is transferred. (from, to, kitty_id)
		Transferred(AccountId, AccountId, KittyIndex),
		/// A kitty is available for sale. (owner, kitty_id, price)
		Ask(AccountId, KittyIndex, Option<Balance>),
		/// A kitty is sold. (from, to, kitty_id, price)
		Sold(AccountId, AccountId, KittyIndex, Balance),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Create a new kitty
		#[weight = 0]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;

			// Generate a random 128bit value
			let dna = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty(dna);
			Self::insert_kitty(&sender, kitty_id, kitty);

			Self::deposit_event(RawEvent::Created(sender, kitty_id));
			
            // 质押token
			Self::reserve_funds(origin, 1000);
		}

		/// Breed kitties
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;

			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));

			// 质押token
			Self::reserve_funds(origin, 1000);
		}

		/// Transfer a kitty to new owner
		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id))), Error::<T>::RequireOwner);

			Self::do_transfer(&sender, &to, kitty_id);

			Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));

			//转移质押
			Self::transfer_funds(origin, to, 1000);
		}

		/// Set a price for a kitty for sale
		/// None to delist the kitty
		#[weight = 0]
 		pub fn ask(origin, kitty_id: T::KittyIndex, new_price: Option<BalanceOf<T>>) {
			let sender = ensure_signed(origin)?;

			ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id))), Error::<T>::RequireOwner);

			<KittyPrices<T>>::mutate_exists(kitty_id, |price| *price = new_price);

			Self::deposit_event(RawEvent::Ask(sender, kitty_id, new_price));
		}

		/// Buy a kitty
		#[weight = 0]
		pub fn buy(origin, kitty_id: T::KittyIndex, price: BalanceOf<T>) {
			let sender = ensure_signed(origin)?;

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;

			let kitty_price = Self::kitty_price(kitty_id).ok_or(Error::<T>::NotForSale)?;

			ensure!(price >= kitty_price, Error::<T>::PriceTooLow);

			T::Currency::transfer(&sender, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;

			<KittyPrices<T>>::remove(kitty_id);

			Self::do_transfer(&owner, &sender, kitty_id);

			Self::deposit_event(RawEvent::Sold(owner, sender, kitty_id, kitty_price));
		}


		/// Reserves the specified amount of funds from the caller
		#[weight = 10_000]
		pub fn reserve_funds(origin, amount: BalanceOf<T>) -> DispatchResult {
			let locker = ensure_signed(origin)?;

			T::Currency::reserve(&locker, amount)
					.map_err(|_| "locker can't afford to lock the amount requested")?;

			let now = <system::Module<T>>::block_number();

			Self::deposit_event(RawEvent::LockFunds(locker, amount, now));
			Ok(())
		}

		/// Unreserves the specified amount of funds from the caller
		#[weight = 10_000]
		pub fn unreserve_funds(origin, amount: BalanceOf<T>) -> DispatchResult {
			let unlocker = ensure_signed(origin)?;

			T::Currency::unreserve(&unlocker, amount);
			// ReservableCurrency::unreserve does not fail (it will lock up as much as amount)

			let now = <system::Module<T>>::block_number();

			Self::deposit_event(RawEvent::UnlockFunds(unlocker, amount, now));
			Ok(())
		}

		/// Transfers funds. Essentially a wrapper around the Currency's own transfer method
		#[weight = 10_000]
		pub fn transfer_funds(origin, dest: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			T::Currency::transfer(&sender, &dest, amount, AllowDeath)?;

			let now = <system::Module<T>>::block_number();

			Self::deposit_event(RawEvent::TransferFunds(sender, dest, amount, now));
			Ok(())
		}

	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	(selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}

	fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err(Error::<T>::KittiesCountOverflow.into());
		}
		Ok(kitty_id)
	}

	fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
		<OwnedKittiesList<T>>::append(owner, kitty_id);
		<KittyOwners<T>>::insert(kitty_id, owner);
	}

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		// Create and store kitty
		Kitties::<T>::insert(kitty_id, kitty);
		KittiesCount::<T>::put(kitty_id + 1.into());

		Self::insert_owned_kitty(owner, kitty_id);
	}

	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
		let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

		ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id_1))), Error::<T>::RequireOwner);
		ensure!(<OwnedKitties<T>>::contains_key((&sender, Some(kitty_id_2))), Error::<T>::RequireOwner);
		ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

		let kitty_id = Self::next_kitty_id()?;

		let kitty1_dna = kitty1.0;
		let kitty2_dna = kitty2.0;

		// Generate a random 128bit value
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		// Combine parents and selector to create new kitty
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));


		// 处理 父母，夫妻，兄弟姐妹，子女等关系。

		<KittyParents<T>>::append(&kitty_id, kitty_id_1);
		<KittyParents<T>>::append(&kitty_id, kitty_id_2);
		<KittyMates<T>>::append(&kitty_id_1, kitty_id_2);
		<KittyMates<T>>::append(&kitty_id_2, kitty_id_1);
		<KittyChildren<T>>::append(&kitty_id_1, kitty_id);
		<KittyChildren<T>>::append(&kitty_id_2, kitty_id);

		kitty_id1_childRen = <KittyChildren<T>>::get(&kitty_id_1);
		for i in 0..kitty_id1_childRen.len() {
			<KittyBrothers<T>>::append(&kitty_id,kitty_id1_childRen[i])
		}

		kitty_id2_childRen = <KittyChildren<T>>::get(&kitty_id_2);
		for i in 0..kitty_id2_childRen.len() {
			<KittyBrothers<T>>::append(&kitty_id,kitty_id2_childRen[i])
		}

		Ok(kitty_id)
	}

	fn do_transfer(from: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex)  {
		<OwnedKittiesList<T>>::remove(&from, kitty_id);
		Self::insert_owned_kitty(&to, kitty_id);
	}
}

}