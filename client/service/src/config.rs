// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
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

//! Service configuration.

use std::net::SocketAddr;

pub use sc_client_db::{Database, DatabaseSettingsSrc as DatabaseConfig, PruningMode};

use sc_chain_spec::ChainSpec;
pub use sc_transaction_pool::txpool::Options as TransactionPoolOptions;

pub use sc_service::config::{BasePath, KeystoreConfig, RpcMethods, TaskExecutor, TaskType};

/// Service configuration.
#[derive(Debug)]
pub struct Configuration {
	/// Implementation name
	pub impl_name: String,
	/// Implementation version (see sc-cli to see an example of format)
	pub impl_version: String,
	/// How to spawn background tasks. Mandatory, otherwise creating a `Service` will error.
	pub task_executor: TaskExecutor,
	/// Extrinsic pool configuration.
	pub transaction_pool: TransactionPoolOptions,
	/// Configuration for the keystore.
	pub keystore: KeystoreConfig,
	/// Configuration for the database.
	pub database: DatabaseConfig,
	/// Size of internal state cache in Bytes
	pub state_cache_size: usize,
	/// Size in percent of cache size dedicated to child tries
	pub state_cache_child_ratio: Option<usize>,
	/// Pruning settings.
	pub pruning: PruningMode,
	/// Chain configuration.
	pub chain_spec: Box<dyn ChainSpec>,
	/// RPC over HTTP binding address. `None` if disabled.
	pub rpc_http: Option<SocketAddr>,
	/// RPC over Websockets binding address. `None` if disabled.
	pub rpc_ws: Option<SocketAddr>,
	/// RPC over IPC binding path. `None` if disabled.
	pub rpc_ipc: Option<String>,
	/// Maximum number of connections for WebSockets RPC server. `None` if default.
	pub rpc_ws_max_connections: Option<usize>,
	/// CORS settings for HTTP & WS servers. `None` if all origins are allowed.
	pub rpc_cors: Option<Vec<String>>,
	/// RPC methods to expose (by default only a safe subset or all of them).
	pub rpc_methods: RpcMethods,
	// /// Should offchain workers be executed.
	// pub offchain_worker: OffchainWorkerConfig, // todo may need offchain in future
	// /// Development key seed.
	// ///
	// /// When running in development mode, the seed will be used to generate authority keys by the keystore.
	// ///
	// /// Should only be set when `node` is running development mode.
	// pub dev_key_seed: Option<String>, // todo may need offchain in future
	/// Tracing targets
	pub tracing_targets: Option<String>,
	/// Announce block automatically after they have been imported
	pub announce_block: bool,
	/// Base path of the configuration
	pub base_path: Option<BasePath>,
	/// Workspace for current node execution environment
	pub workspace: String,
	/// All workspace list
	pub workspace_list: Vec<String>,
	/// Configuration of the output format that the informant uses.
	pub informant_output_format: sc_informant::OutputFormat, // todo may also need in future
}
