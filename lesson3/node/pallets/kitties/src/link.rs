use frame_support::{StorageMap, Parameter};
use sp_runtime::traits::Member;
use codec::{Encode, Decode};

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct LinkedItem<Value> {
	pub prev: Option<Value>,
	pub next: Option<Value>,
}

pub struct LinkedList<Storage, Key, Value>(sp_std::marker::PhantomData<(Storage, Key, Value)>);

impl<Storage, Key, Value> LinkedList<Storage, Key, Value> where
	Value: Parameter + Member + Copy,
	Key: Parameter,
	Storage: StorageMap<(Key, Option<Value>), LinkedItem<Value>, Query = Option<LinkedItem<Value>>>,
{
	fn read_head(key: &Key) -> LinkedItem<Value> {
		Self::read(key, None)
	}

	fn write_head(account: &Key, item: LinkedItem<Value>) {
		Self::write(account, None, item);
	}

	fn read(key: &Key, value: Option<Value>) -> LinkedItem<Value> {
		Storage::get((&key, value)).unwrap_or_else(|| LinkedItem {
			prev: None,
			next: None,
		})
	}

	fn write(key: &Key, value: Option<Value>, item: LinkedItem<Value>) {
		Storage::insert((&key, value), item);
	}

	pub fn append(key: &Key, value: Value) {
		let item = Self::read_head(key);
		if let Some(next) = item.next {
			// 将该节点指向老的头部
			Self::write(key, Some(value), LinkedItem {
				prev: None,
				next: Some(next)
			});
			// 将老的头部的prev指向新的头部
			let old_head = Self::read(key, Some(next));
			Self::write(key, Some(next), LinkedItem {
				prev: Some(value),
				next: old_head.next
			});
			// 更新头节点的指向
			Self::write_head(key, LinkedItem {
				prev: item.prev,
				next: Some(value)
			});
		} else {
			// 如果是第一次插入，可以直接插入一个空的link
			Self::write(key, Some(value), LinkedItem {
				prev: None,
				next: None
			});
			// 再往头部更新
			Self::write(key, None, LinkedItem {
				prev: Some(value),
				next: Some(value)
			});
		}
	}

	pub fn remove(key: &Key, value: Value) {
		let item = Self::read(key, Some(value));
		let prev_item = Self::read(key, item.prev);
		let new_prev_item = LinkedItem {
			prev: prev_item.prev,
			next: item.next
		};
		Self::write(key, item.prev, new_prev_item);
		let next_item = Self::read(key, item.next);
		let new_next_item = LinkedItem {
			prev: item.prev,
			next: next_item.next
		};
		Self::write(key, item.next, new_next_item);
	}
}
