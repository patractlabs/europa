use std::sync::Arc;

use futures::StreamExt;
use jsonrpc_pubsub::manager::SubscriptionManager;
use parking_lot::RwLock;

use sc_client_api::{
	BlockBackend, BlockchainEvents, ExecutorProvider, ProofProvider, StorageProvider, UsageProvider,
};
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::block_validation::Chain;
use sp_core::traits::{CodeExecutor, SpawnNamed};
use sp_runtime::traits::BlockIdTo;
use sp_runtime::{traits::Block as BlockT, BuildStorage};
use sp_transaction_pool::MaintainedTransactionPool;
use sp_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};

use sc_client_api::execution_extensions::ExecutionExtensions;
use sc_client_db::{Backend, DatabaseSettings};
use sc_keystore::Store as Keystore;
use sc_service::{
	config::{Configuration, KeystoreConfig},
	error::Error,
	MallocSizeOfWasm, RpcExtensionBuilder,
};

use ec_executor::{NativeExecutionDispatch, NativeExecutor, RuntimeInfo};

use log::info;

use crate::client::Client;
use crate::start_rpc_servers;
use crate::task_manager::{SpawnTaskHandle, TaskManager};
use crate::RpcHandlers;

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

/// Parameters to pass into `build`.
pub struct SpawnTasksParams<'a, TBl: BlockT, TCl, TExPool, TRpc, Backend> {
	/// The service configuration.
	pub config: Configuration,
	/// A shared client returned by `new_full_parts`/`new_light_parts`.
	pub client: Arc<TCl>,
	/// A shared backend returned by `new_full_parts`/`new_light_parts`.
	pub backend: Arc<Backend>,
	/// A task manager returned by `new_full_parts`/`new_light_parts`.
	pub task_manager: &'a mut TaskManager,
	/// A shared keystore returned by `new_full_parts`/`new_light_parts`.
	pub keystore: Arc<RwLock<Keystore>>,
	/// A shared transaction pool.
	pub transaction_pool: Arc<TExPool>,
	/// A RPC extension builder. Use `NoopRpcExtensionBuilder` if you just want to pass in the
	/// extensions directly.
	pub rpc_extensions_builder: Box<dyn RpcExtensionBuilder<Output = TRpc> + Send>,
	/// A Sender for RPC requests.
	pub system_rpc_tx: TracingUnboundedSender<sc_rpc::system::Request<TBl>>,
}

/// Spawn the tasks that are required to run a node.
pub fn spawn_tasks<TBl, TBackend, TExPool, TRpc, TCl>(
	params: SpawnTasksParams<TBl, TCl, TExPool, TRpc, TBackend>,
) -> Result<RpcHandlers, Error>
where
	TCl: ProvideRuntimeApi<TBl>
		+ HeaderMetadata<TBl, Error = sp_blockchain::Error>
		+ Chain<TBl>
		+ BlockBackend<TBl>
		+ BlockIdTo<TBl, Error = sp_blockchain::Error>
		+ ProofProvider<TBl>
		+ HeaderBackend<TBl>
		+ BlockchainEvents<TBl>
		+ ExecutorProvider<TBl>
		+ UsageProvider<TBl>
		+ StorageProvider<TBl, TBackend>
		+ CallApiAt<TBl, Error = sp_blockchain::Error>
		+ Send
		+ 'static,
	<TCl as ProvideRuntimeApi<TBl>>::Api: sp_api::Metadata<TBl>
		+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<TBl>
		+ sp_session::SessionKeys<TBl>
		+ sp_api::ApiErrorExt<Error = sp_blockchain::Error>
		+ sp_api::ApiExt<TBl, StateBackend = TBackend::State>,
	TBl: BlockT,
	TBackend: 'static + sc_client_api::backend::Backend<TBl> + Send,
	TExPool: MaintainedTransactionPool<Block = TBl, Hash = <TBl as BlockT>::Hash>
		+ MallocSizeOfWasm
		+ 'static,
	TRpc: sc_rpc::RpcExtension<sc_rpc::Metadata>,
{
	let SpawnTasksParams {
		config,
		task_manager,
		client,
		backend: _,
		keystore,
		transaction_pool,
		rpc_extensions_builder,
		system_rpc_tx,
	} = params;

	let chain_info = client.usage_info().chain;

	// TODO may do not need this
	// sp_session::generate_initial_session_keys(
	// 	client.clone(),
	// 	&BlockId::Hash(chain_info.best_hash),
	// 	config.dev_key_seed.clone().map(|s| vec![s]).unwrap_or_default(),
	// )?;

	info!("📦 Highest known block at #{}", chain_info.best_number);

	let spawn_handle = task_manager.spawn_handle();

	// Inform the tx pool about imported and finalized blocks.
	spawn_handle.spawn(
		"txpool-notifications",
		sc_transaction_pool::notification_future(client.clone(), transaction_pool.clone()),
	);

	// RPC
	let gen_handler = |deny_unsafe: sc_rpc::DenyUnsafe,
	                   rpc_middleware: sc_rpc_server::RpcMiddleware| {
		gen_handler(
			deny_unsafe,
			rpc_middleware,
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			keystore.clone(),
			&*rpc_extensions_builder,
			system_rpc_tx.clone(),
		)
	};
	// TODO add custom rpc
	let rpc = start_rpc_servers(&config, gen_handler, None)?;
	// This is used internally, so don't restrict access to unsafe RPC
	let rpc_handlers = RpcHandlers(Arc::new(
		gen_handler(
			sc_rpc::DenyUnsafe::No,
			sc_rpc_server::RpcMiddleware::new(None, "inbrowser"),
		)
		.into(),
	));

	// // Spawn informant task
	// todo use another informant which not include network
	// spawn_handle.spawn("informant", sc_informant::build(
	// 	client.clone(),
	// 	network_status_sinks.status.clone(),
	// 	transaction_pool.clone(),
	// 	config.informant_output_format,
	// ));

	task_manager.keep_alive((config.base_path, rpc, rpc_handlers.clone()));

	Ok(rpc_handlers)
}

fn gen_handler<TBl, TBackend, TExPool, TRpc, TCl>(
	deny_unsafe: sc_rpc::DenyUnsafe,
	rpc_middleware: sc_rpc_server::RpcMiddleware,
	config: &Configuration,
	spawn_handle: SpawnTaskHandle,
	client: Arc<TCl>,
	transaction_pool: Arc<TExPool>,
	keystore: Arc<RwLock<Keystore>>,
	rpc_extensions_builder: &(dyn RpcExtensionBuilder<Output = TRpc> + Send),
	// offchain_storage: Option<<TBackend as sc_client_api::backend::Backend<TBl>>::OffchainStorage>,
	system_rpc_tx: TracingUnboundedSender<sc_rpc::system::Request<TBl>>,
) -> sc_rpc_server::RpcHandler<sc_rpc::Metadata>
where
	TBl: BlockT,
	TCl: ProvideRuntimeApi<TBl>
		+ BlockchainEvents<TBl>
		+ HeaderBackend<TBl>
		+ HeaderMetadata<TBl, Error = sp_blockchain::Error>
		+ ExecutorProvider<TBl>
		+ CallApiAt<TBl, Error = sp_blockchain::Error>
		+ ProofProvider<TBl>
		+ StorageProvider<TBl, TBackend>
		+ BlockBackend<TBl>
		+ Send
		+ Sync
		+ 'static,
	TExPool: MaintainedTransactionPool<Block = TBl, Hash = <TBl as BlockT>::Hash> + 'static,
	TBackend: sc_client_api::backend::Backend<TBl> + 'static,
	TRpc: sc_rpc::RpcExtension<sc_rpc::Metadata>,
	<TCl as ProvideRuntimeApi<TBl>>::Api:
		sp_session::SessionKeys<TBl> + sp_api::Metadata<TBl, Error = sp_blockchain::Error>,
{
	use sc_rpc::{author, chain, state, system};

	let system_info = sc_rpc::system::SystemInfo {
		chain_name: config.chain_spec.name().into(),
		impl_name: config.impl_name.clone(),
		impl_version: config.impl_version.clone(),
		properties: config.chain_spec.properties(),
		chain_type: config.chain_spec.chain_type(),
	};

	let task_executor = sc_rpc::SubscriptionTaskExecutor::new(spawn_handle);
	let subscriptions = SubscriptionManager::new(Arc::new(task_executor.clone()));

	let (chain, state, child_state) = {
		let chain = sc_rpc::chain::new_full(client.clone(), subscriptions.clone());
		let (state, child_state) = sc_rpc::state::new_full(client.clone(), subscriptions.clone());
		(chain, state, child_state)
	};

	let author = sc_rpc::author::Author::new(
		client,
		transaction_pool,
		subscriptions,
		keystore,
		deny_unsafe,
	);
	let system = system::System::new(system_info, system_rpc_tx, deny_unsafe);

	sc_rpc_server::rpc_handler(
		(
			state::StateApi::to_delegate(state),
			state::ChildStateApi::to_delegate(child_state),
			chain::ChainApi::to_delegate(chain),
			author::AuthorApi::to_delegate(author),
			system::SystemApi::to_delegate(system),
			rpc_extensions_builder.build(deny_unsafe, task_executor),
		),
		rpc_middleware,
	)
}

pub fn build_mock_network<TBl>(
	spawn_handle: SpawnTaskHandle,
) -> Result<TracingUnboundedSender<sc_rpc::system::Request<TBl>>, Error>
where
	TBl: BlockT,
{
	let (system_rpc_tx, system_rpc_rx) = tracing_unbounded("mpsc_system_rpc");
	async fn build_network_future<
		B: BlockT,
		// C: BlockchainEvents<B>,
	>(
		mut rpc_rx: TracingUnboundedReceiver<sc_rpc::system::Request<B>>,
	) {
		loop {
			let request = rpc_rx.next().await;
			if let Some(request) = request {
				match request {
					sc_rpc::system::Request::Health(sender) => {
						let _ = sender.send(sc_rpc::system::Health {
							peers: 0,
							is_syncing: false,
							should_have_peers: false,
						});
					}
					sc_rpc::system::Request::LocalPeerId(sender) => {
						let _ = sender.send("".to_string()); // todo use a valid peerid
					}
					sc_rpc::system::Request::LocalListenAddresses(sender) => {
						let _ = sender.send(vec![]);
					}
					sc_rpc::system::Request::Peers(sender) => {
						let _ = sender.send(vec![]);
					}
					sc_rpc::system::Request::NetworkState(sender) => {
						let _ = sender.send(serde_json::Value::Null);
					}
					sc_rpc::system::Request::NetworkAddReservedPeer(_peer_addr, sender) => {
						let _ = sender.send(Ok(()));
					}
					sc_rpc::system::Request::NetworkRemoveReservedPeer(_peer_id, sender) => {
						let _ = sender.send(Ok(()));
					}
					sc_rpc::system::Request::NodeRoles(sender) => {
						let _ = sender.send(vec![]);
					}
				}
			} else {
				// todo print something?
				break;
			}
		}
	}
	let future = build_network_future(system_rpc_rx);
	spawn_handle.spawn_blocking("mock-network-worker", async move { future.await });
	Ok(system_rpc_tx)
}
