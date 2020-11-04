use sp_database::error;
use sp_runtime::traits::Block as BlockT;

pub trait StateKv<Block: BlockT>: Send + Sync {
	/// The transaction type used by the StateKv.
	type Transaction: StateKvTransaction + Default + Send + 'static;

	fn set_kv(&self, hash: Block::Hash, key: &[u8], value: &[u8]) -> error::Result<()>;
	fn transaction(&self, hash: Block::Hash) -> Self::Transaction;
	fn commit(self, t: Self::Transaction) -> error::Result<()>;

	fn get(&self, hash: Block::Hash, key: &[u8]) -> Option<Vec<u8>>;
	fn get_kvs_by_hash(&self, hash: Block::Hash) -> Option<Vec<(Vec<u8>, Vec<u8>)>>;
	fn delete_kvs_by_hash(&self, hash: Block::Hash) -> error::Result<()>;
}

pub trait StateKvTransaction {
	fn set_kv(&mut self, key: &[u8], value: &[u8]);
	fn remove(&mut self, key: &[u8]);
	fn clear(&mut self);
}
