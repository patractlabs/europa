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
	fn commit(self, t: Self::Transaction) -> error::Result<()>;

	fn get(&self, hash: Block::Hash, key: &[u8]) -> Option<Vec<u8>>;
	fn get_child(&self, hash: Block::Hash, child: &[u8], key: &[u8]) -> Option<Vec<u8>>;
	fn get_kvs_by_hash(&self, hash: Block::Hash) -> Option<Vec<(Vec<u8>, Vec<u8>)>>;
	fn get_child_kvs_by_hash(
		&self,
		hash: Block::Hash,
		child: &[u8],
	) -> Option<Vec<(Vec<u8>, Vec<u8>)>>;
	fn delete_kvs_by_hash(&self, hash: Block::Hash) -> error::Result<()>;
	fn delete_child_kvs_by_hash(&self, hash: Block::Hash, child: &[u8]) -> error::Result<()>;

	// hash&number
	fn set_hash_and_number(&self, hash: Block::Hash, number: NumberFor<Block>)
		-> error::Result<()>;
	fn get_number(&self, hash: Block::Hash) -> Option<NumberFor<Block>>;
	fn get_hash(&self, number: NumberFor<Block>) -> Option<Block::Hash>;
}

pub trait StateKvTransaction {
	fn set_kv(&mut self, key: &[u8], value: Option<&[u8]>);
	fn set_child_kv(&mut self, child: &[u8], key: &[u8], value: Option<&[u8]>);
	fn remove(&mut self, key: &[u8]);
	fn remove_child(&mut self, key: &[u8], child: &[u8]);
	fn clear(&mut self);
}
