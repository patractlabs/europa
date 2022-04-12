// This file is part of europa

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

use std::sync::Arc;

use sp_database::error;
use sp_runtime::traits::{Block as BlockT, NumberFor};

pub trait StateKv<Block: BlockT>: Send + Sync {
	/// The transaction type used by the StateKv.
	type Transaction: StateKvTransaction + Default + Send + 'static;

	fn set_kv(&self, hash: Block::Hash, key: &[u8], value: Option<&[u8]>) -> error::Result<()>;
	fn set_child_kv(
		&self,
		hash: Block::Hash,
		child: &[u8],
		key: &[u8],
		value: Option<&[u8]>,
	) -> error::Result<()>;
	fn transaction(&self, hash: Block::Hash) -> Self::Transaction;
	fn commit(&self, t: Self::Transaction) -> error::Result<()>;

	fn get(&self, hash: Block::Hash, key: &[u8]) -> Option<Vec<u8>>;
	fn get_child(&self, hash: Block::Hash, child: &[u8], key: &[u8]) -> Option<Vec<u8>>;
	fn get_kvs_by_hash(&self, hash: Block::Hash) -> Option<Vec<(Vec<u8>, Option<Vec<u8>>)>>;
	fn get_child_kvs_by_hash(
		&self,
		hash: Block::Hash,
		child: &[u8],
	) -> Option<Vec<(Vec<u8>, Option<Vec<u8>>)>>;
	fn delete_kvs_by_hash(&self, hash: Block::Hash) -> error::Result<()>;
	fn delete_child_kvs_by_hash(&self, hash: Block::Hash, child: &[u8]) -> error::Result<()>;
	fn set_extrinsic_changes(
		&self,
		number: NumberFor<Block>,
		index: u32,
		json: String,
	) -> error::Result<()>;
	fn get_extrinsic_changes(&self, number: NumberFor<Block>, index: u32) -> Option<String>;
	fn delete_extrinsic_changes(&self, number: NumberFor<Block>, index: u32) -> error::Result<()>;

	// hash&number
	fn set_hash_and_number(&self, hash: Block::Hash, number: NumberFor<Block>)
		-> error::Result<()>;
	fn get_number(&self, hash: Block::Hash) -> Option<NumberFor<Block>>;
	fn get_hash(&self, number: NumberFor<Block>) -> Option<Block::Hash>;

	fn set_contract_tracing(
		&self,
		number: NumberFor<Block>,
		index: u32,
		tracing: String,
	) -> error::Result<()>;
	fn get_contract_tracing(&self, number: NumberFor<Block>, index: u32) -> Option<String>;
	fn remove_contract_tracing(&self, number: NumberFor<Block>, index: u32) -> error::Result<()>;
	fn remove_contract_tracings_by_number(&self, number: NumberFor<Block>) -> error::Result<()>;

	fn revert_all(&self, number: NumberFor<Block>) -> error::Result<()>;
}

pub trait StateKvTransaction {
	fn set_kv(&mut self, key: &[u8], value: Option<&[u8]>);
	fn set_child_kv(&mut self, child: &[u8], key: &[u8], value: Option<&[u8]>);
	fn remove(&mut self, key: &[u8]);
	fn remove_child(&mut self, key: &[u8], child: &[u8]);
	fn clear(&mut self);
}

pub trait ClientStateKv<B: BlockT, S: StateKv<B>> {
	fn state_kv(&self) -> Arc<S>;
}

impl<Block: BlockT, T: StateKv<Block>> StateKv<Block> for Arc<T> {
	type Transaction = T::Transaction;

	fn set_kv(&self, hash: Block::Hash, key: &[u8], value: Option<&[u8]>) -> error::Result<()> {
		(&**self).set_kv(hash, key, value)
	}

	fn set_child_kv(
		&self,
		hash: Block::Hash,
		child: &[u8],
		key: &[u8],
		value: Option<&[u8]>,
	) -> error::Result<()> {
		(&**self).set_child_kv(hash, child, key, value)
	}

	fn transaction(&self, hash: Block::Hash) -> Self::Transaction {
		(&**self).transaction(hash)
	}

	fn commit(&self, t: Self::Transaction) -> error::Result<()> {
		(&**self).commit(t)
	}

	fn get(&self, hash: Block::Hash, key: &[u8]) -> Option<Vec<u8>> {
		(&**self).get(hash, key)
	}

	fn get_child(&self, hash: Block::Hash, child: &[u8], key: &[u8]) -> Option<Vec<u8>> {
		(&**self).get_child(hash, child, key)
	}

	fn get_kvs_by_hash(&self, hash: Block::Hash) -> Option<Vec<(Vec<u8>, Option<Vec<u8>>)>> {
		(&**self).get_kvs_by_hash(hash)
	}

	fn get_child_kvs_by_hash(
		&self,
		hash: Block::Hash,
		child: &[u8],
	) -> Option<Vec<(Vec<u8>, Option<Vec<u8>>)>> {
		(&**self).get_child_kvs_by_hash(hash, child)
	}

	fn delete_kvs_by_hash(&self, hash: Block::Hash) -> error::Result<()> {
		(&**self).delete_kvs_by_hash(hash)
	}

	fn delete_child_kvs_by_hash(&self, hash: Block::Hash, child: &[u8]) -> error::Result<()> {
		(&**self).delete_child_kvs_by_hash(hash, child)
	}

	fn set_extrinsic_changes(
		&self,
		number: NumberFor<Block>,
		index: u32,
		json: String,
	) -> error::Result<()> {
		(&**self).set_extrinsic_changes(number, index, json)
	}
	fn get_extrinsic_changes(&self, number: NumberFor<Block>, index: u32) -> Option<String> {
		(&**self).get_extrinsic_changes(number, index)
	}
	fn delete_extrinsic_changes(&self, number: NumberFor<Block>, index: u32) -> error::Result<()> {
		(&**self).delete_extrinsic_changes(number, index)
	}

	fn set_hash_and_number(
		&self,
		hash: Block::Hash,
		number: NumberFor<Block>,
	) -> error::Result<()> {
		(&**self).set_hash_and_number(hash, number)
	}

	fn get_number(&self, hash: Block::Hash) -> Option<NumberFor<Block>> {
		(&**self).get_number(hash)
	}

	fn get_hash(&self, number: NumberFor<Block>) -> Option<Block::Hash> {
		(&**self).get_hash(number)
	}

	fn set_contract_tracing(
		&self,
		number: NumberFor<Block>,
		index: u32,
		tracing: String,
	) -> error::Result<()> {
		(&**self).set_contract_tracing(number, index, tracing)
	}

	fn get_contract_tracing(&self, number: NumberFor<Block>, index: u32) -> Option<String> {
		(&**self).get_contract_tracing(number, index)
	}

	fn remove_contract_tracing(&self, number: NumberFor<Block>, index: u32) -> error::Result<()> {
		(&**self).remove_contract_tracing(number, index)
	}

	fn remove_contract_tracings_by_number(&self, number: NumberFor<Block>) -> error::Result<()> {
		(&**self).remove_contract_tracings_by_number(number)
	}

	fn revert_all(&self, number: NumberFor<Block>) -> error::Result<()> {
		(&**self).remove_contract_tracings_by_number(number)
	}
}
