// This file is part of europa which is forked form Substrate.

// Copyright (C) 2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Configuration trait for a CLI based on substrate

use log::warn;
use std::net::SocketAddr;
use std::path::PathBuf;

use sc_cli::{
	arg_enums::Database, generate_node_name, init_logger, DefaultConfigurationValues, Result,
};
// TODO may use local
pub use sc_cli::{DatabaseParams, KeystoreParams, SubstrateCli};

use ec_service::{
	config::{
		BasePath, Configuration, DatabaseConfig, KeystoreConfig, PruningMode, RpcMethods,
		TaskExecutor, TransactionPoolOptions,
	},
	TracingReceiver,
};

use crate::params::{ImportParams, PruningParams, SharedParams};

/// The recommended open file descriptor limit to be configured for the process.
const RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT: u64 = 10_000;

/// A trait that allows converting an object to a Configuration
pub trait CliConfiguration<DCV: DefaultConfigurationValues = ()>: Sized {
	/// Get the SharedParams for this object
	fn shared_params(&self) -> &SharedParams;

	/// Get the ImportParams for this object
	fn import_params(&self) -> Option<&ImportParams> {
		None
	}

	/// Get the PruningParams for this object
	fn pruning_params(&self) -> Option<&PruningParams> {
		self.import_params().map(|x| &x.pruning_params)
	}

	/// Get the KeystoreParams for this object
	fn keystore_params(&self) -> Option<&KeystoreParams> {
		None
	}

	/// Get the DatabaseParams for this object
	fn database_params(&self) -> Option<&DatabaseParams> {
		self.import_params().map(|x| &x.database_params)
	}

	/// Get the base path of the configuration (if any)
	///
	/// By default this is retrieved from `SharedParams`.
	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self.shared_params().base_path())
	}

	/// Get the transaction pool options
	///
	/// By default this is `TransactionPoolOptions::default()`.
	fn transaction_pool(&self) -> Result<TransactionPoolOptions> {
		Ok(Default::default())
	}

	/// Get the keystore configuration.
	///
	/// Bu default this is retrieved from `KeystoreParams` if it is available. Otherwise it uses
	/// `KeystoreConfig::InMemory`.
	fn keystore_config(&self, base_path: &PathBuf) -> Result<KeystoreConfig> {
		self.keystore_params()
			.map(|x| x.keystore_config(base_path))
			.unwrap_or(Ok(KeystoreConfig::InMemory))
	}

	/// Get the database cache size.
	///
	/// By default this is retrieved from `DatabaseParams` if it is available. Otherwise its `None`.
	fn database_cache_size(&self) -> Result<Option<usize>> {
		Ok(self
			.database_params()
			.map(|x| x.database_cache_size())
			.unwrap_or_default())
	}

	/// Get the database backend variant.
	///
	/// By default this is retrieved from `DatabaseParams` if it is available. Otherwise its `None`.
	fn database(&self) -> Result<Option<Database>> {
		Ok(self.database_params().and_then(|x| x.database()))
	}

	/// Get the database configuration object for the parameters provided
	fn database_config(
		&self,
		base_path: &PathBuf,
		cache_size: usize,
		database: Database,
	) -> Result<DatabaseConfig> {
		Ok(match database {
			Database::RocksDb => DatabaseConfig::RocksDb {
				path: base_path.join("db"),
				cache_size,
			},
			Database::ParityDb => DatabaseConfig::ParityDb {
				path: base_path.join("paritydb"),
			},
		})
	}

	/// Get the state cache size.
	///
	/// By default this is retrieved from `ImportParams` if it is available. Otherwise its `0`.
	fn state_cache_size(&self) -> Result<usize> {
		Ok(self
			.import_params()
			.map(|x| x.state_cache_size())
			.unwrap_or_default())
	}

	/// Get the state cache child ratio (if any).
	///
	/// By default this is `None`.
	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		Ok(Default::default())
	}

	/// Get the pruning mode.
	///
	/// By default this is retrieved from `PruningMode` if it is available. Otherwise its
	/// `PruningMode::default()`.
	/// // TODO may remove unsafe_pruning
	fn pruning(&self, unsafe_pruning: bool) -> Result<PruningMode> {
		self.pruning_params()
			.map(|x| x.pruning(unsafe_pruning))
			.unwrap_or_else(|| Ok(Default::default()))
	}

	/// Get the chain ID (string).
	///
	/// By default this is retrieved from `SharedParams`.
	fn chain_id(&self) -> Result<String> {
		Ok(self.shared_params().chain_id())
	}

	/// Get the name of the node.
	///
	/// By default a random name is generated.
	fn node_name(&self) -> Result<String> {
		Ok(generate_node_name())
	}

	/// Get the RPC HTTP address (`None` if disabled).
	///
	/// By default this is `None`.
	fn rpc_http(&self, _default_listen_port: u16) -> Result<Option<SocketAddr>> {
		Ok(None)
	}

	/// Get the RPC IPC path (`None` if disabled).
	///
	/// By default this is `None`.
	fn rpc_ipc(&self) -> Result<Option<String>> {
		Ok(None)
	}

	/// Get the RPC websocket address (`None` if disabled).
	///
	/// By default this is `None`.
	fn rpc_ws(&self, _default_listen_port: u16) -> Result<Option<SocketAddr>> {
		Ok(None)
	}

	/// Returns the RPC method set to expose.
	///
	/// By default this is `RpcMethods::Auto` (unsafe RPCs are denied iff
	/// `{rpc,ws}_external` returns true, respectively).
	fn rpc_methods(&self) -> Result<RpcMethods> {
		Ok(Default::default())
	}

	/// Get the RPC websockets maximum connections (`None` if unlimited).
	///
	/// By default this is `None`.
	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		Ok(None)
	}

	/// Get the RPC cors (`None` if disabled)
	///
	/// By default this is `Some(Vec::new())`.
	fn rpc_cors(&self, _is_dev: bool) -> Result<Option<Vec<String>>> {
		Ok(Some(Vec::new()))
	}

	/// Get the development key seed from the current object
	///
	/// By default this is `None`.
	fn dev_key_seed(&self, _is_dev: bool) -> Result<Option<String>> {
		Ok(Default::default())
	}

	/// Get the tracing targets from the current object (if any)
	///
	/// By default this is retrieved from `ImportParams` if it is available. Otherwise its
	/// `None`.
	fn tracing_targets(&self) -> Result<Option<String>> {
		Ok(self
			.import_params()
			.map(|x| x.tracing_targets())
			.unwrap_or_else(|| Default::default()))
	}

	/// Get the TracingReceiver value from the current object
	///
	/// By default this is retrieved from `ImportParams` if it is available. Otherwise its
	/// `TracingReceiver::default()`.
	fn tracing_receiver(&self) -> Result<TracingReceiver> {
		Ok(self
			.import_params()
			.map(|x| x.tracing_receiver())
			.unwrap_or_default())
	}

	/// Activate or not the automatic announcing of blocks after import
	///
	/// By default this is `false`.
	fn announce_block(&self) -> Result<bool> {
		Ok(true)
	}

	/// Create a Configuration object from the current object
	fn create_configuration<C: SubstrateCli>(
		&self,
		cli: &C,
		task_executor: TaskExecutor,
	) -> Result<Configuration> {
		let chain_id = self.chain_id()?;
		let chain_spec = cli.load_spec(chain_id.as_str())?;
		let base_path = self
			.base_path()?
			.unwrap_or_else(|| BasePath::from_project("", "", &C::executable_name()));
		let config_dir = base_path
			.path()
			.to_path_buf()
			.join("chains")
			.join(chain_spec.id());
		// TODO may need to use this var
		// let client_id = C::client_id();
		let database_cache_size = self.database_cache_size()?.unwrap_or(128);
		let database = self.database()?.unwrap_or(Database::RocksDb);

		let unsafe_pruning = self
			.import_params()
			.map(|p| p.unsafe_pruning)
			.unwrap_or(false);

		Ok(Configuration {
			impl_name: C::impl_name(),
			impl_version: C::impl_version(),
			task_executor,
			transaction_pool: self.transaction_pool()?,
			keystore: self.keystore_config(&config_dir)?,
			database: self.database_config(&config_dir, database_cache_size, database)?,
			state_cache_size: self.state_cache_size()?,
			state_cache_child_ratio: self.state_cache_child_ratio()?,
			pruning: self.pruning(unsafe_pruning)?,
			rpc_http: self.rpc_http(DCV::rpc_http_listen_port())?,
			rpc_ws: self.rpc_ws(DCV::rpc_ws_listen_port())?,
			rpc_ipc: self.rpc_ipc()?,
			rpc_methods: self.rpc_methods()?,
			rpc_ws_max_connections: self.rpc_ws_max_connections()?,
			rpc_cors: self.rpc_cors(true)?,
			dev_key_seed: self.dev_key_seed(true)?, // TODO may remove is_dev
			tracing_targets: self.tracing_targets()?,
			tracing_receiver: self.tracing_receiver()?,
			chain_spec,
			announce_block: self.announce_block()?,
			base_path: Some(base_path),
			informant_output_format: Default::default(),
		})
	}

	/// Get the filters for the logging.
	///
	/// This should be a list of comma-separated values.
	/// Example: `foo=trace,bar=debug,baz=info`
	///
	/// By default this is retrieved from `SharedParams`.
	fn log_filters(&self) -> Result<String> {
		Ok(self.shared_params().log_filters().join(","))
	}

	/// Initialize substrate. This must be done only once per process.
	///
	/// This method:
	///
	/// 1. Sets the panic handler
	/// 2. Initializes the logger
	/// 3. Raises the FD limit
	fn init<C: SubstrateCli>(&self) -> Result<()> {
		let logger_pattern = self.log_filters()?;
		let tracing_receiver = self.tracing_receiver()?;
		let tracing_targets = self.tracing_targets()?;

		sp_panic_handler::set(&C::support_url(), &C::impl_version());

		if let Err(e) = init_logger(&logger_pattern, tracing_receiver, tracing_targets) {
			log::warn!("💬 Problem initializing global logging framework: {:}", e)
		}

		if let Some(new_limit) = fdlimit::raise_fd_limit() {
			if new_limit < RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT {
				warn!(
					"Low open file descriptor limit configured for the process. \
					 Current value: {:?}, recommended value: {:?}.",
					new_limit, RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT,
				);
			}
		}

		Ok(())
	}
}
