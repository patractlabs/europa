use std::sync::Arc;

use sp_inherents::InherentDataProviders;

use ec_service::{config::Configuration, error::Error, TFullParts, TFullStateKv, TaskManager};

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
		|components| {
			let client = components.client.clone();
			let pool = components.transaction_pool.clone();

			Box::new(move |deny_unsafe, _| {
				let deps = europa_rpc::FullDeps::<_, _> {
					client: client.clone(),
					pool: pool.clone(),
					deny_unsafe,
				};

				europa_rpc::create_full(deps)
			})
		},
	)
}

pub fn new_full_parts(
	config: &Configuration,
	read_only: bool,
) -> Result<TFullParts<Block, RuntimeApi, Executor>, Error> {
	ec_service::new_full_parts::<Block, RuntimeApi, Executor>(config, read_only)
}

pub fn new_state_kv(config: &Configuration, read_only: bool) -> Result<Arc<TFullStateKv>, Error> {
	let settings = ec_service::database_settings(config);
	let state_kv = ec_service::new_state_kv(&settings, read_only)?;
	Ok(state_kv)
}
