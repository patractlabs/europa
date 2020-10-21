use std::sync::Arc;

use sp_inherents::InherentDataProviders;

use ec_service::{
	build_mock_network, error::Error as ServiceError, Configuration, PartialComponents, TaskManager,
};

use europa_executor::Executor;
use europa_runtime::{self, opaque::Block, RuntimeApi};

type FullClient = ec_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = ec_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Returns most parts of a service. Not enough to run a full chain,
/// But enough to perform chain operations like purge-chain
pub fn new_partial(
	config: &Configuration,
) -> Result<
	PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sp_consensus::DefaultImportQueue<Block, FullClient>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(),
	>,
	ServiceError,
> {
	let inherent_data_providers = InherentDataProviders::new();
	inherent_data_providers
		.register_provider(sp_timestamp::InherentDataProvider)
		.map_err(Into::into)
		.map_err(sp_consensus::error::Error::InherentData)?;

	let (client, backend, keystore, task_manager) =
		ec_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let import_queue = sc_consensus_manual_seal::import_queue(
		Box::new(client.clone()),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	);

	Ok(ec_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (),
	})
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
	let ec_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue: _,
		keystore,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (),
	} = new_partial(&config)?;

	let role = config.role.clone();

	// let rpc_extensions_builder = {
	//     let client = client.clone();
	//     let pool = transaction_pool.clone();
	//
	//     Box::new(move |deny_unsafe, _| {
	//         let deps = rpc::FullDeps::<_, _, FullBackend> {
	//             client: client.clone(),
	//             pool: pool.clone(),
	//             deny_unsafe,
	//             grandpa: None,
	//         };
	//
	//         rpc::create_full(deps)
	//     })
	// };
	let system_rpc_tx = build_mock_network(task_manager.spawn_handle())?;
	ec_service::spawn_tasks(ec_service::SpawnTasksParams {
		client: client.clone(),
		keystore: keystore.clone(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_extensions_builder: Box::new(ec_service::NoopRpcExtensionBuilder(
			ec_service::IoHandler::default(),
		)), //rpc_extensions_builder,
		backend,
		system_rpc_tx,
		config,
	})?;

	if role.is_authority() {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			client.clone(),
			transaction_pool.clone(),
			None,
		);

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
	};

	Ok(task_manager)
}
