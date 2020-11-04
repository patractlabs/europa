use std::sync::Arc;

use kvdb::{DBTransaction, KeyValueDB};

use sp_database::error;
use sp_runtime::traits::Block as BlockT;

use sc_client_db::{DatabaseSettings, DatabaseSettingsSrc};

const SEPARATOR: u8 = '|' as u8;

pub const NUM_COLUMNS: u32 = 3;
/// Meta column. The set of keys in the column is shared by full storages.
pub const COLUMN_META: u32 = 0;

/// Keys of entries in COLUMN_META.
pub mod meta_keys {}

pub mod columns {
	pub const META: u32 = super::COLUMN_META;
	pub const STATE_KV: u32 = 1;
	// pub const STATE_META: u32 = 2;
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
	let mut k = Vec::with_capacity(hash.as_ref().len() + key.len() + 1);
	k.extend(hash.as_ref());
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
impl<B: BlockT> ec_client_api::statekv::StateKvTransaction for StateKvTransaction<B> {
	fn set_kv(&mut self, key: &[u8], value: &[u8]) {
		let real_key = real_key::<B>(self.hash, key);
		self.inner.put(columns::STATE_KV, &real_key, value)
	}
	fn remove(&mut self, key: &[u8]) {
		let real_key = real_key::<B>(self.hash, key);
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
impl<B: BlockT> ec_client_api::statekv::StateKv<B> for StateKv {
	type Transaction = StateKvTransaction<B>;
	fn set_kv(&self, hash: B::Hash, key: &[u8], value: &[u8]) -> error::Result<()> {
		let mut t = DBTransaction::with_capacity(1);
		let real_key = real_key::<B>(hash, key);
		t.put(columns::STATE_KV, &real_key, value);
		self.state_kv_db
			.write(t)
			.map_err(|e| error::DatabaseError(Box::new(e)))
	}
	fn transaction(&self, hash: B::Hash) -> Self::Transaction {
		StateKvTransaction {
			hash,
			inner: self.state_kv_db.transaction(),
		}
	}
	fn commit(self, t: Self::Transaction) -> error::Result<()> {
		self.state_kv_db
			.write(t.inner)
			.map_err(|e| error::DatabaseError(Box::new(e)))
	}

	fn get(&self, hash: B::Hash, key: &[u8]) -> Option<Vec<u8>> {
		let real_key = real_key::<B>(hash, key);
		handle_err(self.state_kv_db.get(columns::STATE_KV, &real_key))
	}

	fn get_kvs_by_hash(&self, hash: B::Hash) -> Option<Vec<(Vec<u8>, Vec<u8>)>> {
		let hash_len = hash.as_ref().len();
		let r = self
			.state_kv_db
			.iter_with_prefix(columns::STATE_KV, hash.as_ref())
			.map(|(k, v)| {
				assert_eq!(&k[hash_len], &SEPARATOR);
				((&k[hash_len + 1..]).to_vec(), (&v).to_vec())
			})
			.collect::<Vec<_>>();
		if r.len() == 0 {
			None
		} else {
			Some(r)
		}
	}

	fn delete_kvs_by_hash(&self, hash: B::Hash) -> error::Result<()> {
		let mut t = DBTransaction::with_capacity(1);
		t.delete_prefix(columns::STATE_KV, hash.as_ref());
		self.state_kv_db
			.write(t)
			.map_err(|e| error::DatabaseError(Box::new(e)))
	}
}
