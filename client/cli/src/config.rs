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

use serde::{Deserialize, Serialize};

use sc_cli::{arg_enums::Database, generate_node_name, DefaultConfigurationValues, Error, Result};
use sc_tracing::logging::LoggerBuilder;
// TODO may use local
pub use sc_cli::{DatabaseParams, KeystoreParams, SubstrateCli};

use ec_service::{
	config::{
		BasePath, Configuration, DatabaseConfig, ExtTransport, KeepBlocks, KeystoreConfig,
		PrometheusConfig, PruningMode, Role, RpcMethods, TaskExecutor, TransactionPoolOptions,
		TransactionStorageMode,
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

	/// Returns `true` if the node is for development or not
	///
	/// By default this is retrieved from `SharedParams`.
	fn is_dev(&self) -> Result<bool> {
		Ok(self.shared_params().is_dev())
	}

	/// Gets the role
	///
	/// By default this is `Role::Full`.
	fn role(&self, _is_dev: bool) -> Result<Role> {
		Ok(Role::Full)
	}

	/// Get the current workspace or
	fn workspace(&self) -> Option<&str> {
		self.shared_params().workspace()
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
		Ok(self
			.keystore_params()
			.map(|x| x.keystore_config(base_path))
			.unwrap_or(Ok((
				Some(format!(
					"Couldn't find keystore at {}",
					base_path.to_string_lossy()
				)),
				KeystoreConfig::InMemory,
			)))?
			.1)
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

	/// Get the database transaction storage scheme.
	fn database_transaction_storage(&self) -> Result<TransactionStorageMode> {
		Ok(self
			.database_params()
			.map(|x| x.transaction_storage())
			.unwrap_or(TransactionStorageMode::BlockBody))
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

	/// Get the state pruning mode.
	///
	/// By default this is retrieved from `PruningMode` if it is available. Otherwise its
	/// `PruningMode::default()`.
	fn state_pruning(&self, unsafe_pruning: bool, role: &Role) -> Result<PruningMode> {
		self.pruning_params()
			.map(|x| x.state_pruning(unsafe_pruning, role))
			.unwrap_or_else(|| Ok(Default::default()))
	}

	/// Get the block pruning mode.
	///
	/// By default this is retrieved from `block_pruning` if it is available. Otherwise its
	/// `KeepBlocks::All`.
	fn keep_blocks(&self) -> Result<KeepBlocks> {
		self.pruning_params()
			.map(|x| x.keep_blocks())
			.unwrap_or_else(|| Ok(KeepBlocks::All))
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
	fn rpc_cors(&self) -> Result<Option<Vec<String>>> {
		Ok(Some(Vec::new()))
	}

	/// Get the prometheus configuration (`None` if disabled)
	///
	/// By default this is `None`.
	fn prometheus_config(&self, _default_listen_port: u16) -> Result<Option<PrometheusConfig>> {
		Ok(None)
	}

	/// Get the TracingReceiver value from the current object
	///
	/// By default this is retrieved from [`SharedParams`] if it is available. Otherwise its
	/// `TracingReceiver::default()`.
	fn tracing_receiver(&self) -> Result<TracingReceiver> {
		Ok(self.shared_params().tracing_receiver())
	}

	/// Get the tracing targets from the current object (if any)
	///
	/// By default this is retrieved from `ImportParams` if it is available. Otherwise its
	/// `None`.
	fn tracing_targets(&self) -> Result<Option<String>> {
		Ok(self.shared_params().tracing_targets())
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
		let is_dev = self.is_dev()?;
		let chain_id = self.chain_id()?;
		let chain_spec = cli.load_spec(chain_id.as_str())?;
		let mut base_path = self
			.base_path()?
			.unwrap_or_else(|| BasePath::from_project("", "", &C::executable_name()));

		let metadata = metadata(&base_path, |mut metadata| {
			let workspace = self.workspace().unwrap_or(
				metadata
					.current_workspace
					.as_ref()
					.map(AsRef::as_ref)
					.unwrap_or(DEFAULT_WORKSPACE),
			);
			match metadata.workspaces {
				Some(ref mut list) => {
					if !list.iter().any(|x| x == workspace) {
						list.push(workspace.to_string());
					}
				}
				None => metadata.workspaces = Some(vec![workspace.to_string()]),
			}
			metadata.current_workspace = Some(workspace.to_string());
			metadata
		})?;

		let workspace = metadata
			.current_workspace
			.as_ref()
			.map(ToString::to_string)
			.expect("workspace must exist");
		let workspace_list = metadata
			.workspaces
			.as_ref()
			.map(Clone::clone)
			.expect("workspace must exist");
		match base_path {
			BasePath::Permanenent(ref mut p) => {
				// replace old path to new path with workspace
				*p = p.join(&workspace);
			}
			BasePath::Temporary(_) => { /* no thing for temporary dir*/ }
		}

		let config_dir = base_path
			.path()
			.to_path_buf()
			.join("chains")
			.join(chain_spec.id());
		let role = self.role(is_dev)?;
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
			// pruning: self.pruning(unsafe_pruning)?,
			state_pruning: self.state_pruning(unsafe_pruning, &role)?,
			keep_blocks: self.keep_blocks()?,
			prometheus_config: self.prometheus_config(DCV::prometheus_listen_port())?,
			rpc_http: self.rpc_http(DCV::rpc_http_listen_port())?,
			rpc_ws: self.rpc_ws(DCV::rpc_ws_listen_port())?,
			rpc_ipc: self.rpc_ipc()?,
			rpc_methods: self.rpc_methods()?,
			rpc_ws_max_connections: self.rpc_ws_max_connections()?,
			rpc_cors: self.rpc_cors()?,
			tracing_targets: self.tracing_targets()?,
			transaction_storage: self.database_transaction_storage()?,
			chain_spec,
			announce_block: self.announce_block()?,
			role,
			base_path: Some(base_path),
			workspace,
			workspace_list,
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

	/// Is log reloading disabled (enabled by default)
	fn is_log_filter_reloading_disabled(&self) -> Result<bool> {
		Ok(self.shared_params().is_log_filter_reloading_disabled())
	}

	/// Should the log color output be disabled?
	fn disable_log_color(&self) -> Result<bool> {
		Ok(self.shared_params().disable_log_color())
	}

	/// Get the telemetry external transport
	///
	/// By default this is `None`.
	fn telemetry_external_transport(&self) -> Result<Option<ExtTransport>> {
		Ok(None)
	}

	/// Initialize substrate. This must be done only once per process.
	///
	/// This method:
	///
	/// 1. Sets the panic handler
	/// 2. Initializes the logger
	/// 3. Raises the FD limit
	fn init<C: SubstrateCli>(&self) -> Result<sc_telemetry::TelemetryWorker> {
		sp_panic_handler::set(&C::support_url(), &C::impl_version());

		let mut logger = LoggerBuilder::new(self.log_filters()?);
		logger.with_log_reloading(!self.is_log_filter_reloading_disabled()?);

		if let Some(transport) = self.telemetry_external_transport()? {
			logger.with_transport(transport);
		}

		if let Some(tracing_targets) = self.tracing_targets()? {
			let tracing_receiver = self.tracing_receiver()?;
			logger.with_profiling(tracing_receiver, tracing_targets);
		}

		if self.disable_log_color()? {
			logger.with_colors(false);
		}

		let telemetry_worker = logger.init()?;

		if let Some(new_limit) = fdlimit::raise_fd_limit() {
			if new_limit < RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT {
				warn!(
					"Low open file descriptor limit configured for the process. \
					 Current value: {:?}, recommended value: {:?}.",
					new_limit, RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT,
				);
			}
		}

		Ok(telemetry_worker)
	}
}

pub const METADATA_FILE: &'static str = "_metadata";
pub const DEFAULT_WORKSPACE: &'static str = "default";

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Metadata {
	pub workspaces: Option<Vec<String>>,
	pub current_workspace: Option<String>,
}

pub fn metadata(base_path: &BasePath, f: impl Fn(Metadata) -> Metadata) -> Result<Metadata> {
	use std::fs;
	let mut p = base_path.path().to_path_buf();
	if !p.exists() {
		fs::create_dir_all(&p)?;
	}
	p.push(METADATA_FILE);
	if !p.exists() {
		fs::write(&p, "{}")?;
	}
	let data = fs::read(&p)?;
	let metadata: Metadata = serde_json::from_slice(&data).map_err(|e| {
		Error::Application(format!("metadata file do not contains a valid json, e:{:?}", e).into())
	})?;
	let old_metadata = metadata.clone();
	let new_metadata = f(metadata);
	if old_metadata != new_metadata {
		let bytes = serde_json::to_vec(&new_metadata).expect("must be valid metadata struct json");
		fs::write(p, bytes)?;
	}
	Ok(new_metadata)
}
