use sp_inherents::InherentDataProviders;

use ec_service::{error::Error, Configuration, IoHandler, NoopRpcExtensionBuilder, TaskManager};

use europa_executor::Executor;
use europa_runtime::{self, opaque::Block, RuntimeApi};

/// `new_full` is node construction process.
/// Currently could construct 3 types things:
/// * `InherentDataProviders`
/// * config pre-process
/// * custom rpc
pub fn new_full(config: Configuration) -> Result<TaskManager, Error> {
	// construct inherent
	let inherent_data_providers = InherentDataProviders::new();
	inherent_data_providers
		.register_provider(sp_timestamp::InherentDataProvider)
		.map_err(Into::into)
		.map_err(sp_consensus::error::Error::InherentData)?;
	// need Block, RuntimeApi, Executor type
	ec_service::builder_ext::new_node::<Block, RuntimeApi, Executor, _, _>(
		config,
		inherent_data_providers,
		|_components| Box::new(NoopRpcExtensionBuilder(IoHandler::default())),
	)
}
