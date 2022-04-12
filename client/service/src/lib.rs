// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

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

mod builder;
pub mod builder_ext;
pub mod client;
pub mod config;
pub mod task_manager;

use std::net::SocketAddr;
use std::sync::Arc;
use std::{io, pin::Pin};

use futures::{compat::*, Future, FutureExt};
pub use jsonrpc_core::IoHandler;

pub use sp_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};

pub use sc_service::{
	build_network, error, BasePath, BuildNetworkParams, ChainSpec, ChainType, GenericChainSpec,
	NoopRpcExtensionBuilder, RpcExtensionBuilder, RpcMethods, TaskType, TracingReceiver,
	TransactionPoolOptions,
};

use crate::builder::KeystoreContainer;
pub use crate::builder::{
	build_mock_network, database_settings, new_client, new_full_parts, new_state_kv, spawn_tasks,
	SpawnTasksParams, TFullBackend, TFullCallExecutor, TFullClient, TFullParts, TFullStateKv,
};
pub use crate::config::Configuration;
pub use crate::task_manager::{SpawnTaskHandle, TaskManager};

use log::warn;

/// A larger block size limit for debug contract
/// 1024 MB + 512B
pub const BLOCK_SIZE_LIMIT: usize = 1024 * 1024 * 1024 + 512;

/// An imcomplete set of chain components, but enough to run the chain ops subcommands.
pub struct PartialComponents<Client, Backend, SelectChain, ImportQueue, TransactionPool, Other> {
	/// A shared client instance.
	pub client: Arc<Client>,
	/// A shared backend instance.
	pub backend: Arc<Backend>,
	/// The chain task manager.
	pub task_manager: TaskManager,
	/// A keystore container instance..
	pub keystore_container: KeystoreContainer,
	/// A chain selection algorithm instance.
	pub select_chain: SelectChain,
	/// An import queue.
	pub import_queue: ImportQueue,
	/// A shared transaction pool.
	pub transaction_pool: Arc<TransactionPool>,
	/// Everything else that needs to be passed into the main build function.
	pub other: Other,
}

/// An RPC session. Used to perform in-memory RPC queries (ie. RPC queries that don't go through
/// the HTTP or WebSockets server).
#[derive(Clone)]
pub struct RpcSession {
	metadata: sc_rpc::Metadata,
}

impl RpcSession {
	/// Creates an RPC session.
	///
	/// The `sender` is stored inside the `RpcSession` and is used to communicate spontaneous JSON
	/// messages.
	///
	/// The `RpcSession` must be kept alive in order to receive messages on the sender.
	pub fn new(sender: futures01::sync::mpsc::Sender<String>) -> RpcSession {
		RpcSession {
			metadata: sender.into(),
		}
	}
}

/// RPC handlers that can perform RPC queries.
#[derive(Clone)]
pub struct RpcHandlers(
	Arc<jsonrpc_core::MetaIoHandler<sc_rpc::Metadata, sc_rpc_server::RpcMiddleware>>,
);

impl RpcHandlers {
	/// Starts an RPC query.
	///
	/// The query is passed as a string and must be a JSON text similar to what an HTTP client
	/// would for example send.
	///
	/// Returns a `Future` that contains the optional response.
	///
	/// If the request subscribes you to events, the `Sender` in the `RpcSession` object is used to
	/// send back spontaneous events.
	pub fn rpc_query(
		&self,
		mem: &RpcSession,
		request: &str,
	) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> {
		self.0
			.handle_request(request, mem.metadata.clone())
			.compat()
			.map(|res| res.expect("this should never fail"))
			.boxed()
	}

	/// Provides access to the underlying `MetaIoHandler`
	pub fn io_handler(
		&self,
	) -> Arc<jsonrpc_core::MetaIoHandler<sc_rpc::Metadata, sc_rpc_server::RpcMiddleware>> {
		self.0.clone()
	}
}

mod waiting {
	pub struct HttpServer(pub Option<sc_rpc_server::HttpServer>);
	impl Drop for HttpServer {
		fn drop(&mut self) {
			if let Some(server) = self.0.take() {
				server.close_handle().close();
				server.wait();
			}
		}
	}

	pub struct IpcServer(pub Option<sc_rpc_server::IpcServer>);
	impl Drop for IpcServer {
		fn drop(&mut self) {
			if let Some(server) = self.0.take() {
				server.close_handle().close();
				let _ = server.wait();
			}
		}
	}

	pub struct WsServer(pub Option<sc_rpc_server::WsServer>);
	impl Drop for WsServer {
		fn drop(&mut self) {
			if let Some(server) = self.0.take() {
				server.close_handle().close();
				let _ = server.wait();
			}
		}
	}
}

fn start_rpc_servers<
	H: FnMut(
		sc_rpc::DenyUnsafe,
		sc_rpc_server::RpcMiddleware,
	) -> sc_rpc_server::RpcHandler<sc_rpc::Metadata>,
>(
	config: &Configuration,
	mut gen_handler: H,
	rpc_metrics: sc_rpc_server::RpcMetrics, // todo may remove metrics
) -> Result<Box<dyn std::any::Any + Send + Sync>, error::Error> {
	fn maybe_start_server<T, F>(
		address: Option<SocketAddr>,
		mut start: F,
	) -> Result<Option<T>, io::Error>
	where
		F: FnMut(&SocketAddr) -> Result<T, io::Error>,
	{
		Ok(match address {
			Some(mut address) => Some(start(&address).or_else(|e| match e.kind() {
				io::ErrorKind::AddrInUse | io::ErrorKind::PermissionDenied => {
					warn!(
						"Unable to bind RPC server to {}. Trying random port.",
						address
					);
					address.set_port(0);
					start(&address)
				}
				_ => Err(e),
			})?),
			None => None,
		})
	}

	fn deny_unsafe(addr: &SocketAddr, methods: &RpcMethods) -> sc_rpc::DenyUnsafe {
		let is_exposed_addr = !addr.ip().is_loopback();
		match (is_exposed_addr, methods) {
			(_, RpcMethods::Unsafe) | (false, RpcMethods::Auto) => sc_rpc::DenyUnsafe::No,
			_ => sc_rpc::DenyUnsafe::Yes,
		}
	}

	Ok(Box::new((
		config.rpc_ipc.as_ref().map(|path| {
			sc_rpc_server::start_ipc(
				&*path,
				gen_handler(
					sc_rpc::DenyUnsafe::No,
					sc_rpc_server::RpcMiddleware::new(rpc_metrics.clone(), "ipc"),
				),
			)
		}),
		maybe_start_server(config.rpc_http, |address| {
			sc_rpc_server::start_http(
				address,
				None,
				config.rpc_cors.as_ref(),
				gen_handler(
					deny_unsafe(&address, &config.rpc_methods),
					sc_rpc_server::RpcMiddleware::new(rpc_metrics.clone(), "http"),
				),
				None,
			)
		})?
		.map(|s| waiting::HttpServer(Some(s))),
		maybe_start_server(config.rpc_ws, |address| {
			sc_rpc_server::start_ws(
				address,
				config.rpc_ws_max_connections,
				config.rpc_cors.as_ref(),
				gen_handler(
					deny_unsafe(&address, &config.rpc_methods),
					sc_rpc_server::RpcMiddleware::new(rpc_metrics.clone(), "ws"),
				),
				None,
			)
		})?
		.map(|s| waiting::WsServer(Some(s))),
	)))
}
