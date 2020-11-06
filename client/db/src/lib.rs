use std::sync::Arc;

use kvdb::{DBTransaction, KeyValueDB};

use sp_database::error;
use sp_runtime::{
	traits::{Block as BlockT, NumberFor},
	SaturatedConversion,
};

use sc_client_db::{DatabaseSettings, DatabaseSettingsSrc};

const SEPARATOR: u8 = b'|';

pub const NUM_COLUMNS: u32 = 7;
/// Meta column. The set of keys in the column is shared by full storages.
pub const COLUMN_META: u32 = 0;

/// Keys of entries in COLUMN_META.
pub mod meta_keys {}

pub mod columns {
	pub const META: u32 = super::COLUMN_META;
	pub const STATE_KV: u32 = 1;
	pub const STATE_CHILD_KV: u32 = 2;
	pub const STATE_KV_INDEX: u32 = 3; // TODO use index to improve query or other things
	pub const STATE_CHILD_KV_INDEX: u32 = 4;
	pub const HASH_TO_NUMBER: u32 = 5;
	pub const NUMBER_TO_HASH: u32 = 6;
}

const DB_PATH_NAME: &'static str = "state_kv";

pub fn open_state_key_database(
	config: &DatabaseSettings,
) -> sp_blockchain::Result<Arc<dyn KeyValueDB>> {
	#[allow(unused)]
	fn db_open_error(feat: &'static str) -> sp_blockchain::Error {
		sp_blockchain::Error::Backend(format!(
			"`{}` feature not enabled, database can not be opened",
			feat
		))
	}

	let db: Arc<dyn KeyValueDB> = match &config.source {
		DatabaseSettingsSrc::RocksDb {
			path,
			cache_size: _,
		} => {
			// and now open database assuming that it has the latest version
			let mut db_config = kvdb_rocksdb::DatabaseConfig::with_columns(NUM_COLUMNS);

			let mut path = path.to_path_buf();
			// remove "/db" from path
			if !path.pop() {
				return Err(db_open_error("NOT a valid path"));
			}
			path.push(format!("db_{}", DB_PATH_NAME));
			let path = path
				.to_str()
				.ok_or_else(|| sp_blockchain::Error::Backend("Invalid database path".into()))?;

			log::trace!(
				target: "db",
				"Open RocksDB state kv database at {}, column number ({})",
				path,
				NUM_COLUMNS,
			);
			let memory_budget = std::collections::HashMap::new();
			db_config.memory_budget = memory_budget;

			let db = kvdb_rocksdb::Database::open(&db_config, &path)
				.map_err(|err| sp_blockchain::Error::Backend(format!("{}", err)))?;
			Arc::new(db)
		}
		DatabaseSettingsSrc::ParityDb { path: _ } => return Err(db_open_error("with-parity-db")),
		DatabaseSettingsSrc::Custom(_) => return Err(db_open_error("with-custom-db")),
	};
	Ok(db)
}

pub struct StateKv {
	state_kv_db: Arc<dyn KeyValueDB>,
}

impl StateKv {
	pub fn new(config: &DatabaseSettings) -> sp_blockchain::Result<Self> {
		let db = open_state_key_database(config)?;
		Ok(StateKv { state_kv_db: db })
	}
}

fn real_key<B: BlockT>(hash: B::Hash, key: &[u8]) -> Vec<u8> {
	let mut k = Vec::with_capacity(hash.as_ref().len() + 1 + key.len());
	k.extend(hash.as_ref());
	k.push(SEPARATOR);
	k.extend(key);
	k
}

fn real_child_key<B: BlockT>(hash: B::Hash, child: &[u8], key: &[u8]) -> Vec<u8> {
	let mut k = Vec::with_capacity(hash.as_ref().len() + 1 + child.len() + 1 + key.len());
	k.extend(hash.as_ref());
	k.push(SEPARATOR);
	k.extend(child);
	k.push(SEPARATOR);
	k.extend(key);
	k
}

pub struct StateKvTransaction<B: BlockT> {
	hash: B::Hash,
	inner: DBTransaction,
}
impl<B: BlockT> Default for StateKvTransaction<B> {
	fn default() -> Self {
		StateKvTransaction {
			hash: B::Hash::default(),
			inner: Default::default(),
		}
	}
}
impl<B: BlockT> StateKvTransaction<B> {
	fn set_kv_impl(&mut self, col: u32, real_key: &[u8], value: Option<&[u8]>) {
		if let Some(value) = value {
			self.inner.put(col, real_key, value);
		} else {
			self.inner.delete(col, real_key);
		}
	}
}
impl<B: BlockT> ec_client_api::statekv::StateKvTransaction for StateKvTransaction<B> {
	fn set_kv(&mut self, key: &[u8], value: Option<&[u8]>) {
		let real_key = real_key::<B>(self.hash, key);
		self.set_kv_impl(columns::STATE_KV, &real_key, value);
	}

	fn set_child_kv(&mut self, child: &[u8], key: &[u8], value: Option<&[u8]>) {
		let real_key = real_child_key::<B>(self.hash, child, key);
		self.set_kv_impl(columns::STATE_CHILD_KV, &real_key, value);
	}

	fn remove(&mut self, key: &[u8]) {
		let real_key = real_key::<B>(self.hash, key);
		self.inner.delete(columns::STATE_KV, &real_key);
	}
	fn remove_child(&mut self, key: &[u8], child: &[u8]) {
		let real_key = real_child_key::<B>(self.hash, child, key);
		self.inner.delete(columns::STATE_KV, &real_key);
	}
	fn clear(&mut self) {
		self.inner.ops.clear();
	}
}

fn handle_err<T>(result: std::io::Result<T>) -> T {
	match result {
		Ok(r) => r,
		Err(e) => {
			panic!("Critical database eror: {:?}", e);
		}
	}
}

impl StateKv {
	fn set_kv_impl(&self, col: u32, real_key: &[u8], value: Option<&[u8]>) -> error::Result<()> {
		let mut t = DBTransaction::with_capacity(1);
		if let Some(value) = value {
			t.put(col, real_key, value);
		} else {
			t.delete(col, real_key);
		}
		self.state_kv_db
			.write(t)
			.map_err(|e| error::DatabaseError(Box::new(e)))
	}

	fn get_kys_impl(
		&self,
		col: u32,
		prefix: &[u8],
		f: impl FnMut((Box<[u8]>, Box<[u8]>)) -> (Vec<u8>, Vec<u8>),
	) -> Option<Vec<(Vec<u8>, Vec<u8>)>> {
		let r = self
			.state_kv_db
			.iter_with_prefix(col, prefix)
			.map(f)
			.collect::<Vec<(Vec<u8>, Vec<u8>)>>();
		if r.len() == 0 {
			None
		} else {
			Some(r)
		}
	}
}
impl<B: BlockT> ec_client_api::statekv::StateKv<B> for StateKv {
	type Transaction = StateKvTransaction<B>;
	fn set_kv(&self, hash: B::Hash, key: &[u8], value: Option<&[u8]>) -> error::Result<()> {
		let real_key = real_key::<B>(hash, key);
		self.set_kv_impl(columns::STATE_KV, &real_key, value)
	}

	fn set_child_kv(
		&self,
		hash: B::Hash,
		child: &[u8],
		key: &[u8],
		value: Option<&[u8]>,
	) -> error::Result<()> {
		let real_key = real_child_key::<B>(hash, child, key);
		self.set_kv_impl(columns::STATE_CHILD_KV, &real_key, value)
	}

	fn transaction(&self, hash: B::Hash) -> Self::Transaction {
		StateKvTransaction {
			hash,
			inner: self.state_kv_db.transaction(),
		}
	}
	fn commit(&self, t: Self::Transaction) -> error::Result<()> {
		self.state_kv_db
			.write(t.inner)
			.map_err(|e| error::DatabaseError(Box::new(e)))
	}

	fn get(&self, hash: B::Hash, key: &[u8]) -> Option<Vec<u8>> {
		let real_key = real_key::<B>(hash, key);
		handle_err(self.state_kv_db.get(columns::STATE_KV, &real_key))
	}
	fn get_child(&self, hash: B::Hash, child: &[u8], key: &[u8]) -> Option<Vec<u8>> {
		let real_key = real_child_key::<B>(hash, child, key);
		handle_err(self.state_kv_db.get(columns::STATE_CHILD_KV, &real_key))
	}

	fn get_kvs_by_hash(&self, hash: B::Hash) -> Option<Vec<(Vec<u8>, Vec<u8>)>> {
		let prefix = hash.as_ref();
		let hash_len = prefix.len();
		self.get_kys_impl(columns::STATE_KV, prefix, |(k, v)| {
			assert_eq!(&k[hash_len], &SEPARATOR);
			((&k[hash_len + 1..]).to_vec(), (&v).to_vec())
		})
	}

	fn get_child_kvs_by_hash(
		&self,
		hash: B::Hash,
		child: &[u8],
	) -> Option<Vec<(Vec<u8>, Vec<u8>)>> {
		let prefix = hash.as_ref();
		let hash_len = prefix.len();
		let mut lookup_key = Vec::with_capacity(hash_len + 1 + child.len());
		lookup_key.extend(prefix);
		lookup_key.push(SEPARATOR);
		lookup_key.extend(child);

		let lookup_key_len = lookup_key.len();

		self.get_kys_impl(columns::STATE_CHILD_KV, &lookup_key, |(k, v)| {
			assert_eq!(&k[lookup_key_len], &SEPARATOR);
			let sub = &k[lookup_key_len + 1..];
			(sub.to_vec(), (&v).to_vec())
		})
	}

	fn delete_kvs_by_hash(&self, hash: B::Hash) -> error::Result<()> {
		let mut t = DBTransaction::with_capacity(1);
		t.delete_prefix(columns::STATE_KV, hash.as_ref());
		self.state_kv_db
			.write(t)
			.map_err(|e| error::DatabaseError(Box::new(e)))
	}

	fn delete_child_kvs_by_hash(&self, hash: B::Hash, child: &[u8]) -> error::Result<()> {
		let prefix = hash.as_ref();
		let hash_len = prefix.len();
		let mut lookup_key = Vec::with_capacity(hash_len + 1 + child.len());
		lookup_key.extend(prefix);
		lookup_key.push(SEPARATOR);
		lookup_key.extend(child);

		let mut t = DBTransaction::with_capacity(1);
		t.delete_prefix(columns::STATE_CHILD_KV, &lookup_key);
		self.state_kv_db
			.write(t)
			.map_err(|e| error::DatabaseError(Box::new(e)))
	}

	// hash&number
	fn set_hash_and_number(&self, hash: B::Hash, number: NumberFor<B>) -> error::Result<()> {
		let number: u64 = number.saturated_into::<u64>();
		let bytes = &number.to_le_bytes()[..];
		self.set_kv_impl(columns::HASH_TO_NUMBER, hash.as_ref(), Some(bytes))?;
		self.set_kv_impl(columns::NUMBER_TO_HASH, bytes, Some(hash.as_ref()))
	}
	fn get_number(&self, hash: B::Hash) -> Option<NumberFor<B>> {
		let r = handle_err(self.state_kv_db.get(columns::HASH_TO_NUMBER, hash.as_ref()));
		r.map(|v| {
			let mut bytes = [0_u8; 8];
			bytes.copy_from_slice(v.as_slice());
			let number = u64::from_le_bytes(bytes);
			NumberFor::<B>::saturated_from::<u64>(number)
		})
	}
	fn get_hash(&self, number: NumberFor<B>) -> Option<B::Hash> {
		let number: u64 = number.saturated_into::<u64>();
		let bytes = &number.to_le_bytes()[..];
		let r = handle_err(self.state_kv_db.get(columns::NUMBER_TO_HASH, bytes));
		r.map(|v| {
			let mut hash = B::Hash::default();
			hash.as_mut().copy_from_slice(v.as_slice());
			hash
		})
	}
}
