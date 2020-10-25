use std::sync::Arc;

use sp_api::{ApiExt, TransactionFor};
use sp_inherents::InherentDataProviders;
use sp_runtime::traits::Block as BlockT;

use sc_basic_authorship::ProposerFactory;
pub use sc_keystore::KeyStorePtr as KeyStore;
use sc_service::{Configuration, RpcExtensionBuilder};

use ec_executor::NativeExecutionDispatch;

use crate::{build_mock_network, error, spawn_tasks, SpawnTasksParams, TFullBackend};
use crate::{new_full_parts, TFullClient, TaskManager};
use sc_transaction_pool::FullPool;

/// A node components, for rpc construction.
pub struct NodeComponents<TBl: BlockT, TRtApi, TExecDisp: NativeExecutionDispatch + 'static>
where
	TRtApi: sp_api::ConstructRuntimeApi<TBl, TFullClient<TBl, TRtApi, TExecDisp>>
		+ Send
		+ Sync
		+ 'static,
	sp_api::ApiErrorFor<TFullClient<TBl, TRtApi, TExecDisp>, TBl>: Send + std::fmt::Display,
	<TFullClient<TBl, TRtApi, TExecDisp> as sp_api::ProvideRuntimeApi<TBl>>::Api:
		sp_transaction_pool::runtime_api::TaggedTransactionQueue<TBl>,
{
	/// A shared client instance.
	pub client: Arc<TFullClient<TBl, TRtApi, TExecDisp>>,
	/// A shared backend instance.
	pub backend: Arc<TFullBackend<TBl>>,
	/// A shared keystore instance.
	pub keystore: KeyStore,
	/// A chain selection algorithm instance.
	pub select_chain: sc_consensus::LongestChain<TFullBackend<TBl>, TBl>,
	/// An import queue.
	pub import_queue: sp_consensus::import_queue::BasicQueue<
		TBl,
		TransactionFor<TFullClient<TBl, TRtApi, TExecDisp>, TBl>,
	>,
	/// A shared transaction pool.
	pub transaction_pool: Arc<FullPool<TBl, TFullClient<TBl, TRtApi, TExecDisp>>>,
	/// A registry of all providers of `InherentData`.
	pub inherent_data_providers: sp_inherents::InherentDataProviders,
}

pub fn new_node<TBl, TRtApi, TExecDisp, F, TRpc>(
	config: Configuration,
	inherent_data_providers: InherentDataProviders,
	rpc_builder: F,
) -> Result<TaskManager, error::Error>
where
	F: Fn(
		NodeComponents<TBl, TRtApi, TExecDisp>,
	) -> Box<dyn RpcExtensionBuilder<Output = TRpc> + Send>,
	TRpc: sc_rpc::RpcExtension<sc_rpc::Metadata>,
	TBl: BlockT,
	TRtApi: sp_api::ConstructRuntimeApi<TBl, TFullClient<TBl, TRtApi, TExecDisp>>
		+ Send
		+ Sync
		+ 'static,
	TExecDisp: NativeExecutionDispatch + 'static,
	TFullClient<TBl, TRtApi, TExecDisp>: sp_api::ProvideRuntimeApi<TBl>
		+ sc_client_api::BlockBackend<TBl>
		+ sp_runtime::traits::BlockIdTo<TBl>,
	// for transaction_pool
	TFullClient<TBl, TRtApi, TExecDisp>:
		sc_client_api::ExecutorProvider<TBl> + Send + Sync + 'static,
	<TFullClient<TBl, TRtApi, TExecDisp> as sp_api::ProvideRuntimeApi<TBl>>::Api:
		sp_transaction_pool::runtime_api::TaggedTransactionQueue<TBl>,
	sp_api::ApiErrorFor<TFullClient<TBl, TRtApi, TExecDisp>, TBl>: Send + std::fmt::Display,
	// for import_queue
	TFullClient<TBl, TRtApi, TExecDisp>:
		sp_consensus::BlockImport<TBl, Error = sp_consensus::Error>,
	<TFullClient<TBl, TRtApi, TExecDisp> as sp_api::ProvideRuntimeApi<TBl>>::Api: sp_api::Core<TBl, Error = sp_blockchain::Error>
		+ ApiExt<
			TBl,
			StateBackend = <TFullBackend<TBl> as sc_client_api::backend::Backend<TBl>>::State,
		>,
	// spawn_tasks
	TFullClient<TBl, TRtApi, TExecDisp>: sp_blockchain::HeaderMetadata<TBl, Error = sp_blockchain::Error>
		+ sp_consensus::block_validation::Chain<TBl>
		+ sp_runtime::traits::BlockIdTo<TBl, Error = sp_blockchain::Error>
		+ sc_client_api::ProofProvider<TBl>
		+ sp_blockchain::HeaderBackend<TBl>
		+ sc_client_api::BlockchainEvents<TBl>
		+ sc_client_api::UsageProvider<TBl>
		+ sc_client_api::StorageProvider<TBl, TFullBackend<TBl>>
		+ sp_api::CallApiAt<TBl, Error = sp_blockchain::Error>
		+ Send
		+ 'static,
	<TFullClient<TBl, TRtApi, TExecDisp> as sp_api::ProvideRuntimeApi<TBl>>::Api:
		sp_api::Metadata<TBl>
			+ sp_session::SessionKeys<TBl>
			+ sp_api::ApiErrorExt<Error = sp_blockchain::Error>,
	// manual_seal
	<TFullClient<TBl, TRtApi, TExecDisp> as sp_api::ProvideRuntimeApi<TBl>>::Api:
		sc_block_builder::BlockBuilderApi<TBl, Error = sp_blockchain::Error>,
{
	let (client, backend, keystore, mut task_manager) =
		new_full_parts::<TBl, TRtApi, TExecDisp>(&config)?;
	let client = Arc::new(client);

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let import_queue = sc_consensus_manual_seal::import_queue(
		Box::new(client.clone()),
		&task_manager.spawn_handle(),
		None,
	);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let components = NodeComponents {
		client: client.clone(),
		backend: backend.clone(),
		import_queue,
		keystore: keystore.clone(),
		select_chain: select_chain.clone(),
		transaction_pool: transaction_pool.clone(),
		inherent_data_providers: inherent_data_providers.clone(),
	};

	let rpc_extensions_builder = rpc_builder(components);
	let system_rpc_tx = build_mock_network::<TBl>(task_manager.spawn_handle())?;
	spawn_tasks(SpawnTasksParams {
		client: client.clone(),
		keystore,
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_extensions_builder,
		backend,
		system_rpc_tx,
		config,
	})?;

	let proposer: ProposerFactory<_, TFullBackend<TBl>, _> =
		sc_basic_authorship::ProposerFactory::new(client.clone(), transaction_pool.clone(), None);

	let params = sc_consensus_manual_seal::InstantSealParams {
		block_import: client.clone(),
		env: proposer,
		client: client.clone(),
		pool: transaction_pool.pool().clone(),
		select_chain,
		consensus_data_provider: None,
		inherent_data_providers,
	};
	let authorship_future = sc_consensus_manual_seal::run_instant_seal(params);

	task_manager
		.spawn_essential_handle()
		.spawn_blocking("instant-seal", authorship_future);

	Ok(task_manager)
}
