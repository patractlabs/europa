use parking_lot::RwLock;
use std::sync::Arc;

use sp_core::traits::{CodeExecutor, SpawnNamed};
use sp_runtime::{traits::Block as BlockT, BuildStorage};

use sc_client_api::execution_extensions::ExecutionExtensions;
use sc_client_db::{Backend, DatabaseSettings};
use sc_keystore::Store as Keystore;
use sc_service::{
	config::{Configuration, KeystoreConfig},
	error::Error,
};

use ec_executor::{NativeExecutionDispatch, NativeExecutor, RuntimeInfo};

use crate::client::Client;
use crate::task_manager::TaskManager;

/// Full client type.
pub type TFullClient<TBl, TRtApi, TExecDisp> =
	Client<TFullBackend<TBl>, TFullCallExecutor<TBl, TExecDisp>, TBl, TRtApi>;

/// Full client backend type.
pub type TFullBackend<TBl> = sc_client_db::Backend<TBl>;

/// Full client call executor type.
pub type TFullCallExecutor<TBl, TExecDisp> =
	crate::client::LocalCallExecutor<sc_client_db::Backend<TBl>, NativeExecutor<TExecDisp>>;

type TFullParts<TBl, TRtApi, TExecDisp> = (
	TFullClient<TBl, TRtApi, TExecDisp>,
	Arc<TFullBackend<TBl>>,
	Arc<RwLock<sc_keystore::Store>>,
	TaskManager,
);

/// Create the initial parts of a full node.
pub fn new_full_parts<TBl, TRtApi, TExecDisp>(
	config: &Configuration,
) -> Result<TFullParts<TBl, TRtApi, TExecDisp>, Error>
where
	TBl: BlockT,
	TExecDisp: NativeExecutionDispatch + 'static,
{
	let keystore = match &config.keystore {
		KeystoreConfig::Path { path, password } => Keystore::open(path.clone(), password.clone())?,
		KeystoreConfig::InMemory => Keystore::new_in_memory(),
	};

	let task_manager = TaskManager::new(config.task_executor.clone());

	let executor = NativeExecutor::<TExecDisp>::new();

	let chain_spec = &config.chain_spec;

	let (client, backend) = {
		let db_config = sc_client_db::DatabaseSettings {
			state_cache_size: config.state_cache_size,
			state_cache_child_ratio: config.state_cache_child_ratio.map(|v| (v, 100)),
			pruning: config.pruning.clone(),
			source: config.database.clone(),
		};

		let extensions = sc_client_api::execution_extensions::ExecutionExtensions::new(
			config.execution_strategies.clone(),
			Some(keystore.clone()),
		);

		new_client(
			db_config,
			executor,
			chain_spec.as_storage_builder(),
			extensions,
			Box::new(task_manager.spawn_handle()),
		)?
	};

	Ok((client, backend, keystore, task_manager))
}

/// Create an instance of db-backed client.
pub fn new_client<E, Block, RA>(
	settings: DatabaseSettings,
	executor: E,
	genesis_storage: &dyn BuildStorage,
	execution_extensions: ExecutionExtensions<Block>,
	spawn_handle: Box<dyn SpawnNamed>,
) -> Result<
	(
		crate::client::Client<
			Backend<Block>,
			crate::client::LocalCallExecutor<Backend<Block>, E>,
			Block,
			RA,
		>,
		Arc<Backend<Block>>,
	),
	sp_blockchain::Error,
>
where
	Block: BlockT,
	E: CodeExecutor + RuntimeInfo,
{
	const CANONICALIZATION_DELAY: u64 = 4096;

	let backend = Arc::new(Backend::new(settings, CANONICALIZATION_DELAY)?);
	let executor = crate::client::LocalCallExecutor::new(backend.clone(), executor, spawn_handle);
	Ok((
		crate::client::Client::new(
			backend.clone(),
			executor,
			genesis_storage,
			execution_extensions,
		)?,
		backend,
	))
}
